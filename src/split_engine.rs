use crate::ghostty::ffi;
use gtk4::prelude::*;

/// Direction for pane focus navigation (Ctrl+Alt+arrows per D-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Recursive pane layout tree. Each workspace has one root SplitNode.
/// - Leaf: a single terminal pane (GtkGLArea + Ghostty surface)
/// - Split: two child subtrees separated by a GtkPaned divider
///
/// Per SPLIT-06: this is the Bonsplit Rust port — immutable-style tree where
/// split/close operations return a new root.
#[derive(Clone)]
pub enum SplitNode {
    Leaf {
        pane_id: u64,
        gl_area: gtk4::GLArea,
        surface: ffi::ghostty_surface_t,
    },
    Split {
        orientation: gtk4::Orientation,
        paned: gtk4::Paned,
        start: Box<SplitNode>,
        end: Box<SplitNode>,
    },
}

impl SplitNode {
    /// Returns the root GTK widget for this node (GLArea for Leaf, Paned for Split).
    pub fn widget(&self) -> gtk4::Widget {
        match self {
            SplitNode::Leaf { gl_area, .. } => gl_area.clone().upcast(),
            SplitNode::Split { paned, .. } => paned.clone().upcast(),
        }
    }

    /// Find the pane_id of the active (focused) leaf by checking CSS class.
    pub fn find_active_pane_id(&self) -> Option<u64> {
        match self {
            SplitNode::Leaf {
                pane_id, gl_area, ..
            } => {
                if gl_area.has_css_class("active-pane") {
                    Some(*pane_id)
                } else {
                    None
                }
            }
            SplitNode::Split { start, end, .. } => start
                .find_active_pane_id()
                .or_else(|| end.find_active_pane_id()),
        }
    }

    /// Apply the active-pane CSS class to the leaf matching active_pane_id.
    /// Removes the class from all other leaves.
    pub fn update_focus_css(&self, active_pane_id: u64) {
        match self {
            SplitNode::Leaf {
                pane_id, gl_area, ..
            } => {
                if *pane_id == active_pane_id {
                    gl_area.add_css_class("active-pane");
                } else {
                    gl_area.remove_css_class("active-pane");
                }
            }
            SplitNode::Split { start, end, .. } => {
                start.update_focus_css(active_pane_id);
                end.update_focus_css(active_pane_id);
            }
        }
    }

    /// Collect all leaf pane_ids into a Vec (for cleanup on workspace close).
    pub fn collect_pane_ids(&self, out: &mut Vec<u64>) {
        match self {
            SplitNode::Leaf { pane_id, .. } => out.push(*pane_id),
            SplitNode::Split { start, end, .. } => {
                start.collect_pane_ids(out);
                end.collect_pane_ids(out);
            }
        }
    }

    /// Collect all surfaces into a Vec (for ghostty_surface_free on workspace close).
    pub fn collect_surfaces(&self, out: &mut Vec<ffi::ghostty_surface_t>) {
        match self {
            SplitNode::Leaf { surface, .. } => out.push(*surface),
            SplitNode::Split { start, end, .. } => {
                start.collect_surfaces(out);
                end.collect_surfaces(out);
            }
        }
    }
}

/// SplitEngine manages one workspace's pane layout tree.
pub struct SplitEngine {
    pub root: SplitNode,
    pub active_pane_id: u64,
    /// Monotonically increasing pane ID counter.
    next_pane_id: u64,
    /// GTK Application handle needed to create new GLAreas.
    app: gtk4::Application,
    /// Ghostty app handle needed to create new surfaces.
    ghostty_app: ffi::ghostty_app_t,
}

