# Phase 3: Socket API + Session Persistence - Research

**Researched:** 2026-03-25
**Domain:** Tokio Unix socket server, glib::MainContext channel, v2 JSON-RPC protocol, serde session serialization
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**tests_v2 Coverage Scope**
- D-01: Phase 3 targets core socket + session test subset: `test_cli_*`, `test_close_*`, `test_ctrl_socket.py`, `test_nested_split_*`, `test_initial_terminal_*`, and any other tests exercising workspace/surface/pane operations without browser or command palette.
- D-02: Browser (`test_browser_*`), command palette (`test_command_palette_*`), and macOS-specific tests (`test_lint_swiftui_patterns.py`, SwiftUI split tests) are out of scope for Phase 3.
- D-03: SOCK-04 is satisfied by the Linux-applicable subset running green. macOS-only tests excluded by platform check or skip marker.

**CLI Tool**
- D-04: The "cmux CLI" for Phase 3 is the Python client `tests_v2/cmux.py`. A native Rust CLI binary comes in Phase 5.
- D-05: Update `tests_v2/cmux.py` to check XDG paths on Linux — add `$XDG_RUNTIME_DIR/cmux/cmux.sock` (and `/run/user/{uid}/cmux/cmux.sock` fallback) to socket discovery list when `platform.system() == "Linux"`. The `CMUX_SOCKET` env var override already works.

**Socket Server**
- D-06: Socket path: `$XDG_RUNTIME_DIR/cmux/cmux.sock`. Fallback: `/run/user/{getuid()}/cmux/cmux.sock`. Create `cmux/` subdirectory at startup (mode 0700). Socket file mode: 0600.
- D-07: Authentication: `SO_PEERCRED` uid validation on every `accept()`. Reject connections from UIDs that don't match the app owner UID. No HMAC-SHA256.
- D-08: Threading: Replace current polling `mpsc::channel` + 100ms timer in `main.rs` with `glib::MainContext::channel::<SocketCommand>()`. Socket accept loop runs in tokio; commands sent via channel sender and processed in GTK main loop receiver.

**v2 Protocol Command Tier**
- D-09: Tier 1 — fully implemented in Phase 3: `system.ping`, `system.identify`, `system.capabilities`, `workspace.*` (list/current/create/select/close/rename/next/previous/last/reorder), `surface.*` (list/split/focus/close/send_text/send_key/read_text/health/refresh), `pane.list`, `pane.focus`, `pane.last`, `debug.layout`, `debug.type`, `window.list`, `window.current`.
- D-10: Tier 2 — stub with `{"ok": false, "error": "not_implemented"}` in Phase 3: all `browser.*`, all `notification.*`, `window.create/close/focus`, `pane.create/break/join/swap/surfaces`, `surface.drag_to_split/move/reorder/trigger_flash/clear_history/create`.

**Socket Command Threading Policy**
- D-11: Per CLAUDE.md: parse and validate arguments off-main; schedule state mutations with `glib::MainContext::channel` receiver (main thread). Non-focus-intent commands never call GTK focus APIs.

**Session Persistence**
- D-12: Save path: `~/.local/share/cmux/session.json` (respects `$XDG_DATA_HOME/cmux/session.json` if set).
- D-13: Save content: workspace list (name, UUID, active_pane_id) + per-workspace SplitNode tree (recursive branch/leaf, each leaf: surface UUID, shell command, CWD path).
- D-14: Save trigger: debounced 500ms after any workspace/pane mutation.
- D-15: Atomic write: write to `session.json.tmp`, then `rename()` to `session.json`.
- D-16: Restore: on app launch, read `session.json` before building UI. If missing or invalid, log and start with one default workspace.

**Claude's Discretion**
- v2 JSON-RPC framing: one JSON object per line, newline-delimited, as specified by `tests_v2/cmux.py` protocol docstring.
- Error response schema: `{"id": N, "ok": false, "error": "error_code", "message": "human text"}`.
- Request ID type: accept both integer and string IDs, echo back what was sent.
- `system.identify` response shape: include `version`, `platform: "linux"`, `socket_path`.
- `glib::MainContext::channel` buffer capacity: 256 commands.
- `SO_PEERCRED` implementation: `getsockopt(SOL_SOCKET, SO_PEERCRED)` on accepted fd.
- Session JSON schema: versioned (`"version": 1`) for forward-compatibility.

