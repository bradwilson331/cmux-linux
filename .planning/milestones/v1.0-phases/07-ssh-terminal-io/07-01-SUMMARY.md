---
phase: 07-ssh-terminal-io
plan: 01
subsystem: ssh
tags: [ssh, pty, ghostty, ffi, json-rpc, base64, go, rust]

# Dependency graph
requires:
  - phase: 04-notifications-hidpi-ssh
    provides: SSH connection lifecycle and deploy infrastructure
provides:
  - ghostty.h manual I/O mode FFI types (io_mode, io_write_cb, process_output)
  - SshBridge per-pane stream state management with bidirectional lookup
  - IoWriteContext and ssh_io_write_cb for C-compatible Ghostty callback
  - Bidirectional proxy protocol routing in tunnel.rs
  - cmuxd-remote PTY shell spawning via session.spawn RPC
  - stream.resize RPC for PTY window size changes
affects: [07-ssh-terminal-io]

# Tech tracking
tech-stack:
  added: [base64 0.22.1 (Rust), creack/pty v1.1.24 (Go)]
  patterns: [Arc<Mutex<HashMap>> for bidirectional stream-to-pane mapping, oneshot channels for RPC response routing, ptyConn net.Conn adapter for PTY fd]

key-files:
  created:
    - src/ssh/bridge.rs
    - daemon/remote/go.sum
  modified:
    - ghostty.h
    - Cargo.toml
    - Cargo.lock
    - src/ssh/mod.rs
    - src/ssh/tunnel.rs
    - src/main.rs
    - src/socket/handlers.rs
    - daemon/remote/cmd/cmuxd-remote/main.go
    - daemon/remote/go.mod

key-decisions:
  - "Per-workspace SshBridge created at workspace.create time with dedicated write/output channels"
  - "ptyConn adapter wraps os.File as net.Conn so existing streamState/streamPump infrastructure is reused"
  - "PendingMap (Arc<Mutex<HashMap<u64, oneshot::Sender>>>) for correlating RPC responses to requests"

patterns-established:
  - "SshBridge bidirectional mapping: streams (pane_id->PaneStream) + stream_to_pane (stream_id->pane_id)"
  - "WriteRequest channel pattern: user types -> io_write_cb -> base64 encode -> mpsc -> tunnel write path -> JSON-RPC"

requirements-completed: [SSH-03]

# Metrics
duration: 10min
completed: 2026-03-27
---

# Phase 07 Plan 01: SSH Terminal I/O Foundation Summary

**Manual I/O mode FFI types in ghostty.h, SshBridge stream management, cmuxd-remote PTY spawn via session.spawn RPC, and bidirectional proxy routing in tunnel.rs**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-27T01:06:20Z
- **Completed:** 2026-03-27T01:16:09Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- Added manual I/O mode types to ghostty.h (io_mode enum, io_write_cb typedef, process_output declaration) preserving Linux GTK4 platform extensions
- Created bridge.rs with SshBridge for per-pane stream state, IoWriteContext for C callback userdata, and ssh_io_write_cb for Ghostty keyboard input routing
- Enhanced cmuxd-remote with session.spawn RPC that creates PTY shells with configurable size, plus stream.resize for window resize
- Rewrote tunnel.rs with bidirectional proxy routing: read path parses events (proxy.stream.data/eof/error), write path sends proxy.write RPCs, and open_remote_stream helper chains session.spawn + proxy.stream.subscribe

## Task Commits

Each task was committed atomically:

1. **Task 1: Update ghostty.h and create bridge.rs module with types** - `100411f9` (feat)
2. **Task 2: Enhance cmuxd-remote with PTY spawn and extend tunnel.rs proxy routing** - `489d61fe` (feat)
3. **Fix: Include ghostty.h changes in main repo preserving GTK4 platform** - `33a7a57e` (fix)

## Files Created/Modified
- `ghostty.h` - Added ghostty_surface_io_mode_e, ghostty_io_write_cb, io_mode/io_write_cb/io_write_userdata fields, ghostty_surface_process_output
- `Cargo.toml` - Added base64 0.22.1 dependency
- `src/ssh/bridge.rs` - New: SshBridge, PaneStream, WriteRequest, OutputEvent, IoWriteContext, ssh_io_write_cb
- `src/ssh/mod.rs` - Added bridge module, RemoteOutput/RemoteEof/StreamOpened event variants
- `src/ssh/tunnel.rs` - Rewritten with bidirectional proxy routing and open_remote_stream helper
- `src/main.rs` - Handle new SshEvent variants in GTK event loop
- `src/socket/handlers.rs` - Create per-workspace SshBridge at workspace.create
- `daemon/remote/cmd/cmuxd-remote/main.go` - Added session.spawn, stream.resize RPCs, ptyConn adapter
- `daemon/remote/go.mod` - Added creack/pty v1.1.24
- `daemon/remote/go.sum` - New: dependency checksums

## Decisions Made
- Per-workspace SshBridge created at workspace.create time -- each SSH workspace gets its own bridge with dedicated write/output channels
- ptyConn adapter wraps os.File (PTY master) as net.Conn so existing streamState and streamPump goroutine infrastructure can be reused without modification
- PendingMap using Arc<Mutex<HashMap<u64, oneshot::Sender>>> correlates RPC request IDs to response channels for async request/response over SSH

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Non-exhaustive match on SshEvent in main.rs**
- **Found during:** Task 1
- **Issue:** Adding new SshEvent variants (RemoteOutput, RemoteEof, StreamOpened) broke the existing match in the GTK timer callback
- **Fix:** Added match arms with TODO comments for Plan 02 wiring
- **Files modified:** src/main.rs
- **Verification:** cargo check passes
- **Committed in:** 100411f9 (Task 1 commit)

**2. [Rule 1 - Bug] ghostty.h copied from macOS worktree removed Linux GTK4 platform types**
- **Found during:** Task 2 verification
- **Issue:** Copying ghostty.h from the macOS worktree removed GHOSTTY_PLATFORM_GTK4, ghostty_platform_gtk4_s, and the gtk4 union field that the Linux port requires
- **Fix:** Reverted to main repo's ghostty.h and applied changes surgically (io_mode enum, io_write_cb, process_output) without touching platform types
- **Files modified:** ghostty.h
- **Verification:** cargo check passes, all FFI bindings resolve
- **Committed in:** 33a7a57e

**3. [Rule 3 - Blocking] Socket handler needed SshBridge argument for run_ssh_lifecycle**
- **Found during:** Task 2
- **Issue:** Updated run_ssh_lifecycle signature requires Arc<SshBridge> parameter; existing call site in handlers.rs had 3 args
- **Fix:** Created per-workspace bridge with write_tx/output_tx channels at workspace creation
- **Files modified:** src/socket/handlers.rs
- **Verification:** cargo check passes
- **Committed in:** 489d61fe (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 blocking)
**Impact on plan:** All auto-fixes necessary for correctness. No scope creep.

## Issues Encountered
None beyond the deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All types and infrastructure ready for Plan 02 to wire remote surfaces
- bridge.rs exports all types Plan 02 needs: SshBridge, IoWriteContext, ssh_io_write_cb
- tunnel.rs open_remote_stream helper ready to be called when creating remote panes
- cmuxd-remote session.spawn tested via go build; ready for integration testing

---
*Phase: 07-ssh-terminal-io*
*Completed: 2026-03-27*
