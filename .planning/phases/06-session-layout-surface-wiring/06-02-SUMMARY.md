---
phase: 06-session-layout-surface-wiring
plan: 02
subsystem: session
tags: [session-restore, split-tree, gtk4-paned, ghostty-surface, uuid]

requires:
  - phase: 06-session-layout-surface-wiring
    provides: "Version 2 session format with SplitNodeData tree topology, ratio, CWD, active_pane_uuid"
provides:
  - "SplitEngine::from_data() reconstructs full split tree from session JSON"
  - "Recursive set_initial_surface for nested pane trees"
  - "sync_surfaces_from_registry wires Ghostty surfaces after GLArea realize"
  - "Version-aware session restore: v2 full tree, v1 name-only with auto-upgrade"
affects: [socket-commands, session-persistence]

tech-stack:
  added: []
  patterns: ["from_data() constructor pattern for deserializing GTK widget trees from serde data"]

key-files:
  created: []
  modified:
    - src/split_engine.rs
    - src/app_state.rs
    - src/main.rs

key-decisions:
  - "Reuse existing find_pane_id_by_uuid instead of adding duplicate find_pane_id_by_uuid_str"
  - "sync_surfaces_from_registry uses idle_add_local_once to run after GLArea realize completes"

patterns-established:
  - "from_data pattern: recursive tree reconstruction with depth guard for safety"

requirements-completed: [SESS-02, SOCK-02, SOCK-03]

duration: 3min
completed: 2026-03-26
---

# Phase 06 Plan 02: Session Restore Tree Rebuild Summary

**SplitEngine::from_data() reconstructs full split tree with live Ghostty surfaces from v2 session JSON, enabling socket commands on restored panes**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-26T19:15:47Z
- **Completed:** 2026-03-26T19:18:26Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- SplitEngine::from_data() rebuilds full SplitNode tree from SplitNodeData with GTK widgets and Ghostty surface placeholders
- set_initial_surface is now recursive, wiring surfaces for all leaves in nested splits (not just root)
- sync_surfaces_from_registry fills null surface pointers from GL_TO_SURFACE after GLArea realize
- main.rs branches on session version: v2 uses restore_workspace with full tree rebuild, v1 preserves name-only restore
- AppState::restore_workspace() creates sidebar row and split engine from session data in one call

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement from_data() and recursive set_surface_for_pane()** - `2db71933` (feat)
2. **Task 2: Update main.rs restore logic to use from_data() for full tree rebuild** - `b9d14106` (feat)

## Files Created/Modified
- `src/split_engine.rs` - Added from_data(), node_from_data(), recursive set_initial_surface, sync_surfaces_from_registry
- `src/app_state.rs` - Added restore_workspace() method using SplitEngine::from_data()
- `src/main.rs` - Version-aware restore branching with idle surface sync after GLArea realize

## Decisions Made
- Reused existing find_pane_id_by_uuid() instead of adding duplicate find_pane_id_by_uuid_str() -- identical functionality
- sync_surfaces_from_registry deferred via idle_add_local_once to let GTK realize all widgets first

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full session restore loop is complete: save (Plan 01) and restore (Plan 02) both handle v2 format
- Socket commands (surface.send_text, debug.type) work against restored panes via surface pointer wiring
- All 21 existing tests pass

---
*Phase: 06-session-layout-surface-wiring*
*Completed: 2026-03-26*

## Self-Check: PASSED
