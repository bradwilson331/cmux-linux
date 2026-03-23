# Architecture Research

**Domain:** Linux terminal multiplexer — Rust + GTK4 + Ghostty libghostty C API
**Researched:** 2026-03-23
**Confidence:** MEDIUM — Ghostty submodule not initialized; analysis based on ghostty.h C API contract, macOS embedding patterns, and GTK4/Rust ecosystem knowledge.

---

## Critical Constraint: The libghostty Embedding API

The `ghostty.h` file in this repo reveals a hard constraint that shapes the entire architecture.

The current embedding API defines only two platform types:

```c
typedef struct { void* nsview; } ghostty_platform_macos_s;
typedef struct { void* uiview;  } ghostty_platform_ios_s;
```

There is no `ghostty_platform_linux_s` or GTK widget pointer. This means the macOS embedding model — where the host app creates a native view and passes it to `ghostty_surface_new()` — does not have a direct Linux equivalent in the current public API.

**What this means for the port:** The manaflow-ai fork must be extended to add a Linux platform variant to `ghostty_platform_u` that accepts a GDK/GTK drawing context or an OpenGL/Vulkan surface handle. This is a required prerequisite before any Rust GTK4 embedding can occur. This fork work is the single highest-risk item in the project.

**Ghostty's own Linux app** bypasses this entirely — it is written in Zig and uses Ghostty's internal `apprt/gtk` module directly, not the embedding C API. That route is not available to a Rust host app.

---

## How Ghostty Surfaces Work (C API Model)

Based on `ghostty.h`:

**Lifecycle:**
1. `ghostty_init()` — one-time global init (must call before anything else)
2. `ghostty_config_new()` / `ghostty_config_load_*()` / `ghostty_config_finalize()` — load config
3. `ghostty_app_new(runtime_config, config)` — create one app instance; pass callback table
4. `ghostty_surface_new(app, surface_config)` — create one surface per terminal pane
5. App is responsible for pumping the event loop: `ghostty_app_tick()` must be called periodically
6. `ghostty_surface_free()` / `ghostty_app_free()` — teardown

**Callback contract (`ghostty_runtime_config_s`):**

The host app provides these callbacks at `ghostty_app_new` time:
- `wakeup_cb` — Ghostty signals it needs a redraw; host must schedule a draw call
- `action_cb` — Ghostty requests a UI action (new tab, close, split, goto tab, set title, ring bell, etc.)
- `read_clipboard_cb` / `write_clipboard_cb` — clipboard bridge
- `close_surface_cb` — surface wants to close (child process exited)

**Input contract:**
- Host feeds keyboard: `ghostty_surface_key()`, `ghostty_surface_text()`
- Host feeds mouse: `ghostty_surface_mouse_button()`, `ghostty_surface_mouse_pos()`, `ghostty_surface_mouse_scroll()`
- Host feeds sizing: `ghostty_surface_set_size()`, `ghostty_surface_set_content_scale()`
- Host feeds focus: `ghostty_surface_set_focus()`, `ghostty_surface_set_occlusion()`

**Render contract:**
- Ghostty renders internally; on macOS it uses Metal + IOSurface
- On Linux it will need to render into an OpenGL or Vulkan context
- `ghostty_surface_draw()` triggers a synchronous draw; `ghostty_surface_refresh()` marks dirty

**Context types for surfaces:**
```c
GHOSTTY_SURFACE_CONTEXT_WINDOW = 0
GHOSTTY_SURFACE_CONTEXT_TAB = 1
GHOSTTY_SURFACE_CONTEXT_SPLIT = 2
```
This context affects inherited config (font size can differ between window/tab/split contexts).

---

## System Overview

