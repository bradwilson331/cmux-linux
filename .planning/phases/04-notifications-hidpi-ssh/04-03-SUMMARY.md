---
phase: 04-notifications-hidpi-ssh
plan: 03
subsystem: api
tags: [socket, json-rpc, notifications, attention-state]

# Dependency graph
requires:
  - phase: 04-01
    provides: "has_attention field on Workspace, set_pane_attention/clear_workspace_attention on AppState"
provides:
  - "notification.list socket command returning per-workspace attention state"
  - "notification.clear socket command clearing attention by workspace UUID"
  - "has_attention field in surface.health response"
  - "pane_has_attention() method on SplitNode"
affects: [socket-cli, notification-ui, external-tooling]

# Tech tracking
tech-stack:
  added: []
  patterns: ["notification socket commands follow SOCK-05 no-focus-side-effects pattern"]

key-files:
  created: []
  modified:
    - src/socket/commands.rs
    - src/socket/mod.rs
    - src/socket/handlers.rs
    - src/split_engine.rs

key-decisions:
  - "notification.list returns workspace-level attention (not pane-level) matching macOS socket API"
  - "notification.clear takes workspace UUID as id param, consistent with workspace.select pattern"

patterns-established:
  - "Notification commands: read-only list + mutating clear, both no-focus-side-effects"

requirements-completed: [NOTF-01, NOTF-02]

# Metrics
duration: 1min
completed: 2026-03-26
---

# Phase 04 Plan 03: Notification Socket Commands Summary

**notification.list and notification.clear socket commands with per-workspace attention state and surface.health attention enrichment**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-26T13:00:26Z
- **Completed:** 2026-03-26T13:01:37Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments
- notification.list returns per-workspace attention state (uuid, name, has_attention)
- notification.clear clears attention on a workspace by UUID without focus side effects
- surface.health response enriched with has_attention field for per-pane attention querying
- pane_has_attention() method added to SplitNode for targeted pane attention checks
- Both notification methods added to system.capabilities response

## Task Commits

Each task was committed atomically:

1. **Task 1: Add notification.* socket command variants and dispatch** - `75fa47fd` (feat)

## Files Created/Modified
- `src/socket/commands.rs` - Added NotificationList and NotificationClear enum variants
- `src/socket/mod.rs` - Added dispatch rules for notification.list and notification.clear
- `src/socket/handlers.rs` - Added handlers for both commands, capabilities listing, surface.health attention
- `src/split_engine.rs` - Added pane_has_attention() method to SplitNode

## Decisions Made
- notification.list returns workspace-level attention (not pane-level) matching macOS socket API
- notification.clear takes workspace UUID as id param, consistent with workspace.select pattern

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Notification socket commands are fully operational
- External tools and CLI can now query and clear attention state
- Ready for Plan 04 (sidebar attention dots UI integration)

---
*Phase: 04-notifications-hidpi-ssh*
*Completed: 2026-03-26*

## Self-Check: PASSED