### Deferred Ideas (OUT OF SCOPE)
- `@agent-browser` integration — targeting Phase 4/5
- HMAC-SHA256 password auth — not needed for Phase 3
- Rust CLI binary (`src/bin/cmux.rs`) — Phase 5
- `notification.*` socket commands — Phase 4
- `pane.break`, `pane.join`, `pane.swap` — beyond Phase 3
- `window.create` / multi-window — Linux Phase 3 is single-window
- Systemd socket activation (SYS-01) — out of scope for Phase 3
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SOCK-01 | Unix socket server starts at `$XDG_RUNTIME_DIR/cmux/cmux.sock` (mode 0600) | tokio::net::UnixListener, SO_PEERCRED, XDG path logic |
| SOCK-02 | v2 JSON-RPC protocol wire-compatible with macOS cmux | cmux.py protocol docstring, full response schema documented below |
| SOCK-03 | `cmux` CLI (Python client) can connect and control the Linux app | cmux.py XDG path update (D-05) |
| SOCK-04 | `tests_v2/` Python protocol suite passes against Linux socket server | In-scope test file list identified; platform skip pattern documented |
| SOCK-05 | Non-focus-intent commands never call `gtk_window_present()` or `ghostty_surface_set_focus()` | Command routing architecture documented; focus-intent list enumerated |
| SOCK-06 | `SO_PEERCRED` uid validation on every connection accept | libc::ucred, getsockopt pattern documented |
| SESS-01 | Workspace and pane layout saved to `~/.local/share/cmux/session.json` debounced | serde + tokio debounce pattern |
| SESS-02 | Layout fully restored on next app launch | Session restore before UI build; SplitNode deserialization |
| SESS-03 | Session file written atomically (write `.tmp`, then `rename()`) | `std::fs::rename` atomicity on Linux, tmp path pattern |
| SESS-04 | App launches cleanly if session file is missing or corrupted | Graceful fallback to single default workspace |
</phase_requirements>

---

## Summary

Phase 3 adds two independent subsystems: a Unix socket JSON-RPC server and a session persistence mechanism. Both are pure backend — no new UI.

The socket server runs a tokio async accept loop that authenticates connections via `SO_PEERCRED` and dispatches parsed commands to the GTK main thread via `glib::MainContext::channel`. This replaces the current 100ms polling mpsc channel in `main.rs` with an event-driven, zero-latency delivery mechanism. The server speaks the v2 JSON-RPC line protocol already defined by `tests_v2/cmux.py` — one JSON object per line, with stable UUID handles for workspaces/surfaces/panes.

Session persistence uses serde + serde_json to serialize the full workspace layout (names, UUIDs, SplitNode tree structure, per-leaf CWD) to `~/.local/share/cmux/session.json`. Writes are atomic via rename-from-tmp. The debounce (500ms) is implemented with a tokio task that resets its sleep on each mutation event. On restore, the session file is read before the GTK window is presented so that the initial workspace state reflects the saved layout.

**Primary recommendation:** Implement the socket server as a dedicated `src/socket/` module with a `SocketServer` struct, a `SocketCommand` enum for all Tier 1 commands, and a `SessionManager` in `src/session.rs`. Keep all GTK/Ghostty state mutations on the main thread via the glib channel. Parse JSON and validate arguments in the tokio accept task, never on main.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.50.0 | Async runtime, UnixListener, debounce sleep | Already in Cargo.toml; `features = ["full"]` includes all needed APIs |
| glib | 0.21.5 (pinned) | MainContext::channel — bridges tokio to GTK main loop | Already in Cargo.toml; idiomatic gtk4-rs IPC pattern |
| serde | 1.0.228 | Derive Serialize/Deserialize for session types | Standard Rust serialization |
| serde_json | 1.0.149 | JSON line encoding/decoding for wire protocol and session file | Standard JSON crate, pairs with serde |
| uuid | 1.22.0 | Generate stable UUIDs for workspace/surface/pane handles | Required by protocol (v2 uses UUIDs) |
| libc | latest | `getsockopt(SOL_SOCKET, SO_PEERCRED)` via `libc::ucred` | Lowest-level SO_PEERCRED access; no wrapper crate needed |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio::sync::Notify | (in tokio) | Debounce trigger for session saves | Preferred over channel for pure signaling |

**Cargo.toml additions:**
```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
libc = "0.2"
```

Note: `tokio` and `glib` are already in Cargo.toml. `glib` stays pinned at 0.21.5 — the latest is 0.22.3 but upgrading requires verifying gtk4 0.10 compatibility. Do not upgrade without testing.

**Version verification:** Confirmed against `cargo search` output on 2026-03-25. serde_json 1.0.149, uuid 1.22.0, serde 1.0.228.

---

## Architecture Patterns

### Recommended Project Structure

