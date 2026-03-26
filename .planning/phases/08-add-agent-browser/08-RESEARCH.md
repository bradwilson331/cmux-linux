# Phase 8: Add Agent-Browser - Research

**Researched:** 2026-03-26
**Domain:** Headless browser integration, CDP streaming, GTK4 image rendering
**Confidence:** HIGH

## Summary

Phase 8 integrates the existing `agent-browser/` Rust CLI (a headless Chrome automation tool using CDP) into cmux-linux. The integration has three main pillars: (1) bundling the agent-browser binary alongside cmux, (2) adding `browser.*` socket commands for lifecycle and streaming control, and (3) rendering CDP screencast frames in a GTK4 preview pane.

The agent-browser project is already a complete Rust CLI at `agent-browser/cli/` with Unix socket daemon architecture, CDP WebSocket streaming via `Page.startScreencast`, and a well-defined JSON protocol. The cmux integration layer is thin: spawn the daemon, connect to its stream WebSocket, decode base64 JPEG frames, and display them in a `gtk4::Picture` widget. The existing socket infrastructure in `src/socket/` provides a clear pattern for adding the `browser.*` command namespace.

**Primary recommendation:** Build the daemon spawn in a new `src/browser.rs` module, add `browser.*` variants to `SocketCommand` enum and handlers, and use `gtk4::Picture` with `gdk4::Texture::from_bytes()` to render incoming JPEG frames in a non-terminal pane leaf.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Hybrid integration -- agent-browser binary bundled with cmux, plus a thin socket API layer for key operations. Agents can use either the CLI directly from terminal panes or the cmux socket for lifecycle/streaming commands.
- **D-02:** CDP streaming to preview pane -- agent-browser runs headless Chrome, streams screenshots/DOM snapshots via its `stream` command, cmux renders them in a pane. Not a real browser widget, but shows live feedback of what the agent sees.
- **D-03:** Embedded browser pane (WebKitGTK or CEF) is a stretch goal / future enhancement, not in initial scope.
- **D-04:** Lifecycle + streaming commands only: `browser.open`, `browser.close`, `browser.stream.enable`, `browser.stream.disable`, `browser.snapshot`, `browser.screenshot`. Interaction commands (`click`, `fill`, `eval`, etc.) go through agent-browser CLI directly.
- **D-05:** Auto-start on first use -- cmux spawns agent-browser daemon automatically when the first `browser.*` socket command is issued. User stops manually via `browser.close` or cmux shutdown.

### Claude's Discretion
- How to bundle the agent-browser binary (embed in AppImage, ship alongside, or expect system install)
- CDP streaming transport (WebSocket from agent-browser `stream` command vs polling screenshots)
- Preview pane widget implementation (GTK4 image widget refreshed on stream events vs custom drawing area)
- Chrome download/install management (delegate to `agent-browser install` or bundle Chrome)
- Error handling for Chrome not found / agent-browser not installed

### Deferred Ideas (OUT OF SCOPE)
- Full embedded browser pane via WebKitGTK or CEF (D-03 stretch goal)
- Browser tab management within cmux (multiple browser tabs per pane)
- JavaScript console UI in cmux
- Find-in-page UI overlay on preview pane
- Browser history / bookmarks
- macOS `browser.*` socket command wire-compatibility
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| gtk4 | 0.10.3 | GTK4 bindings (already in project) | Required for all UI; `gtk4::Picture` for image display |
| gdk4 | 0.10.x | GDK texture/pixbuf for frame decoding | `gdk4::Texture::from_bytes()` decodes JPEG to GPU texture |
| tokio | 1.x | Async runtime (already in project) | WebSocket client, daemon spawn, stream processing |
| tokio-tungstenite | 0.24.x | WebSocket client | Connect to agent-browser stream server; same version as agent-browser uses |
| futures-util | 0.3.x | Stream combinators for WebSocket | Process incoming WS messages |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| glib | 0.21.5 | GLib main loop integration (already in project) | Bridge tokio stream events to GTK main thread |
| serde_json | 1.x | JSON parsing (already in project) | Parse stream frame messages |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `gtk4::Picture` | `gtk4::DrawingArea` with cairo | Picture handles texture lifecycle; DrawingArea needs manual cairo rendering -- Picture is simpler for image-only display |
| tokio-tungstenite | tungstenite (sync) | Project already uses tokio everywhere; async WS fits naturally |
| Polling screenshots | WebSocket streaming | Streaming is lower latency and already built into agent-browser; polling would add complexity for no benefit |

