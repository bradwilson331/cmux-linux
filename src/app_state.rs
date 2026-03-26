use crate::ghostty::ffi;
use crate::split_engine::SplitEngine;
use crate::workspace::{ConnectionState, Workspace};
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
    /// Sender for SSH events (cloned into SSH lifecycle tokio tasks).
    pub ssh_event_tx: Option<crate::ssh::SshEventTx>,
    /// Tokio runtime handle for spawning SSH lifecycle tasks.
    pub runtime_handle: Option<tokio::runtime::Handle>,
    /// Handles to SSH lifecycle tasks, keyed by workspace id. Used for cleanup on close.
    pub ssh_task_handles: std::collections::HashMap<u64, tokio::task::JoinHandle<()>>,
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
            ssh_event_tx: None,
            runtime_handle: None,
            ssh_task_handles: std::collections::HashMap::new(),
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

        // Phase 4: Row layout: GtkBox(H, 4) > [GtkBox(V, 0) > [GtkLabel(name)], GtkLabel(dot)]
        // VBox allows Plan 04 to add a connection-state subtitle without restructuring.
        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let label = gtk4::Label::new(Some(&name));
        label.set_halign(gtk4::Align::Start);
        label.set_hexpand(true);
        vbox.append(&label);
        vbox.set_hexpand(true);
        hbox.append(&vbox);

        // Attention dot — hidden by default, shown when has_attention
        let dot = gtk4::Label::new(None);
        dot.add_css_class("attention-dot");
        dot.set_visible(false);
        hbox.append(&dot);

        let row = gtk4::ListBoxRow::new();
        row.set_child(Some(&hbox));
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

    /// Build a sidebar row for a workspace. Used by create_workspace and create_remote_workspace.
    fn build_sidebar_row(&self, workspace: &Workspace) -> gtk4::ListBoxRow {
        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let label = gtk4::Label::new(Some(&workspace.name));
        label.set_halign(gtk4::Align::Start);
        label.set_hexpand(true);
        vbox.append(&label);

        // SSH connection state subtitle (only for remote workspaces)
        if workspace.connection_state.is_remote() {
            let status = gtk4::Label::new(Some(workspace.connection_state.display_text()));
            status.set_halign(gtk4::Align::Start);
            status.add_css_class("connection-state");
            status.add_css_class(workspace.connection_state.css_class());
            vbox.append(&status);
        }
        vbox.set_hexpand(true);
        hbox.append(&vbox);

        // Attention dot — hidden by default, shown when has_attention
        let dot = gtk4::Label::new(None);
        dot.add_css_class("attention-dot");
        dot.set_visible(false);
        hbox.append(&dot);

        let row = gtk4::ListBoxRow::new();
        row.set_child(Some(&hbox));
        unsafe {
            row.set_data("workspace-id", workspace.id);
        }
        row
    }

    /// Create a remote SSH workspace. Returns workspace id.
    pub fn create_remote_workspace(&mut self, target: String) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let display_number = self.next_display_number;
        self.next_display_number += 1;

        let workspace = Workspace::new_remote(id, display_number, target);
        let row = self.build_sidebar_row(&workspace);
        self.sidebar_list.append(&row);

        // Create surface and split engine (same as local)
        let pane_id = id * 1000;
        let (gl_area, surface_cell) =
            crate::ghostty::surface::create_surface(&self.gtk_app, self.ghostty_app, None, pane_id);
        let engine = SplitEngine::new(
            self.gtk_app.clone(),
            self.ghostty_app,
            gl_area,
            surface_cell,
            pane_id,
        );

        let page_name = workspace.stack_page_name.clone();
        self.stack
            .add_named(&engine.root_widget(), Some(&page_name));

        self.workspaces.push(workspace);
        self.split_engines.push(engine);

        let new_index = self.workspaces.len() - 1;
        self.switch_to_index(new_index);
        self.trigger_session_save();
        id
    }

    /// Update the connection state of a workspace and refresh its sidebar row.
    pub fn update_connection_state(&mut self, workspace_id: u64, state: ConnectionState) {
        if let Some(idx) = self.workspaces.iter().position(|ws| ws.id == workspace_id) {
            self.workspaces[idx].connection_state = state.clone();
            // Update sidebar subtitle
            if let Some(row) = self.sidebar_list.row_at_index(idx as i32) {
                if let Some(hbox) = row.child().and_downcast::<gtk4::Box>() {
                    if let Some(vbox) = hbox.first_child().and_downcast::<gtk4::Box>() {
                        // Last child in vbox is the status label (if it has connection-state class)
                        if let Some(status) = vbox.last_child().and_downcast::<gtk4::Label>() {
                            if status.has_css_class("connection-state") {
                                status.set_text(state.display_text());
                                status.remove_css_class("connected");
                                status.remove_css_class("disconnected");
                                status.remove_css_class("reconnecting");
                                status.add_css_class(state.css_class());
                            }
                        }
                    }
                }
            }
        }
    }

    /// Close the workspace at `index`. Removes the sidebar row and GtkStack page.
    /// Returns false if there is only one workspace (cannot close the last one).
    /// The caller (Plan 04) is responsible for calling ghostty_surface_free on all panes first.
    pub fn close_workspace(&mut self, index: usize) -> bool {
        if self.workspaces.len() <= 1 {
            return false; // Cannot close the last workspace
        }

        // Abort SSH lifecycle task if this is a remote workspace.
        if let Some(ws) = self.workspaces.get(index) {
            if let Some(handle) = self.ssh_task_handles.remove(&ws.id) {
                handle.abort();
            }
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
        // Phase 4: clear attention when user switches to a workspace (D-05).
        self.clear_workspace_attention(index);
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
                    // Phase 4: navigate nested layout: row > hbox > vbox > label
                    if let Some(hbox) = r.child().and_downcast::<gtk4::Box>() {
                        if let Some(vbox) = hbox.first_child().and_downcast::<gtk4::Box>() {
                            if let Some(label) = vbox.first_child().and_downcast::<gtk4::Label>() {
                                label.set_css_classes(&[]);
                            }
                        }
                    }
                }
            }
            row.add_css_class("active-workspace");
            // Phase 4: navigate nested layout: row > hbox > vbox > label
            if let Some(hbox) = row.child().and_downcast::<gtk4::Box>() {
                if let Some(vbox) = hbox.first_child().and_downcast::<gtk4::Box>() {
                    if let Some(label) = vbox.first_child().and_downcast::<gtk4::Label>() {
                        label.add_css_class("active-workspace-label");
                    }
                }
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
            // Update the sidebar label (Phase 4 nested layout: row > hbox > vbox > label).
            if let Some(row) = self.sidebar_list.row_at_index(self.active_index as i32) {
                if let Some(hbox) = row.child().and_downcast::<gtk4::Box>() {
                    if let Some(vbox) = hbox.first_child().and_downcast::<gtk4::Box>() {
                        if let Some(label) = vbox.first_child().and_downcast::<gtk4::Label>() {
                            label.set_text(&new_name);
                        }
                    }
                }
            }
            self.trigger_session_save();
        }
    }

    /// Returns the active workspace, if any.
    pub fn active_workspace(&self) -> Option<&Workspace> {
        self.workspaces.get(self.active_index)
    }

    /// Set attention on a specific pane. Called from bell handler.
    /// Updates workspace has_attention and sidebar dot.
    pub fn set_pane_attention(&mut self, pane_id: u64) {
        for (idx, engine) in self.split_engines.iter_mut().enumerate() {
            if engine.root.set_attention(pane_id, true) {
                self.workspaces[idx].has_attention = engine.root.any_attention();
                self.update_sidebar_attention(idx);

                // Desktop notification when window is unfocused (NOTF-03)
                let window_focused = self.gtk_app.active_window()
                    .map(|w| w.is_active())
                    .unwrap_or(false);
                if !window_focused && self.workspaces[idx].has_attention {
                    let should_notify = self.workspaces[idx].last_notification
                        .map(|t| t.elapsed() >= std::time::Duration::from_secs(5))
                        .unwrap_or(true);
                    if should_notify {
                        self.workspaces[idx].last_notification = Some(std::time::Instant::now());
                        send_bell_notification(&self.gtk_app, &self.workspaces[idx].name, idx);
                    }
                }
                break;
            }
        }
    }

    /// Clear all attention in the workspace at `index`.
    pub fn clear_workspace_attention(&mut self, index: usize) {
        if let Some(engine) = self.split_engines.get_mut(index) {
            engine.root.clear_all_attention();
        }
        if let Some(ws) = self.workspaces.get_mut(index) {
            ws.has_attention = false;
        }
        self.update_sidebar_attention(index);
    }

    /// Update the sidebar dot visibility for workspace at `index`.
    fn update_sidebar_attention(&self, index: usize) {
        if let Some(row) = self.sidebar_list.row_at_index(index as i32) {
            let has_attention = self.workspaces.get(index)
                .map(|ws| ws.has_attention)
                .unwrap_or(false);
            // Row layout: GtkBox(H) > [GtkBox(V) > [GtkLabel(name)], GtkLabel(dot)]
            if let Some(hbox) = row.child().and_downcast::<gtk4::Box>() {
                if let Some(dot) = hbox.last_child() {
                    dot.set_visible(has_attention);
                }
            }
        }
    }

    /// Trigger a debounced session save. Call after any workspace/pane mutation.
    /// Snapshots SessionData on the main thread (safe for Rc) and sends to the
    /// tokio debounce task which handles the file I/O.
    pub fn trigger_session_save(&self) {
        if let Some(ref notify) = self.save_notify {
            // Snapshot session data on main thread where Rc<RefCell<AppState>> is safe.
            if let Some(ref tx) = self.session_tx {
                let session = crate::session::SessionData {
                    version: 2, // D-01: bump to version 2 for full tree serialization
                    active_index: self.active_index,
                    workspaces: self.workspaces.iter().enumerate().map(|(i, ws)| {
                        // D-02: save full split tree for ALL workspaces
                        let layout = if i < self.split_engines.len() {
                            self.split_engines[i].root.to_data()
                        } else {
                            // Fallback: shouldn't happen, but be safe
                            crate::split_engine::SplitNodeData::Leaf {
                                pane_id: 0,
                                surface_uuid: uuid::Uuid::nil(),
                                shell: String::new(),
                                cwd: String::new(),
                            }
                        };
                        // D-04: save active_pane_uuid per workspace
                        let active_pane_uuid = if i < self.split_engines.len() {
                            self.split_engines[i].active_pane_uuid()
                        } else {
                            None
                        };
                        crate::session::WorkspaceSession {
                            uuid: ws.uuid.to_string(),
                            name: ws.name.clone(),
                            active_pane_uuid,
                            layout,
                        }
                    }).collect(),
                };
                let _ = tx.send(session);
            }
            notify.notify_one();
        }
    }
}

/// Send a desktop notification for a bell in the given workspace.
/// Uses GNotification via gio. Rate limiting is handled by the caller.
fn send_bell_notification(app: &gtk4::Application, workspace_name: &str, workspace_index: usize) {
    use gtk4::gio;
    let notification = gio::Notification::new("Terminal Bell");
    notification.set_body(Some(&format!("{} - Terminal bell", workspace_name)));
    notification.set_priority(gio::NotificationPriority::Normal);
    let notif_id = format!("bell-{}", workspace_index);
    app.send_notification(Some(&notif_id), &notification);
}