```
src/
├── main.rs              # Remove polling mpsc; wire glib::MainContext::channel
├── app_state.rs         # Add UUID fields to Workspace; add apply_session() method
├── workspace.rs         # Add uuid: Uuid field
├── split_engine.rs      # Add serialize_tree() method; SplitNodeData for serde
├── session.rs           # NEW: SessionData, load_session(), save_session_atomic()
├── socket/
│   ├── mod.rs           # SocketServer struct, start_socket_server()
│   ├── commands.rs      # SocketCommand enum (all Tier 1 variants)
│   ├── handlers.rs      # Handler functions called on GTK main thread
│   └── auth.rs          # SO_PEERCRED uid validation
└── sidebar.rs           # (unchanged)
```

### Pattern 1: glib::MainContext::channel for Tokio-to-GTK Bridge

**What:** `glib::MainContext::channel::<SocketCommand>(glib::Priority::DEFAULT)` creates a `(Sender<T>, Receiver<T>)` pair. The tokio task holds the `Sender`; the GLib main loop calls the `Receiver`'s `attach()` callback on every message.

**When to use:** Any time a tokio task needs to mutate GTK/AppState. This is the only safe way to cross the thread boundary.

**Example:**
```rust
// In main() / build_ui(), before socket server starts:
let (cmd_tx, cmd_rx) = glib::MainContext::channel::<SocketCommand>(glib::Priority::DEFAULT);

// Attach receiver to GTK main loop — called on main thread for each message:
cmd_rx.attach(None, {
    let state = state.clone();
    move |cmd| {
        handle_socket_command(cmd, &state);
        glib::ControlFlow::Continue
    }
});

// Pass cmd_tx (Clone-able) into the tokio socket accept task:
let tx = cmd_tx.clone();
runtime_handle.spawn(async move {
    socket_server_loop(tx).await;
});
```

**Key property:** The `Sender<T>` is `Send + Clone`. It can be held by tokio tasks on any thread and sent across task boundaries. The `Receiver<T>` stays on the main thread after `attach()`.

**Capacity:** The channel is unbounded in glib 0.21 (no backpressure). 256-command burst is safe.

### Pattern 2: Tokio Unix Socket Accept Loop with SO_PEERCRED Auth

**What:** `tokio::net::UnixListener::bind()` creates the socket; `listener.accept()` awaits new connections. Each accepted connection is spawned as a separate tokio task.

**SO_PEERCRED validation (Linux-specific):**
```rust
// In auth.rs — called immediately after accept(), before any data is read:
use std::os::unix::io::AsRawFd;

pub fn validate_peer_uid(stream: &tokio::net::UnixStream) -> std::io::Result<bool> {
    let fd = stream.as_raw_fd();
    let mut cred = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let ret = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut cred as *mut _ as *mut libc::c_void,
            &mut len,
        )
    };
    if ret != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let expected_uid = unsafe { libc::getuid() };
    Ok(cred.uid == expected_uid)
}
```

**Socket file setup:**
```rust
use std::os::unix::fs::PermissionsExt;

// Create directory with mode 0700:
std::fs::create_dir_all(&socket_dir)?;
std::fs::set_permissions(&socket_dir, std::fs::Permissions::from_mode(0o700))?;

// Remove stale socket file from previous run:
let _ = std::fs::remove_file(&socket_path);

let listener = tokio::net::UnixListener::bind(&socket_path)?;

// Set socket file mode to 0600:
std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;
```

**Pitfall:** `tokio::net::UnixListener::bind` fails if the socket file already exists. Always `remove_file` first (ignore ENOENT). This handles the case where the app crashed without cleanup.

### Pattern 3: JSON-RPC Line Protocol Handler

**Wire format** (source of truth: `tests_v2/cmux.py` docstring):
- Request: `{"id": 1, "method": "surface.list", "params": {...}}\n`
- Success response: `{"id": 1, "ok": true, "result": {...}}\n`
- Error response: `{"id": 1, "ok": false, "error": {"code": "error_code", "message": "human text"}}\n`

**Request ID:** The `id` field in the Python client is always an integer, but the spec says accept both integers and strings and echo back what was received. Use `serde_json::Value` for the id field to handle both.

**Per-connection handler pattern:**
```rust
// Each accepted connection runs this in its own tokio::spawn:
async fn handle_connection(
    stream: tokio::net::UnixStream,
    cmd_tx: glib::Sender<SocketCommand>,
) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let response = dispatch_line(&line, &cmd_tx).await;
        let _ = writer.write_all(response.as_bytes()).await;
        let _ = writer.write_all(b"\n").await;
    }
}
```

**Synchronous response problem:** Many commands (e.g., `workspace.list`, `pane.list`) need to read AppState from the GTK main thread and send the result back to the tokio connection handler. The solution is a `oneshot` channel:

