pub struct Workspace {
    pub id: u64,
    pub name: String,
    pub display_number: u32,
    pub stack_page_name: String,
}

impl Workspace {
    pub fn new(id: u64, display_number: u32) -> Self {
        // Dummy implementation
        Self {
            id,
            name: "".to_string(),
            display_number,
            stack_page_name: "".to_string(),
        }
    }

    pub fn rename(&mut self, name: String) {
        // Dummy implementation
        self.name = name;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // WS-01: Can create a workspace with a default name
    #[test]
    fn test_workspace_default_name() {
        let ws = Workspace::new(1, 1);
        assert_eq!(ws.name, "Workspace 1");
        assert_eq!(ws.id, 1);
        assert_eq!(ws.display_number, 1);
    }

    // WS-01: Second workspace gets incremented display number
    #[test]
    fn test_workspace_default_name_increments() {
        let ws1 = Workspace::new(1, 1);
        let ws2 = Workspace::new(2, 2);
        assert_eq!(ws1.name, "Workspace 1");
        assert_eq!(ws2.name, "Workspace 2");
    }

    // WS-04: Can rename a workspace
    #[test]
    fn test_workspace_rename() {
        let mut ws = Workspace::new(1, 1);
        ws.rename("My Terminal".to_string());
        assert_eq!(ws.name, "My Terminal");
        // id and display_number unchanged
        assert_eq!(ws.id, 1);
    }

    // WS-06: stack_page_name is derived from workspace id
    #[test]
    fn test_workspace_stack_page_name() {
        let ws = Workspace::new(42, 3);
        assert_eq!(ws.stack_page_name, "workspace-42");
    }

    // WS-02: Workspace with id 0 is a valid state (close-last guard is in AppState)
    #[test]
    fn test_workspace_new_id_zero() {
        let ws = Workspace::new(0, 1);
        assert_eq!(ws.id, 0);
    }
}
