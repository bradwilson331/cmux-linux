---
phase: 07-ssh-terminal-io
verified: 2026-03-28T14:30:00Z
status: human_needed
score: 10/10 must-haves verified
re_verification:
  previous_status: human_needed
  previous_score: 10/10
  gaps_closed: []
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Create an SSH workspace and verify bidirectional terminal I/O"
    expected: "Keystrokes appear in remote shell; remote shell output renders in local Ghostty surface"
    why_human: "Requires a running SSH connection to a real remote host with cmuxd-remote deployed"
  - test: "Kill the SSH connection during an active remote session, then let it reconnect"
    expected: "Yellow disconnect message appears, then green reconnect message with a fresh shell"
    why_human: "Requires network interruption simulation and visual verification of ANSI-colored terminal output"
  - test: "Exit the remote shell (type 'exit') and verify exit message"
    expected: "Gray '[Remote shell exited. Press any key to close]' message appears in the pane"
    why_human: "Requires active remote session and visual verification"
---

# Phase 07: SSH Terminal I/O Verification Report

**Phase Goal:** Wire SSH terminal I/O through Ghostty manual mode -- keypresses reach remote shell, output returns to surface
**Verified:** 2026-03-28T14:30:00Z
**Status:** human_needed
**Re-verification:** Yes -- regression check against previous human_needed (10/10)

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | ghostty.h contains io_mode, io_write_cb, io_write_userdata fields and ghostty_surface_process_output declaration | VERIFIED | Artifact exists, unchanged |
| 2 | bridge.rs exports PaneStream, SshBridge, WriteRequest, IoWriteContext, ssh_io_write_cb types with swappable write channel | VERIFIED | 177 lines; take_or_recreate_write_rx (line 59), clear_stream_ids (line 71), clone_write_tx (line 84), ssh_io_write_cb (line 159) all present |
| 3 | tunnel.rs sends session.spawn, proxy.stream.subscribe, proxy.write RPCs and routes proxy.stream.data/eof events | VERIFIED | 487 lines; open_remote_stream (line 332) sends session.spawn (line 346) and proxy.stream.subscribe (line 395); write loop sends proxy.write (line 240); proxy.stream.data (line 278) and proxy.stream.eof (line 290) handled |
| 4 | cmuxd-remote spawns a PTY shell when session.spawn is called and bridges it to a proxy stream | VERIFIED | main.go 1241 lines; handleSessionSpawn at line 651 |
| 5 | Remote workspace panes use Ghostty manual I/O mode surfaces | VERIFIED | app_state.rs line 232: SurfaceIoMode::Manual used in create_remote_workspace |
| 6 | Keystrokes typed in SSH pane are sent to remote shell via proxy.write | VERIFIED | ssh_io_write_cb -> bridge.write_tx -> take_or_recreate_write_rx -> run_proxy_routing write loop -> proxy.write JSON-RPC |
| 7 | Remote shell output renders in local terminal surface | VERIFIED | proxy.stream.data -> SshEvent::RemoteOutput -> main.rs line 387 -> ghostty_surface_process_output (line 395) |
| 8 | SSH disconnect shows disconnect message | VERIFIED | tunnel.rs: yellow ANSI message injected for all panes on connection drop |
| 9 | SSH reconnect starts fresh shell and shows reconnect message | VERIFIED | tunnel.rs: green reconnect message; run_proxy_routing re-entered in loop -> clear_stream_ids + open_remote_stream for all panes; take_or_recreate_write_rx creates fresh channel |
| 10 | Remote shell exit shows exit message | VERIFIED | proxy.stream.eof -> SshEvent::RemoteEof -> main.rs line 415 -> ghostty_surface_process_output (line 424) |

**Score:** 10/10 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/ssh/bridge.rs` | Per-pane stream state, swappable write channel, io_write_cb | VERIFIED | 177 lines, all key functions present |
| `src/ssh/tunnel.rs` | Bidirectional proxy protocol routing | VERIFIED | 487 lines, write path and read path both wired |
| `src/ssh/mod.rs` | SSH module root with SshEvent enum | VERIFIED | 887 bytes |
| `daemon/remote/cmd/cmuxd-remote/main.go` | PTY shell spawning | VERIFIED | 1241 lines, handleSessionSpawn present |
| `src/app_state.rs` | Remote workspace creation with manual I/O | VERIFIED | SurfaceIoMode::Manual at line 232, create_remote_workspace at line 206 |
| `src/main.rs` | GTK dispatch for RemoteOutput/RemoteEof | VERIFIED | RemoteOutput handler at line 387, RemoteEof at line 415, both call ghostty_surface_process_output |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| bridge.rs (ssh_io_write_cb) | tunnel.rs (write loop) | write_tx -> take_or_recreate_write_rx | WIRED | Regression check passed |
| tunnel.rs (run_proxy_routing) | cmuxd-remote | JSON-RPC (session.spawn, proxy.write, proxy.stream) | WIRED | All RPC methods present |
| app_state.rs | bridge.rs | SurfaceIoMode::Manual with IoWriteContext | WIRED | create_remote_workspace uses Manual mode |
| main.rs | ssh/mod.rs | SshEvent::RemoteOutput -> ghostty_surface_process_output | WIRED | Handler dispatches correctly |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| main.rs (RemoteOutput) | data: Vec<u8> | SshEvent::RemoteOutput from tunnel | Yes -- proxy.stream.data decoded from base64 | FLOWING |
| tunnel.rs (write path) | WriteRequest | bridge.take_or_recreate_write_rx() | Yes -- ssh_io_write_cb sends keystrokes | FLOWING |

### Behavioral Spot-Checks

Step 7b: SKIPPED (requires running SSH connection to remote host; not testable without server)

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-----------|-------------|--------|----------|
| SSH-03 | 07-01, 07-02, 07-03 | Terminal sessions in an SSH workspace run on the remote host | SATISFIED | Full bidirectional I/O pipeline wired: keystrokes -> ssh_io_write_cb -> write_tx -> proxy.write -> cmuxd-remote PTY; remote output -> proxy.stream.data -> RemoteOutput -> ghostty_surface_process_output |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None found | - | No TODOs, FIXMEs, or placeholders in SSH module | - | Clean |

### Human Verification Required

### 1. End-to-End SSH Terminal I/O

**Test:** Create an SSH workspace (workspace.create with remote_target) and type commands
**Expected:** Keystrokes appear in remote shell; remote shell output renders in local Ghostty surface
**Why human:** Requires a running SSH connection to a real remote host with cmuxd-remote deployed

### 2. Disconnect/Reconnect Messages

**Test:** Kill the SSH connection during an active remote session, then let it reconnect
**Expected:** Yellow disconnect message appears, then green reconnect message with a fresh shell
**Why human:** Requires network interruption simulation and visual verification of ANSI-colored terminal output

### 3. Remote Shell Exit

**Test:** Exit the remote shell (type 'exit') and verify exit message
**Expected:** Gray '[Remote shell exited. Press any key to close]' message appears in the pane
**Why human:** Requires active remote session and visual verification

### Gaps Summary

No gaps found. All 10 automated truths verified with no regressions from previous verification. The codebase state is consistent with the previous human_needed result -- all code-level checks pass, but end-to-end SSH I/O requires a live remote host for human verification.

---

_Verified: 2026-03-28T14:30:00Z_
_Verifier: Claude (gsd-verifier)_