**Installation (new deps only):**
```bash
# Add to Cargo.toml [dependencies]
tokio-tungstenite = { version = "0.24", features = ["rustls-tls-webpki-roots"] }
futures-util = "0.3"
```

Note: `gdk4` is already available transitively through `gtk4` but may need explicit import for `Texture` APIs.

## Architecture Patterns

### Recommended Project Structure
```
src/
  browser.rs           # NEW: BrowserManager struct -- daemon lifecycle, stream connection
  socket/
    commands.rs         # MODIFY: Add Browser* variants to SocketCommand enum
    handlers.rs         # MODIFY: Add browser.* match arms
    mod.rs              # MODIFY: Add browser.* dispatch in dispatch_line()
  split_engine.rs       # MODIFY: Add Preview leaf variant to SplitNode
  app_state.rs          # MODIFY: Add BrowserManager field
  main.rs               # MODIFY: Wire BrowserManager initialization
```

### Pattern 1: Browser Daemon Lifecycle (BrowserManager)
**What:** A struct that owns the agent-browser child process, tracks its state, and manages the WebSocket stream connection.
**When to use:** All `browser.*` socket commands route through this.
**Example:**
```rust
// src/browser.rs
pub struct BrowserManager {
    /// Child process handle for the agent-browser daemon
    daemon_process: Option<std::process::Child>,
    /// Session name used for socket path discovery
    session_name: String,
    /// WebSocket stream task handle
    stream_task: Option<tokio::task::JoinHandle<()>>,
    /// Sender to forward decoded frames to GTK main thread
    frame_tx: Option<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,
    /// Connection to daemon's Unix socket for sending commands
    daemon_socket: Option<std::path::PathBuf>,
}

impl BrowserManager {
    /// Spawn agent-browser daemon if not running. Auto-start per D-05.
    pub fn ensure_daemon(&mut self) -> Result<(), String> { ... }

    /// Send a command to the agent-browser daemon via its Unix socket
    pub fn send_command(&self, action: &str, params: serde_json::Value) -> Result<serde_json::Value, String> { ... }

    /// Connect to stream WebSocket, start forwarding frames
    pub fn start_stream(&mut self, port: u16) -> Result<(), String> { ... }
}
```

### Pattern 2: Non-Terminal Pane (Preview Leaf)
**What:** Extend `SplitNode` with a variant for non-terminal content display.
**When to use:** Browser preview pane in the split tree.
**Example:**
```rust
// In split_engine.rs, add to SplitNode enum:
pub enum SplitNode {
    Leaf { ... },                    // existing terminal pane
    Preview {                        // NEW: non-terminal content pane
        pane_id: u64,
        picture: gtk4::Picture,      // displays latest frame
        uuid: Uuid,
        pane_type: PreviewType,      // Browser, future: other types
    },
    Split { ... },                   // existing split container
}

pub enum PreviewType {
    Browser,
}
```

### Pattern 3: Frame Delivery Pipeline
**What:** WebSocket frames flow: tokio task -> mpsc channel -> GTK main thread -> Picture widget update.
**When to use:** Continuous stream rendering.
**Example:**
```rust
// In tokio task (browser.rs):
// 1. Connect to ws://127.0.0.1:{port}
// 2. For each message:
//    - Parse JSON, extract "type": "frame", "data": base64
//    - Decode base64 to JPEG bytes
//    - Send bytes via mpsc to GTK thread

// On GTK main thread (triggered by mpsc receiver):
// 1. Receive JPEG bytes
// 2. let bytes = glib::Bytes::from(&jpeg_data);
// 3. let texture = gdk4::Texture::from_bytes(&bytes)?;
// 4. picture.set_paintable(Some(&texture));
```

### Pattern 4: Socket Command Dispatch (existing pattern)
**What:** Add `browser.*` commands following the established pattern in `dispatch_line()`.
**When to use:** All 6 socket commands from D-04.