```
┌───────────────────────────────────────────────────────────────────┐
│                        GTK4 Main Thread                           │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                    AppShell (GtkApplicationWindow)          │  │
│  │  ┌──────────────┐  ┌────────────────────────────────────┐   │  │
│  │  │  Sidebar     │  │  WorkspaceView                     │   │  │
│  │  │  (workspace  │  │  ┌──────────┐  ┌──────────────┐   │   │  │
│  │  │   list)      │  │  │ PaneNode │  │  PaneNode    │   │   │  │
│  │  │              │  │  │ (leaf)   │  │  (branch)    │   │   │  │
│  │  │              │  │  │          │  │ ┌────┐ ┌───┐ │   │   │  │
│  │  │              │  │  │ GhosttyWi│  │ │    │ │   │ │   │   │  │
│  │  │              │  │  │ dget     │  │ └────┘ └───┘ │   │   │  │
│  │  └──────────────┘  │  └──────────┘  └──────────────┘   │   │  │
│  │                    └────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │  AppState      │  │  WorkspaceManager│  │  SplitEngine     │  │
│  │  (Rc<RefCell>) │  │                  │  │  (tree layout)   │  │
│  └────────────────┘  └──────────────────┘  └──────────────────┘  │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │              GhosttyBridge (unsafe FFI)                    │   │
│  │  ghostty_app_t  +  per-surface ghostty_surface_t handles   │   │
│  └────────────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────────────┘
                              │ glib::idle_add / channel
┌───────────────────────────────────────────────────────────────────┐
│                    Tokio Async Runtime (separate thread pool)     │
│                                                                   │
│  ┌─────────────────────────────────────┐  ┌────────────────────┐  │
│  │  SocketServer (UnixListener)        │  │  SessionPersist    │  │
│  │  v1 line protocol + v2 JSON-RPC     │  │  (async file I/O)  │  │
│  └─────────────────────────────────────┘  └────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
```

---

## Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| **AppShell** | GTK4 `GtkApplicationWindow`, menu bar, keyboard shortcut routing, window lifecycle | WorkspaceManager, GhosttyBridge, SplitEngine |
| **WorkspaceManager** | Ordered list of workspaces; create/close/switch/rename; owns workspace state | AppShell (UI updates), SessionPersistence, SocketServer (command dispatch) |
| **SplitEngine** | Recursive tree of pane nodes (branch = split axis + ratio, leaf = surface ID); layout math | WorkspaceManager (owns tree per workspace), AppShell (triggers GTK layout) |
| **GhosttyBridge** | Owns `ghostty_app_t`; creates/frees `ghostty_surface_t` handles; feeds input; routes action callbacks | All components via callback dispatch, GTK main thread only |
| **GhosttyWidget** | GTK4 custom widget per surface; hosts OpenGL/Vulkan context; forwards GTK events to GhosttyBridge | GhosttyBridge (draw/input), SplitEngine (sizing) |
| **SocketServer** | Tokio async `UnixListener`; parses v1/v2 protocol; sends commands to main thread via channel | WorkspaceManager via `glib::idle_add` dispatch channel |
| **ConfigLoader** | Loads `~/.config/cmux/config.toml` and Ghostty config; provides merged config to all components | GhosttyBridge (ghostty config), AppShell (shortcuts), WorkspaceManager (defaults) |
| **SessionPersistence** | Serialize/deserialize workspace+pane tree to `~/.local/share/cmux/session.json` | WorkspaceManager (save/restore), async file I/O via Tokio |
| **NotificationStore** | Per-surface unread/attention state; parses OSC 99 / bell signals from action callbacks | GhosttyBridge (bell/notification actions), AppShell (sidebar badge rendering) |

---

## Ghostty Embedding Strategy (Concrete)

### Required Fork Work

The manaflow-ai/ghostty fork needs a Linux platform entry added to the embedding API:

```c
// NEW: to be added to ghostty.h + Zig apprt/embedded.zig
typedef struct {
  // Pointer to the host's GDK/EGL surface or OpenGL context handle.
  // Exact type determined during fork investigation.
  void* gl_context;  // or: GdkGLContext*, EGLSurface, etc.
} ghostty_platform_linux_s;
```

Until this exists, the embedding approach on Linux is **blocked**. The initial phase of the project must include a spike/investigation into what Ghostty's embedded renderer needs on Linux to render into a host-provided GL context.