impl SplitEngine {
    /// Create a new SplitEngine with a single leaf pane.
    /// The initial GLArea and surface are created by the caller (Plan 04) and passed in.
    pub fn new(
        app: gtk4::Application,
        ghostty_app: ffi::ghostty_app_t,
        initial_gl_area: gtk4::GLArea,
        initial_surface_cell: std::rc::Rc<std::cell::RefCell<Option<ffi::ghostty_surface_t>>>,
        pane_id: u64,
    ) -> Self {
        // The initial surface may not be realized yet. SplitEngine stores the cell
        // so it can read the surface pointer after realize. For focus/split operations
        // that run after realize, we read from the cell.
        // For the tree structure, we store null initially and update after realize.
        let surface_placeholder: ffi::ghostty_surface_t = std::ptr::null_mut();
        let root = SplitNode::Leaf {
            pane_id,
            gl_area: initial_gl_area,
            surface: surface_placeholder,
        };
        SplitEngine {
            root,
            active_pane_id: pane_id,
            next_pane_id: pane_id + 1,
            app,
            ghostty_app,
        }
    }

    /// Update the surface pointer for the initial leaf after realize.
    /// Called by Plan 04 in the GLArea realize callback.
    pub fn set_initial_surface(&mut self, pane_id: u64, surface: ffi::ghostty_surface_t) {
        if let SplitNode::Leaf {
            pane_id: id,
            surface: s,
            ..
        } = &mut self.root
        {
            if *id == pane_id {
                *s = surface;
            }
        }
    }

    /// Returns the root widget of this workspace's split tree.
    pub fn root_widget(&self) -> gtk4::Widget {
        self.root.widget()
    }

    /// Grab GTK keyboard focus for the active pane's GLArea.
    /// Called after workspace switch so key events route to Ghostty, not the sidebar.
    pub fn grab_active_focus(&self) {
        if let Some(gl_area) = self.find_gl_area(self.active_pane_id) {
            gl_area.grab_focus();
        }
    }

    /// Split the active pane to the right (Ctrl+D per D-10).
    /// Replaces the active Leaf with a Split(Horizontal) containing the old leaf + new leaf.
    /// Per D-08: new surface inherits CWD via ghostty_surface_inherited_config.
    /// Per D-09: initial split ratio is 50/50 (set in paned.connect_realize).
    /// Per SPLIT-07: new pane receives focus immediately.
    pub fn split_right(&mut self) -> Option<u64> {
        self.split_active(gtk4::Orientation::Horizontal)
    }

    /// Split the active pane downward (Ctrl+Shift+D per D-10).
    pub fn split_down(&mut self) -> Option<u64> {
        self.split_active(gtk4::Orientation::Vertical)
    }

    fn split_active(&mut self, orientation: gtk4::Orientation) -> Option<u64> {
        let active_id = self.active_pane_id;
        let new_pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        // Find the active leaf's surface for inherited config.
        let inherited_surface = self.find_surface(active_id)?;

        // Get inherited config from the active surface (for CWD inheritance per D-08).
        let inherited = unsafe {
            let mut inherited_config = ffi::ghostty_surface_inherited_config(
                inherited_surface,
                ffi::ghostty_surface_context_e_GHOSTTY_SURFACE_CONTEXT_SPLIT,
            );
            Some(&mut inherited_config as *mut _)
        };

        // Create new GLArea + surface for the new pane.
        let (new_gl_area, _surface_cell) = crate::ghostty::surface::create_surface(
            &self.app,
            self.ghostty_app,
            inherited,
            new_pane_id,
        );

        // Replace the active leaf in the tree with a Split node.
        let new_surface_placeholder: ffi::ghostty_surface_t = std::ptr::null_mut();
        let new_leaf = SplitNode::Leaf {
            pane_id: new_pane_id,
            gl_area: new_gl_area.clone(),
            surface: new_surface_placeholder, // updated after realize via SURFACE_REGISTRY
        };

        let _replaced = self.replace_leaf_with_split(active_id, new_leaf, orientation)?;

        // After realize, update active focus to the new pane.
        self.active_pane_id = new_pane_id;
        self.root.update_focus_css(new_pane_id);

        // Focus the new GLArea widget so it receives keyboard events.
        new_gl_area.grab_focus();

        Some(new_pane_id)
    }

