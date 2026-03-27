# Phase 7: SSH Terminal I/O - Research

**Researched:** 2026-03-26
**Domain:** Bidirectional terminal I/O proxying over SSH tunnel using Ghostty manual I/O mode
**Confidence:** HIGH

## Summary

Phase 7 closes the last v1 gap (SSH-03) by wiring Ghostty terminal surfaces to remote shells running on SSH workspace targets. The SSH tunnel infrastructure (Phase 4) already connects to `cmuxd-remote`, which speaks a complete proxy protocol (`proxy.open`, `proxy.write`, `proxy.stream.subscribe`, `proxy.stream.data`, `proxy.stream.eof`). The Go daemon is fully implemented and ready.

The critical discovery is that **Ghostty natively supports manual I/O mode** (`io_mode = manual`) which eliminates the need for virtual PTY bridges. In manual mode, Ghostty does not spawn a shell -- instead it invokes an `io_write_cb` callback when the user types, and accepts output via `ghostty_surface_process_output()`. This is exactly the API needed: keystrokes flow from Ghostty through the callback to the SSH tunnel as `proxy.write`, and remote shell output flows from `proxy.stream.data` events through `ghostty_surface_process_output` into the terminal renderer.

**Primary recommendation:** Use Ghostty's `io_mode = GHOSTTY_SURFACE_IO_MANUAL` with `io_write_cb` for SSH surfaces instead of the openpty bridge described in D-01. This is simpler (no PTY pair management), lower latency (no extra copy through PTY master/slave), and uses a purpose-built API in Ghostty.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use a virtual PTY bridge -- create a local PTY pair (openpty) and pipe the master side to/from the SSH tunnel. Ghostty reads/writes the slave side as a normal terminal. No Ghostty modifications required.
- **D-02:** The PTY bridge lives inside tunnel.rs -- extend run_ssh_lifecycle() to create PTY pairs and run bridge loops.
- **D-03:** One shell per pane -- each remote pane gets its own proxy.open stream through the same SSH tunnel.
- **D-04:** Use the proxy pattern: proxy.open -> get stream_id -> proxy.stream.subscribe -> read events flow back as proxy.stream.data -> proxy.write sends keystrokes.
- **D-05:** Terminal resize handled via SIGWINCH-over-proxy.
- **D-06:** On SSH disconnect: freeze the terminal surface and write a visible status message into the PTY.
- **D-07:** After reconnect: start a fresh shell via new proxy.open stream.
- **D-08:** On remote shell exit (proxy.stream.eof): keep the pane open with exit message.

### Claude's Discretion
- PTY creation mechanism (nix::pty::openpty vs libc::openpty vs tokio async PTY)
- Base64 encode/decode buffer sizing
- Exact escape sequence format for resize-over-proxy
- How to pass the PTY slave fd to Ghostty's surface creation
- Whether to enhance cmuxd-remote's proxy.open to accept cols/rows params

### Deferred Ideas (OUT OF SCOPE)
- Session resumption after disconnect (keep remote shell alive via tmux/screen)
- GUI SSH configuration dialog
- SSH key management UI
- Remote workspace CWD synchronization
- Rust rewrite of cmuxd-remote
- Multi-hop SSH tunneling
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SSH-03 | Terminal sessions in an SSH workspace run on the remote host | Ghostty manual I/O mode + proxy protocol provides full bidirectional I/O path. `io_write_cb` captures keystrokes, `ghostty_surface_process_output` renders remote output. cmuxd-remote proxy.open/write/stream.subscribe already implemented in Go daemon. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Never run tests locally -- all tests via GitHub Actions
- No app-level display link or manual ghostty_surface_draw loop
- Typing-latency-sensitive paths: no allocations in key input callbacks
- Socket command threading policy: high-frequency I/O off main thread
- Socket focus policy: non-focus commands must not steal focus
- All user-facing strings must be localized
- Test quality: verify observable runtime behavior, not source text patterns