**Alternative approach to investigate:** Rather than extending the C embedding API, the host Rust app could instantiate a Ghostty GTK4 app runtime (`apprt/gtk`) as a library — treating Ghostty as the outer event loop and the Rust app as a plugin. This is architecturally inverted from the macOS model but may be the path of least resistance if Ghostty's GTK apprt exposes enough hooks. LOW confidence — needs investigation.

### Preferred Embedding Model (if fork extension succeeds)

Each terminal pane = one `ghostty_surface_t` + one custom `GtkWidget`:

```
GhosttyWidget (GtkGLArea or GtkDrawingArea subclass)
  owns: ghostty_surface_t
  on realize: create GL context, call ghostty_surface_set_size()
  on draw: call ghostty_surface_draw()
  on key-press: call ghostty_surface_key()
  on focus-in/out: call ghostty_surface_set_focus()
  on resize: call ghostty_surface_set_size() + ghostty_surface_set_content_scale()
```

`GtkGLArea` is the natural host widget — it manages an OpenGL context and calls `realize`/`render` signals that map directly to the Ghostty surface lifecycle.

### Action Callback Routing

Ghostty calls back into the host via `action_cb` for user-initiated actions (new tab, split, close, goto tab, etc.). These arrive on whatever thread Ghostty's internal renderer runs on — **not necessarily the GTK main thread**. The bridge must:

1. Receive the action callback (any thread)
2. Package it as a message and send it to the GTK main thread via `glib::idle_add` or a `glib::MainContext::channel`
3. Execute the state mutation on the GTK main thread

This is the primary concurrency boundary. All GTK and AppState mutations must happen on the GTK main thread.

---

## Thread Model

```
GTK Main Thread                   Tokio Thread Pool
─────────────────────────────     ──────────────────────────────────
GtkApplication event loop         SocketServer accept loop
AppState mutations                Command parsing (off-thread)
GhosttyBridge calls               SessionPersistence async I/O
SplitEngine layout
Widget draw/input
                │                              │
                └──── glib::MainContext ────────┘
                      channel (mpsc sender)
                      idle_add callbacks
```

**Rule:** GTK APIs and `ghostty_*` calls are GTK-main-thread-only. Tokio tasks send work to the main thread via a `glib::MainContext::channel` sender; they never call GTK or Ghostty APIs directly.

**Ghostty wakeup callback:** When Ghostty calls `wakeup_cb`, this may arrive from an internal Ghostty renderer thread. The wakeup handler must call `widget.queue_draw()` via `glib::idle_add` — never directly from the callback if called off-main-thread.

---

## GTK4 Event Loop + Tokio Integration

GTK4's GLib event loop and Tokio's async runtime cannot share the same thread pool directly. The standard Rust pattern:

```rust
fn main() {
    // Start Tokio runtime on background threads
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();
    let _guard = rt.enter();

    // Create a glib channel for cross-thread communication
    let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);

    // Attach receiver to GTK main context — runs on GTK main thread
    receiver.attach(None, move |msg: AppCommand| {
        handle_command(msg);
        glib::ControlFlow::Continue
    });

    // Pass sender clone to Tokio tasks (SocketServer, etc.)
    rt.spawn(socket_server(sender.clone()));
    rt.spawn(session_persistence_loop(sender.clone()));

    // GTK main loop — blocks until app exits
    app.run();

    // Shutdown Tokio after GTK exits
    rt.shutdown_background();
}
```

**Do not use** `tokio::main` as the outer runtime — GTK must own the main thread. Tokio runs on worker threads with `rt.spawn()`.

**gtk4-rs `glib::MainContext::spawn_local`** can schedule async tasks on the GTK main thread for lightweight async work (e.g., a debounced save), but this is cooperative not preemptive and must not block.

---

## Data Flow

### Keyboard Input → Terminal

```
GtkWidget key-press-event (GTK main thread)
  → GhosttyWidget::on_key_press()
  → ghostty_surface_key(surface, key_event)     [C FFI, synchronous]
  → Ghostty internal: key → PTY stdin
  → Terminal renders → Ghostty wakeup_cb fires
  → wakeup_cb: glib::idle_add(|| widget.queue_draw())
  → GTK schedules redraw → GhosttyWidget::on_draw()
  → ghostty_surface_draw(surface)               [C FFI, synchronous]
```

