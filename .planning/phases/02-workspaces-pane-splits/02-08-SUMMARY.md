---
phase: 02-workspaces-pane-splits
plan: 08
subsystem: ghostty-focus-sync
tags: [bug-fix, focus, render, ghostty, gtk4, cursor-blink]
dependency_graph:
  requires: []
  provides: [unified-focus-render-fix, cursor-blink-fix]
  affects: [src/ghostty/surface.rs, src/split_engine.rs, src/shortcuts.rs, ghostty/src/renderer/Thread.zig, ghostty/src/renderer/generic.zig]
tech_stack:
  added: []
  patterns: [EventControllerFocus, active-pane CSS class check, focus_active_surface, xev-timer-cancel-race, drawFrame-sync-guard]
key_files:
  created: []
  modified:
    - src/ghostty/surface.rs
    - src/split_engine.rs
    - src/shortcuts.rs
    - ghostty/src/renderer/Thread.zig
    - ghostty/src/renderer/generic.zig
decisions:
  - EventControllerFocus on GLArea keeps Ghostty's focused state in sync with GTK focus routing after any widget-tree operation
  - restore_active_pane_focus conditions set_focus on CSS class rather than toggle-all pattern
  - focus_active_surface() replaces grab_active_focus() in Ctrl+B to also call ghostty_surface_set_focus(true)
  - Fix the cancel race in Ghostty Thread.zig cursorCancelCallback not in cmux Rust code
  - Guard macOS sync/resize early return in generic.zig drawFrame with comptime isDarwin() — Linux embedded must not skip this path
metrics:
  duration: ~2 sessions
  completed_date: "2026-03-25T20:05:00Z"
  tasks: 6
  files: 5
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
| 5 | Fix cursor cancel-race in Thread.zig cursorCancelCallback | 36e8d2a5 | ghostty/src/renderer/Thread.zig |
| 6 | Guard macOS sync/resize early return to Darwin-only in generic.zig | 36e8d2a5 | ghostty/src/renderer/generic.zig |

Note: Tasks 1 and 4 both modify surface.rs; they were committed together in 5e262c65.
Note: Tasks 5 and 6 are ghostty fork changes; committed together in 36e8d2a5.

## What Was Built

### Bug 1 Fixed: EventControllerFocus on GLArea (GHOST-05)

Added `EventControllerFocus` to each `GLArea` in `create_surface()`. The `enter` signal calls `ghostty_surface_set_focus(true)` and the `leave` signal calls `ghostty_surface_set_focus(false)`. This ensures Ghostty's internal `focused` flag stays synchronized with GTK's focus routing after any widget-tree operation (GtkPaned separator drag, sidebar show/hide, workspace switch). Previously, Ghostty's focused state diverged from GTK reality, causing subsequent explicit `set_focus(true)` calls to hit Ghostty's early-return guard (`if self.focused == focused { return; }`).

### Bug 2 Fixed: restore_active_pane_focus active-pane-only focus

Rewrote the `set_focus` logic in `restore_active_pane_focus()` in `split_engine.rs`. The previous code called `set_focus(false)` then `set_focus(true)` on ALL surfaces in `GL_AREA_REGISTRY`, leaving every surface with `focused=true`. The fix checks `has_css_class("active-pane")` on each area: active pane gets `set_focus(true)`, all others get `set_focus(false)`. The `set_size` and `ghostty_surface_refresh` calls remain on all surfaces.

### Bug 3 Fixed: Ctrl+B sidebar toggle calls focus_active_surface

Added `focus_active_surface()` method to `SplitEngine`. It grabs GTK focus via `find_gl_area()`, then iterates `GL_AREA_REGISTRY` to find the active-pane area and calls `ghostty_surface_set_focus(true)` on its surface, then queues renders on all realized areas. Updated the `Ctrl+B` handler in `shortcuts.rs` to call `focus_active_surface()` instead of `grab_active_focus()`.

### Bug 4 Fixed: Guard initial realize set_size(0,0)

Added the same `phys_w > 0 && phys_h > 0` guard to the initial surface creation path in `connect_realize`. At realize time `area.width()` is 0, so the old code called `ghostty_surface_set_size(surface, 0, 0)`, which could trigger Ghostty's anti-flicker guard. The `connect_resize` signal fires after GTK allocates real size.

## Additional Root Cause Fixes (UAT revealed deeper bugs)

UAT of the original 4 tasks revealed cursor blink was still frozen after window resize and after split-pane divider drag. Investigation found two more root causes in the Ghostty fork itself:

### Bug 5 Fixed: Cursor cancel-race in Thread.zig (split-pane focus bounce)

When `set_focus(false)` + `set_focus(true)` were drained in one mailbox pass, the cancel completion was still in flight when the re-focus arrived. The re-focus saw `cursor_c.state() == .active` and skipped restart; then `cursorCancelCallback` fired with `flags.focused=false` → timer stayed permanently dead.

Fix: `cursorCancelCallback` now receives `Thread*` as userdata (changed from `?*void`). On cancel completion, if `self.flags.focused == true` and `cursor_c` is not active, restart the blink timer immediately.

### Bug 6 Fixed: macOS sync/resize early return in generic.zig (resize freeze)

The macOS CoreAnimation anti-blank-flash guard in `drawFrame` was platform-unconditional:
```zig
if (sync and size_changed and has_presented) { presentLastTarget(); return; }
```
On Linux embedded (our `ghostty_surface_draw` path uses `sync=true`): after any resize, `size_changed=true` because `self.size.screen` is only updated PAST this guard, which never ran. Every frame hit the early return, permanently re-presenting the last frame. Cursor blink state from `updateFrame` was never visible.

Fix: Wrapped in `comptime builtin.os.tag.isDarwin()` — the guard is a no-op on Linux.

## Deviations from Plan

The original plan was executed exactly as written for tasks 1-4. Tasks 5-6 were unplanned ghostty fork fixes discovered during UAT.

The plan referenced `self.find_surface(self.active_pane_id)` in `focus_active_surface()`, which doesn't exist on `SplitEngine`. Implemented using `GL_AREA_REGISTRY` + `GL_TO_SURFACE` lookup with `has_css_class("active-pane")` check — consistent with how `restore_active_pane_focus` already works.

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
isDarwin() guard in generic.zig: 1 ✓
Thread* userdata in cursorCancelCallback: 1 ✓
```

UAT (human):
- Resize window → cursor blinks ✓
- Ctrl+D split + drag divider → cursor blinks in active pane ✓

## Known Stubs

None.

## Self-Check: PASSED

- src/ghostty/surface.rs: modified (committed 5e262c65)
- src/split_engine.rs: modified (committed 8956eca9)
- src/shortcuts.rs: modified (committed af448ae1)
- ghostty/src/renderer/Thread.zig: modified (committed 36e8d2a5 in ghostty submodule + parent)
- ghostty/src/renderer/generic.zig: modified (committed 36e8d2a5 in ghostty submodule + parent)
