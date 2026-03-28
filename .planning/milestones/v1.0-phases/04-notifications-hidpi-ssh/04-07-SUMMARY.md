---
phase: 04-notifications-hidpi-ssh
plan: 07
subsystem: ssh
tags: [ssh, deploy, retry, error-handling]

# Dependency graph
requires:
  - phase: 04-notifications-hidpi-ssh
    provides: SSH workspace lifecycle and deploy infrastructure
provides:
  - Dev install script for cmuxd-remote binary
  - Permanent vs transient failure classification in SSH lifecycle
  - Max retry bound (10 attempts) preventing infinite reconnect loops
affects: [ssh, workspace]

# Tech tracking
tech-stack:
  added: []
  patterns: [FailureKind enum for permanent vs transient error classification]

key-files:
  created: [scripts/install-cmuxd-remote.sh]
  modified: [src/ssh/deploy.rs, src/ssh/tunnel.rs]

key-decisions:
  - "Classify binary-not-found as permanent failure via string match on error message"
  - "MAX_RETRIES=10 as bounded retry count for transient failures"

patterns-established:
  - "FailureKind enum: classify errors as Permanent or Transient to control retry behavior"

requirements-completed: [SSH-01, SSH-02, SSH-03, SSH-04]

# Metrics
duration: 3min
completed: 2026-03-27
---

# Phase 04 Plan 07: SSH Gap Closure Summary

**Dev install script for cmuxd-remote, permanent failure exit, and max retry bound (10 attempts) preventing infinite reconnect loops**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-27T18:40:00Z
- **Completed:** 2026-03-27T18:43:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Created dev install script (scripts/install-cmuxd-remote.sh) that builds cmuxd-remote to XDG data path
- Updated deploy error message to reference install script instead of raw Go build command
- Added FailureKind enum and permanent failure detection (binary not found exits immediately)
- Added MAX_RETRIES=10 with Disconnected state on exhaustion

## Task Commits

Each task was committed atomically:

1. **Task 1: Add cmuxd-remote dev install script and improve deploy error** - `4ddd5e63` (feat)
2. **Task 2: Add max retry and permanent failure detection to SSH lifecycle** - `3fb743aa` (feat)

## Files Created/Modified
- `scripts/install-cmuxd-remote.sh` - Dev install script for cmuxd-remote binary
- `src/ssh/deploy.rs` - Updated error message to reference install script
- `src/ssh/tunnel.rs` - Added MAX_RETRIES, FailureKind enum, permanent failure exit, max retry check

## Decisions Made
- Classify binary-not-found as permanent failure via string match on "not found at" in error message
- MAX_RETRIES=10 as bounded retry count for transient failures
- FailureKind enum keeps classification simple and extensible

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- SSH lifecycle is now bounded and exits cleanly on permanent failures
- All 34 tests pass including new test_max_retries_is_reasonable

---
*Phase: 04-notifications-hidpi-ssh*
*Completed: 2026-03-27*
