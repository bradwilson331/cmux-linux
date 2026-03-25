---
phase: 02-workspaces-pane-splits
plan: 08
subsystem: ghostty-focus-sync
tags: [bug-fix, focus, render, ghostty, gtk4]
dependency_graph:
  requires: []
  provides: [unified-focus-render-fix]
  affects: [src/ghostty/surface.rs, src/split_engine.rs, src/shortcuts.rs]
tech_stack:
  added: []
  patterns: [EventControllerFocus, active-pane CSS class check, focus_active_surface]
key_files:
  created: []
  modified:
    - src/ghostty/surface.rs
    - src/split_engine.rs
    - src/shortcuts.rs
decisions:
  - EventControllerFocus on GLArea keeps Ghostty's focused state in sync with GTK focus routing after any widget-tree operation
  - restore_active_pane_focus conditions set_focus on CSS class rather than toggle-all pattern
  - focus_active_surface() replaces grab_active_focus() in Ctrl+B to also call ghostty_surface_set_focus(true)
metrics:
  duration: ~10 minutes
  completed_date: "2026-03-25T03:52:56Z"
  tasks: 4
  files: 3
---

# Phase 02 Plan 08: Unified Focus/Render Regression Fix Summary

**One-liner:** Fixed four compounding bugs causing frozen cursor and invisible input after divider drag, pane close, sidebar toggle, and fresh-launch resize by wiring EventControllerFocus, fixing active-pane-only focus restore, and calling ghostty_surface_set_focus after sidebar toggle.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add EventControllerFocus to GLArea and guard re-realize set_size | 5e262c65 | src/ghostty/surface.rs |
| 2 | Fix restore_active_pane_focus to set focus only on the active surface | 8956eca9 | src/split_engine.rs |
| 3 | Fix sidebar toggle — call focus_active_surface after grab_active_focus | af448ae1 | src/shortcuts.rs |
| 4 | Guard initial realize set_size(0,0) — fresh-launch window resize | 5e262c65 | src/ghostty/surface.rs |

Note: Tasks 1 and 4 both modify surface.rs; they were committed together in 5e262c65.

## What Was Built

### Bug 1 Fixed: EventControllerFocus on GLArea (GHOST-05)

Added `EventControllerFocus` to each `GLArea` in `create_surface()`. The `enter` signal calls `ghostty_surface_set_focus(true)` and the `leave` signal calls `ghostty_surface_set_focus(false)`. This ensures Ghostty's internal `focused` flag stays synchronized with GTK's focus routing after any widget-tree operation (GtkPaned separator drag, sidebar show/hide, workspace switch). Previously, Ghostty's focused state diverged from GTK reality, causing subsequent explicit `set_focus(true)` calls to hit Ghostty's early-return guard (`if self.focused == focused { return; }`).

### Bug 2 Fixed: restore_active_pane_focus active-pane-only focus

Rewrote the `set_focus` logic in `restore_active_pane_focus()` in `split_engine.rs`. The previous code called `set_focus(false)` then `set_focus(true)` on ALL surfaces in `GL_AREA_REGISTRY`, leaving every surface with `focused=true`. The fix checks `has_css_class("active-pane")` on each area: active pane gets `set_focus(true)`, all others get `set_focus(false)`. The `set_size` and `ghostty_surface_refresh` calls remain on all surfaces.

### Bug 3 Fixed: Ctrl+B sidebar toggle calls focus_active_surface

Added `focus_active_surface()` method to `SplitEngine`. It grabs GTK focus via `find_gl_area()`, then iterates `GL_AREA_REGISTRY` to find the active-pane area and calls `ghostty_surface_set_focus(true)` on its surface, then queues renders on all realized areas. Updated the `Ctrl+B` handler in `shortcuts.rs` to call `focus_active_surface()` instead of `grab_active_focus()`.

### Bug 4 Fixed: Guard initial realize set_size(0,0)

Added the same `phys_w > 0 && phys_h > 0` guard to the initial surface creation path in `connect_realize`. At realize time `area.width()` is 0, so the old code called `ghostty_surface_set_size(surface, 0, 0)`, which could trigger Ghostty's anti-flicker guard. The `connect_resize` signal fires after GTK allocates real size.

## Deviations from Plan

None — plan executed exactly as written.

The plan referenced `self.find_surface(self.active_pane_id)` in `focus_active_surface()`, which doesn't exist on `SplitEngine`. I implemented the method using `GL_AREA_REGISTRY` + `GL_TO_SURFACE` lookup with `has_css_class("active-pane")` check instead — consistent with how `restore_active_pane_focus` already works, achieving the same outcome without adding a new `find_surface` traversal helper.

## Verification

```
EventControllerFocus::new count: 1 ✓
phys_w > 0 guards in surface.rs: 2 ✓
ghostty_surface_set_size skipped message: 1 ✓
focus_active_surface in split_engine: 1 ✓
focus_active_surface in shortcuts: 1 ✓
grab_active_focus in shortcuts: 0 (replaced) ✓
has_css_class("active-pane") in split_engine: 5 ✓
cargo build errors: 0 ✓
```

## Known Stubs

None.

## Self-Check: PASSED

- src/ghostty/surface.rs: modified (committed 5e262c65)
- src/split_engine.rs: modified (committed 8956eca9)
- src/shortcuts.rs: modified (committed af448ae1)
- git log confirms all three commits exist
