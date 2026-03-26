use uuid::Uuid;

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
    /// Stable UUID for session persistence and v2 socket protocol identity.
    pub uuid: Uuid,
    /// Phase 4 NOTF-01: true when any pane in this workspace has unread bell activity.
    pub has_attention: bool,
    /// Phase 4: rate-limit desktop notifications to 1 per workspace per 5 seconds.
    pub last_notification: Option<std::time::Instant>,
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
            uuid: Uuid::new_v4(),
            has_attention: false,
            last_notification: None,
        }
    }

    /// Rename this workspace to a new display name.
    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_new_has_uuid() {
        let w = Workspace::new(1, 1);
        // uuid must not be nil (all-zeros)
        assert_ne!(w.uuid, Uuid::nil(), "Workspace::new() must generate a non-nil UUID");
    }

    #[test]
    fn workspace_uuids_are_unique() {
        let w1 = Workspace::new(1, 1);
        let w2 = Workspace::new(2, 2);
        assert_ne!(w1.uuid, w2.uuid, "Two workspaces must have distinct UUIDs");
    }
}
