---
phase: 02-workspaces-pane-splits
plan: 07
subsystem: ui
tags: [gtk4, gtkpaned, focus, keyboard-shortcuts, rust]

# Dependency graph
requires:
  - phase: 02-workspaces-pane-splits
    provides: SplitEngine, GL_AREA_REGISTRY, pane splits
provides:
  - Post-drag cursor blink restoration via notify::position handler
  - CSS border repaint after divider drag via queue_draw()
  - GLArea registry cleanup before GObject finalization (crash fix)
  - Ctrl+Shift+Arrow pane focus shortcuts (Linux-compatible)
affects: [02-UAT, session-persistence]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "GtkPaned notify::position handler for focus restoration"
    - "GLArea registry cleanup before tree drop"

key-files:
  created: []
  modified:
    - src/split_engine.rs
    - src/ghostty/surface.rs
    - src/shortcuts.rs

key-decisions:
  - "Use notify::position on GtkPaned to detect drag end and restore active pane focus"
  - "Call queue_draw() alongside queue_render() to repaint CSS borders after resize"
  - "Capture raw GLArea pointer before remove_leaf_from_tree and purge from registry before surface free"
  - "Change Ctrl+Alt+Arrow to Ctrl+Shift+Arrow to avoid Linux compositor interception"

patterns-established:
  - "GtkPaned drag focus: connect notify::position to restore focus after divider drag steals it"
  - "GLArea cleanup: always remove from GL_AREA_REGISTRY before GObject drop to prevent dangling pointer"

requirements-completed: [SPLIT-03, SPLIT-04, SPLIT-05]

# Metrics
duration: 2min
completed: 2026-03-24
---

# Phase 02 Plan 07: UAT Gap Closure Summary

**Fixed post-drag cursor freeze, pane-close crash (dangling GLArea pointer), and Ctrl+Alt+Arrow interception on Linux desktops**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-24T18:00:50Z
- **Completed:** 2026-03-24T18:03:00Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Post-drag cursor blink restored via GtkPaned notify::position handler (Gap 1A)
- CSS border repaint after divider drag via queue_draw() (Gap 1B)
- Pane-close crash fixed by removing GLArea from registry before GObject finalization (Gap 2)
- Pane focus shortcuts changed to Ctrl+Shift+Arrow, avoiding Linux compositor interception (Gap 3)

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix post-drag cursor freeze and border repaint (Gap 1)** - `ff54ebe2` (fix)
2. **Task 2: Fix pane-close crash — remove GLArea from registry before free (Gap 2)** - `9cc42698` (fix)
3. **Task 3: Change pane focus shortcuts from Ctrl+Alt+Arrow to Ctrl+Shift+Arrow (Gap 3)** - `037a0c65` (fix)

## Files Created/Modified
- `src/split_engine.rs` - Added notify::position handler in replace_leaf_with_split; added raw GLArea capture and registry cleanup in close_active; updated doc comment
- `src/ghostty/surface.rs` - Added queue_draw() alongside queue_render() in resize idle loop
- `src/shortcuts.rs` - Changed all four pane focus direction arms from (true, false, true, k) to (true, true, false, k)

## Decisions Made
- Used notify::position callback on GtkPaned to detect when divider drag ends — this is when GTK focus needs restoration
- Added queue_draw() for CSS border repaint in addition to queue_render() for GL framebuffer
- Captured raw GLArea pointer before tree drop so we have the exact value to remove from registry
- Adopted Ctrl+Shift+Arrow convention (matches Terminator) instead of Ctrl+Alt+Arrow which is claimed by GNOME/KDE for virtual desktop switching

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all three fixes were straightforward implementations as specified in the plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- All three UAT gaps closed
- Ready to complete Phase 2 UAT verification
- No blockers for subsequent phases

---
*Phase: 02-workspaces-pane-splits*
*Completed: 2026-03-24*

## Self-Check: PASSED

All files verified to exist. All commits verified in git history.