### Socket Command → State Change

```
Tokio: UnixListener accept() → read line(s)
  → parse v1/v2 command (off-thread)
  → validate args (off-thread)
  → sender.send(AppCommand::WorkspaceCreate { ... })
  → glib::MainContext receiver fires on GTK main thread
  → WorkspaceManager::create_workspace()
  → SplitEngine creates initial leaf node
  → GhosttyBridge::create_surface()
  → AppShell updates GTK widget tree
  → sender.send(AppResponse::Ok { ... }) back to Tokio
  → Tokio writes JSON response to socket
```

### Split Divider Drag → Layout Update

```
GtkGestureDrag on divider widget (GTK main thread)
  → SplitEngine::update_ratio(node_id, delta)
  → Returns new tree (immutable update)
  → WorkspaceManager stores updated tree
  → AppShell triggers GTK re-layout
  → Each GhosttyWidget receives new allocation
  → ghostty_surface_set_size() for each resized surface
```

### Session Persistence

```
WorkspaceManager state change
  → debounce timer (glib::timeout_add)
  → serialize AppState to SessionSnapshot struct
  → sender.send(AppCommand::PersistSession(snapshot))
  → Tokio task: serde_json::to_string() + async file write
  → atomic write to ~/.local/share/cmux/session.json

App launch:
  → ConfigLoader reads session.json synchronously before GTK show
  → WorkspaceManager::restore_from_snapshot()
  → GhosttyBridge creates surfaces for each pane in snapshot
  → initial_input replayed per pane (working dir, shell cmd)
```

### Terminal Output → Notification State

```
Ghostty internal: terminal output parsed
  → action_cb fires: GHOSTTY_ACTION_RING_BELL or notification OSC 99
  → GhosttyBridge packages: NotificationEvent { surface_id, kind }
  → glib::idle_add delivers to GTK main thread
  → NotificationStore::update(surface_id, kind)
  → AppShell sidebar re-renders badge for affected workspace
```

---

## Recommended Project Structure

```
src/
├── main.rs                    # Entry point: Tokio + GTK setup, channel wiring
├── app/
│   ├── mod.rs                 # AppShell: GtkApplicationWindow, menus
│   ├── state.rs               # AppState: Rc<RefCell<AppStateInner>>, command enum
│   └── commands.rs            # AppCommand enum (cross-thread message types)
├── workspace/
│   ├── mod.rs                 # WorkspaceManager: create/close/switch
│   ├── workspace.rs           # Workspace struct: id, name, split_tree, notification state
│   └── session.rs             # SessionSnapshot serde types, save/restore
├── split/
│   ├── mod.rs                 # SplitEngine: layout tree operations
│   └── tree.rs                # SplitNode enum (Branch { axis, ratio, left, right }, Leaf { surface_id })
├── ghostty/
│   ├── mod.rs                 # GhosttyBridge: app/surface lifecycle, callback routing
│   ├── ffi.rs                 # Raw bindgen bindings to ghostty.h
│   ├── surface.rs             # GhosttyWidget (GtkGLArea subclass)
│   └── callbacks.rs           # Action callback dispatch (wakeup, action, clipboard, close)
├── socket/
│   ├── mod.rs                 # SocketServer: Tokio UnixListener, request routing
│   ├── v1.rs                  # V1 line protocol parser
│   └── v2.rs                  # V2 JSON-RPC parser + handler
├── config/
│   ├── mod.rs                 # ConfigLoader: cmux TOML + Ghostty config
│   └── shortcuts.rs           # Keybinding config → GTK shortcut registration
├── notifications/
│   └── mod.rs                 # NotificationStore: per-surface attention state
└── persistence/
    └── mod.rs                 # Async session file I/O (Tokio)
```

### Structure Rationale

