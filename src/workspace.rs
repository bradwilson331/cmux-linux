/// Workspace: one tab in the cmux sidebar.
/// Each workspace has an independent pane split tree (managed by SplitEngine in split_engine.rs).
/// The root GTK widget of a workspace's split tree is added as a named page in the GtkStack.
#[derive(Debug)]
pub struct Workspace {
    /// Unique workspace ID — used as the GtkStack page name.
    pub id: u64,
    /// Display name shown in the sidebar GtkListBox row.
    pub name: String,
    /// The name key used with GtkStack::add_named / set_visible_child_name.
    pub stack_page_name: String,
    /// Sequential number used for default naming ("Workspace N").
    /// Preserved even after renames so we don't reuse numbers.
    pub display_number: usize,
}

impl Workspace {
    /// Create a new workspace with a default "Workspace N" name.
    pub fn new(id: u64, display_number: usize) -> Self {
        let name = format!("Workspace {}", display_number);
        let stack_page_name = format!("workspace-{}", id);
        Self {
            id,
            name,
            stack_page_name,
            display_number,
        }
    }

    /// Rename this workspace to a new display name.
    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }
}
