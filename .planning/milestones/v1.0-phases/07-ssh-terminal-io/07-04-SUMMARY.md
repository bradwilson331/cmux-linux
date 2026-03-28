---
phase: 07-ssh-terminal-io
plan: 04
subsystem: ssh
tags: [ssh, bridge, pane-registration, terminal-io]

requires:
  - phase: 07-ssh-terminal-io plan 03
    provides: SSH tunnel lifecycle with run_proxy_routing and open_remote_stream
provides:
  - Initial SSH pane registered in bridge.streams at workspace creation time
  - run_proxy_routing can find and open remote streams for initial pane
affects: [ssh-terminal-io, session-persistence]

tech-stack:
  added: []
  patterns: [placeholder-then-overwrite pane registration for SSH bridge]

key-files:
  created: []
  modified:
    - src/ssh/bridge.rs
    - src/app_state.rs

key-decisions:
  - "Placeholder PaneStream with empty stream_id inserted at creation time; overwritten by register_pane when proxy.open succeeds"

patterns-established:
  - "Placeholder registration: register pane in bridge.streams before SSH handshake so proxy routing discovers it"

requirements-completed: [SSH-03]

duration: 1min
completed: 2026-03-28
---

# Phase 07 Plan 04: SSH Pane Registration Fix Summary

**Fixed SSH terminal I/O pipeline by registering initial remote pane in bridge.streams before SSH handshake, unblocking keystroke and output routing**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-28T04:35:02Z
- **Completed:** 2026-03-28T04:36:21Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Added `register_pane_placeholder` method to SshBridge that inserts a pane_id with empty PaneStream
- Called `register_pane_placeholder` from `create_remote_workspace` before returning, so `run_proxy_routing` finds the pane after SSH handshake
- Closes the root cause of UAT Test 1 failure: keystrokes not reaching remote shell because `open_remote_stream` was never called for the initial pane

## Task Commits

Each task was committed atomically:

1. **Task 1: Add register_pane_placeholder to SshBridge and call it from create_remote_workspace** - `96d7e72a` (feat)

## Files Created/Modified
- `src/ssh/bridge.rs` - Added `register_pane_placeholder` method for pre-handshake pane registration
- `src/app_state.rs` - Added `bridge.register_pane_placeholder(pane_id)` call in `create_remote_workspace`

## Decisions Made
- Placeholder PaneStream uses empty `stream_id` and `subscribed: false`; existing `clear_stream_ids` and `register_pane` handle the overwrite when proxy.open succeeds

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- SSH terminal I/O pipeline is now complete: pane registration -> SSH handshake -> run_proxy_routing finds pane -> open_remote_stream -> stream subscribed -> keystrokes flow via proxy.write, output flows via proxy.stream.data
- Ready for end-to-end SSH testing

---
*Phase: 07-ssh-terminal-io*
*Completed: 2026-03-28*