```rust
// SocketCommand carries a response sender for commands needing a result:
pub enum SocketCommand {
    WorkspaceList {
        req_id: serde_json::Value,
        resp_tx: tokio::sync::oneshot::Sender<serde_json::Value>,
    },
    WorkspaceCreate {
        req_id: serde_json::Value,
        resp_tx: tokio::sync::oneshot::Sender<serde_json::Value>,
    },
    // ... fire-and-forget commands (no resp_tx needed):
    SurfaceSendText {
        req_id: serde_json::Value,
        surface_id: Option<uuid::Uuid>,
        text: String,
        resp_tx: tokio::sync::oneshot::Sender<serde_json::Value>,
    },
    // etc.
}

// In the tokio connection handler:
let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
cmd_tx.send(SocketCommand::WorkspaceList { req_id, resp_tx }).ok();
let response = resp_rx.await.unwrap_or_else(|_| error_json(req_id, "internal_error", "handler dropped"));

// In the GTK main thread handler:
fn handle_socket_command(cmd: SocketCommand, state: &AppStateRef) {
    match cmd {
        SocketCommand::WorkspaceList { req_id, resp_tx } => {
            let result = build_workspace_list_result(state);
            let _ = resp_tx.send(build_ok_response(req_id, result));
        }
        // ...
    }
}
```

### Pattern 4: Session Save with Debounce

**Debounce via tokio::sync::Notify:**
```rust
// Shared notify in Arc:
let save_notify = std::sync::Arc::new(tokio::sync::Notify::new());

// Debounce task — runs for the lifetime of the app:
let notify = save_notify.clone();
let state_for_save = state.clone();  // must snapshot on main thread, not in tokio
runtime_handle.spawn(async move {
    loop {
        notify.notified().await;
        // Drain additional notifications that arrived during debounce window:
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        while notify.try_notified() { /* drain */ }
        // Snapshot happens on GTK main thread via glib oneshot, or via Arc<Mutex<SessionSnapshot>>
        // (see Pitfall 3 below — use glib::idle_add_once to snapshot on main thread)
    }
});

// After any mutation in AppState, trigger save:
save_notify.notify_one();
```

**Atomic write:**
```rust
fn save_session_atomic(path: &std::path::Path, data: &SessionData) -> std::io::Result<()> {
    let tmp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(&tmp_path, json.as_bytes())?;
    std::fs::rename(&tmp_path, path)?;  // atomic on Linux (same filesystem)
    Ok(())
}
```

`rename()` is atomic on Linux when src and dst are on the same filesystem (which they always are for `~/.local/share/cmux/`). kill -9 mid-write leaves the `.tmp` file but never touches `session.json` until rename succeeds.

### Pattern 5: SplitNode Serialization

The `SplitNode` tree contains GTK widget references (`gtk4::GLArea`, `gtk4::Paned`) which cannot be serialized. Create a parallel serde-friendly data type:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum SplitNodeData {
    Leaf {
        pane_id: u64,
        surface_uuid: uuid::Uuid,
        shell: String,   // e.g. "/bin/zsh"
        cwd: String,     // absolute path string
    },
    Split {
        orientation: String,  // "horizontal" or "vertical"
        start: Box<SplitNodeData>,
        end: Box<SplitNodeData>,
    },
}
```

Add a `to_data()` method to `SplitNode` that produces `SplitNodeData` by walking the tree. The `cwd` for each leaf is obtained by reading `/proc/{pid}/cwd` where `pid` is the shell PID for that pane (this requires the pane to track its shell PID, or use `ghostty_surface_cwd()` if available in the Ghostty API).

**Important gap:** The current `Workspace` struct uses `u64` IDs, not UUIDs. Phase 3 must add `pub uuid: uuid::Uuid` to `Workspace` (generated at `Workspace::new()` time). Similarly, panes (leaves in SplitNode) need UUID handles for the v2 protocol. The existing `pane_id: u64` can coexist; the UUID is added for protocol-facing identity.

### Pattern 6: cmux.py Linux Socket Discovery

Add to `_default_socket_path()` in `tests_v2/cmux.py`:

```python
import platform

def _default_socket_path() -> str:
    override = os.environ.get("CMUX_SOCKET_PATH") or os.environ.get("CMUX_SOCKET")
    if override:
        # ... existing logic unchanged

    # Linux: check XDG_RUNTIME_DIR paths before macOS paths
    if platform.system() == "Linux":
        xdg_runtime = os.environ.get("XDG_RUNTIME_DIR") or f"/run/user/{os.getuid()}"
        linux_candidates = [
            os.path.join(xdg_runtime, "cmux", "cmux.sock"),
            os.path.join(xdg_runtime, "cmux", "last-socket-path"),  # marker file
        ]
        for path in linux_candidates:
            if os.path.exists(path):
                return path

    # ... existing macOS/tmp fallback logic
