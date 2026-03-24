use crate::ghostty::ffi;
use crate::workspace::Workspace;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub type AppStateRef = Rc<RefCell<AppState>>;

pub struct AppState {
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
}

impl AppState {
    /// Create a new AppState. Does NOT create the first workspace — caller must call
    /// create_workspace() after constructing the GTK widget tree (Plan 04 wires this).
    pub fn new(
        stack: gtk4::Stack,
        sidebar_list: gtk4::ListBox,
        ghostty_app: ffi::ghostty_app_t,
    ) -> AppStateRef {
        let state = AppState {
            workspaces: Vec::new(),
            active_index: 0,
            stack,
            sidebar_list,
            ghostty_app,
            next_id: 1,
            next_display_number: 1,
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

        let workspace = Workspace::new(id, display_number);
        let name = workspace.name.clone();

        // Append a row to the sidebar GtkListBox.
        let label = gtk4::Label::new(Some(&name));
        label.set_halign(gtk4::Align::Start);
        let row = gtk4::ListBoxRow::new();
        row.set_child(Some(&label));
        // Store workspace id on the row for click-to-switch routing (Plan 04).
        unsafe {
            row.set_data("workspace-id", id);
        }
        self.sidebar_list.append(&row);

        self.workspaces.push(workspace);
        let new_index = self.workspaces.len() - 1;

        // Switch to the new workspace in the sidebar.
        // The GtkStack page is added by Plan 04 (requires the GLArea root widget).
        self.active_index = new_index;
        if let Some(row) = self.sidebar_list.row_at_index(new_index as i32) {
            self.sidebar_list.select_row(Some(&row));
        }

        id
    }

    /// Close the workspace at `index`. Removes the sidebar row and GtkStack page.
    /// Returns false if there is only one workspace (cannot close the last one).
    /// The caller (Plan 04) is responsible for calling ghostty_surface_free on all panes first.
    pub fn close_workspace(&mut self, index: usize) -> bool {
        if self.workspaces.len() <= 1 {
            return false; // Cannot close the last workspace
        }

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
        }
    }

    /// Returns the active workspace, if any.
    pub fn active_workspace(&self) -> Option<&Workspace> {
        self.workspaces.get(self.active_index)
    }
}