- **`ghostty/`:** All unsafe FFI isolated here. GhosttyBridge is the only component allowed to hold raw `ghostty_*` pointers. Everything else works with `SurfaceId` (a typed integer wrapper).
- **`split/`:** Pure Rust layout logic with no GTK dependencies. Can be unit-tested in isolation. The tree is an immutable value — updates return a new tree.
- **`socket/`:** Entirely async/Tokio. Zero GTK imports. Communicates via `AppCommand` channel only.
- **`workspace/`:** Owns the split tree and notification state per workspace. GTK main thread only.
- **`app/state.rs`:** `AppState` is `Rc<RefCell<...>>` (single-threaded GTK) not `Arc<Mutex<...>>`. Only GTK main thread touches it.

---

## Architectural Patterns

### Pattern 1: Surface ID Indirection

**What:** GhosttyBridge owns all `ghostty_surface_t` pointers internally. Other components reference surfaces by `SurfaceId` (a newtype over u64). To perform a surface operation, they call `bridge.with_surface(id, |s| ghostty_surface_key(s, ...))`.

**When to use:** Always. This prevents raw pointer leakage into non-FFI code and makes ownership explicit.

**Trade-offs:** Slight indirection overhead; simplifies lifetime management significantly.

```rust
pub struct SurfaceId(u64);

impl GhosttyBridge {
    pub fn with_surface<F, R>(&self, id: SurfaceId, f: F) -> Option<R>
    where F: FnOnce(ghostty_surface_t) -> R
    {
        self.surfaces.get(&id).map(|&ptr| f(ptr))
    }
}
```

### Pattern 2: GTK Main Thread Assertion

**What:** All methods on `AppState`, `WorkspaceManager`, `GhosttyBridge`, and GTK widgets include a debug assertion that they are called on the main thread.

**When to use:** Any type that owns GTK objects or `ghostty_*` handles.

**Trade-offs:** Negligible runtime cost in debug builds; catches threading bugs during development.

```rust
fn assert_main_thread() {
    debug_assert!(
        glib::MainContext::default().is_owner(),
        "Must be called on GTK main thread"
    );
}
```

### Pattern 3: Command Channel for Cross-Thread State Mutation

**What:** All state mutations from async (Tokio) context are expressed as `AppCommand` enum variants sent through a `glib::MainContext::channel`. The GTK-side receiver is the single point of entry for async-originated mutations.

**When to use:** Any Tokio task that needs to mutate app state or trigger UI changes.

**Trade-offs:** All async→main operations are serialized through one channel; for high-frequency telemetry (socket polling), use a bounded channel with backpressure.

### Pattern 4: Immutable Split Tree Updates

**What:** `SplitNode` is an immutable recursive enum. Every split/close/resize operation returns a new tree root. WorkspaceManager stores the current root and replaces it atomically.

**When to use:** All layout operations.

**Trade-offs:** Clone cost for large trees (typically shallow — rarely >20 nodes); enables trivial undo/session snapshot via tree comparison.

```rust
pub enum SplitNode {
    Leaf { surface_id: SurfaceId },
    Branch { axis: SplitAxis, ratio: f64, left: Box<SplitNode>, right: Box<SplitNode> },
}
```

---

## Build Order / Dependency Graph

Components must be built in this order because of hard dependencies:

```
1. ghostty/ffi.rs           — bindgen from ghostty.h; no Rust deps
      ↓
2. ghostty/callbacks.rs     — defines action callback types; depends on ffi
      ↓
3. split/tree.rs            — pure Rust; no external deps
      ↓
4. app/commands.rs          — AppCommand enum; depends on split/tree (for layout commands)
      ↓
5. ghostty/surface.rs       — GtkGLArea subclass; depends on ffi + commands
6. ghostty/mod.rs           — GhosttyBridge; depends on surface + callbacks
      ↓
7. notifications/mod.rs     — depends on commands (for bell/notification actions)
8. workspace/workspace.rs   — depends on split/tree + notifications
9. workspace/session.rs     — serde types; depends on workspace
10. workspace/mod.rs        — WorkspaceManager; depends on workspace + session + GhosttyBridge
      ↓
11. config/mod.rs            — ConfigLoader; depends on workspace defaults
12. socket/v1.rs + v2.rs    — protocol parsers; depend on commands
13. socket/mod.rs            — SocketServer; depends on v1/v2 + commands
      ↓
14. persistence/mod.rs       — depends on workspace/session
      ↓
15. app/mod.rs               — AppShell; depends on everything
      ↓
16. main.rs                  — wires Tokio + GTK + channel; entry point
```