```

Also write the `last-socket-path` marker file at `$XDG_RUNTIME_DIR/cmux/last-socket-path` from the Rust app at startup, matching the macOS convention the Python client already checks.

### Anti-Patterns to Avoid

- **Do not call `gtk_window_present()` from socket command handlers.** Only focus-intent commands (`workspace.select`, `pane.focus`, `pane.last`, `surface.focus`) may touch focus state. All others must read/mutate data only.
- **Do not read `AppState` from the tokio task thread.** `AppState` is `Rc<RefCell<AppState>>` — `!Send`. All reads must happen in the glib channel callback on the main thread. Use `oneshot::Sender` to return results.
- **Do not call `ghostty_surface_set_focus()` for non-focus commands.** The `surface.send_text` and `surface.send_key` handlers must send input to the current active surface without changing focus.
- **Do not upgrade `glib` past 0.21.5 in this phase.** The `gtk4 = "0.10.3"` pin constrains glib compatibility. Test before upgrading.
- **Do not skip socket auth even for localhost.** SO_PEERCRED validation must run on every accepted connection before reading any data.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON encoding | Custom serializer | `serde_json` | Handles Unicode escaping, number precision, all edge cases |
| UUID generation | Custom random IDs | `uuid::Uuid::new_v4()` | Collision-free, standard format clients expect |
| Session versioning | Ad-hoc field checking | `"version": 1` in root + serde flatten | Easy to add fields; readers can reject unknown versions |
| Atomic file write | Temp + copy | `write tmp + rename()` | `rename()` is the only atomic write on Linux; copy is not |
| Peer credential check | Parsing /proc/net/unix | `getsockopt(SO_PEERCRED)` | Kernel-provided, unforgeable, O(1) |
| Line protocol framing | Custom length-prefix | Newline-delimited JSON | What the Python test client speaks |

**Key insight:** The tokio + glib channel pattern is the only safe way to bridge async I/O to GTK. Any attempt to share `AppStateRef` across threads will fail at compile time — Rust's ownership enforces this correctly.

---

## Common Pitfalls

### Pitfall 1: Stale Socket File from Previous Crash
**What goes wrong:** If the app crashes or is killed without cleanup, the socket file remains. The next `UnixListener::bind()` call returns `EADDRINUSE`.
**Why it happens:** Unix domain sockets are filesystem objects that persist after the process exits.
**How to avoid:** Always call `std::fs::remove_file(&socket_path)` immediately before `UnixListener::bind()`. Ignore `ENOENT` (file doesn't exist is fine). This is standard practice.
**Warning signs:** `bind: address already in use` error at app startup.

### Pitfall 2: AppState is Rc<RefCell<...>> — Not Send
**What goes wrong:** Attempting to share `AppStateRef` with a tokio task fails at compile time: "`Rc<RefCell<AppState>>` cannot be sent between threads safely".
**Why it happens:** GTK objects (`GLArea`, `ListBox`, etc.) are not thread-safe. `Rc<RefCell<T>>` enforces this by being `!Send`.
**How to avoid:** All AppState access happens exclusively in the glib `MainContext::channel` callback. Tokio tasks communicate via `glib::Sender<SocketCommand>` (which IS `Send + Clone`) and `tokio::sync::oneshot::Sender<serde_json::Value>` for responses.
**Warning signs:** Compile error mentioning `Rc` or `RefCell` not being `Send`.

### Pitfall 3: Session Snapshot Must Happen on Main Thread
**What goes wrong:** The debounce task runs in tokio. If it tries to read `AppState` to build the session snapshot, it will fail (Rc is !Send). If it clones the session data incorrectly, it may serialize an inconsistent state.
**Why it happens:** The save trigger and the snapshot extraction are on different threads.
**How to avoid:** Use `glib::idle_add_local_once` in the debounce task's callback to request a snapshot on the main thread. The snapshot is written to an `Arc<Mutex<Option<SessionData>>>` that the tokio save task can read. Alternatively, trigger the save entirely on the main thread: when a mutation happens in AppState, call `session_manager.schedule_save(state)` synchronously from the glib receiver, which extracts the snapshot immediately and spawns the async file write with the already-captured data.
**Warning signs:** Borrow errors involving `RefCell` inside `async` blocks.

### Pitfall 4: glib::Sender::send() Returns Err When Receiver is Dropped
**What goes wrong:** If the GTK main loop exits (app shutting down), the glib channel receiver is dropped. Subsequent `cmd_tx.send()` calls return `Err`. If the tokio task panics on this error, it may cause visible crashes on shutdown.
**Why it happens:** Normal app shutdown sequence: GTK exits first, then tokio tasks get dropped.
**How to avoid:** Handle `cmd_tx.send(cmd)` errors gracefully — log and continue or break the accept loop. Tokio tasks should treat a send error as "app is shutting down, stop accepting connections."
**Warning signs:** Occasional panic on app close: "send on closed channel".

### Pitfall 5: response ID Must Match Request ID Type
**What goes wrong:** The Python client checks `resp.get("id") != req_id` with strict equality. If the request sends `{"id": 1}` (integer) and the server responds `{"id": "1"}` (string), the client raises `cmuxError("Mismatched response id")`.
**Why it happens:** `serde_json::Value` preserves the original type. If the server parses the id as a string and serializes it back as a string, integer ids become strings.
**How to avoid:** Parse the request `id` as `serde_json::Value` (not as `u64`). Echo the exact `Value` back in the response. The Python client always sends integer ids (1, 2, 3...) so in practice they will always be `Value::Number`.
**Warning signs:** Test failures with "Mismatched response id" even when the server is running.

### Pitfall 6: Workspace and Pane UUIDs Not Yet Present
**What goes wrong:** The v2 protocol requires stable UUID handles for workspaces and panes. The current `Workspace` and `SplitNode` use monotonic `u64` IDs. These are not valid v2 protocol handles.
**Why it happens:** Phase 2 was designed before the socket API; IDs were implementation-internal.
**How to avoid:** Add `pub uuid: uuid::Uuid` to `Workspace` (generated in `Workspace::new()`) and a UUID field to each `SplitNode::Leaf` (generated at split time). The `workspace.list` response uses these UUIDs as the `id` field. Keep the u64 IDs as internal implementation detail. The SURFACE_REGISTRY already maps pane_id to surface pointer — extend it or add a parallel UUID registry.
**Warning signs:** Python client UUID validation fails with "Invalid workspace id".

### Pitfall 7: XDG_RUNTIME_DIR May Be Unset
**What goes wrong:** In some environments (running via SSH without PAM, inside containers, or with unusual session types), `XDG_RUNTIME_DIR` is not set. Directly reading it without fallback causes a panic.
**Why it happens:** `XDG_RUNTIME_DIR` is set by logind/PAM. Headless and container environments may skip it.
**How to avoid:** Use the fallback: `std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }))`. Always validate the directory exists and is writable before creating the socket.
**Warning signs:** "No such file or directory" when creating socket directory.

---

## Wire Protocol Reference

### Request/Response Schema (source: tests_v2/cmux.py)

```
Request:  {"id": <int|str>, "method": "<namespace>.<command>", "params": {...}}\n
Success:  {"id": <int|str>, "ok": true, "result": {...}}\n
Error:    {"id": <int|str>, "ok": false, "error": {"code": "<code>", "message": "<text>"}}\n
```

### Tier 1 Command Response Shapes

These are inferred from `cmux.py` client methods and test files:

| Method | Result Fields |
|--------|--------------|
| `system.ping` | `{"pong": true}` |
| `system.identify` | `{"version": "...", "platform": "linux", "socket_path": "...", "focused": {"surface_id": "...", "pane_id": "...", "workspace_id": "..."}}` |
| `system.capabilities` | `{"methods": ["..."]}` — list of all supported method names |
| `workspace.list` | `{"workspaces": [{"index": 0, "id": "<uuid>", "title": "...", "selected": bool}]}` |
| `workspace.current` | `{"workspace_id": "<uuid>"}` |
| `workspace.create` | `{"workspace_id": "<uuid>"}` |
| `workspace.select` | `{}` |
| `workspace.close` | `{}` |
| `workspace.rename` | `{}` |
| `workspace.next` | `{"workspace_id": "<uuid>"}` |
| `workspace.previous` | `{"workspace_id": "<uuid>"}` |
| `workspace.last` | `{"workspace_id": "<uuid>"}` |
| `workspace.reorder` | `{}` |
| `surface.list` | `{"surfaces": [{"index": 0, "id": "<uuid>", "focused": bool}]}` |
| `surface.split` | `{"surface_id": "<uuid>"}` |
| `surface.focus` | `{}` — FOCUS INTENT: may call ghostty_surface_set_focus |
| `surface.close` | `{}` |
| `surface.send_text` | `{}` |
| `surface.send_key` | `{}` |
| `surface.read_text` | `{"text": "..."}` or `{"base64": "..."}` |
| `surface.health` | `{"surfaces": [...]}` |
| `surface.refresh` | `{}` |
| `pane.list` | `{"panes": [{"index": 0, "id": "<uuid>", "surface_count": 1, "focused": bool}]}` |
| `pane.focus` | `{}` — FOCUS INTENT |
| `pane.last` | `{"pane_id": "<uuid>"}` |
| `window.list` | `{"windows": [{"id": "<uuid>", "index": 0}]}` |
| `window.current` | `{"window_id": "<uuid>"}` |
| `debug.layout` | `{"layout": {...}}` — tree structure for debugging |
| `debug.type` | `{}` — sends keystrokes to active surface (same as send_key internally) |

### Focus-Intent Command List (may call focus APIs)
- `workspace.select` — switches active workspace, calls focus_active_surface()
- `workspace.next`, `workspace.previous`, `workspace.last` — same
- `pane.focus` — focuses a specific pane
- `pane.last` — focuses the previously focused pane
- `surface.focus` — focuses a specific surface

**All other commands** must NOT call `gtk_window_present()`, `ghostty_surface_set_focus()`, or `GLArea::grab_focus()`.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `std::sync::mpsc` + 100ms poll in main.rs | `glib::MainContext::channel` | Phase 3 migration | Zero-latency delivery, no CPU waste, idiomatic gtk4-rs |
| u64 monotonic IDs | UUID v4 per workspace/pane | Phase 3 | v2 protocol compatibility |
| No session | serde_json atomic write to XDG_DATA_HOME | Phase 3 | Survives restarts |
| macOS-only socket path discovery | Linux XDG path + fallback in cmux.py | Phase 3 | Python test suite connects on Linux |

---

## Open Questions

1. **CWD Extraction for Session Save**
   - What we know: Session must save per-leaf CWD. Ghostty may expose `ghostty_surface_cwd()` in the embedded API, or we read `/proc/{pid}/cwd` (requires knowing the shell PID).
   - What's unclear: Does `ghostty.h` expose a CWD accessor? Need to grep `ghostty/include/ghostty.h` before implementing.
   - Recommendation: Check `ghostty.h` in Wave 0 setup plan. If no API, use `/proc/{pid}/cwd` with the shell PID tracked per-surface. Fall back to `$HOME` if unavailable.

2. **workspace.last semantic**
   - What we know: `pane.last` focuses the previously focused pane; `workspace.last` returns the previously selected workspace.
   - What's unclear: Is there a "previously focused workspace" already tracked in AppState? No — AppState only has `active_index`.
   - Recommendation: Add `previous_index: Option<usize>` to AppState. Set it whenever `switch_to_index()` is called.

3. **surface.read_text implementation**
   - What we know: The Python client expects either `{"text": "..."}` or `{"base64": "..."}`. Ghostty may expose a terminal content read API.
   - What's unclear: Does `ghostty.h` have a terminal content accessor?
   - Recommendation: Check `ghostty.h`. If not available, return `{"text": ""}` with a comment explaining the stub. The in-scope tests that use `read_terminal_text` may not be blocking for Phase 3 pass criteria.

4. **workspace.reorder direction**
   - What we know: The Python `reorder_workspace` call accepts `index`, `before_workspace_id`, or `after_workspace_id`. The current AppState.workspaces is a `Vec`.
   - What's unclear: Which `reorder_workspace` operations are exercised by in-scope tests?
   - Recommendation: Implement all three reorder variants on the Vec. If a test doesn't exercise this, the stub is still needed for the method to exist.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| tokio | Socket server, debounce | Yes (Cargo.toml) | 1 (full) | — |
| glib | MainContext::channel | Yes (Cargo.toml) | 0.21.5 | — |
| serde_json | JSON line protocol, session file | No (add to Cargo.toml) | 1.0.149 | — |
| serde | Derive macros | No (add to Cargo.toml) | 1.0.228 | — |
| uuid | Workspace/pane UUID generation | No (add to Cargo.toml) | 1.22.0 | — |
| libc | SO_PEERCRED getsockopt | No (add to Cargo.toml) | 0.2.x | — |
| XDG_RUNTIME_DIR | Socket path | Yes (Linux standard, /run/user/{uid} fallback) | n/a | /run/user/{uid}/cmux/ |
| Python 3 + tests_v2/ | Test harness | Present in repo | — | — |

**Missing dependencies with no fallback:** serde, serde_json, uuid, libc — all must be added to Cargo.toml in Wave 0.

**Missing dependencies with fallback:** XDG_RUNTIME_DIR — always use the `/run/user/{uid}` fallback if unset.

---

## Validation Architecture

`workflow.nyquist_validation` is `true` in `.planning/config.json`.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `#[cfg(test)]` unit tests (`cargo test`) + Python integration tests (tests_v2/) |
| Config file | No separate config file — `cargo test` uses Cargo.toml |
| Quick run command | `cargo test --lib 2>&1 \| tail -20` |
| Full suite command | `cargo test 2>&1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SOCK-01 | Socket file created at XDG path with mode 0600 | unit | `cargo test socket::tests::test_socket_path_creation` | ❌ Wave 0 |
| SOCK-02 | JSON-RPC request/response roundtrip (ping) | integration | `python3 tests_v2/test_ctrl_socket.py` | ✅ |
| SOCK-03 | Python client discovers Linux socket path | unit | `cargo test session::tests::test_xdg_path_fallback` | ❌ Wave 0 |
| SOCK-04 | Core test subset passes | integration | `python3 tests_v2/test_ctrl_socket.py && python3 tests_v2/test_close_workspace_selection.py` | ✅ |
| SOCK-05 | Non-focus commands do not call focus APIs | unit | `cargo test socket::tests::test_focus_policy` | ❌ Wave 0 |
| SOCK-06 | SO_PEERCRED rejects wrong-uid connections | unit | `cargo test socket::auth::tests::test_peercred_rejection` | ❌ Wave 0 |
| SESS-01 | Session file written after mutation | unit | `cargo test session::tests::test_save_triggered` | ❌ Wave 0 |
| SESS-02 | Session restored on relaunch | unit | `cargo test session::tests::test_restore_roundtrip` | ❌ Wave 0 |
| SESS-03 | Atomic write — tmp + rename | unit | `cargo test session::tests::test_atomic_write` | ❌ Wave 0 |
| SESS-04 | Graceful fallback on missing/corrupt session | unit | `cargo test session::tests::test_graceful_fallback` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib 2>&1 | tail -20`
- **Per wave merge:** `cargo test 2>&1`
- **Phase gate:** Full suite green + Python integration test subset passes before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/socket/mod.rs` — placeholder with `#[cfg(test)] mod tests` block
- [ ] `src/socket/auth.rs` — `test_peercred_rejection` stub
- [ ] `src/session.rs` — `test_save_triggered`, `test_restore_roundtrip`, `test_atomic_write`, `test_graceful_fallback` stubs
- [ ] Cargo.toml: add `serde`, `serde_json`, `uuid`, `libc` dependencies before any source compiles