### Anti-Patterns to Avoid
- **Don't embed WebKitGTK/CEF:** D-03 explicitly defers this. The preview pane shows static frames, not a live browser.
- **Don't poll screenshots:** Agent-browser already has WebSocket streaming with `Page.startScreencast`. Use it.
- **Don't run agent-browser commands synchronously on GTK main thread:** All daemon communication must happen in tokio tasks, with results forwarded via mpsc channels.
- **Don't allocate per-frame on the GTK main thread hot path:** Reuse the `Picture` widget; `set_paintable()` is the only call needed per frame.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Chrome automation | Custom CDP client | agent-browser CLI/daemon | Full CDP protocol implementation with 50+ commands |
| Screenshot streaming | Manual Page.screencastFrame polling | agent-browser `stream enable` | Already handles WebSocket server, auto-start/stop, frame metadata |
| Chrome installation | Manual Chrome download logic | `agent-browser install --with-deps` | Handles Chrome for Testing download, Linux deps |
| JPEG decoding for display | Manual image decoding | `gdk4::Texture::from_bytes()` | GTK4's built-in loader handles JPEG/PNG natively |
| WebSocket client | Raw TCP + HTTP upgrade | tokio-tungstenite | Robust WS implementation, matches agent-browser's server library |

**Key insight:** Agent-browser is a complete, battle-tested automation tool. cmux's job is purely lifecycle management and frame rendering -- not reimplementing any browser automation logic.

## Common Pitfalls

### Pitfall 1: Agent-Browser Daemon Socket Race
**What goes wrong:** cmux sends `browser.open` before the daemon is ready, gets connection refused.
**Why it happens:** Daemon startup takes 1-3 seconds (Chrome launch). The Unix socket does not exist until the daemon binds it.
**How to avoid:** Poll for `$XDG_RUNTIME_DIR/agent-browser/{session}.sock` existence with backoff, matching `ensure_daemon()` in `agent-browser/cli/src/connection.rs` (which uses a 200ms poll with 50 retries).
**Warning signs:** "Connection refused" errors on first `browser.open`.

### Pitfall 2: Stream Port Discovery
**What goes wrong:** cmux tries to connect to stream WebSocket on wrong port.
**Why it happens:** Agent-browser writes the stream port to `{session}.stream` file in the socket dir. If you hardcode port 0 or guess, you miss the actual bound port.
**How to avoid:** After `stream enable`, read `$XDG_RUNTIME_DIR/agent-browser/{session}.stream` file for the actual port. Or parse the response from `stream_enable` command which returns the port.
**Warning signs:** WebSocket connection failures after `stream enable` succeeds.

### Pitfall 3: GTK Main Thread Frame Delivery
**What goes wrong:** Frame updates don't appear, or app freezes.
**Why it happens:** `gtk4::Picture::set_paintable()` must be called on the GTK main thread. Calling from a tokio task causes undefined behavior or silent failure.
**How to avoid:** Use `glib::MainContext::default().spawn_local()` or the existing `mpsc::UnboundedSender` pattern (tokio task sends, GTK main thread receives via `glib::idle_add_local_once`).
**Warning signs:** Frames arrive in tokio task log but Picture never updates.

### Pitfall 4: SplitNode Variant Breakage
**What goes wrong:** Adding `Preview` variant to `SplitNode` breaks all existing `match` arms.
**Why it happens:** Rust's exhaustive pattern matching requires handling every variant.
**How to avoid:** Audit every `match` on `SplitNode` in `split_engine.rs` and `handlers.rs`. Preview panes should be skipped/ignored for terminal-specific operations (surface pointer, GLArea access, attention state).
**Warning signs:** Compiler errors on every `SplitNode` match after adding the variant.

### Pitfall 5: Daemon Cleanup on App Exit
**What goes wrong:** Orphaned Chrome processes after cmux closes.
**Why it happens:** If cmux exits without sending `close` to agent-browser daemon, Chrome stays running.
**How to avoid:** In cmux shutdown hook, send `close` command to daemon socket, then kill the daemon process if it does not exit within 2 seconds. Also handle SIGTERM/SIGINT.
**Warning signs:** `ps aux | grep chrome` shows lingering processes after cmux exit.

### Pitfall 6: Chrome Not Installed
**What goes wrong:** First `browser.open` fails with cryptic error about Chrome binary.
**Why it happens:** User has not run `agent-browser install` to download Chrome for Testing.
**How to avoid:** When daemon fails to start, check error output for Chrome-not-found messages. Return a clear error via socket: `{"error": {"code": "chrome_not_found", "message": "Run 'agent-browser install' to download Chrome"}}`.
**Warning signs:** Daemon process exits immediately after spawn.

## Code Examples

