# Technology Stack

**Project:** cmux-linux
**Researched:** 2026-03-23
**Overall confidence:** MEDIUM (C API analysis is HIGH from source; gtk4-rs/Rust ecosystem from training data + crates.io knowledge, capped at MEDIUM without live verification)

---

## Critical Pre-Decision: The Embedded C API Does Not Support Linux

Before listing the stack, this constraint must be understood because it shapes every other decision.

The `ghostty.h` C API in this repo (and upstream Ghostty) only defines two platform variants:

```c
typedef enum {
  GHOSTTY_PLATFORM_INVALID,
  GHOSTTY_PLATFORM_MACOS,
  GHOSTTY_PLATFORM_IOS,
} ghostty_platform_e;
```

The `ghostty_surface_config_s.platform` union only has `macos` (void* nsview) and `ios` (void* uiview) members. There is no Linux/GTK4 platform entry. The `#ifdef __APPLE__` blocks guard Metal-specific surface functions.

**Conclusion (HIGH confidence — from source):** The embedded C API (`ghostty_surface_new` + platform union) cannot embed Ghostty terminal surfaces into a custom Linux application today. On Linux, Ghostty operates via its own full GTK4 application runtime (`src/apprt/gtk`), not via the embedding API.

**Implication:** The Linux port cannot simply call `ghostty_surface_new` with a GTK4 widget pointer the way macOS calls it with an NSView. There are three viable approaches, ranked:

| Approach | What it means | Feasibility |
|----------|---------------|-------------|
| A: Extend libghostty C API for Linux | Add `GHOSTTY_PLATFORM_GTK4` + GdkSurface/GtkWidget pointer to the platform union; build libghostty.so with an embedded GTK4 apprt | HIGH effort, requires Zig + upstream contribution |
| B: Fork Ghostty's GTK4 apprt as a multi-surface host | Instead of a separate Rust app, add multi-tab/split/socket control to Ghostty's own GTK4 app, written in Zig + C | Not Rust, departs from project goals |
| C: IPC/Process approach | Run multiple Ghostty instances, position their windows inside a Rust GTK4 container via GtkSocket/XEmbed or Wayland foreign-toplevel protocols | Fragile on Wayland (XEmbed is X11-only); not recommended |

**Recommended path: Approach A.** Extend the Ghostty manaflow-ai fork to add a Linux embedded apprt (`GHOSTTY_PLATFORM_GTK4`) that accepts a `GtkWidget*` (specifically a `GtkGLArea` or `GdkSurface`), builds as libghostty.so on Linux, and exposes the same C API surface as macOS. This is significant upfront work but the only path to a clean multi-surface Rust app matching the macOS architecture. The fork infrastructure already exists for macOS-specific patches; this is the same mechanism.

---

## Recommended Stack

### Core Language

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust | 1.77+ (stable) | Application layer | Memory safety, strong GTK4 bindings, system programming, no GC |
| Zig | 0.14+ | Build libghostty.so | Same toolchain already used for macOS GhosttyKit; required to build Ghostty |

### UI Framework

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| gtk4-rs (`gtk4` crate) | 0.9.x | Application shell — sidebar, tab bar, split layout, dialogs | GTK4 is the only viable choice for embedding Ghostty terminal surfaces on Linux. GTK4 widgets host `GtkGLArea` surfaces where Ghostty renders via OpenGL/Vulkan. No other Rust GUI framework (iced, egui, slint) can host a GTK4 GdkSurface. |
| gtk4-rs (`gdk4` crate) | 0.9.x | GDK display, surface, and event plumbing | Companion crate; needed for content scale, focus, input events |
| gtk4-rs (`glib` crate) | 0.20.x | GLib main loop integration, signals | GTK4 requires GLib event loop; used for idle/timeout callbacks |