**Phase implications:**
- Phases 1-2: Items 1-4 (FFI scaffolding + split tree — can be built and tested without GTK)
- Phase 3: Items 5-10 (GTK widget + workspace management — first runnable app)
- Phase 4: Items 11-13 (config + socket server — CLI-controllable app)
- Phase 5: Items 14-16 (persistence + polish)

---

## Integration Points

### External: libghostty (C FFI)

| Integration | Pattern | Notes |
|-------------|---------|-------|
| Build | `build.rs` with `bindgen` on `ghostty.h` + link to `libghostty.a` | Zig build step required first; see CLAUDE.md |
| Thread safety | `ghostty_*` calls: GTK main thread only | `ghostty_app_t` and `ghostty_surface_t` are not `Send`; wrap in `Rc` not `Arc` |
| Render loop | GTK drives redraws via `GtkGLArea::render` signal | Do not add independent render loop — causes typing lag (per macOS CLAUDE.md pitfall) |

### External: GTK4 (gtk4-rs)

| Integration | Pattern | Notes |
|-------------|---------|-------|
| Custom widget | `GtkGLArea` subclass via `glib::subclass` | Use `glib-macros::Properties` for surface_id binding |
| Event handling | `GtkEventControllerKey`, `GtkGestureClick`, `GtkEventControllerMotion` | Attach to each GhosttyWidget instance |
| Cross-thread | `glib::MainContext::channel` + `glib::idle_add` | Never call GTK APIs from Tokio threads |

### Internal: Socket ↔ AppState

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Tokio → GTK | `glib::MainContext::channel` mpsc sender | Bounded channel with backpressure for high-frequency commands |
| GTK → Tokio | `tokio::sync::oneshot` via `AppCommand` response field | Socket server awaits response for synchronous RPC methods |
| GhosttyBridge → GTK | `glib::idle_add` from Ghostty callbacks | Wakeup and action callbacks may fire from non-main threads |

---

## Architectural Risks

### Risk 1: Linux Platform Missing from Embedding API (HIGH severity)

**What:** `ghostty.h` has no `ghostty_platform_linux_s`. The surface config only supports macOS/iOS platform pointers. Without a Linux platform variant, `ghostty_surface_new()` cannot be called from a Rust/GTK host.

**Mitigation:** First milestone deliverable must be a fork investigation spike: either (a) add a Linux GTK platform struct to the embedding API in manaflow-ai/ghostty, or (b) determine if Ghostty's GTK apprt can be used as a library with the Rust app driving the GLib main loop.

**Detection:** If this is not resolved, no terminal surfaces can be displayed. It will block Phase 3 entirely.

### Risk 2: Ghostty Renderer Thread Model on Linux (MEDIUM severity)

**What:** On macOS, Ghostty renders via a CVDisplayLink callback on a separate Metal renderer thread. On Linux (GTK4), rendering likely uses a GDK frame clock or GtkGLArea render signal. The wakeup callback threading model may differ from macOS, affecting how `glib::idle_add` should be used.

**Mitigation:** After the embedding API is established, validate that `wakeup_cb` → `queue_draw()` round-trip produces correct rendering without stutter. Test with high-frequency terminal output (e.g., `cat /dev/urandom | head -c 10M`).

### Risk 3: GTK4 + Tokio Event Loop Contention (MEDIUM severity)

**What:** GLib and Tokio both want to drive their own event loops. Integrating them requires care — blocking the GTK main thread (e.g., with `block_on`) will freeze the UI.