### Agent-Browser Daemon Communication Pattern
```rust
// Source: agent-browser/cli/src/connection.rs
// The daemon uses Unix sockets with newline-delimited JSON (same as cmux socket protocol).
// Send: {"id": "r123", "action": "navigate", "url": "https://example.com"}\n
// Recv: {"success": true, "data": {...}}\n

use std::os::unix::net::UnixStream;
use std::io::{BufRead, BufReader, Write};

fn send_to_daemon(socket_path: &str, command: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut stream = UnixStream::connect(socket_path)
        .map_err(|e| format!("Cannot connect to agent-browser daemon: {}", e))?;
    let mut msg = serde_json::to_string(&command).unwrap();
    msg.push('\n');
    stream.write_all(msg.as_bytes()).map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|e| e.to_string())
}
```

### Stream WebSocket Frame Format
```json
// Source: agent-browser/cli/src/native/stream.rs broadcast_screencast_frame()
{
    "type": "frame",
    "data": "<base64-encoded-jpeg>",
    "metadata": {
        "offsetTop": 0.0,
        "pageScaleFactor": 1.0,
        "deviceWidth": 1280,
        "deviceHeight": 720,
        "scrollOffsetX": 0.0,
        "scrollOffsetY": 0.0,
        "timestamp": 1234567890
    }
}
```

### GTK4 Picture Update from JPEG Bytes
```rust
// Source: gtk4-rs docs, gdk4::Texture API
use gdk4::prelude::*;
use gtk4::prelude::*;

fn update_preview(picture: &gtk4::Picture, jpeg_bytes: &[u8]) {
    let bytes = glib::Bytes::from(jpeg_bytes);
    match gdk4::Texture::from_bytes(&bytes) {
        Ok(texture) => {
            picture.set_paintable(Some(&texture));
        }
        Err(e) => {
            eprintln!("cmux: failed to decode browser frame: {}", e);
        }
    }
}
```

### Socket Path Discovery for Agent-Browser
```rust
// Source: agent-browser/cli/src/connection.rs get_socket_dir()
fn agent_browser_socket_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("AGENT_BROWSER_SOCKET_DIR") {
        if !dir.is_empty() {
            return std::path::PathBuf::from(dir);
        }
    }
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime_dir.is_empty() {
            return std::path::PathBuf::from(runtime_dir).join("agent-browser");
        }
    }
    dirs::home_dir()
        .map(|h| h.join(".agent-browser"))
        .unwrap_or_else(|| std::env::temp_dir().join("agent-browser"))
}

fn agent_browser_socket_path(session: &str) -> std::path::PathBuf {
    agent_browser_socket_dir().join(format!("{}.sock", session))
}

fn agent_browser_stream_port_path(session: &str) -> std::path::PathBuf {
    agent_browser_socket_dir().join(format!("{}.stream", session))
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Embedding WebKitGTK/CEF | CDP streaming to image preview | D-02/D-03 decision | Dramatically simpler; no WebKitGTK dependency; no GTK widget embedding conflicts |
| Polling `screenshot` command | WebSocket `stream enable` | Built into agent-browser | Real-time ~10fps screencast vs one-shot screenshots |
| Spawning separate chrome process | Agent-browser daemon manages Chrome lifecycle | agent-browser architecture | Clean process tree; daemon handles Chrome download/launch/cleanup |

## Open Questions

1. **Binary bundling strategy**
   - What we know: agent-browser is a Rust binary that can be cargo-built. The cmux AppImage ships as a single file.
   - What's unclear: Whether to cargo-build agent-browser as part of cmux CI and embed in AppImage, or expect users to install separately via `npm install -g agent-browser`.
   - Recommendation: Build agent-browser from the in-repo source during CI and embed alongside cmux binary in AppImage. Fall back to `$PATH` lookup if bundled binary not found. This matches cmuxd bundling pattern referenced in CONTEXT.md.

2. **Session naming**
   - What we know: Agent-browser uses session names to isolate multiple instances (default: "default"). Each session gets its own socket/pid/stream files.
   - What's unclear: Should cmux use a unique session name (e.g., "cmux-{workspace-uuid}") or always use "default"?
   - Recommendation: Use "cmux" as session name. One daemon per cmux instance. Multiple browser tabs are handled by agent-browser's `tab` command, not by spawning multiple daemons.

3. **Frame rate / resource usage**
   - What we know: CDP `Page.startScreencast` can be configured for quality and max FPS. Agent-browser's stream server broadcasts all frames.
   - What's unclear: What frame rate/quality is appropriate for a terminal-adjacent preview pane.
   - Recommendation: Default to 5fps, JPEG quality 60. This is sufficient for seeing "what the agent sees" without excessive CPU/bandwidth. Make configurable later if needed.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo (Rust) | Building agent-browser binary | Yes | 1.91.1 | -- |
| google-chrome | CDP target browser | Yes | (system) | `agent-browser install` downloads Chrome for Testing |
| agent-browser CLI | All browser.* commands | No (not in PATH) | -- | Build from in-repo source at `agent-browser/cli/` |
| GTK4 | UI framework | Yes | (project dependency) | -- |
| tokio-tungstenite | WebSocket client | No (not yet in Cargo.toml) | -- | Add to dependencies |

**Missing dependencies with no fallback:**
- None -- all can be resolved at build time

**Missing dependencies with fallback:**
- agent-browser CLI: Build from `agent-browser/cli/` source, or look up in `$PATH`

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust unit tests) + Python socket tests |
| Config file | Cargo.toml |
| Quick run command | `cargo test --bin cmux-linux` |
| Full suite command | `cargo test --bin cmux-linux` + CI socket tests |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 | agent-browser binary found and spawnable | unit | `cargo test --bin cmux-linux browser_manager` | No -- Wave 0 |
| D-02 | CDP stream frames render in preview pane | manual | Visual verification (stream frames appear) | N/A |
| D-04 | browser.* socket commands work | integration | `CMUX_SOCKET=... python tests_v2/test_browser_api_p0.py` | Exists (macOS) |
| D-05 | Auto-start daemon on first browser.* command | unit | `cargo test --bin cmux-linux browser_auto_start` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --bin cmux-linux`
- **Per wave merge:** `cargo test --bin cmux-linux` + `cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/browser.rs` unit tests for daemon lifecycle (spawn, connect, cleanup)
- [ ] Socket command dispatch tests for `browser.*` namespace
- [ ] Note: Existing `tests_v2/test_browser_api_p0.py` tests the macOS socket protocol and may need adaptation for Linux