---

## Project Constraints (from CLAUDE.md)

| Constraint | Implication for Phase 3 |
|-----------|------------------------|
| Socket command threading policy: parse off-main, mutate on main | SocketCommand enum carries pre-parsed values; no raw JSON on main thread |
| Socket focus policy: non-focus-intent commands must not steal focus | handlers.rs must gate all focus API calls to the explicit focus-intent list |
| Never use `DispatchQueue.main.sync` for high-freq commands (macOS policy — Rust analogue: never block tokio future waiting for main thread) | Use tokio oneshot channels for all command→response flows; tokio task awaits response, never blocks |
| Testing policy: never run tests locally (E2E/UI/Python socket tests run via GH Actions) | Unit tests only locally via `cargo test --lib`; Python integration tests go in CI |
| Test quality policy: no tests that only verify source code text or file contents | All session/socket tests must exercise runtime behavior through Rust code paths |

---

## Sources

### Primary (HIGH confidence)
- `tests_v2/cmux.py` — v2 protocol docstring, method implementations, response parsing — defines exact wire format
- `src/main.rs`, `src/app_state.rs`, `src/workspace.rs`, `src/split_engine.rs` — current implementation state
- `Cargo.toml` — pinned dependency versions (gtk4 0.10.3, glib 0.21.5, tokio 1)
- `.planning/phases/03-socket-api-session-persistence/03-CONTEXT.md` — all locked decisions

### Secondary (MEDIUM confidence)
- `cargo search` output (2026-03-25) — verified current crate versions for serde, serde_json, uuid, glib
- glib `MainContext::channel` pattern: documented in gtk4-rs project examples; matches current glib 0.21.5 API surface (glib::Sender is Send + Clone)

### Tertiary (LOW confidence)
- CWD extraction via `ghostty_surface_cwd()` — not verified against ghostty.h; may not exist. Needs confirmation in Wave 0.
- `libc::ucred` struct layout — standard Linux ABI; confirmed in libc crate docs but not tested against this specific kernel version (6.17.0-19-generic).

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — versions confirmed via `cargo search`; glib/tokio already in use
- Architecture: HIGH — glib MainContext::channel pattern is canonical gtk4-rs; tokio UnixListener is standard; SO_PEERCRED is Linux kernel guarantee
- Protocol compliance: HIGH — derived directly from `tests_v2/cmux.py` source code
- Pitfalls: HIGH — stale socket, Rc !Send, and atomic write patterns are well-established Rust/Linux facts
- CWD extraction: LOW — ghostty.h API not verified

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable ecosystem; glib/gtk4 pin means no drift risk within 30 days)
