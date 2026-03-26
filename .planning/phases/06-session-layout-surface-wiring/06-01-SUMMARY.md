---
phase: 06-session-layout-surface-wiring
plan: 01
subsystem: session
tags: [session-persistence, split-tree, serde, proc-fs]

requires:
  - phase: 03-socket-api-session-persistence
    provides: "Session save/load infrastructure, SplitNodeData enum, trigger_session_save()"
provides:
  - "Version 2 session format with full split tree topology"
  - "Divider ratio capture from GtkPaned position"
  - "Best-effort CWD capture via /proc for leaf panes"
  - "active_pane_uuid per workspace in session data"
  - "Backward-compatible deserialization for v1 session files"
affects: [06-02, session-restore]

tech-stack:
  added: []
  patterns: ["proc-fs CWD capture for child processes", "serde default for backward-compatible schema evolution"]

key-files:
  created: []
  modified:
    - src/split_engine.rs
    - src/app_state.rs
    - src/session.rs

key-decisions:
  - "CWD capture scans /proc for child processes of cmux PID since no direct Ghostty FFI for PTY fd exists"
  - "serde(default = default_ratio) on ratio field enables v1 session files to deserialize with 0.5 default"

patterns-established:
  - "Schema versioning: version field bump + backward-compat serde defaults for additive fields"

requirements-completed: [SESS-02]

duration: 3min
completed: 2026-03-26
---

# Phase 06 Plan 01: Session Save Tree Topology Summary

**Version 2 session format with real split tree serialization, divider ratios from GtkPaned, CWD via /proc, and active_pane_uuid per workspace**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-26T19:10:42Z
- **Completed:** 2026-03-26T19:13:58Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- SplitNodeData::Split now carries a ratio: f64 field capturing real GtkPaned divider positions
- to_data() captures CWD via /proc child process scanning and shell from $SHELL
- trigger_session_save() writes version 2 with real tree topology instead of dummy leaf
- active_pane_uuid tracked per workspace via find_uuid_for_pane() tree traversal
- load_session_from() accepts both version 1 and 2 for seamless upgrade

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ratio field to SplitNodeData and capture CWD in to_data()** - `99f2d1f3` (feat)
2. **Task 2: Wire real tree serialization into trigger_session_save() with version 2 and active_pane_uuid** - `b154dcc6` (feat)

## Files Created/Modified
- `src/split_engine.rs` - Added ratio field, get_surface_cwd() helper, find_uuid_for_pane(), active_pane_uuid(), updated to_data() and tests
- `src/app_state.rs` - Updated trigger_session_save() to version 2 with real tree serialization and active_pane_uuid
- `src/session.rs` - Updated load_session_from() to accept version 1 and 2

## Decisions Made
- CWD capture scans /proc for child processes of cmux PID since no direct Ghostty FFI for PTY fd exists; falls back to $HOME
- serde(default = "default_ratio") on ratio field enables v1 session files to deserialize with 0.5 default

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Session save now writes full tree topology -- ready for Plan 02's restore logic
- Version 2 format includes all data needed for layout reconstruction: orientation, ratio, CWD, shell, active_pane_uuid

---
*Phase: 06-session-layout-surface-wiring*
*Completed: 2026-03-26*