## Project Constraints (from CLAUDE.md)

- **Socket command threading policy:** `browser.*` commands should parse/validate off-main. Only UI mutations (Picture widget updates) go to GTK main thread via `glib::idle_add_local_once`.
- **Socket focus policy:** `browser.*` commands must not steal app focus. No window raising.
- **Test quality policy:** Tests must verify runtime behavior, not source text or metadata files. Stream frame rendering is inherently visual -- unit test the daemon lifecycle and socket protocol, not the rendering.
- **Never run tests locally.** All integration tests run via CI.
- **Typing-latency paths:** The browser preview pane must not introduce work in existing keyboard event paths. SplitNode variant additions must handle Preview gracefully (skip/no-op) in terminal-specific hot paths.

## Sources

### Primary (HIGH confidence)
- `agent-browser/cli/src/native/stream.rs` -- WebSocket stream server implementation, frame format, screencast control
- `agent-browser/cli/src/connection.rs` -- Daemon socket protocol, socket dir discovery, ensure_daemon logic
- `agent-browser/cli/src/native/daemon.rs` -- Daemon startup, stream server initialization
- `agent-browser/cli/src/commands.rs` -- CLI command parsing, stream enable/disable/status
- `agent-browser/README.md` -- Full CLI reference, installation instructions
- `src/socket/handlers.rs` -- Existing socket command handler pattern
- `src/socket/commands.rs` -- SocketCommand enum structure
- `src/socket/mod.rs` -- Socket server architecture, dispatch_line pattern
- `src/split_engine.rs` -- SplitNode enum, pane tree structure
- `src/app_state.rs` -- AppState struct, lifecycle management pattern

### Secondary (MEDIUM confidence)
- GTK4 `Picture` widget docs -- `set_paintable()` for texture display
- GDK4 `Texture::from_bytes()` -- JPEG/PNG decoding to GPU texture

### Tertiary (LOW confidence)
- Frame rate recommendations (5fps/quality 60) -- based on typical CDP screencast usage, not benchmarked for this specific use case

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries are either already in project or standard Rust ecosystem choices matching agent-browser's own dependencies
- Architecture: HIGH -- patterns follow established cmux conventions (socket commands, mpsc bridge, SplitNode tree) and agent-browser's daemon/stream architecture
- Pitfalls: HIGH -- derived from reading actual source code of both projects; daemon race condition and socket discovery are well-documented in agent-browser's own connection.rs

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (stable -- agent-browser CLI and cmux socket patterns are mature)
