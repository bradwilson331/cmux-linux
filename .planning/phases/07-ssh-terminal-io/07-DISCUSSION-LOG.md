# Phase 7: SSH Terminal I/O - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-26
**Phase:** 07-ssh-terminal-io
**Areas discussed:** Surface I/O interception, Protocol flow, Error handling & UX

---

## Surface I/O Interception

### Q1: How should remote pane I/O be routed through Ghostty?

| Option | Description | Selected |
|--------|-------------|----------|
| Virtual PTY bridge | Create local PTY pair, pipe master to/from SSH tunnel. Ghostty reads/writes slave side as normal. | ✓ |
| Ghostty surface config override | Pass custom config that sets command to a local proxy process. | |
| Direct FFI interception | Hook into ghostty_surface_text() and render callback at FFI level. | |

**User's choice:** Virtual PTY bridge (Recommended)
**Notes:** Keeps Ghostty completely unmodified.

### Q2: Where should the PTY bridge live in the architecture?

| Option | Description | Selected |
|--------|-------------|----------|
| Inside tunnel.rs | Extend run_ssh_lifecycle() to create PTY pairs and run bridge loop. | ✓ |
| Separate proxy module | New src/ssh/proxy.rs with ProxyBridge struct. | |

**User's choice:** Inside tunnel.rs (Recommended)
**Notes:** Keeps all SSH logic in one module.

### Q3: Should remote panes support multiple shells per SSH workspace?

| Option | Description | Selected |
|--------|-------------|----------|
| One shell per pane | Each remote pane gets own proxy.open stream. Splitting opens another remote shell. | ✓ |
| Single shared session | All panes share one cmuxd-remote session. | |

**User's choice:** One shell per pane (Recommended)
**Notes:** Matches local pane behavior exactly.

---

## Protocol Flow

### Q1: Which cmuxd-remote protocol pattern should remote terminal panes use?

| Option | Description | Selected |
|--------|-------------|----------|
| Proxy pattern | proxy.open → stream_id → proxy.stream.subscribe → proxy.write. Simple, stateless, fully implemented. | ✓ |
| Session pattern | session.open → session.attach with cols/rows. Structured but sessions are metadata-only in Go code. | |
| You decide | Claude evaluates both. | |

**User's choice:** Proxy pattern (Recommended)

### Q2: How should terminal resize be handled with the proxy pattern?

| Option | Description | Selected |
|--------|-------------|----------|
| SIGWINCH over proxy | Send special escape/OOB message through proxy stream on resize. | ✓ |
| Session.resize alongside proxy | Hybrid — open session alongside proxy stream for resize coordination. | |
| You decide | Claude picks simplest approach. | |

**User's choice:** SIGWINCH over proxy (Recommended)

---

## Error Handling & UX

### Q1: What should happen to the Ghostty surface when SSH disconnects?

| Option | Description | Selected |
|--------|-------------|----------|
| Freeze + status message | Keep last output, print status line into PTY, resume on reconnect. | ✓ |
| Close pane automatically | Close remote pane on tunnel drop. | |
| You decide | Claude picks best UX. | |

**User's choice:** Freeze + status message (Recommended)

### Q2: After SSH reconnect, resume or start fresh?

| Option | Description | Selected |
|--------|-------------|----------|
| Start fresh shell | New proxy.open stream. Write '[Reconnected — new session]' to PTY. | ✓ |
| Attempt resume via session | Use session.open to try resuming previous session. | |

**User's choice:** Start fresh shell (Recommended)

### Q3: What happens when the remote shell exits?

| Option | Description | Selected |
|--------|-------------|----------|
| Close pane | Close pane on proxy.stream.eof, same as local shells. | |
| Show message + keep open | Keep pane with '[Remote shell exited. Press any key to close]' message. | ✓ |

**User's choice:** Show message + keep open

---

## Claude's Discretion

- PTY creation mechanism
- Base64 buffer sizing
- Resize escape sequence format
- How to pass PTY slave fd to Ghostty surface creation
- Whether to enhance cmuxd-remote's proxy.open with cols/rows params

## Deferred Ideas

- Session resumption after disconnect
- GUI SSH configuration dialog
- SSH key management UI
- Remote workspace CWD synchronization
- Rust rewrite of cmuxd-remote
- Multi-hop SSH tunneling