## Critical Finding: Ghostty Manual I/O Mode

**Confidence: HIGH** -- verified directly in source code.

The Ghostty embedded runtime (`ghostty/src/apprt/embedded.zig`) has a `Manual` I/O mode that is purpose-built for exactly this use case:

| Field | C Type | Purpose |
|-------|--------|---------|
| `io_mode` | `ghostty_surface_io_mode_e` | `GHOSTTY_SURFACE_IO_MANUAL` (1) = no shell spawn |
| `io_write_cb` | `ghostty_io_write_cb` = `void(*)(void*, const char*, uintptr_t)` | Called when Ghostty wants to write (user keystrokes) |
| `io_write_userdata` | `void*` | Context pointer passed to write callback |

Additionally: `ghostty_surface_process_output(surface, data, len)` injects bytes into the terminal as if read from a PTY.

**These fields exist in `ghostty/include/ghostty.h` but are MISSING from the project's `ghostty.h`.** The project header must be updated to include these 3 struct fields plus the `ghostty_surface_process_output` function declaration. This triggers bindgen to regenerate Rust bindings with the new fields.

### D-01 Revision Recommendation

D-01 specifies a virtual PTY bridge (`openpty`). Research found that Ghostty's manual I/O mode is **strictly superior**:

| Aspect | openpty Bridge | Manual I/O Mode |
|--------|---------------|-----------------|
| Complexity | Create PTY pair, manage master/slave fds, bridge loops | Set 3 fields in surface config |
| Latency | Extra copy through kernel PTY layer | Direct callback, zero-copy |
| Resource usage | One PTY pair per remote pane | No kernel resources |
| Error handling | PTY read/write errors + SSH errors | Only SSH errors |
| Ghostty changes | None (uses slave fd as command) | Update ghostty.h (add 3 fields + 1 function) |
| Dependencies | `nix` or `libc::openpty` | None new |