    /// Replace the leaf with `target_pane_id` with a Split(orientation) node.
    /// Returns Some(()) on success, None if the leaf was not found.
    fn replace_leaf_with_split(
        &mut self,
        target_pane_id: u64,
        new_leaf: SplitNode,
        orientation: gtk4::Orientation,
    ) -> Option<()> {
        let orientation_cap = orientation;
        let mut replacer = Some(|old_leaf: SplitNode| {
            let old_widget = old_leaf.widget();
            let new_widget = new_leaf.widget();

            let paned = gtk4::Paned::new(orientation_cap);
            paned.set_start_child(Some(&old_widget));
            paned.set_end_child(Some(&new_widget));

            // Set 50/50 position after realize (per D-09 and RESEARCH Pitfall 2).
            paned.connect_realize(move |p| {
                let size = if orientation_cap == gtk4::Orientation::Horizontal {
                    p.width()
                } else {
                    p.height()
                };
                if size > 0 {
                    p.set_position(size / 2);
                }
            });

            SplitNode::Split {
                orientation: orientation_cap,
                paned: paned.clone(),
                start: Box::new(old_leaf),
                end: Box::new(new_leaf),
            }
        });
        replace_in_tree(&mut self.root, target_pane_id, &mut replacer)
    }

    /// Close the active pane (Ctrl+Shift+X per UI-SPEC).
    /// Removes the active leaf, replaces its parent Split with the surviving sibling.
    /// Returns the new active pane_id, or None if this was the last pane.
    pub fn close_active(&mut self) -> Option<u64> {
        let active_id = self.active_pane_id;

        // Cannot close the last pane — workspace close is handled at AppState level.
        if matches!(&self.root, SplitNode::Leaf { pane_id, .. } if *pane_id == active_id) {
            return None; // Signal to AppState: close the workspace instead
        }

        // Find the surface before removing it from the tree.
        let surface_to_free = self.find_surface(active_id)?;

        // Remove the leaf from the tree and get the surviving sibling's pane_id.
        let surviving_id = remove_leaf_from_tree(&mut self.root, active_id)?;

        // Deregister from SURFACE_REGISTRY and free the surface.
        unsafe {
            ffi::ghostty_surface_free(surface_to_free);
        }
        if let Ok(mut registry) = crate::ghostty::callbacks::SURFACE_REGISTRY.lock() {
            registry.remove(&(surface_to_free as usize));
        }

        // Update focus to the surviving pane.
        self.active_pane_id = surviving_id;
        self.root.update_focus_css(surviving_id);

        // Call ghostty_surface_set_focus on the surviving surface (SPLIT-07).
        if let Some(surface) = self.find_surface(surviving_id) {
            unsafe {
                ffi::ghostty_surface_set_focus(surface, true);
            }
        }

        // Grab GTK focus on the surviving GLArea.
        if let Some(gl_area) = self.find_gl_area(surviving_id) {
            gl_area.grab_focus();
        }

        Some(surviving_id)
    }

    /// Navigate focus to the pane adjacent in `direction` (Ctrl+Alt+arrows per D-10).
    pub fn focus_next_in_direction(&mut self, direction: FocusDirection) -> bool {
        let active_id = self.active_pane_id;
        if let Some(new_id) = find_adjacent(&self.root, active_id, direction) {
            // Unfocus old surface.
            if let Some(old_surface) = self.find_surface(active_id) {
                unsafe {
                    ffi::ghostty_surface_set_focus(old_surface, false);
                }
            }
            self.active_pane_id = new_id;
            self.root.update_focus_css(new_id);
            // Focus new surface (SPLIT-07).
            if let Some(new_surface) = self.find_surface(new_id) {
                unsafe {
                    ffi::ghostty_surface_set_focus(new_surface, true);
                }
            }
            if let Some(gl_area) = self.find_gl_area(new_id) {
                gl_area.grab_focus();
            }
            true
        } else {
            false
        }
    }

    /// Update the surface pointer for a pane after its GLArea realize callback fires.
    /// Called by Plan 04 wiring after SURFACE_REGISTRY is populated.
    pub fn update_surface(&mut self, pane_id: u64, surface: ffi::ghostty_surface_t) {
        update_surface_in_tree(&mut self.root, pane_id, surface);
    }

