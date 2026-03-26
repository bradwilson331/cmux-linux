use crate::ghostty::ffi;
use crate::split_engine::SplitEngine;
use crate::workspace::Workspace;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub type AppStateRef = Rc<RefCell<AppState>>;

pub struct AppState {
    pub split_engines: Vec<SplitEngine>,
    pub gtk_app: gtk4::Application,
    /// All open workspaces. Never empty after initialization — create_workspace is called in new().
    pub workspaces: Vec<Workspace>,
    /// Index into workspaces of the currently visible workspace.
    pub active_index: usize,
    /// GtkStack holding one page per workspace (the workspace's root GTK widget).
    pub stack: gtk4::Stack,
    /// GtkListBox in the sidebar showing workspace names.
    pub sidebar_list: gtk4::ListBox,
    /// Ghostty app handle — used by create_surface() for new panes.
    pub ghostty_app: ffi::ghostty_app_t,
    /// Next workspace ID (monotonically increasing).
    next_id: u64,
    /// Next display number for default names ("Workspace N").
    next_display_number: usize,
    /// Notified after any workspace/pane mutation to trigger a debounced session save.
    pub save_notify: Option<std::sync::Arc<tokio::sync::Notify>>,
    /// Sender for session snapshots to the debounce task.
    /// Each mutation snapshots SessionData on the main thread and sends it here.
    pub session_tx: Option<tokio::sync::mpsc::UnboundedSender<crate::session::SessionData>>,
}

impl AppState {
    /// Create a new AppState. Does NOT create the first workspace — caller must call
    /// create_workspace() after constructing the GTK widget tree (Plan 04 wires this).
    pub fn new(
        stack: gtk4::Stack,
        sidebar_list: gtk4::ListBox,
        ghostty_app: ffi::ghostty_app_t,
        gtk_app: gtk4::Application,
    ) -> AppStateRef {
        let state = AppState {
            workspaces: Vec::new(),
            split_engines: Vec::new(),
            active_index: 0,
            stack,
            sidebar_list,
            ghostty_app,
            gtk_app,
            next_id: 1,
            next_display_number: 1,
            save_notify: None, // Set to Some(...) after tokio runtime is available in main.rs
            session_tx: None,
        };
        Rc::new(RefCell::new(state))
    }

    /// Create a new workspace. Allocates an ID, creates a sidebar row, and adds a placeholder
    /// page to the GtkStack. The actual GLArea/split root is added by the caller (Plan 04).
    /// Returns the new workspace id.
    pub fn create_workspace(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let display_number = self.next_display_number;
        self.next_display_number += 1;

        let mut workspace = Workspace::new(id, display_number);
        let name = workspace.name.clone();

        // Append a row to the sidebar GtkListBox.
        let label = gtk4::Label::new(Some(&name));
        label.set_halign(gtk4::Align::Start);
        let row = gtk4::ListBoxRow::new();
        row.set_child(Some(&label));
        unsafe {
            row.set_data("workspace-id", id);
        }
        self.sidebar_list.append(&row);

        // Create surface and split engine
        let pane_id = id * 1000;
        eprintln!(
            "cmux: create_workspace calling create_surface for workspace_id={}, pane_id={}",
            id, pane_id
        );
        let (gl_area, surface_cell) =
            crate::ghostty::surface::create_surface(&self.gtk_app, self.ghostty_app, None, pane_id);
        let engine = SplitEngine::new(
            self.gtk_app.clone(),
            self.ghostty_app,
            gl_area.clone(),
            surface_cell,
            pane_id,
        );

        // Add to stack
        let page_name = format!("workspace-{}", id);
        self.stack
            .add_named(&engine.root_widget(), Some(&page_name));
        workspace.stack_page_name = page_name;

        self.workspaces.push(workspace);
        self.split_engines.push(engine);

        let new_index = self.workspaces.len() - 1;
        self.switch_to_index(new_index);

        self.trigger_session_save();
        id
    }

    /// Close the workspace at `index`. Removes the sidebar row and GtkStack page.
    /// Returns false if there is only one workspace (cannot close the last one).
    /// The caller (Plan 04) is responsible for calling ghostty_surface_free on all panes first.
    pub fn close_workspace(&mut self, index: usize) -> bool {
        if self.workspaces.len() <= 1 {
            return false; // Cannot close the last workspace
        }

        // Before removing from workspaces, free all Ghostty surfaces in the split engine.
        if let Some(engine) = self.split_engines.get(index) {
            let mut surfaces = Vec::new();
            engine.root.collect_surfaces(&mut surfaces);
            for surface in surfaces {
                if !surface.is_null() {
                    unsafe {
                        crate::ghostty::ffi::ghostty_surface_free(surface);
                    }
                    if let Ok(mut reg) = crate::ghostty::callbacks::SURFACE_REGISTRY.lock() {
                        reg.remove(&(surface as usize));
                    }
                }
            }
        }
        self.split_engines.remove(index);

        let workspace = self.workspaces.remove(index);

        // Remove sidebar row.
        if let Some(row) = self.sidebar_list.row_at_index(index as i32) {
            self.sidebar_list.remove(&row);
        }

        // Remove GtkStack page.
        if let Some(child) = self.stack.child_by_name(&workspace.stack_page_name) {
            self.stack.remove(&child);
        }

        // Adjust active_index: if we removed before or at active, clamp.
        if self.active_index >= self.workspaces.len() {
            self.active_index = self.workspaces.len() - 1;
        } else if index <= self.active_index && self.active_index > 0 {
            self.active_index -= 1;
        }

        self.switch_to_index(self.active_index);
        self.trigger_session_save();
        true
    }

