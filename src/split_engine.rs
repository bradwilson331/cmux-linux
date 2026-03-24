use gtk4::prelude::*;
use gtk4::{GLArea, Orientation, Paned};

pub enum SplitNode {
    Leaf {
        pane_id: u64,
        gl_area: GLArea,
        surface: *mut std::ffi::c_void,
    },
    Split {
        orientation: Orientation,
        paned: Paned,
        start: Box<SplitNode>,
        end: Box<SplitNode>,
    },
}

impl SplitNode {
    pub fn widget(&self) -> gtk4::Widget {
        match self {
            SplitNode::Leaf { gl_area, .. } => gl_area.clone().upcast(),
            SplitNode::Split { paned, .. } => paned.clone().upcast(),
        }
    }

    pub fn collect_pane_ids(&self, _ids: &mut Vec<u64>) {
        // dummy
    }

    pub fn find_active_pane_id(&self) -> Option<u64> {
        // dummy
        None
    }
}

pub struct SplitEngine;

impl SplitEngine {
    pub fn close_active(&mut self) -> Option<u64> {
        // dummy
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal helper: create a Leaf SplitNode without a real GTK GLArea.
    /// Tests that only check tree structure can use gtk4::GLArea::new() if GTK is
    /// initialized, or skip GTK-dependent methods. For pure tree structure tests,
    /// we test SplitNode enum variants directly.
    fn make_leaf(pane_id: u64) -> SplitNode {
        // gtk4 tests require GTK init. Use gtk4::init() if available.
        // If GTK is not available in test environment, these tests are skipped.
        SplitNode::Leaf {
            pane_id,
            gl_area: gtk4::GLArea::new(),
            surface: std::ptr::null_mut(),
        }
    }

    // SPLIT-06: SplitNode::Leaf holds a pane_id
    #[test]
    fn test_split_node_leaf_pane_id() {
        if gtk4::init().is_err() {
            return;
        } // skip if no display
        let leaf = make_leaf(42);
        if let SplitNode::Leaf { pane_id, .. } = leaf {
            assert_eq!(pane_id, 42);
        } else {
            panic!("expected Leaf variant");
        }
    }

    // SPLIT-06: collect_pane_ids on a Leaf returns the single pane_id
    #[test]
    fn test_collect_pane_ids_leaf() {
        if gtk4::init().is_err() {
            return;
        }
        let leaf = make_leaf(7);
        let mut ids = Vec::new();
        leaf.collect_pane_ids(&mut ids);
        assert_eq!(ids, vec![7]);
    }

    // SPLIT-06: collect_pane_ids on a Split returns both children
    #[test]
    fn test_collect_pane_ids_split() {
        if gtk4::init().is_err() {
            return;
        }
        let left = make_leaf(1);
        let right = make_leaf(2);
        let left_widget = left.widget();
        let right_widget = right.widget();
        let paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        paned.set_start_child(Some(&left_widget));
        paned.set_end_child(Some(&right_widget));
        let split = SplitNode::Split {
            orientation: gtk4::Orientation::Horizontal,
            paned,
            start: Box::new(left),
            end: Box::new(right),
        };
        let mut ids = Vec::new();
        split.collect_pane_ids(&mut ids);
        assert_eq!(ids, vec![1, 2]);
    }

    // SPLIT-01/SPLIT-02: find_active_pane_id returns None when no active-pane CSS class
    #[test]
    fn test_find_active_pane_id_none_without_css() {
        if gtk4::init().is_err() {
            return;
        }
        let leaf = make_leaf(5);
        // No CSS class added — find_active_pane_id should return None
        assert_eq!(leaf.find_active_pane_id(), None);
    }

    // SPLIT-01: find_active_pane_id returns Some when active-pane CSS class is set
    #[test]
    fn test_find_active_pane_id_with_css() {
        if gtk4::init().is_err() {
            return;
        }
        let leaf = make_leaf(5);
        if let SplitNode::Leaf { ref gl_area, .. } = leaf {
            gl_area.add_css_class("active-pane");
        }
        assert_eq!(leaf.find_active_pane_id(), Some(5));
    }

    // SPLIT-05: close_active on a single-leaf SplitEngine returns None (last pane)
    // This is a contract test — not yet wired to GTK Application. Stub only.
    #[test]
    fn test_close_active_last_pane_returns_none() {
        // Full test requires SplitEngine with GTK Application — tested via integration
        // in Plan 02-05. This stub documents the contract: close_active() on the last
        // pane must return None to signal AppState to close the workspace.
        // Verify: SplitEngine::close_active return type is Option<u64>.
        // Compile-time check: if close_active returns (), this test file won't compile.
        let _: fn(&mut SplitEngine) -> Option<u64> = SplitEngine::close_active;
    }
}
