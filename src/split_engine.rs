use crate::ghostty::ffi;
use gtk4::prelude::*;
use uuid::Uuid;

/// Direction for pane focus navigation (Ctrl+Shift+arrows per D-10).
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
        /// Stable UUID for session persistence and v2 socket protocol pane identity.
        uuid: Uuid,
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

    /// Find the Ghostty surface handle for a specific pane by pane_id.
    /// Used by debug.type to send text to a specific pane's surface.
    pub fn find_surface_for_pane(&self, target_id: u64) -> Option<ffi::ghostty_surface_t> {
        match self {
            SplitNode::Leaf { pane_id, surface, .. } => {
                if *pane_id == target_id { Some(*surface) } else { None }
            }
            SplitNode::Split { start, end, .. } => {
                start.find_surface_for_pane(target_id)
                    .or_else(|| end.find_surface_for_pane(target_id))
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
            uuid: Uuid::new_v4(),
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

    /// Grab GTK keyboard focus AND notify Ghostty of focus for the active pane.
    /// Use this after any operation that may have moved focus away from the terminal
    /// (sidebar toggle, workspace switch, etc.). grab_active_focus() only handles the
    /// GTK side; this method ensures Ghostty's internal focused state is also updated.
    pub fn focus_active_surface(&self) {
        if let Some(gl_area) = self.find_gl_area(self.active_pane_id) {
            gl_area.grab_focus();
        }
        // Call ghostty_surface_set_focus(true) on the active surface via registry lookup.
        if let Ok(areas) = crate::ghostty::callbacks::GL_AREA_REGISTRY.lock() {
            if let Ok(gl_to_surface) = crate::ghostty::callbacks::GL_TO_SURFACE.lock() {
                for area_ptr in areas.iter() {
                    let area: gtk4::glib::translate::Borrowed<gtk4::GLArea> =
                        unsafe { gtk4::glib::translate::from_glib_borrow(area_ptr.0) };
                    if area.has_css_class("active-pane") {
                        if let Some(&surface_ptr) = gl_to_surface.get(&(area_ptr.0 as usize)) {
                            unsafe {
                                crate::ghostty::ffi::ghostty_surface_set_focus(
                                    surface_ptr as ffi::ghostty_surface_t,
                                    true,
                                );
                            }
                        }
                        break;
                    }
                }
            }
        }
        // Kick the render loop to repaint after focus restore.
        if let Ok(areas) = crate::ghostty::callbacks::GL_AREA_REGISTRY.lock() {
            for area_ptr in areas.iter() {
                let area: gtk4::glib::translate::Borrowed<gtk4::GLArea> =
                    unsafe { gtk4::glib::translate::from_glib_borrow(area_ptr.0) };
                if area.is_realized() {
                    area.queue_render();
                }
            }
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

        // When the root is a Leaf (first split), the GLArea is a direct child of the GtkStack
        // page. The replacer will remove it from the Stack (via remove_widget_from_parent) and
        // place it inside the new Paned. We then need to add the Paned to the Stack page.
        // Only capture this for Leaf roots — for nested splits the outer Paned stays in the Stack.
        let old_root_widget = self.root.widget();
        let stack_slot: Option<(gtk4::Stack, String)> =
            if matches!(self.root, SplitNode::Leaf { .. }) {
                old_root_widget
                    .parent()
                    .and_then(|p| p.downcast::<gtk4::Stack>().ok())
                    .and_then(|stack| {
                        let name = stack.page(&old_root_widget).name()?.to_string();
                        Some((stack, name))
                    })
            } else {
                None
            };

        // Find the active leaf's surface for inherited config.
        let inherited_surface = self.find_surface(active_id)?;

        // Unfocus the old surface before the split — Ghostty routes input by focus state,
        // so without this the old pane continues receiving keystrokes after the new pane
        // is created (SPLIT-07).
        unsafe {
            ffi::ghostty_surface_set_focus(inherited_surface, false);
        }

        // Get inherited config from the active surface (for CWD inheritance per D-08).
        // Pass by value (ghostty_surface_config_s is Copy) — avoids dangling pointer
        // in the GLArea realize callback, which fires asynchronously after this returns.
        let inherited_config = unsafe {
            ffi::ghostty_surface_inherited_config(
                inherited_surface,
                ffi::ghostty_surface_context_e_GHOSTTY_SURFACE_CONTEXT_SPLIT,
            )
        };

        // Create new GLArea + surface for the new pane.
        eprintln!(
            "cmux: split_active calling create_surface for new_pane_id={}",
            new_pane_id
        );
        let (new_gl_area, _surface_cell) = crate::ghostty::surface::create_surface(
            &self.app,
            self.ghostty_app,
            Some(inherited_config),
            new_pane_id,
        );

        // Replace the active leaf in the tree with a Split node.
        let new_surface_placeholder: ffi::ghostty_surface_t = std::ptr::null_mut();
        let new_leaf = SplitNode::Leaf {
            pane_id: new_pane_id,
            gl_area: new_gl_area.clone(),
            surface: new_surface_placeholder, // updated after realize via SURFACE_REGISTRY
            uuid: Uuid::new_v4(),
        };

        let _replaced = self.replace_leaf_with_split(active_id, new_leaf, orientation)?;

        // If the root was a Leaf, it's now a Split whose Paned has no parent.
        // Re-parent the new Paned root into the GtkStack page we saved above.
        if let Some((stack, name)) = stack_slot {
            let new_root = self.root.widget();
            stack.add_named(&new_root, Some(&name));
            stack.set_visible_child_name(&name);
        }

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

            // GTK4 requires a widget to have no parent before set_start/end_child.
            // old_widget may be parented to the Stack (first split) or an outer Paned (nested).
            remove_widget_from_parent(&old_widget);

            let paned = gtk4::Paned::new(orientation_cap);
            // Both children must be allowed to resize — GTK4 default for resize_end_child
            // is TRUE but be explicit to ensure drag works in both directions.
            paned.set_resize_start_child(true);
            paned.set_resize_end_child(true);
            // Prevent children from collapsing to 0px when dragging to an extreme.
            paned.set_shrink_start_child(false);
            paned.set_shrink_end_child(false);
            // Wide handle makes the divider grabable (default is ~5px, hard to click).
            paned.set_wide_handle(true);

            paned.set_start_child(Some(&old_widget));
            paned.set_end_child(Some(&new_widget));

            // Set 50/50 position after the first layout pass (per D-09 and RESEARCH Pitfall 2).
            // connect_realize fires before GTK allocates sizes, so p.width() is 0 there.
            // idle_add_local_once defers to the next main-loop idle, after layout completes.
            {
                let paned_ref = paned.clone();
                gtk4::glib::idle_add_local_once(move || {
                    let size = if orientation_cap == gtk4::Orientation::Horizontal {
                        paned_ref.width()
                    } else {
                        paned_ref.height()
                    };
                    if size > 0 {
                        paned_ref.set_position(size / 2);
                    }
                });
            }

            // Restore GTK focus AND Ghostty focus after GtkPaned drag ends (Gap 1A).
            //
            // The divider drag temporarily moves GTK focus to the separator handle,
            // which causes Ghostty's cursor blink and keyboard input to stop.
            //
            // IMPORTANT: We must NOT restore focus on every `notify::position` change
            // during the drag — that fires on every pixel of movement and causes:
            //   1. grab_focus() thrashing that fights the active gesture
            //   2. ghostty_surface_set_focus(true) message storms to the render thread
            //   3. GL_AREA_REGISTRY lock contention with the resize idle handler
            // Instead, we detect the drag lifecycle via the paned's internal GestureDrag
            // controller and restore focus only once when the drag ends.
            {
                // Find the GtkPaned's internal GestureDrag on its separator handle.
                // GTK4's Paned uses a GestureDrag controller internally — we observe
                // the controller list and connect to its drag-end signal.
                //
                // The gesture lives on the separator handle widget, not the Paned
                // itself. Walk the Paned's children to find the separator, then
                // inspect its controllers.
                let mut found_gesture = false;

                // First try: controllers on the Paned itself
                let controllers = paned.observe_controllers();
                let n = controllers.n_items();
                eprintln!("cmux: Paned has {} controllers", n);
                for i in 0..n {
                    if let Some(obj) = controllers.item(i) {
                        eprintln!("cmux:   controller[{}]: {}", i, obj.type_().name());
                        if let Ok(gesture) = obj.downcast::<gtk4::GestureDrag>() {
                            eprintln!("cmux:   -> found GestureDrag on Paned, connecting drag-end");
                            gesture.connect_drag_end(|_gesture, _offset_x, _offset_y| {
                                eprintln!("cmux: GestureDrag drag-end fired on Paned — deferring focus restore to idle");
                                // Defer to idle so GTK has time to fully release the gesture
                                // and clean up the event sequence before we move focus.
                                gtk4::glib::idle_add_local_once(|| {
                                    eprintln!("cmux: drag-end idle: restoring focus now");
                                    restore_active_pane_focus();
                                });
                            });
                            found_gesture = true;
                            break;
                        }
                    }
                }

                // Second try: walk children to find the separator handle widget
                if !found_gesture {
                    let mut child = paned.first_child();
                    while let Some(ref widget) = child {
                        let type_name = widget.type_().name();
                        eprintln!("cmux: Paned child: {}", type_name);
                        let ctrl_list = widget.observe_controllers();
                        let cn = ctrl_list.n_items();
                        for i in 0..cn {
                            if let Some(obj) = ctrl_list.item(i) {
                                eprintln!(
                                    "cmux:   child controller[{}]: {}",
                                    i,
                                    obj.type_().name()
                                );
                                if let Ok(gesture) = obj.downcast::<gtk4::GestureDrag>() {
                                    eprintln!(
                                        "cmux:   -> found GestureDrag on {}, connecting drag-end",
                                        type_name
                                    );
                                    gesture.connect_drag_end(|_gesture, _offset_x, _offset_y| {
                                        eprintln!("cmux: GestureDrag drag-end fired on separator — deferring focus restore to idle");
                                        gtk4::glib::idle_add_local_once(|| {
                                            eprintln!("cmux: separator drag-end idle: restoring focus now");
                                            restore_active_pane_focus();
                                        });
                                    });
                                    found_gesture = true;
                                    break;
                                }
                            }
                        }
                        if found_gesture {
                            break;
                        }
                        child = widget.next_sibling();
                    }
                }

                if !found_gesture {
                    eprintln!("cmux: WARNING — no GestureDrag found on Paned or its children, falling back to notify::position");
                    // Fallback: use notify::position but debounced via idle.
                    // connect_notify requires Send+Sync, so use AtomicBool instead of Rc<Cell>.
                    let restore_pending =
                        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                    paned.connect_notify(Some("position"), move |_paned, _pspec| {
                        if restore_pending.swap(true, std::sync::atomic::Ordering::SeqCst) {
                            return;
                        }
                        let pending = restore_pending.clone();
                        gtk4::glib::idle_add_once(move || {
                            pending.store(false, std::sync::atomic::Ordering::SeqCst);
                            restore_active_pane_focus();
                        });
                    });
                }
            }

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

        // Capture the raw GLArea pointer BEFORE the tree removal drops the GObject.
        // GL_AREA_REGISTRY holds raw pointers; once GTK finalizes the GObject the
        // pointer becomes dangling. Remove it here while the GLArea is still alive.
        let raw_gl_area: Option<*mut gtk4::ffi::GtkGLArea> = self
            .find_gl_area(active_id)
            .map(|a| a.as_ptr() as *mut gtk4::ffi::GtkGLArea);

        // Remove the leaf from the tree and get the surviving sibling's pane_id.
        let surviving_id = remove_leaf_from_tree(&mut self.root, active_id)?;

        // Remove the now-dropped GLArea from GL_AREA_REGISTRY before any further
        // callbacks can dereference the dangling pointer (Gap 2 fix).
        if let Some(raw_ptr) = raw_gl_area {
            if let Ok(mut areas) = crate::ghostty::callbacks::GL_AREA_REGISTRY.lock() {
                areas.retain(|p| p.0 != raw_ptr);
            }
            // Also remove from GL_TO_SURFACE mapping.
            if let Ok(mut gl_to_surface) = crate::ghostty::callbacks::GL_TO_SURFACE.lock() {
                gl_to_surface.remove(&(raw_ptr as usize));
            }
        }

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
                        uuid: Uuid::new_v4(),
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
        let page = stack.page(old_widget);
        if let Some(name) = page.name() {
            let name_str = name.to_string();
            stack.remove(old_widget);
            // new_widget may still be parented to the Paned we're replacing; unparent first.
            remove_widget_from_parent(new_widget);
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

/// Remove `widget` from its current GTK parent so it can be reparented.
/// GTK4 requires `gtk_widget_get_parent(child) == NULL` before set_start/end_child.
fn remove_widget_from_parent(widget: &gtk4::Widget) {
    let Some(parent) = widget.parent() else {
        return;
    };
    if let Some(paned) = parent.downcast_ref::<gtk4::Paned>() {
        if paned
            .start_child()
            .as_ref()
            .map(|w| w == widget)
            .unwrap_or(false)
        {
            paned.set_start_child(None::<&gtk4::Widget>);
        } else {
            paned.set_end_child(None::<&gtk4::Widget>);
        }
    } else if let Some(stack) = parent.downcast_ref::<gtk4::Stack>() {
        stack.remove(widget);
    }
}

/// Restore GTK keyboard focus and Ghostty surface focus to the active pane.
/// Called once when a GtkPaned drag ends — NOT on every pixel of movement.
/// Re-syncs each surface's cached size with the GLArea's current allocation to
/// break any anti-flicker stall in Ghostty's drawFrame() guard, then kicks
/// the render thread with ghostty_surface_refresh + queue_render.
///
/// Does NOT touch focus state. The cursor blink timer runs independently of
/// resize. Calling set_focus(false→true) here kills the timer due to an async
/// cancel race in Ghostty's renderer thread: the false message enqueues a timer
/// cancel, but the true message is processed before the cancel callback fires,
/// so the guard `if cursor_c.state() != .active` sees `.active` and skips the
/// restart. The cancel then completes, leaving the timer permanently dead.
fn restore_active_pane_focus() {
    // Re-set size + refresh ALL surfaces to break the anti-flicker stall.
    if let Ok(areas) = crate::ghostty::callbacks::GL_AREA_REGISTRY.lock() {
        if let Ok(gl_to_surface) = crate::ghostty::callbacks::GL_TO_SURFACE.lock() {
            for area_ptr in areas.iter() {
                let area: gtk4::glib::translate::Borrowed<gtk4::GLArea> =
                    unsafe { gtk4::glib::translate::from_glib_borrow(area_ptr.0) };
                if let Some(&surface_ptr) = gl_to_surface.get(&(area_ptr.0 as usize)) {
                    let scale = area.scale_factor();
                    let w = (area.width() * scale) as u32;
                    let h = (area.height() * scale) as u32;
                    if w > 0 && h > 0 {
                        unsafe {
                            let surface = surface_ptr as ffi::ghostty_surface_t;
                            ffi::ghostty_surface_set_size(surface, w, h);
                            ffi::ghostty_surface_refresh(surface);
                        }
                    }
                }
                if area.is_realized() {
                    area.queue_render();
                    area.queue_draw();
                }
            }
        }
    }

    // Drive the app tick to process any pending mailbox messages (redraw_surface, etc.)
    let app_ptr = crate::ghostty::callbacks::APP_PTR.load(std::sync::atomic::Ordering::SeqCst);
    if app_ptr != 0 {
        unsafe {
            let app = app_ptr as ffi::ghostty_app_t;
            ffi::ghostty_app_tick(app);
        }
    }

    // Restore GTK keyboard focus to the active pane.
    if let Ok(areas) = crate::ghostty::callbacks::GL_AREA_REGISTRY.lock() {
        for area_ptr in areas.iter() {
            let area: gtk4::glib::translate::Borrowed<gtk4::GLArea> =
                unsafe { gtk4::glib::translate::from_glib_borrow(area_ptr.0) };
            if area.has_css_class("active-pane") {
                area.grab_focus();
                area.queue_render();
                break;
            }
        }
    }

    // The set_size → IO thread → render thread → updateFrame → cells rebuild pipeline
    // is asynchronous. The immediate queue_render above may still draw stale content
    // because cells haven't been rebuilt yet. Schedule follow-up recovery ticks to
    // give the pipeline time to converge (50ms, 150ms, 300ms).
    for delay_ms in [50u32, 150, 300] {
        gtk4::glib::timeout_add_local_once(
            std::time::Duration::from_millis(delay_ms as u64),
            move || {
                let app_ptr =
                    crate::ghostty::callbacks::APP_PTR.load(std::sync::atomic::Ordering::SeqCst);
                if app_ptr != 0 {
                    unsafe {
                        ffi::ghostty_app_tick(app_ptr as ffi::ghostty_app_t);
                    }
                }
                if let Ok(areas) = crate::ghostty::callbacks::GL_AREA_REGISTRY.lock() {
                    for area_ptr in areas.iter() {
                        let area: gtk4::glib::translate::Borrowed<gtk4::GLArea> =
                            unsafe { gtk4::glib::translate::from_glib_borrow(area_ptr.0) };
                        if area.is_realized() {
                            area.queue_render();
                        }
                    }
                }
            },
        );
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

/// Serde-friendly mirror of SplitNode for session persistence.
/// GTK widget references (GLArea, Paned) cannot be serialized — this parallel type holds
/// only the data needed to reconstruct the tree on restore.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum SplitNodeData {
    Leaf {
        pane_id: u64,
        surface_uuid: Uuid,
        /// Shell executable path, e.g. "/bin/zsh" or "/bin/bash"
        shell: String,
        /// Absolute working directory path (best-effort; may be empty if /proc unavailable)
        cwd: String,
    },
    Split {
        /// "horizontal" or "vertical"
        orientation: String,
        start: Box<SplitNodeData>,
        end: Box<SplitNodeData>,
    },
}

impl SplitNode {
    /// Produce a serializable snapshot of this node's tree structure.
    /// `shell` and `cwd` are best-effort: Plan 05 fills these via /proc.
    /// Falls back to empty strings if /proc is unavailable or the pid is unknown.
    pub fn to_data(&self) -> SplitNodeData {
        match self {
            SplitNode::Leaf { pane_id, uuid, .. } => SplitNodeData::Leaf {
                pane_id: *pane_id,
                surface_uuid: *uuid,
                // Best-effort CWD: not yet implemented — Plan 05 fills this via /proc.
                shell: String::new(),
                cwd: String::new(),
            },
            SplitNode::Split { orientation, start, end, .. } => SplitNodeData::Split {
                orientation: match orientation {
                    gtk4::Orientation::Horizontal => "horizontal".to_string(),
                    gtk4::Orientation::Vertical => "vertical".to_string(),
                    _ => "horizontal".to_string(),
                },
                start: Box::new(start.to_data()),
                end: Box::new(end.to_data()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_node_data_leaf_has_surface_uuid() {
        // Build a minimal SplitNodeData::Leaf directly and verify surface_uuid field exists.
        let id = Uuid::new_v4();
        let data = SplitNodeData::Leaf {
            pane_id: 42,
            surface_uuid: id,
            shell: "/bin/bash".to_string(),
            cwd: "/home/user".to_string(),
        };
        if let SplitNodeData::Leaf { surface_uuid, pane_id, .. } = data {
            assert_eq!(surface_uuid, id);
            assert_eq!(pane_id, 42);
        } else {
            panic!("Expected SplitNodeData::Leaf");
        }
    }

    #[test]
    fn split_node_data_roundtrip_json() {
        // Verify SplitNodeData serializes and deserializes via serde_json.
        let leaf = SplitNodeData::Leaf {
            pane_id: 1,
            surface_uuid: Uuid::new_v4(),
            shell: "/bin/zsh".to_string(),
            cwd: "/tmp".to_string(),
        };
        let json = serde_json::to_string(&leaf).expect("serialize failed");
        let restored: SplitNodeData = serde_json::from_str(&json).expect("deserialize failed");
        if let (
            SplitNodeData::Leaf { pane_id: p1, surface_uuid: u1, .. },
            SplitNodeData::Leaf { pane_id: p2, surface_uuid: u2, .. },
        ) = (&leaf, &restored)
        {
            assert_eq!(p1, p2);
            assert_eq!(u1, u2);
        } else {
            panic!("Roundtrip changed variant");
        }
    }

    #[test]
    fn split_node_data_split_roundtrip_json() {
        // Verify nested SplitNodeData serializes correctly.
        let split = SplitNodeData::Split {
            orientation: "horizontal".to_string(),
            start: Box::new(SplitNodeData::Leaf {
                pane_id: 1,
                surface_uuid: Uuid::new_v4(),
                shell: String::new(),
                cwd: String::new(),
            }),
            end: Box::new(SplitNodeData::Leaf {
                pane_id: 2,
                surface_uuid: Uuid::new_v4(),
                shell: String::new(),
                cwd: String::new(),
            }),
        };
        let json = serde_json::to_string(&split).expect("serialize failed");
        let restored: SplitNodeData = serde_json::from_str(&json).expect("deserialize failed");
        if let SplitNodeData::Split { orientation, .. } = restored {
            assert_eq!(orientation, "horizontal");
        } else {
            panic!("Roundtrip changed variant to non-Split");
        }
    }
}
