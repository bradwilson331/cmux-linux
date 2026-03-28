---
phase: 04-notifications-hidpi-ssh
plan: 04
subsystem: ssh
tags: [ssh, tokio, json-rpc, gtk4, scp, reconnection, exponential-backoff]

# Dependency graph
requires:
  - phase: 04-notifications-hidpi-ssh plan 01
    provides: nested sidebar row layout (GtkBox(H) > GtkBox(V) > Label + dot), attention state tracking
provides:
  - ConnectionState enum for SSH workspace lifecycle
  - SSH workspace creation via socket API (workspace.create with remote_target)
  - cmuxd-remote deployment via scp
  - SSH tunnel with JSON-RPC over stdio
  - Exponential backoff reconnection (1s-30s cap)
  - Connection state sidebar indicator with CSS classes
affects: [session-persistence, socket-api]

# Tech tracking
tech-stack:
  added: [tokio::process for SSH spawning]
  patterns: [SSH event channel (tokio mpsc to GTK timer), exponential backoff reconnection]

key-files:
  created:
    - src/ssh/mod.rs
    - src/ssh/tunnel.rs
    - src/ssh/deploy.rs
  modified:
    - src/workspace.rs
    - src/app_state.rs
    - src/main.rs
    - src/socket/commands.rs
    - src/socket/handlers.rs
    - src/socket/mod.rs

key-decisions:
  - "SSH events processed in existing 100ms GTK timer alongside bell notifications"
  - "Deploy only on first connection attempt; reconnections skip deployment"
  - "proxy.stream terminal I/O routing deferred as known gap for future work"

patterns-established:
  - "SSH event channel: tokio mpsc unbounded -> GTK timer try_recv polling"
  - "SSH lifecycle as tokio::spawn task with JoinHandle tracked for cleanup"

requirements-completed: [SSH-01, SSH-02, SSH-03, SSH-04]

# Metrics
duration: 4min
completed: 2026-03-26
---

# Phase 04 Plan 04: SSH Remote Workspaces Summary

**SSH workspace lifecycle with socket-driven creation, cmuxd-remote scp deployment, JSON-RPC stdio tunnel, exponential backoff reconnection, and connection state sidebar indicator**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-26T13:03:36Z
- **Completed:** 2026-03-26T13:07:44Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments
- ConnectionState enum (Local/Connected/Disconnected/Reconnecting) with CSS classes for sidebar display
- SSH workspace creation via workspace.create socket command with remote_target parameter
- cmuxd-remote binary deployment via scp with remote directory setup
- SSH tunnel with JSON-RPC handshake (system.hello) and response reading over stdio
- Exponential backoff reconnection (1s, 2s, 4s, 8s, 16s, 30s cap)
- SSH task lifecycle management with JoinHandle cleanup on workspace close

## Task Commits

Each task was committed atomically:

1. **Task 1: SSH data model, connection state, and sidebar indicator** - `74798d59` (feat)
2. **Task 2: SSH module, deployment, and event channel wiring** - `2dd683a0` (feat)
3. **Task 3: SSH tunnel, reconnection, and socket API integration** - `5444ee87` (feat)

## Files Created/Modified
- `src/ssh/mod.rs` - SSH module: SshEvent enum, SshEventTx/Rx channel types
- `src/ssh/tunnel.rs` - SSH lifecycle: run_ssh_lifecycle, start_ssh, backoff_duration
- `src/ssh/deploy.rs` - cmuxd-remote deployment via scp to remote host
- `src/workspace.rs` - ConnectionState enum, remote_target field, Workspace::new_remote()
- `src/app_state.rs` - build_sidebar_row, create_remote_workspace, update_connection_state, SSH cleanup
- `src/main.rs` - SSH event channel wiring, connection-state CSS
- `src/socket/commands.rs` - remote_target field on WorkspaceCreate
- `src/socket/handlers.rs` - SSH lifecycle spawn on remote workspace creation
- `src/socket/mod.rs` - remote_target extraction from workspace.create params

## Decisions Made
- SSH events processed in existing 100ms GTK timer alongside bell notifications -- avoids adding a separate polling loop
- Deploy only on first connection attempt; reconnections skip deployment -- avoids unnecessary scp on transient network interruptions
- proxy.stream terminal I/O routing deferred as known gap -- Phase 4 MVP logs RPC responses, full proxy routing requires terminal surface I/O integration

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Created SSH module files in Task 1 instead of Task 2**
- **Found during:** Task 1
- **Issue:** app_state.rs references crate::ssh::SshEventTx which requires the ssh module to exist for compilation
- **Fix:** Created ssh/mod.rs, ssh/deploy.rs, and ssh/tunnel.rs (stub) in Task 1 alongside the data model changes
- **Files modified:** src/ssh/mod.rs, src/ssh/deploy.rs, src/ssh/tunnel.rs
- **Verification:** cargo test passes
- **Committed in:** 74798d59 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Moved file creation earlier for compilation. No scope creep.

## Known Stubs

- `src/ssh/tunnel.rs` line 78-82: JSON-RPC responses from cmuxd-remote are logged but not routed to terminal surfaces (TODO comment). Full proxy.stream routing requires terminal surface I/O integration tracked for future work.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- SSH workspace lifecycle complete for socket-driven creation
- Full proxy.stream terminal I/O routing is a known gap for future enhancement
- cmuxd-remote binary must be pre-compiled and placed at ~/.local/share/cmux/bin/cmuxd-remote-linux-amd64

---
*Phase: 04-notifications-hidpi-ssh*
*Completed: 2026-03-26*
