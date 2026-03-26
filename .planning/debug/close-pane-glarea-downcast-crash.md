---
status: diagnosed
trigger: "crash in cmux-linux when closing active pane with Ctrl+Shift+X when 3 panes are open (one split right + one split down)"
created: 2026-03-24T00:00:00Z
updated: 2026-03-24T00:00:00Z
---

## Current Focus

hypothesis: GL_AREA_REGISTRY holds raw GtkGLArea pointers to destroyed panes; after close, wakeup_cb or resize idle dereferences a stale pointer and performs a from_glib_borrow cast to gtk4::GLArea, which panics because the GObject type check fails on the dead pointer.
test: confirmed by code reading — GL_AREA_REGISTRY is never pruned on pane close.
expecting: fix = remove the closing pane's GtkGLAreaPtr from GL_AREA_REGISTRY in close_active() before freeing the surface.
next_action: report root cause to user.

## Symptoms

expected: Closing a pane with Ctrl+Shift+X removes it cleanly; remaining panes continue rendering.
actual: Process aborts with GTK4 assertion failure: `assertion failed: ::glib::types::instance_of::<Self>(ptr as *const _)` in gtk4::auto::gl_area.rs:18:1
errors: |
  thread 'main' (565900) panicked at .../gtk4-0.10.3/src/auto/gl_area.rs:18:1:
  assertion failed: ::glib::types::instance_of::<Self>(ptr as *const _)
  note: run with RUST_BACKTRACE=1 environment variable to display a backtrace
  fatal runtime error: failed to initiate panic, error 5, aborting
  Aborted (core dumped)
reproduction: Open cmux. Split right (Ctrl+D). Split down (Ctrl+Shift+D). Close active pane (Ctrl+Shift+X).
started: After pane-split feature was added (Phase 02 SPLIT-06).

## Eliminated

- hypothesis: The crash is caused by dangling surface pointer in find_surface() after close_active() frees it.
  evidence: find_surface() is called BEFORE ghostty_surface_free. The crash is in GTK widget code, not ghostty FFI. The assertion is instance_of::<GLArea>, not a segfault in ghostty code.
  timestamp: 2026-03-24

- hypothesis: The crash is in replace_child_in_parent() when the tree is mutated.
  evidence: remove_leaf_from_tree() handles tree mutation before the render callbacks fire. The panic location (gl_area.rs:18) is in the gtk4-rs auto-generated downcast code inside from_glib_borrow, not tree-mutation code.
  timestamp: 2026-03-24

## Evidence

- timestamp: 2026-03-24
  checked: src/ghostty/callbacks.rs — GL_AREA_REGISTRY and wakeup_cb
  found: |
    GL_AREA_REGISTRY is a global Mutex<Vec<GtkGLAreaPtr>> (line 31).
    GtkGLAreaPtr stores a raw *mut GtkGLArea.
    wakeup_cb (line 39) and action_cb (line 83) and the resize idle in surface.rs (line 217) all iterate GL_AREA_REGISTRY and call from_glib_borrow(area_ptr.0) to get a gtk4::GLArea.
    from_glib_borrow is implemented in gtk4-0.10.3/src/auto/gl_area.rs:18 — it asserts instance_of::<GLArea>(ptr).
    NOTHING removes a GtkGLAreaPtr from GL_AREA_REGISTRY when a pane is closed.
  implication: |
    After close_active() calls ghostty_surface_free() and removes the GLArea from the GTK widget tree,
    the raw GtkGLAreaPtr in GL_AREA_REGISTRY still points to that widget's GObject.
    GTK4 may finalize (destroy) the GObject once it has no parent and no Rust strong reference.
    The next time wakeup_cb or an idle fires, it calls from_glib_borrow on the dangling/finalized pointer.
    The GObject type check instance_of::<GLArea> fails → panic → abort.

- timestamp: 2026-03-24
  checked: src/ghostty/surface.rs — where GtkGLAreaPtr is inserted into GL_AREA_REGISTRY
  found: |
    Line 146-150 (realize callback): inserts the raw GtkGLArea pointer into GL_AREA_REGISTRY.
    There is no corresponding removal anywhere in the codebase.
  implication: Every closed pane leaves a dangling pointer in GL_AREA_REGISTRY.

