---
phase: 04-notifications-hidpi-ssh
plan: 01
subsystem: notifications
tags: [gtk4, gnotification, bell, attention-state, sidebar]

# Dependency graph
requires:
  - phase: 02-workspaces-pane-splits
    provides: SplitNode tree, workspace sidebar, AppState with switch_to_index
  - phase: 03-socket-api-session-persistence
    provides: Session save/restore, socket command dispatch
provides:
  - Per-pane attention state (has_attention on SplitNode::Leaf)
  - Per-workspace attention state (has_attention on Workspace)
  - Bell event handler in action_cb (RING_BELL -> BELL_PENDING -> set_pane_attention)
  - Sidebar attention dot (amber circle, hidden by default)
  - Desktop notification via GNotification on unfocused window bell
  - Nested sidebar row layout ready for Plan 04 SSH status label
affects: [04-04-ssh-workspaces, 04-05-integration-verification]

# Tech tracking
tech-stack:
  added: [GNotification (gio)]
  patterns: [atomic flag bridge from action_cb to GTK main thread via glib::timeout_add_local]

key-files:
  created: []
  modified:
    - src/split_engine.rs
    - src/workspace.rs
    - src/app_state.rs
    - src/ghostty/callbacks.rs
    - src/main.rs
    - src/sidebar.rs

key-decisions:
  - "Bell processing via glib::timeout_add_local(100ms) polling BELL_PENDING atomic instead of spawn_local socket loop -- action_cb fires during ghostty_app_tick on main thread, but AppState is Rc<RefCell> not accessible from wakeup_cb idle handler"
  - "Nested sidebar row layout (GtkBox(H) > GtkBox(V) > GtkLabel + GtkLabel(dot)) established now to avoid double-refactor when Plan 04 adds SSH status label"

patterns-established:
  - "Atomic flag bridge: action_cb sets AtomicBool+AtomicU64, glib timeout polls and dispatches to AppState"
  - "Sidebar row structure: GtkBox(H,4) > [GtkBox(V,0) > [GtkLabel(name)], GtkLabel(dot)] -- all code navigating rows must use this hierarchy"

requirements-completed: [NOTF-01, NOTF-02, NOTF-03]

# Metrics
duration: 7min
completed: 2026-03-26
---

# Phase 04 Plan 01: Bell Attention Tracking Summary

**Bell-driven per-pane attention state with sidebar amber dot indicator and GNotification desktop notifications, rate-limited to 1 per workspace per 5 seconds**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-26T12:45:51Z
- **Completed:** 2026-03-26T12:52:44Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Bell events propagate from Ghostty action_cb through BELL_PENDING atomic to AppState::set_pane_attention
- Sidebar shows amber dot (8px, #e8a444) next to workspace name when any pane has unread bell activity
- Attention clears automatically when user switches to the workspace
- Desktop notification fires via GNotification when window is unfocused, rate-limited per workspace
- Sidebar row uses nested VBox layout ready for Plan 04 SSH status label

## Task Commits

Each task was committed atomically:

1. **Task 1: Add attention state to data model and action_cb bell handler** - `048160cb` (feat)
2. **Task 2: Sidebar attention dot, desktop notifications, and CSS** - `5d81c1c2` (feat)

## Files Created/Modified
- `src/split_engine.rs` - Added has_attention field to SplitNode::Leaf, set_attention/any_attention/clear_all_attention methods
- `src/workspace.rs` - Added has_attention and last_notification fields to Workspace
- `src/app_state.rs` - Added set_pane_attention, clear_workspace_attention, update_sidebar_attention, send_bell_notification; updated create_workspace/rename_active/switch_to_index for nested row layout
- `src/ghostty/callbacks.rs` - Added RING_BELL handler in action_cb, BELL_PENDING/BELL_PANE_ID atomics
- `src/main.rs` - Added .attention-dot CSS, bell processing glib timeout
- `src/sidebar.rs` - Updated inline rename for nested row layout, added rebuild_sidebar_row_content helper

## Decisions Made
- Used glib::timeout_add_local(100ms) to poll BELL_PENDING instead of integrating into spawn_local socket loop (socket loop only runs on socket commands, not suitable for bell events)
- Established nested sidebar row layout now (GtkBox(H) > GtkBox(V) > Label + dot) to avoid restructuring in Plan 04

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Bell dispatch via glib timeout instead of spawn_local socket loop**
- **Found during:** Task 1 (Bell handler integration)
- **Issue:** Plan suggested adding bell check inside spawn_local socket command loop, but that only runs when socket commands arrive -- bells would not be processed without socket traffic
- **Fix:** Used glib::timeout_add_local(100ms) to poll BELL_PENDING atomic and dispatch to AppState
- **Files modified:** src/main.rs
- **Verification:** cargo test passes, bell processing runs independently of socket commands
- **Committed in:** 048160cb (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Bell dispatch mechanism changed from socket loop to dedicated timer. Functionally equivalent, more reliable.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Known Stubs
None - all data paths are wired from action_cb through to sidebar dot and desktop notification.

## Next Phase Readiness
- Attention state infrastructure ready for Plan 05 integration verification
- Sidebar row layout ready for Plan 04 SSH status label addition
- Desktop notification infrastructure can be extended for OSC 99 in future

---
*Phase: 04-notifications-hidpi-ssh*
*Completed: 2026-03-26*

## Self-Check: PASSED
- All 6 source files verified present
- Both task commits (048160cb, 5d81c1c2) verified in git log