**Why NOT iced:** iced is an Elm-style Rust framework backed by wgpu (its own GPU renderer). It has no concept of hosting a GTK4 widget or GdkSurface. iced cannot receive GTK4 input events on behalf of embedded native widgets. To use iced, you would need to also run a GTK4 main loop in parallel and bridge input/render — this is architecturally broken and not supported. **iced is eliminated.** (MEDIUM confidence — based on iced architecture as of 0.12.x; confirmed by the fact that iced's renderer bypasses GTK entirely.)

**Why NOT egui:** Same issue — egui runs on wgpu or glow, not GTK4. Cannot host GTK4 surfaces.

**Why NOT slint:** Slint has a GTK4 backend in experimental state, but it renders its own widgets — it cannot embed a `GtkGLArea` from Ghostty. Eliminated.

**gtk4-rs maturity:** gtk-rs is the official Rust binding project maintained by the GTK team. The `gtk4` crate follows GTK4's release cadence. As of GTK 4.14/4.16 (2024-2025), the bindings are stable and production-ready. Used in production by GNOME apps written in Rust (e.g., Loupe image viewer). (MEDIUM confidence — from training data; verify exact version at `https://crates.io/crates/gtk4` before pinning.)

### Terminal Engine Integration

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| libghostty (custom Linux build) | manaflow fork | Terminal rendering, PTY management, VTE protocol | Must remain Ghostty; this is a hard project constraint |
| bindgen | 0.70.x | Generate Rust FFI bindings from ghostty.h | Automated, tracks API changes; alternative is manual `unsafe extern "C"` bindings |
| `cc` crate | 1.x | Link libghostty.so from build.rs | Standard approach for linking C libraries in Rust |

**FFI approach — bindgen vs manual:**
- **bindgen** (recommended): Run at build time via `build.rs`. Automatically regenerates when `ghostty.h` changes. Handles the extensive enum/struct surface of the API without manual error. The `ghostty.h` is stable enough for automated bindings.
- Manual FFI: Only appropriate if bindgen output needs heavy curation. Given ~1,169 lines of `ghostty.h` with ~100 function declarations and many enums/structs, manual is too error-prone.

**Linux surface embedding (what needs to be built):** The `ghostty_surface_config_s.platform` union needs a new `linux` variant containing a `GtkWidget*` (specifically the `GtkGLArea` that Ghostty will render into). The Zig embedded apprt on Linux will need to:
1. Accept the `GtkGLArea` widget pointer
2. Set up OpenGL context via GDK
3. Connect GTK4 `realize`/`resize`/`render` signals to Ghostty's renderer
4. Forward key/mouse events from GTK4 signal handlers to `ghostty_surface_key`/`ghostty_surface_mouse_pos` etc.

This mirrors what the macOS NSView does: the host app provides the view, Ghostty renders into it via the platform's GPU API.

### Pane/Split Layout

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Custom Rust tree (port of Bonsplit) | n/a | Recursive binary split tree for pane layout | Bonsplit is Swift; reimplement the same `enum SplitNode { Leaf(PaneId), Branch { left, right, direction, ratio } }` data model in Rust. It's ~500 lines of logic. No crate exactly matches Bonsplit's API needs. |
| GTK4 `GtkPaned` | system | Physical divider widget between panes | Use `GtkPaned` (horizontal/vertical) as the GTK4 widget backing each Branch node in the tree. The Rust tree tracks layout state; `GtkPaned` provides the actual draggable divider. |

**Why NOT a crate for split layout:** The split tree itself is simple business logic (enum + update functions). Bringing in a tiling layout crate adds more complexity than it saves. Port Bonsplit's model directly.

### Unix Socket IPC

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tokio | 1.x | Async runtime for socket server | Standard choice for async I/O in Rust; `tokio::net::UnixListener` provides the socket accept loop. Eliminates the per-client thread spawn pattern (a known concern in the macOS codebase). |
| serde_json | 1.x | Parse/serialize v2 JSON-RPC commands | Protocol compatibility with macOS cmux is a hard constraint; JSON-RPC v2 protocol. |
| serde | 1.x | Derive ser/deserialize for command types | Pair with serde_json; `#[derive(Serialize, Deserialize)]` on all command structs |

**tokio vs std threads:** The macOS codebase uses per-connection thread spawning with `nonisolated(unsafe)` state — explicitly flagged as tech debt in CONCERNS.md. tokio's async model avoids this entirely: one thread pool, async accept loop, per-client tasks. Use tokio from day one.

**Socket path:** Mirror macOS — `~/.config/cmux/cmux.sock` (Linux XDG convention).

### Session Persistence

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| serde | 1.x | Derive serialization for session types | Same serde already used for IPC |
| serde_json | 1.x | JSON format for session file | Protocol-compatible with macOS `session.json` format; allows cross-platform tooling |

**Session file location:** `~/.local/share/cmux/session.json` (XDG_DATA_HOME convention on Linux).

### Config File Parsing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| toml | 0.8.x (via `toml` crate) | Parse cmux-specific config (shortcuts, socket path, etc.) | TOML is idiomatic Rust config format; used by Cargo, rust-analyzer, most Rust tooling. Simple, well-specified. |
| serde | 1.x | Deserialize TOML config into typed structs | Same serde used elsewhere; `#[derive(Deserialize)]` on config structs |

**Note on Ghostty config:** Ghostty reads its own config (`~/.config/ghostty/config`) via the C API (`ghostty_config_load_default_files`, `ghostty_config_load_file`). cmux-linux does not re-parse Ghostty's config format. The Rust config layer handles cmux-specific settings only (keyboard shortcuts, socket path, theme overrides passed to the Ghostty config API).

**Config file location:** `~/.config/cmux/config.toml` (XDG_CONFIG_HOME convention).

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tracing` | 0.1.x | Structured logging | Replace `dlog()` equivalent; structured spans for socket commands, surface lifecycle |
| `tracing-subscriber` | 0.3.x | Log output formatting | Debug builds write to file; release builds configurable |
| `anyhow` | 1.x | Error handling | Application-level error propagation; not for library boundaries |
| `thiserror` | 1.x | Define typed error enums | For socket command parsing errors, config errors |
| `uuid` | 1.x | Workspace/pane/surface UUIDs | Protocol compatibility with macOS (uses UUID-based IDs) |
| `dirs` | 5.x | XDG path resolution | `dirs::config_dir()`, `dirs::data_dir()` for cross-distro correctness |

### Build System

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Cargo | (Rust stable) | Rust dependency management and build | Standard |
| `build.rs` | n/a | Compile/link libghostty.so, run bindgen | Standard Cargo build script pattern for native dependencies |
| Zig | 0.14+ | Build libghostty.so from Ghostty source | Same toolchain as macOS; Ghostty's build system is Zig-native |

### Distribution

| Technology | Purpose | Why |
|------------|---------|-----|
| Flatpak | Primary distribution target | Handles GTK4/GLib runtime dependencies; standard for Linux GUI apps |
| `.deb` | Ubuntu/Debian secondary | Broad coverage for common distros |
| AppImage | Portable fallback | No installation required; useful for CI artifacts |

### CI/CD

| Technology | Purpose |
|------------|---------|
| GitHub Actions (ubuntu-latest runners) | Build, test, release |
| `cargo test` | Unit tests |
| `cargo clippy` | Linting |

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| UI framework | gtk4-rs | iced | iced cannot host GTK4 native surfaces; different renderer entirely |
| UI framework | gtk4-rs | egui | Same issue as iced; wgpu-based, no GTK4 widget hosting |
| UI framework | gtk4-rs | slint | No GTK4 widget embedding; experimental GTK backend for Slint's own widgets only |
| UI framework | gtk4-rs | raw GTK4 via bindgen | gtk4-rs provides safe wrappers and type system integration; raw FFI is unsafe and verbose |
| Async runtime | tokio | std threads | Per-thread model is tech debt in macOS codebase; tokio's async model is cleaner |
| Config format | TOML | YAML | TOML is simpler, more Rust-idiomatic; YAML has edge cases around types |
| Config format | TOML | Ghostty config format | Ghostty's format is intentionally not a standard; parsing it in Rust duplicates Ghostty's parser |
| FFI | bindgen | Manual FFI | ghostty.h is ~1,169 lines; manual maintenance is error-prone; bindgen is standard practice |
| Split layout | Custom Rust enum | `tui-layout` or similar | TUI layout crates target text/ncurses grids, not pixel-accurate GTK4 split panes |

---

## The iced vs GTK4 Question — Resolved

**Decision: GTK4 via gtk4-rs. iced is not viable.**

The project context noted this as the key tension. The analysis resolves it definitively:

Ghostty's terminal surfaces on Linux render via OpenGL/Vulkan into a GDK surface (backed by a `GtkGLArea`). The libghostty embedded API (once extended for Linux) will hand Ghostty a GtkWidget to render into. That widget must live inside a GTK4 widget hierarchy with a live GDK display connection.

iced, egui, and slint all run their own GPU rendering pipeline outside of GTK4. They cannot provide a `GtkWidget*` to libghostty because they do not speak GTK4. Bridging them would require:
- Running a GTK4 main loop alongside the iced event loop
- X11/Wayland window embedding hacks to place one renderer's output inside another

This is not a viable architecture. GTK4 is the only choice.

The Rust developer experience with gtk4-rs is acceptable for this project's needs: the crate provides safe, idiomatic wrappers with `#[derive]`-style signal handlers and builder patterns. It is not as ergonomic as SwiftUI, but it is the right tool.

---

## Key Unknowns / Research Flags

1. **libghostty Linux embedded apprt** (HIGH priority): The exact API extension needed — specifically, whether `GHOSTTY_PLATFORM_GTK4` should pass a `GtkGLArea*`, a `GdkSurface*`, or a `GdkGLContext*` — requires reading Ghostty's `src/apprt/gtk` source and `src/apprt/embedded.zig` to understand what the embedded apprt expects vs. what the GTK4 apprt provides. This is the single most critical technical unknown for Phase 1.

2. **gtk4-rs version** (LOW priority): Verify the exact latest stable version at crates.io before pinning in Cargo.toml. Training data suggests 0.9.x but patch versions change frequently.

3. **Wayland + OpenGL surface embedding** (MEDIUM priority): On Wayland, `GtkGLArea` gets a `wl_surface`. Confirm that Ghostty's renderer can target a `GtkGLArea`'s GL context on Wayland, not just X11. Ghostty's GTK4 app already runs on Wayland, so this should work — but confirm the embedded path behaves the same.

4. **GLib main loop + tokio integration** (MEDIUM priority): GTK4 requires the GLib main loop (`gtk4::main()`). tokio has its own runtime. The standard pattern is to run GTK4's main loop on the main thread and use `glib::MainContext::spawn_local` for GTK4-thread tasks, while tokio handles socket I/O on a background thread. Verify this architecture doesn't cause event loop conflicts.

---

## Installation

```toml
# Cargo.toml [dependencies]
gtk4 = "0.9"
gdk4 = "0.9"
glib = "0.20"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
anyhow = "1"
thiserror = "1"
uuid = { version = "1", features = ["v4"] }
tracing = "0.1"
dirs = "5"

# Cargo.toml [dev-dependencies]
tracing-subscriber = "0.3"

# build.rs dependencies
[build-dependencies]
bindgen = "0.70"
cc = "1"
```

## Sources

- `ghostty.h` in this repo (HIGH confidence — authoritative C API source, read directly)
- macOS `GhosttyTerminalView.swift` (HIGH confidence — shows exact libghostty usage patterns to port)
- `docs/ghostty-fork.md` (HIGH confidence — fork patch inventory; all patches are macOS-specific, no Linux embedded apprt work yet)
- `.planning/codebase/CONCERNS.md` (HIGH confidence — per-thread socket model flagged as tech debt, confirms tokio is the right fix)
- gtk-rs project architecture (MEDIUM confidence — from training data; official project at https://gtk-rs.org)
- iced architecture (MEDIUM confidence — from training data; project at https://iced.rs)
- tokio async runtime (HIGH confidence — widely used, stable API)