**Mitigation:** Use the `glib::MainContext::channel` pattern exclusively. Never use `tokio::runtime::Handle::current().block_on()` from GTK callbacks. For async GTK work, use `glib::MainContext::spawn_local` with non-blocking futures.

### Risk 4: Ghostty Config API Ownership (LOW severity)

**What:** `ghostty_config_t` is heap-allocated by Ghostty and must be freed with `ghostty_config_free`. On Linux, the config file path conventions differ from macOS (`~/.config/ghostty/` vs macOS app support dir). The Rust config layer needs to bridge cmux's own config to Ghostty's config loader.

**Mitigation:** Call `ghostty_config_load_default_files()` to handle platform-specific defaults, then apply cmux overrides with `ghostty_config_load_file()` for a cmux-managed config fragment.

---

## Anti-Patterns

### Anti-Pattern 1: Independent Render Loop

**What people do:** Call `ghostty_surface_draw()` on a timer or in a background thread to ensure smooth rendering.

**Why it's wrong:** Causes input latency (same macOS pitfall documented in CLAUDE.md). Ghostty's wakeup callback already signals when a redraw is needed. Adding an independent loop creates double-renders and competes with GTK's frame clock.

**Do this instead:** Only draw in response to wakeup_cb → `queue_draw()` → GtkGLArea `render` signal.

### Anti-Pattern 2: Shared Mutable State Across Threads

**What people do:** Wrap `AppState` in `Arc<Mutex<>>` so Tokio tasks can mutate it directly.

**Why it's wrong:** GTK objects (`GtkWidget`, etc.) are not `Send`. Ghostty surface handles are not thread-safe. `Arc<Mutex<AppState>>` that contains GTK refs will panic or cause UB when accessed from Tokio threads.

**Do this instead:** `AppState` is `Rc<RefCell<>>` (main thread only). Tokio tasks send `AppCommand` messages via channel. The GTK-side receiver applies mutations on the main thread.

### Anti-Pattern 3: One GhosttyBridge Per Surface

**What people do:** Create a separate `ghostty_app_t` for each terminal surface to avoid coordination.

**Why it's wrong:** `ghostty_app_t` represents the entire application runtime including config, font cache, and global keybind state. Multiple app instances cause config duplication, memory bloat, and incorrect keybind behavior.

**Do this instead:** One `ghostty_app_t` per process. Multiple `ghostty_surface_t` per app (one per pane).

### Anti-Pattern 4: Blocking Socket Responses on GTK Main Thread

**What people do:** Handle socket commands synchronously on the GTK main thread by blocking until a response is ready.

**Why it's wrong:** A slow or misbehaving socket client can freeze the entire UI.

**Do this instead:** Socket parsing and validation happens entirely in Tokio. State queries that need the main thread use a `oneshot::channel` — the Tokio task awaits the oneshot while the GTK-side handler computes the result and sends it back. GTK main thread is never blocked.

---

## Sources

- `ghostty.h` in this repo — definitive C API contract (confidence: HIGH, first-party)
- `.planning/codebase/ARCHITECTURE.md` — macOS embedding patterns to mirror (confidence: HIGH, first-party analysis)
- `docs/ghostty-fork.md` — manaflow fork scope and current patches (confidence: HIGH, first-party)
- `docs/v2-api-migration.md` — V2 JSON-RPC wire protocol (confidence: HIGH, first-party)
- `Sources/GhosttyTerminalView.swift` — macOS rendering contract patterns (confidence: HIGH, first-party)
- `Sources/TerminalController.swift` — macOS socket server patterns (confidence: HIGH, first-party)
- gtk4-rs book (https://gtk-rs.org/gtk4-rs/stable/latest/book/) — GTK4 Rust patterns (confidence: MEDIUM, not verified during this session due to WebFetch restriction)
- GLib main context channel pattern — standard gtk-rs cross-thread communication (confidence: MEDIUM, based on training knowledge, needs verification against current gtk4-rs docs)

---

*Architecture research for: cmux Linux port (Rust + GTK4 + Ghostty)*
*Researched: 2026-03-23*