    /// Switch to the workspace at `index` (0-based). Updates GtkStack visible child and
    /// sidebar selection. Does nothing if index is out of bounds.
    pub fn switch_to_index(&mut self, index: usize) {
        if index >= self.workspaces.len() {
            return;
        }
        self.active_index = index;
        let page_name = self.workspaces[index].stack_page_name.clone();
        self.stack.set_visible_child_name(&page_name);
        if let Some(row) = self.sidebar_list.row_at_index(index as i32) {
            self.sidebar_list.select_row(Some(&row));
            // Update CSS classes: active row gets "active-workspace" for styling.
            // All rows: remove first, then add to active.
            let count = self.workspaces.len() as i32;
            for i in 0..count {
                if let Some(r) = self.sidebar_list.row_at_index(i) {
                    r.remove_css_class("active-workspace");
                    if let Some(label) = r.child().and_downcast::<gtk4::Label>() {
                        label.set_css_classes(&[]);
                    }
                }
            }
            row.add_css_class("active-workspace");
            if let Some(label) = row.child().and_downcast::<gtk4::Label>() {
                label.add_css_class("active-workspace-label");
            }
        }
        // Grab GTK keyboard focus on the active pane so key events reach Ghostty.
        if let Some(engine) = self.split_engines.get(index) {
            engine.grab_active_focus();
        }
    }

    /// Switch to next workspace (wrap-around). Per D-10: Ctrl+].
    pub fn switch_next(&mut self) {
        if self.workspaces.is_empty() {
            return;
        }
        let next = (self.active_index + 1) % self.workspaces.len();
        self.switch_to_index(next);
    }

    /// Switch to previous workspace (wrap-around). Per D-10: Ctrl+[.
    pub fn switch_prev(&mut self) {
        if self.workspaces.is_empty() {
            return;
        }
        let prev = if self.active_index == 0 {
            self.workspaces.len() - 1
        } else {
            self.active_index - 1
        };
        self.switch_to_index(prev);
    }

    pub fn active_split_engine_mut(&mut self) -> Option<&mut SplitEngine> {
        self.split_engines.get_mut(self.active_index)
    }

    /// Rename the active workspace. Per D-03/D-10: Ctrl+Shift+R (UI wired in Plan 04/05).
    pub fn rename_active(&mut self, new_name: String) {
        if let Some(ws) = self.workspaces.get_mut(self.active_index) {
            ws.rename(new_name.clone());
            // Update the sidebar label.
            if let Some(row) = self.sidebar_list.row_at_index(self.active_index as i32) {
                if let Some(label) = row.child().and_downcast::<gtk4::Label>() {
                    label.set_text(&new_name);
                }
            }
            self.trigger_session_save();
        }
    }

    /// Returns the active workspace, if any.
    pub fn active_workspace(&self) -> Option<&Workspace> {
        self.workspaces.get(self.active_index)
    }

    /// Trigger a debounced session save. Call after any workspace/pane mutation.
    /// Snapshots SessionData on the main thread (safe for Rc) and sends to the
    /// tokio debounce task which handles the file I/O.
    pub fn trigger_session_save(&self) {
        if let Some(ref notify) = self.save_notify {
            // Snapshot session data on main thread where Rc<RefCell<AppState>> is safe.
            if let Some(ref tx) = self.session_tx {
                let session = crate::session::SessionData {
                    version: 1,
                    active_index: self.active_index,
                    workspaces: self.workspaces.iter().map(|ws| {
                        crate::session::WorkspaceSession {
                            uuid: ws.uuid.to_string(),
                            name: ws.name.clone(),
                            active_pane_uuid: None, // Phase 4: fill from split engine
                            layout: crate::split_engine::SplitNodeData::Leaf {
                                pane_id: 0,
                                surface_uuid: uuid::Uuid::nil(),
                                shell: String::new(),
                                cwd: String::new(),
                            },
                        }
                    }).collect(),
                };
                let _ = tx.send(session);
            }
            notify.notify_one();
        }
    }
}