- timestamp: 2026-03-24
  checked: src/split_engine.rs — close_active() method (lines 326-365)
  found: |
    close_active() calls remove_leaf_from_tree() which removes the GLArea from the GTK widget tree.
    It calls ghostty_surface_free() and removes from SURFACE_REGISTRY.
    It does NOT remove the closing pane's GtkGLAreaPtr from GL_AREA_REGISTRY.
    The gl_area local (the closing pane's GLArea object) is still held in the SplitNode::Leaf
    that is being dropped, but only as a gtk4::GLArea GObject reference.
    When the SplitNode::Leaf is dropped (at end of remove_leaf_from_tree via *node = surviving),
    the Rust gtk4::GLArea wrapper is dropped, which decrements the GObject ref count.
    If GTK finalizes the GObject at that point, the raw pointer in GL_AREA_REGISTRY is dangling.
  implication: The very next idle or wakeup callback crash is guaranteed once a pane is closed.

- timestamp: 2026-03-24
  checked: src/ghostty/callbacks.rs — resize idle in surface.rs lines 217-225
  found: |
    The resize idle (queued by connect_resize on ANY GLArea) also iterates GL_AREA_REGISTRY.
    If a resize event is pending from the paned drag that preceded pane creation, this idle
    fires immediately after the pane close, hitting the stale pointer before wakeup_cb.
    This explains why the crash specifically manifests in the 3-pane scenario:
    the second split (split_down) involves a GtkPaned resize, which queues the resize idle,
    which races with close_active() and fires on the now-dead GLArea pointer.
  implication: The resize idle path is the most likely crash site in the reported scenario.

## Resolution

root_cause: |
  GL_AREA_REGISTRY (src/ghostty/callbacks.rs:31) is a global Vec of raw GtkGLArea pointers
  used by wakeup_cb, action_cb, and the resize idle to call queue_render() on all panes.

  When close_active() removes a pane (src/split_engine.rs:326-365), it:
    1. Removes the GLArea from the GTK widget tree (via remove_leaf_from_tree)
    2. Frees the ghostty surface
    3. Removes from SURFACE_REGISTRY

  But it NEVER removes the closing pane's raw GtkGLAreaPtr from GL_AREA_REGISTRY.

  Once the SplitNode::Leaf holding the closing pane's gtk4::GLArea is dropped, GTK may
  finalize the underlying GObject. The raw pointer in GL_AREA_REGISTRY then becomes
  dangling/stale.

  The next time any of these iterates GL_AREA_REGISTRY:
    - wakeup_cb (src/ghostty/callbacks.rs:54-62)
    - action_cb (src/ghostty/callbacks.rs:91-98)
    - resize idle (src/ghostty/surface.rs:217-225)

  ...it calls `glib::translate::from_glib_borrow(area_ptr.0)` which invokes the gtk4-rs
  auto-generated GLArea wrapper at gtk4-0.10.3/src/auto/gl_area.rs:18, which asserts
  `instance_of::<GLArea>(ptr)`. On a finalized GObject this type check fails → panic → abort.

fix: |
  In src/split_engine.rs, close_active() must remove the closing pane's GLArea from
  GL_AREA_REGISTRY before the SplitNode::Leaf (and thus its gtk4::GLArea) is dropped.

  Specifically, before calling remove_leaf_from_tree(), retrieve the GLArea for the closing
  pane (via find_gl_area), then after tree removal, remove its raw pointer from GL_AREA_REGISTRY:

  ```rust
  // In close_active(), after finding surface_to_free and before remove_leaf_from_tree:
  let closing_gl_area = self.find_gl_area(active_id);

  let surviving_id = remove_leaf_from_tree(&mut self.root, active_id)?;

  // Remove the closing pane's GLArea from GL_AREA_REGISTRY so wakeup_cb / resize idle
  // cannot dereference the stale raw pointer after the widget is destroyed.
  if let Some(area) = closing_gl_area {
      let raw_ptr = area.as_ptr();
      if let Ok(mut areas) = crate::ghostty::callbacks::GL_AREA_REGISTRY.lock() {
          areas.retain(|p| p.0 != raw_ptr);
      }
  }
  ```

  This must happen AFTER remove_leaf_from_tree (so the widget is unparented/removed from GTK tree)
  but BEFORE ghostty_surface_free and before the function returns (before the SplitNode drop).
  The gtk4::GLArea smart pointer returned by find_gl_area keeps the GObject alive through this window.

verification: Diagnosis only (goal: find_root_cause_only). Not yet fixed.
files_changed: []
