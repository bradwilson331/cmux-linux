---
phase: 07-ssh-terminal-io
plan: 02
subsystem: ssh
tags: [ssh, ghostty, ffi, gtk4, manual-io, surface, bridge]

# Dependency graph
requires:
  - phase: 07-ssh-terminal-io
    plan: 01
    provides: SshBridge, IoWriteContext, ssh_io_write_cb, tunnel proxy routing, ghostty.h manual I/O types
provides:
  - SurfaceIoMode enum for Exec vs Manual I/O mode surface creation
  - Remote workspace creation with manual I/O mode surfaces wired to SSH bridge
  - GTK main thread dispatch for RemoteOutput, RemoteEof, StreamOpened events
  - Disconnect/reconnect/exit message injection per D-06/D-07/D-08
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: [SurfaceIoMode enum parameter on create_surface for I/O mode branching, SURFACE_REGISTRY reverse lookup for pane_id-to-surface dispatch]

key-files:
  created: []
  modified:
    - src/ghostty/surface.rs
    - src/app_state.rs
    - src/split_engine.rs
    - src/socket/handlers.rs
    - src/ssh/bridge.rs
    - src/main.rs
    - src/ssh/tunnel.rs

key-decisions:
  - "SurfaceIoMode enum added as parameter to create_surface rather than separate create_remote_surface function -- avoids duplicating 600-line function"
  - "ssh_io_write_cb signature changed to c_char to match ghostty.h FFI typedef"
  - "SURFACE_REGISTRY reverse lookup (iterate to find pane_id -> surface_ptr) used for remote output dispatch -- acceptable O(n) for small pane counts"
  - "Disconnect/reconnect messages injected by tunnel.rs into SSH event channel, rendered by main.rs GTK timer"

patterns-established:
  - "remote_pane_contexts HashMap on AppState for IoWriteContext lookup when StreamOpened arrives"
  - "workspace_bridges HashMap on AppState for SshBridge access from socket handlers"

requirements-completed: [SSH-03]

# Metrics
duration: 4min
completed: 2026-03-27
---

# Phase 07 Plan 02: SSH Surface Wiring and Event Dispatch Summary

**SurfaceIoMode enum enabling manual I/O mode Ghostty surfaces for SSH workspaces, with bidirectional event dispatch and disconnect/reconnect/exit terminal messages**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-27T01:20:50Z
- **Completed:** 2026-03-27T01:24:57Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Added SurfaceIoMode enum (Exec/Manual) to create_surface, avoiding 600-line function duplication while enabling SSH remote surfaces with io_write_cb wired to the SSH bridge
- Fixed ssh_io_write_cb signature to use c_char matching the ghostty.h FFI typedef (was u8, causing type mismatch)
- Wired RemoteOutput events to ghostty_surface_process_output for terminal rendering with GLArea queue_render
- Wired RemoteEof to display gray exit message per D-08, StreamOpened to set stream_id on IoWriteContext
- Added yellow disconnect message (D-06) and green reconnect message (D-07) injection in tunnel.rs
- Added remote_pane_contexts and workspace_bridges HashMaps to AppState for cross-component access

## Task Commits

Each task was committed atomically:

1. **Task 1: Create remote surface factory and wire app_state remote workspace creation** - `8f8877a6` (feat)
2. **Task 2: GTK main thread dispatch for remote I/O events and disconnect/reconnect/exit handling** - `dc1fe64a` (feat)

## Files Created/Modified
- `src/ghostty/surface.rs` - Added SurfaceIoMode enum, io_mode parameter on create_surface, manual I/O mode config in realize callback
- `src/app_state.rs` - Updated create_remote_workspace to accept bridge, added remote_pane_contexts and workspace_bridges fields
- `src/split_engine.rs` - Updated two create_surface call sites to pass SurfaceIoMode::Exec
- `src/socket/handlers.rs` - Moved bridge creation before workspace creation, pass bridge to create_remote_workspace
- `src/ssh/bridge.rs` - Fixed ssh_io_write_cb data parameter type from *const u8 to *const c_char
- `src/main.rs` - Replaced TODO stubs with full RemoteOutput/RemoteEof/StreamOpened event handlers
- `src/ssh/tunnel.rs` - Added disconnect message injection on connection drop, reconnect message on successful retry

## Decisions Made
- SurfaceIoMode enum parameter approach chosen over separate create_remote_surface function to avoid duplicating the entire 600-line create_surface function
- ssh_io_write_cb signature fixed to c_char to match ghostty.h typedef -- the FFI uses `const char*` which maps to c_char in Rust
- SURFACE_REGISTRY reverse lookup (O(n) scan) used for pane_id to surface pointer resolution -- acceptable for typical pane counts
- Disconnect/reconnect messages injected via SshEvent channel rather than direct surface manipulation from tunnel task

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ssh_io_write_cb type mismatch with FFI typedef**
- **Found during:** Task 1
- **Issue:** ssh_io_write_cb used `*const u8` for data parameter but ghostty.h typedef declares `const char*` which maps to `*const c_char` (i8 on Linux)
- **Fix:** Changed data parameter to `*const std::ffi::c_char` and cast to `*const u8` internally
- **Files modified:** src/ssh/bridge.rs
- **Committed in:** 8f8877a6

## Known Stubs
None -- all TODO stubs from Plan 01 have been replaced with working implementations.

---
*Phase: 07-ssh-terminal-io*
*Completed: 2026-03-27*