    fn find_surface(&self, pane_id: u64) -> Option<ffi::ghostty_surface_t> {
        find_surface_in_tree(&self.root, pane_id).or_else(|| {
            // Fallback: look up in global SURFACE_REGISTRY by scanning for pane_id.
            // SURFACE_REGISTRY maps surface_ptr (usize) → pane_id; need reverse lookup.
            if let Ok(reg) = crate::ghostty::callbacks::SURFACE_REGISTRY.lock() {
                reg.iter()
                    .find(|(_, &pid)| pid == pane_id)
                    .map(|(&ptr, _)| ptr as ffi::ghostty_surface_t)
            } else {
                None
            }
        })
    }

    fn find_gl_area(&self, pane_id: u64) -> Option<gtk4::GLArea> {
        find_gl_area_in_tree(&self.root, pane_id)
    }
}

// ── Tree traversal helpers ───────────────────────────────────────────────────

/// Replace the leaf with `target_id` using `replacer` function. Returns Some(()) if found.
fn replace_in_tree<F>(node: &mut SplitNode, target_id: u64, replacer: &mut Option<F>) -> Option<()>
where
    F: FnOnce(SplitNode) -> SplitNode,
{
    match node {
        SplitNode::Leaf { pane_id, .. } if *pane_id == target_id => {
            if let Some(r) = replacer.take() {
                // Take ownership of the old node to pass to replacer.
                let old = std::mem::replace(
                    node,
                    SplitNode::Leaf {
                        pane_id: 0,
                        gl_area: gtk4::GLArea::new(),
                        surface: std::ptr::null_mut(),
                    },
                );
                *node = r(old);
                Some(())
            } else {
                None
            }
        }
        SplitNode::Leaf { .. } => None,
        SplitNode::Split {
            start, end, paned, ..
        } => {
            if let Some(()) = replace_in_tree(start, target_id, replacer) {
                // Update paned start child to new widget.
                paned.set_start_child(Some(&start.widget()));
                Some(())
            } else if let Some(()) = replace_in_tree(end, target_id, replacer) {
                paned.set_end_child(Some(&end.widget()));
                Some(())
            } else {
                None
            }
        }
    }
}

/// Remove leaf `target_id` from the tree. Returns the surviving sibling's pane_id.
/// Replaces the parent Split with the surviving sibling in the GTK widget tree.
fn remove_leaf_from_tree(node: &mut SplitNode, target_id: u64) -> Option<u64> {
    match node {
        SplitNode::Leaf { .. } => None, // Caller ensures we never remove the root leaf
        SplitNode::Split {
            start, end, paned, ..
        } => {
            // Check if start is the target leaf.
            if let SplitNode::Leaf { pane_id, .. } = start.as_ref() {
                if *pane_id == target_id {
                    // Surviving sibling is end. Replace this Split with end in the GTK tree.
                    let surviving = *end.clone();
                    let surviving_widget = surviving.widget();
                    // Find the paned's parent and replace it with the surviving widget.
                    if let Some(parent) = paned.parent() {
                        replace_child_in_parent(
                            &parent,
                            &paned.clone().upcast(),
                            &surviving_widget,
                        );
                    }
                    let surviving_id = first_pane_id(&surviving);
                    *node = surviving;
                    return Some(surviving_id);
                }
            }
            // Check if end is the target leaf.
            if let SplitNode::Leaf { pane_id, .. } = end.as_ref() {
                if *pane_id == target_id {
                    let surviving = *start.clone();
                    let surviving_widget = surviving.widget();
                    if let Some(parent) = paned.parent() {
                        replace_child_in_parent(
                            &parent,
                            &paned.clone().upcast(),
                            &surviving_widget,
                        );
                    }
                    let surviving_id = first_pane_id(&surviving);
                    *node = surviving;
                    return Some(surviving_id);
                }
            }
            // Recurse into start subtree.
            if let Some(id) = remove_leaf_from_tree(start, target_id) {
                paned.set_start_child(Some(&start.widget()));
                return Some(id);
            }
            // Recurse into end subtree.
            if let Some(id) = remove_leaf_from_tree(end, target_id) {
                paned.set_end_child(Some(&end.widget()));
                return Some(id);
            }
            None
        }
    }
}

