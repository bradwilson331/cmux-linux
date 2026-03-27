# Phase 7: SSH Terminal I/O - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement bidirectional terminal I/O routing so SSH workspace panes run shells on the remote host. The SSH tunnel infrastructure (deploy, connect, handshake, reconnect) is already built in Phase 4. This phase wires the proxy.stream protocol to Ghostty surfaces via virtual PTY pairs, completing SSH-03.

No new SSH connection logic. No GUI SSH configuration. No session resumption. Just the I/O bridge.

</domain>

<decisions>
## Implementation Decisions

### Surface I/O Interception

- **D-01:** ~~Use a virtual PTY bridge~~ **OVERRIDDEN:** Use Ghostty's native manual I/O mode (`io_mode = GHOSTTY_SURFACE_IO_MANUAL`) with `io_write_cb` callback for SSH surfaces. Research found this eliminates PTY pair management, reduces latency (direct callback vs master/slave copy), and uses a purpose-built API. Requires updating `ghostty.h` with 3 fields + 1 function from fork header.
- **D-02:** ~~The PTY bridge lives inside `tunnel.rs`~~ **OVERRIDDEN:** SSH I/O bridge logic lives in a new `src/ssh/bridge.rs` module. `tunnel.rs` handles proxy protocol routing; `bridge.rs` manages per-pane stream state and the `io_write_cb` → `proxy.write` path. Separation keeps tunnel lifecycle and I/O bridging concerns distinct.
- **D-03:** One shell per pane — each remote pane gets its own `proxy.open` stream through the same SSH tunnel. Splitting a remote pane opens another remote shell. Matches local pane behavior exactly.

### Protocol Flow

- **D-04:** Use the proxy pattern: `proxy.open` → get `stream_id` → `proxy.stream.subscribe` → read events flow back as `proxy.stream.data` → `proxy.write` sends keystrokes. Simple, stateless on the wire, fully implemented in the Go daemon.
- **D-05:** Terminal resize handled via SIGWINCH-over-proxy — send a special escape sequence or out-of-band message through the proxy stream when the local pane resizes. cmuxd-remote needs a small enhancement to handle resize signals for proxy streams.

### Error Handling & UX

- **D-06:** On SSH disconnect: freeze the terminal surface (keep showing last output) and write a visible status message (e.g., `[SSH disconnected — reconnecting...]`) into the PTY. Resume I/O when tunnel reconnects.
- **D-07:** After reconnect: start a fresh shell via new `proxy.open` stream. The old remote shell is presumed gone. Write `[Reconnected — new session]` to the PTY. No session resumption complexity.
- **D-08:** On remote shell exit (`proxy.stream.eof`): keep the pane open with `[Remote shell exited. Press any key to close]` message. Gives user a chance to see final output before pane disappears.

### Claude's Discretion

- PTY creation mechanism (`nix::pty::openpty` vs `libc::openpty` vs tokio async PTY)
- Base64 encode/decode buffer sizing for proxy.write/proxy.stream.data
- Exact escape sequence format for resize-over-proxy (could be a custom JSON-RPC method like `proxy.resize` instead)
- How to pass the PTY slave fd to Ghostty's surface creation (may need `ghostty_surface_config_s` with custom command/fd)
- Whether to enhance cmuxd-remote's proxy.open to accept `cols`/`rows` params for initial terminal size

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §SSH Remote Workspaces — SSH-03 (the only pending requirement)

### Roadmap
- `.planning/ROADMAP.md` §Phase 7 — success criteria, phase goal

### Prior Phase Context
- `.planning/phases/04-notifications-hidpi-ssh/04-CONTEXT.md` — SSH decisions D-12 through D-15 (daemon reuse, socket-driven config, auto-reconnect, lifecycle)

### Existing SSH Implementation (read before modifying)
- `src/ssh/tunnel.rs` — SSH lifecycle with TODO at lines 73-74 for I/O routing
- `src/ssh/deploy.rs` — cmuxd-remote deployment to remote host
- `src/ssh/mod.rs` — SshEvent enum, event channels
- `src/workspace.rs` — `remote_target: Option<String>`, `ConnectionState` enum
- `src/app_state.rs` — Workspace create with remote target (line 197+)
- `src/socket/handlers.rs` — `workspace.create` with `remote_target` param (line 88+)

### Go Daemon Protocol (the remote side)
- `daemon/remote/cmd/cmuxd-remote/main.go` — Full RPC server: `proxy.open`, `proxy.write`, `proxy.close`, `proxy.stream.subscribe`, `streamPump()` for async read events, base64 data encoding

### Ghostty Surface Creation
- `src/ghostty/surface.rs` — `create_surface()` function, GLArea realize callback
- `src/ghostty/ffi.rs` — `ghostty_surface_config_s`, `ghostty_surface_config_new`, surface creation FFI

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tunnel.rs:run_ssh_lifecycle()` — Already connects, deploys, handshakes, reads JSON-RPC lines. The I/O bridge plugs directly into the existing read loop (line 67-85).
- `tunnel.rs:start_ssh()` — Returns `Child` with piped stdin/stdout — the SSH transport is ready.
- `SshEvent` enum + channel — State change mechanism for workspace connection status. Can be extended for per-pane I/O events.
- `create_surface()` in `surface.rs` — Creates GLArea + Ghostty surface. May need a variant that accepts a pre-created PTY fd instead of spawning a shell.

### Established Patterns
- Tokio tasks for async I/O (`run_ssh_lifecycle` is already a tokio task)
- `mpsc::UnboundedSender` for cross-task communication (SSH events → GTK main thread)
- JSON-RPC line protocol over stdin/stdout (already parsed in tunnel.rs)
- Base64 encoding for binary data in proxy protocol (Go daemon handles encode/decode)

### Integration Points
- `tunnel.rs` line 73-74 — The TODO where proxy.stream routing plugs in
- `app_state.rs` workspace creation — Needs to create remote-aware split engines with virtual PTYs
- `split_engine.rs` — Remote pane leaves need PTY fd association for I/O bridge
- `surface.rs:create_surface()` — May need overload accepting external PTY fd

</code_context>

<specifics>
## Specific Ideas

- Virtual PTY bridge: `openpty()` → Ghostty gets slave fd, tokio task reads/writes master fd, shuttles bytes to/from SSH tunnel as base64
- Freeze-on-disconnect: write `\r\n[SSH disconnected — reconnecting...]\r\n` directly to PTY master, which Ghostty renders as terminal output
- One SSH tunnel per workspace, multiple proxy streams per tunnel (one per pane)
- cmuxd-remote's `proxy.open` connects to `localhost:{port}` on remote — need to decide what port (likely a shell spawner or cmuxd-remote's own PTY manager)

</specifics>

<deferred>
## Deferred Ideas

- Session resumption after disconnect (keep remote shell alive via tmux/screen on remote)
- GUI SSH configuration dialog
- SSH key management UI
- Remote workspace CWD synchronization
- Rust rewrite of cmuxd-remote
- Multi-hop SSH tunneling

</deferred>

---

*Phase: 07-ssh-terminal-io*
*Context gathered: 2026-03-26 via discuss-phase*
