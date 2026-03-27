---
phase: 07-ssh-terminal-io
plan: 03
subsystem: ssh
tags: [ssh, tokio, mpsc, channel, reconnect, proxy, terminal-io]

# Dependency graph
requires:
  - phase: 07-01
    provides: "SshBridge struct, IoWriteContext, ssh_io_write_cb, PaneStream, WriteRequest types"
  - phase: 07-02
    provides: "SurfaceIoMode::Manual, create_remote_workspace, open_remote_stream function, run_proxy_routing"
provides:
  - "Connected write channel: ssh_io_write_cb -> bridge.write_tx -> write_rx -> run_proxy_routing write loop -> proxy.write JSON-RPC"
  - "open_remote_stream called after SSH handshake for all registered panes"
  - "Reconnect-safe channel recreation via take_or_recreate_write_rx"
  - "Stream state cleanup via clear_stream_ids on reconnect"
affects: [ssh-terminal-io, session-persistence]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Swappable mpsc sender behind Arc<Mutex> for reconnect channel recreation"
    - "take-or-recreate pattern: consume receiver on first call, create fresh pair on subsequent calls"

key-files:
  created: []
  modified:
    - "src/ssh/bridge.rs"
    - "src/ssh/tunnel.rs"
    - "src/socket/handlers.rs"
    - "src/app_state.rs"

key-decisions:
  - "Arc<Mutex<UnboundedSender>> for swappable write_tx enables reconnect without restarting bridge"
  - "Known limitation: existing IoWriteContext holds old sender after reconnect; new panes work, old panes need restart"

patterns-established:
  - "take_or_recreate pattern: first call takes stored receiver, subsequent calls create fresh channel pair"

requirements-completed: [SSH-03]

# Metrics
duration: 2min
completed: 2026-03-27
---

# Phase 07 Plan 03: Gap Closure Summary

**Connected write channel and open_remote_stream call sites to complete bidirectional SSH terminal I/O pipeline**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-27T01:50:51Z
- **Completed:** 2026-03-27T01:52:32Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments
- Connected the write path: keystrokes from ssh_io_write_cb now flow through bridge.write_tx to write_rx (consumed by run_proxy_routing) to proxy.write JSON-RPC over SSH stdin
- open_remote_stream is now called for all registered panes after SSH handshake, spawning remote PTY sessions
- Reconnect creates fresh channel pair via take_or_recreate_write_rx; stale stream state cleared via clear_stream_ids

## Task Commits

Each task was committed atomically:

1. **Task 1: Connect write channel and call open_remote_stream** - `0cc971e7` (feat)

## Files Created/Modified
- `src/ssh/bridge.rs` - Made write_tx swappable (Arc<Mutex>), added write_rx storage, take_or_recreate_write_rx, clear_stream_ids, clone_write_tx methods
- `src/ssh/tunnel.rs` - Replaced disconnected local channel with bridge.take_or_recreate_write_rx(); added open_remote_stream calls after handshake
- `src/socket/handlers.rs` - Passed write_rx to SshBridge::new constructor
- `src/app_state.rs` - Used bridge.clone_write_tx() instead of direct field clone

## Decisions Made
- Arc<Mutex<UnboundedSender>> for swappable write_tx enables reconnect without restarting bridge
- Known limitation accepted: existing IoWriteContext holds old sender after reconnect; per D-07, reconnect starts a fresh shell anyway

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- SSH terminal I/O pipeline is fully wired end-to-end
- Both BLOCKER gaps from 07-VERIFICATION.md are resolved
- Ready for integration testing with actual SSH connections
- Known limitation: after reconnect, old panes' IoWriteContext points to dropped channel (acceptable per D-07)

---
*Phase: 07-ssh-terminal-io*
*Completed: 2026-03-27*

## Self-Check: PASSED