/// Replace `old_widget` with `new_widget` in `parent`. Handles GtkPaned children and GtkStack pages.
fn replace_child_in_parent(
    parent: &gtk4::Widget,
    old_widget: &gtk4::Widget,
    new_widget: &gtk4::Widget,
) {
    if let Some(paned) = parent.downcast_ref::<gtk4::Paned>() {
        if paned
            .start_child()
            .as_ref()
            .map(|w| w == old_widget)
            .unwrap_or(false)
        {
            paned.set_start_child(Some(new_widget));
        } else {
            paned.set_end_child(Some(new_widget));
        }
    } else if let Some(stack) = parent.downcast_ref::<gtk4::Stack>() {
        // Find the page name for the old widget, add new widget with same name, remove old.
        let page = stack.page(old_widget);
        if let Some(name) = page.name() {
            let name_str = name.to_string();
            stack.remove(old_widget);
            stack.add_named(new_widget, Some(&name_str));
            stack.set_visible_child_name(&name_str);
        } else {
            stack.remove(old_widget);
        }
    }
    // If parent is something else, the widget swap is a no-op (should not happen in Phase 2).
}

/// Return the first (leftmost/topmost) pane_id in a subtree.
fn first_pane_id(node: &SplitNode) -> u64 {
    match node {
        SplitNode::Leaf { pane_id, .. } => *pane_id,
        SplitNode::Split { start, .. } => first_pane_id(start),
    }
}

fn find_surface_in_tree(node: &SplitNode, pane_id: u64) -> Option<ffi::ghostty_surface_t> {
    match node {
        SplitNode::Leaf {
            pane_id: id,
            surface,
            ..
        } if *id == pane_id => {
            if surface.is_null() {
                None
            } else {
                Some(*surface)
            }
        }
        SplitNode::Leaf { .. } => None,
        SplitNode::Split { start, end, .. } => {
            find_surface_in_tree(start, pane_id).or_else(|| find_surface_in_tree(end, pane_id))
        }
    }
}

fn find_gl_area_in_tree(node: &SplitNode, pane_id: u64) -> Option<gtk4::GLArea> {
    match node {
        SplitNode::Leaf {
            pane_id: id,
            gl_area,
            ..
        } if *id == pane_id => Some(gl_area.clone()),
        SplitNode::Leaf { .. } => None,
        SplitNode::Split { start, end, .. } => {
            find_gl_area_in_tree(start, pane_id).or_else(|| find_gl_area_in_tree(end, pane_id))
        }
    }
}

fn update_surface_in_tree(node: &mut SplitNode, pane_id: u64, surface: ffi::ghostty_surface_t) {
    match node {
        SplitNode::Leaf {
            pane_id: id,
            surface: s,
            ..
        } if *id == pane_id => *s = surface,
        SplitNode::Leaf { .. } => {}
        SplitNode::Split { start, end, .. } => {
            update_surface_in_tree(start, pane_id, surface);
            update_surface_in_tree(end, pane_id, surface);
        }
    }
}

/// Find the pane adjacent to `active_id` in `direction`.
/// Strategy: collect ordered leaf positions and find the neighbor.
/// This is a directional approximation: Left/Up = previous leaf, Right/Down = next leaf.
/// A full spatial algorithm (comparing widget coordinates) can be added in a future phase.
fn find_adjacent(root: &SplitNode, active_id: u64, direction: FocusDirection) -> Option<u64> {
    let mut leaves = Vec::new();
    collect_leaves_in_order(root, &mut leaves);
    let pos = leaves.iter().position(|&id| id == active_id)?;
    match direction {
        FocusDirection::Left | FocusDirection::Up => {
            if pos > 0 {
                Some(leaves[pos - 1])
            } else {
                None
            }
        }
        FocusDirection::Right | FocusDirection::Down => {
            if pos + 1 < leaves.len() {
                Some(leaves[pos + 1])
            } else {
                None
            }
        }
    }
}

fn collect_leaves_in_order(node: &SplitNode, out: &mut Vec<u64>) {
    match node {
        SplitNode::Leaf { pane_id, .. } => out.push(*pane_id),
        SplitNode::Split { start, end, .. } => {
            collect_leaves_in_order(start, out);
            collect_leaves_in_order(end, out);
        }
    }
}