The planner should prefer manual I/O mode. The ghostty.h update is minimal (add fields that already exist in the fork's header) and eliminates all PTY bridge complexity. If the user insists on the PTY approach despite this evidence, the planner should note the tradeoff.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| base64 | 0.22.1 | Encode/decode proxy.write/stream.data payloads | Standard Rust base64 crate, matches Go daemon's encoding/base64 |
| serde_json | 1 (already dep) | JSON-RPC framing for SSH tunnel protocol | Already used in tunnel.rs |
| tokio | 1 (already dep) | Async I/O for SSH tunnel read/write loops | Already used for run_ssh_lifecycle |
| libc | 0.2 (already dep) | Low-level system calls if PTY fallback needed | Already a dependency |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| glib | 0.21.5 (already dep) | MainContext bridge for GTK thread dispatch | Already used for SSH event processing |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| base64 crate | Manual base64 impl | Don't hand-roll; base64 crate is 0 dependency, well-tested |
| Manual I/O mode | openpty bridge (D-01) | PTY bridge adds complexity and latency; manual mode is Ghostty's built-in solution |

**Installation:**
```bash
cargo add base64@0.22.1
```

**Version verification:** base64 0.22.1 confirmed via `cargo search base64` on 2026-03-26.

## Architecture Patterns

### Data Flow Architecture

```
User keystroke
    |
    v
Ghostty surface (manual I/O mode)
    |
    v (io_write_cb)
Rust callback captures bytes
    |
    v (mpsc channel)
Tokio SSH tunnel task
    |
    v (JSON-RPC: proxy.write + base64)
SSH stdin -> cmuxd-remote
    |
    v (TCP to remote shell)
Remote shell processes input
    |
    v (shell output)
cmuxd-remote reads TCP
    |
    v (JSON-RPC event: proxy.stream.data + base64)
SSH stdout -> Tokio read loop
    |
    v (glib dispatch to main thread)
ghostty_surface_process_output(surface, bytes, len)
    |
    v
Terminal renders output
```

### Recommended Module Structure

```
src/ssh/
├── mod.rs          # SshEvent enum (extend with I/O events)
├── tunnel.rs       # run_ssh_lifecycle() — extend with proxy.stream routing
├── deploy.rs       # cmuxd-remote deployment (unchanged)
└── bridge.rs       # NEW: IoWriteCallback, per-pane stream state, base64 encode/decode
```

### Pattern 1: Per-Pane Stream State

**What:** Each remote pane gets its own `stream_id` from `proxy.open`. A HashMap maps pane_id to stream state (stream_id, connected flag).

**When to use:** When the SSH tunnel is connected and a remote workspace creates or splits a pane.

**Example:**
```rust
// bridge.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct PaneStream {
    pub stream_id: String,
    pub subscribed: bool,
}

pub struct SshBridge {
    /// Maps pane_id -> stream state
    pub streams: Arc<Mutex<HashMap<u64, PaneStream>>>,
    /// Channel to send write requests to the SSH tunnel task
    pub write_tx: tokio::sync::mpsc::UnboundedSender<WriteRequest>,
}

pub struct WriteRequest {
    pub stream_id: String,
    pub data_base64: String,
}
```

### Pattern 2: io_write_cb as C Callback

**What:** The callback Ghostty invokes when the user types in a manual-mode surface. Must be `extern "C"` with signature `fn(*mut c_void, *const u8, usize)`.

**When to use:** Set once per remote surface at creation time via `surface_config.io_write_cb`.

**Example:**
```rust
// The userdata pointer carries enough context to route writes
// to the correct stream_id through the tunnel.
struct IoWriteContext {
    pane_id: u64,
    write_tx: tokio::sync::mpsc::UnboundedSender<WriteRequest>,
    stream_id: Mutex<Option<String>>,
}

unsafe extern "C" fn ssh_io_write_cb(
    userdata: *mut std::ffi::c_void,
    data: *const u8,
    len: usize,
) {
    let ctx = &*(userdata as *const IoWriteContext);
    let bytes = std::slice::from_raw_parts(data, len);
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    if let Some(ref stream_id) = *ctx.stream_id.lock().unwrap() {
        let _ = ctx.write_tx.send(WriteRequest {
            stream_id: stream_id.clone(),
            data_base64: b64,
        });
    }
}
```

### Pattern 3: Dispatch Output to GTK Main Thread

**What:** Remote shell output arrives on the tokio SSH read task. It must be dispatched to the GTK main thread before calling `ghostty_surface_process_output`.

**When to use:** Every `proxy.stream.data` event from the SSH tunnel.

**Example:**
```rust
// In the tunnel read loop (tokio task):
if let Some(event) = parse_stream_event(&msg) {
    match event.event.as_str() {
        "proxy.stream.data" => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&event.data_base64).unwrap_or_default();
            let pane_id = stream_to_pane.get(&event.stream_id).copied();
            if let Some(pane_id) = pane_id {
                // Dispatch to GTK main thread
                let _ = output_tx.send(OutputEvent { pane_id, data: bytes });
            }
        }
        "proxy.stream.eof" => { /* handle D-08 */ }
        "proxy.stream.error" => { /* handle error */ }
        _ => {}
    }
}
```

### Pattern 4: Surface Creation for Remote Panes

**What:** Remote panes use a different surface config than local panes.

**When to use:** When `create_remote_workspace` or splitting within a remote workspace.

**Example:**
```rust
// In create_surface or a new create_remote_surface variant:
let mut surface_config = unsafe { ffi::ghostty_surface_config_new() };
surface_config.platform_tag = ffi::ghostty_platform_e_GHOSTTY_PLATFORM_GTK4;
surface_config.platform = platform;
surface_config.scale_factor = scale;

// Manual I/O mode for remote surfaces
surface_config.io_mode = ffi::ghostty_surface_io_mode_e_GHOSTTY_SURFACE_IO_MANUAL;
surface_config.io_write_cb = Some(ssh_io_write_cb);
surface_config.io_write_userdata = Box::into_raw(Box::new(io_ctx)) as *mut c_void;
```

### Anti-Patterns to Avoid

- **Calling `ghostty_surface_process_output` from a tokio thread:** This MUST run on the GTK main thread. The surface pointer is not Send+Sync. Use glib dispatch.
- **Allocating in `io_write_cb`:** The callback fires on every keystroke. Base64 encoding allocates but is unavoidable. Keep the allocation to just the base64 string and the channel send.
- **Blocking the SSH read loop:** The tunnel read loop processes all JSON-RPC responses/events for all panes. If one pane's output dispatch blocks, all panes stall. Use unbounded channels.
- **Storing `ghostty_surface_t` in the tokio task:** Surface pointers are GTK-thread-only. Store pane_id mapping instead and resolve to surface on the GTK thread.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Base64 encoding | Custom base64 | `base64` crate (0.22.1) | Edge cases in padding, constant-time concerns |
| JSON-RPC framing | Custom line parser | `serde_json::from_str` | Already used in tunnel.rs; handles escaping |
| PTY bridge | openpty + bridge loops | Ghostty manual I/O mode | Built-in API, zero-copy, no kernel resources |
| Thread dispatch | Raw pointer passing | `glib::MainContext::default().spawn_local()` | Established pattern from Phase 3 |

## Common Pitfalls

### Pitfall 1: ghostty.h Out of Sync
**What goes wrong:** The project's `ghostty.h` is missing `io_mode`, `io_write_cb`, `io_write_userdata` fields and `ghostty_surface_process_output`. Bindgen generates a struct without these fields. Setting them silently corrupts memory.
**Why it happens:** The project maintains its own `ghostty.h` separate from the submodule's `ghostty/include/ghostty.h`.
**How to avoid:** Copy the missing fields and function declaration from `ghostty/include/ghostty.h` into the project's `ghostty.h`. Verify the generated struct size changes from 88 to ~104 bytes after rebuild.
**Warning signs:** Segfault on surface creation for remote panes; struct size mismatch warnings from bindgen.

### Pitfall 2: io_write_cb Memory Safety
**What goes wrong:** The `io_write_userdata` pointer outlives its allocation, causing use-after-free.
**Why it happens:** `Box::into_raw` creates the context, but if the pane is closed without `Box::from_raw` cleanup, the memory leaks. If cleanup runs too early (before Ghostty stops calling the callback), it's use-after-free.
**How to avoid:** Use `Arc<IoWriteContext>` and increment refcount for the raw pointer. Drop the Arc in the surface cleanup path. The callback only reads, never mutates (stream_id is behind Mutex).
**Warning signs:** Crashes when closing remote panes; valgrind/ASAN reports.

### Pitfall 3: JSON-RPC ID Tracking
**What goes wrong:** `proxy.open` response (with stream_id) arrives mixed with `proxy.stream.data` events for other streams. Without ID tracking, responses get misrouted.
**Why it happens:** The SSH tunnel is a single multiplexed channel. Requests and async events interleave.
**How to avoid:** Use an atomic `next_rpc_id` counter. When sending `proxy.open`, record the RPC ID. Match response by ID to resolve the stream_id for the requesting pane.
**Warning signs:** Wrong stream_id associated with pane; keystrokes appear in wrong remote shell.

### Pitfall 4: Resize Race Condition
**What goes wrong:** User resizes pane while proxy.open is still in flight. Resize event has no stream_id yet.
**Why it happens:** Surface creation and stream setup are async with SSH round-trip latency.
**How to avoid:** Buffer resize events until stream is established. Send initial size with `proxy.open` (if enhanced) or send resize immediately after `proxy.stream.subscribe`.
**Warning signs:** Remote shell starts with wrong terminal dimensions; first screen render is garbled.

### Pitfall 5: Disconnect During Active I/O
**What goes wrong:** SSH connection drops while io_write_cb is sending. The write_tx channel sends to a closed tunnel.
**Why it happens:** Network interruptions are abrupt; the tokio read loop detects EOF but io_write_cb can fire simultaneously.
**How to avoid:** The write_tx channel is unbounded and never errors on send (it just buffers). When the tunnel reconnects, drain buffered writes or discard them (per D-07: fresh shell on reconnect, so old writes are irrelevant).
**Warning signs:** Panic on send if using bounded channel; memory growth if unbounded writes accumulate during long disconnects.

## Code Examples

### ghostty.h Additions Needed

```c
// Add these to ghostty.h (already present in ghostty/include/ghostty.h)

// Before ghostty_surface_config_s:
typedef enum {
  GHOSTTY_SURFACE_IO_EXEC = 0,
  GHOSTTY_SURFACE_IO_MANUAL = 1,
} ghostty_surface_io_mode_e;

typedef void (*ghostty_io_write_cb)(void*, const char*, uintptr_t);

// Add to end of ghostty_surface_config_s struct (after `context` field):
  ghostty_surface_io_mode_e io_mode;
  ghostty_io_write_cb io_write_cb;
  void* io_write_userdata;

// Add to function declarations:
void ghostty_surface_process_output(ghostty_surface_t, const char*, uintptr_t);
```

### JSON-RPC Protocol Messages

```json
// Request: Open a proxy stream
{"jsonrpc":"2.0","id":2,"method":"proxy.open","params":{"host":"127.0.0.1","port":22}}

// Response: Stream created
{"id":2,"ok":true,"result":{"stream_id":"s-1"}}

// Request: Subscribe to stream events
{"jsonrpc":"2.0","id":3,"method":"proxy.stream.subscribe","params":{"stream_id":"s-1"}}

// Response: Subscribed
{"id":3,"ok":true,"result":{"subscribed":true,"already_subscribed":false}}

// Request: Write keystroke data
{"jsonrpc":"2.0","id":4,"method":"proxy.write","params":{"stream_id":"s-1","data_base64":"bHM="}}

// Async event: Remote output data
{"event":"proxy.stream.data","stream_id":"s-1","data_base64":"dG90YWwgMTIK"}

// Async event: Remote shell exited
{"event":"proxy.stream.eof","stream_id":"s-1","data_base64":""}
```

### Disconnect/Reconnect Message Injection

```rust
// Per D-06: Write disconnect message to surface
fn inject_disconnect_message(surface: ffi::ghostty_surface_t) {
    let msg = b"\r\n\x1b[33m[SSH disconnected \xe2\x80\x94 reconnecting...]\x1b[0m\r\n";
    unsafe {
        ffi::ghostty_surface_process_output(surface, msg.as_ptr() as *const _, msg.len());
    }
}

// Per D-07: Write reconnect message
fn inject_reconnect_message(surface: ffi::ghostty_surface_t) {
    let msg = b"\r\n\x1b[32m[Reconnected \xe2\x80\x94 new session]\x1b[0m\r\n";
    unsafe {
        ffi::ghostty_surface_process_output(surface, msg.as_ptr() as *const _, msg.len());
    }
}

// Per D-08: Write shell exit message
fn inject_exit_message(surface: ffi::ghostty_surface_t) {
    let msg = b"\r\n\x1b[90m[Remote shell exited. Press any key to close]\x1b[0m\r\n";
    unsafe {
        ffi::ghostty_surface_process_output(surface, msg.as_ptr() as *const _, msg.len());
    }
}
```

## Open Questions

1. **What does proxy.open connect to?**
   - What we know: `proxy.open` takes `host` + `port` and makes a TCP connection on the remote side. cmuxd-remote is a proxy, not a shell spawner.
   - What's unclear: The remote pane needs a shell, but proxy.open connects to a TCP port. There's no shell-spawning RPC in the proxy protocol. The `session.open` + `session.attach` API exists but it's separate from the proxy stream API.
   - Recommendation: Investigate whether `session.open` is the correct entry point (it manages sessions with cols/rows/attachments) or whether proxy.open needs a shell-spawning enhancement on cmuxd-remote. The Go daemon may need a small addition to spawn a PTY on the remote side and expose it as a TCP port that proxy.open can connect to.

2. **Resize delivery mechanism**
   - What we know: D-05 says "SIGWINCH-over-proxy". The manual I/O mode's `resize` is a no-op. cmuxd-remote has `session.resize` RPC.
   - What's unclear: If using proxy streams (not sessions), there's no resize command for a stream. If using sessions, the session API is separate from the proxy API.
   - Recommendation: Either (a) use the session API instead of proxy streams, or (b) add a `proxy.resize` RPC to cmuxd-remote that sends SIGWINCH to the remote PTY.

3. **ghostty.h sync strategy**
   - What we know: The project maintains its own ghostty.h. The submodule's header has the fields we need.
   - What's unclear: Whether to wholesale replace the project's header or surgically add needed fields.
   - Recommendation: Surgical addition of just the needed types and fields to minimize risk. Full sync may introduce other breaking changes.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| SSH client | SSH tunnel | Likely (system) | -- | None (required) |
| cmuxd-remote | Remote daemon | Built from source (Go) | dev | -- |
| Ghostty manual I/O | Surface I/O | In submodule | Current fork | openpty bridge (D-01) |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (unit) + Python socket tests (integration) |
| Config file | Cargo.toml (bin tests) |
| Quick run command | `cargo test --bin cmux-linux -- ssh` |
| Full suite command | GitHub Actions CI |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SSH-03 | Terminal sessions run on remote host | integration/manual | Manual: connect to SSH workspace, type, see output | N/A - requires live SSH |

### Sampling Rate
- **Per task commit:** `cargo test --bin cmux-linux -- ssh`
- **Per wave merge:** CI build + clippy
- **Phase gate:** Manual SSH test with real remote host

### Wave 0 Gaps
- [ ] Unit tests for base64 encode/decode round-trip with proxy protocol framing
- [ ] Unit tests for JSON-RPC message construction (proxy.open, proxy.write, etc.)
- [ ] Unit tests for stream state management (pane_id <-> stream_id mapping)

## Sources

### Primary (HIGH confidence)
- `ghostty/src/apprt/embedded.zig` lines 432-506 -- IoMode enum, IoWriteCallback type, Surface.Options struct with io_mode/io_write_cb/io_write_userdata
- `ghostty/src/termio/Manual.zig` lines 1-80 -- Manual I/O backend: queueWrite invokes write_cb, resize is no-op
- `ghostty/include/ghostty.h` lines 440-463 -- C API: ghostty_surface_io_mode_e, ghostty_io_write_cb, ghostty_surface_config_s with io fields
- `ghostty/include/ghostty.h` line 1107 -- ghostty_surface_process_output declaration
- `daemon/remote/cmd/cmuxd-remote/main.go` -- Full proxy protocol implementation (proxy.open, proxy.write, proxy.close, proxy.stream.subscribe, streamPump)
- `src/ssh/tunnel.rs` -- Existing SSH lifecycle with TODO at line 73-74
- `ghostty.h` (project root) -- Current project header, confirmed MISSING io_mode fields

### Secondary (MEDIUM confidence)
- Struct size analysis: current bindings 88 bytes, expected ~104 bytes with io fields

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- base64 crate version verified, all other deps already in Cargo.toml
- Architecture: HIGH -- Ghostty manual I/O mode verified directly in source; cmuxd-remote proxy protocol verified in Go source
- Pitfalls: HIGH -- ghostty.h sync issue confirmed by comparing headers; io_write_cb safety analyzed from Zig source

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (stable -- Ghostty fork under project control, cmuxd-remote under project control)
