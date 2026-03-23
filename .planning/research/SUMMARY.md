# Project Research Summary

**Project:** cmux-linux
**Domain:** Linux GPU-accelerated terminal multiplexer (Rust + GTK4 + Ghostty)
**Researched:** 2026-03-23
**Confidence:** MEDIUM

## Executive Summary

cmux-linux is a ground-up Rust rewrite of a macOS terminal multiplexer, targeting full feature parity: GPU-accelerated terminals via Ghostty, workspace/tab management, recursive pane splits, and a v2 JSON-RPC socket API wire-compatible with the macOS app. The core recommendation is clear: the UI toolkit must be GTK4 via gtk4-rs — iced, egui, and slint are eliminated because Ghostty's terminal surfaces render into a GTK4 GdkSurface (a `GtkGLArea`) and no other Rust GUI framework can host that surface. The socket server should use tokio from day one, avoiding the per-thread model that is already flagged as tech debt in the macOS codebase.

The single highest-risk item in the project is that the `ghostty.h` C API does not define a Linux platform variant. The `ghostty_platform_u` union today has only `macos` (NSView pointer) and `ios` (UIView pointer) members. Before any terminal surface can be embedded in the Rust GTK4 app, the manaflow-ai/ghostty fork must be extended to add a `GHOSTTY_PLATFORM_GTK4` variant accepting a `GtkWidget*` (specifically a `GtkGLArea`). This is a required prerequisite and the first Phase 1 deliverable must be a fork investigation spike. Until this is resolved, no terminal surfaces can be displayed — it blocks all subsequent work.

The second major constraint is thread discipline: all `ghostty_*` C API calls are GTK-main-thread-only, and this must be enforced architecturally from the first line of code. The recommended architecture uses a two-loop model: GTK4's GLib main loop owns the main thread (all AppState, GhosttyBridge, and widget mutations); a tokio runtime on background threads handles socket I/O and session file writes. The two communicate exclusively through a `glib::MainContext::channel`. This pattern avoids the threading tech debt in the macOS codebase and maps cleanly to GTK4's threading model.

## Key Findings

### Recommended Stack

The stack is dominated by the Ghostty embedding constraint. GTK4 via gtk4-rs is mandatory; all other UI framework choices are eliminated. The async story uses tokio for socket I/O, with a `glib::MainContext::channel` bridging tokio tasks to the GTK main thread. Config is TOML via the `toml` crate; session persistence reuses `serde_json` (same format as macOS `session.json` for cross-platform compatibility). The Zig toolchain is required to build libghostty.so from the manaflow-ai fork — same mechanism as macOS GhosttyKit.

**Core technologies:**
- Rust (stable 1.77+): application language — memory safety, strong GTK4 bindings, no GC
- gtk4-rs (`gtk4` + `gdk4` + `glib` crates, 0.9.x): UI framework — mandatory for Ghostty surface hosting; no alternative works
- libghostty (manaflow fork, custom Linux build): terminal engine — hard project constraint; requires Zig to build
- bindgen (0.70.x): FFI generation from `ghostty.h` — automated; handles the ~1,169-line header reliably
- tokio (1.x): async runtime for socket server — avoids the per-thread tech debt from macOS codebase
- serde + serde_json (1.x): IPC and session serialization — wire-compatible with macOS session format
- toml (0.8.x): cmux config file — idiomatic Rust; Ghostty's own config loaded via C API separately
- Custom Rust split tree (Bonsplit port): ~500 lines of pure Rust logic; no crate matches Bonsplit's API needs; backed by GTK4 `GtkPaned` widgets for dividers

**Distribution:** Flatpak (primary), .deb (secondary), AppImage (CI artifacts). Note: Flatpak + PTY/shell sandboxing conflicts are a P3 research item.

### Expected Features

The product occupies a third category beyond TUI multiplexers (tmux/zellij) and plain GPU terminal emulators (wezterm/kitty): a GPU-rendered terminal with workspace management AND a rich programmatic socket API. The socket API is the key differentiator — no competitor has a wire-compatible cross-platform JSON-RPC API with workspace/pane/surface addressing.

**Must have (table stakes — v1 launch):**
- Single GTK4 window with Ghostty terminal surface (proves embedding works; everything else depends on this)
- Tab/workspace creation, naming, switching
- Horizontal and vertical pane splits with drag dividers (Bonsplit tree port)
- Unix socket server (v2 JSON-RPC, wire-compatible with macOS cmux)
- Session persistence (save/restore layout on relaunch; key differentiator vs kitty/wezterm)
- XDG Base Directory compliance (`~/.config/cmux/`, `~/.local/share/cmux/`, `$XDG_RUNTIME_DIR/cmux.sock`) — must be correct from day one; retrofitting breaks installs
- Keyboard shortcut configuration (config file)
- Clipboard integration (X11 + Wayland — GTK4 API handles most of this)
- .desktop file + AppImage packaging

**Should have (competitive — v1.x):**
- Terminal notification/attention state with visual indicator
- SSH remote workspaces (cmuxd-remote Go daemon is platform-agnostic; potentially reusable as-is)
- HiDPI / fractional scaling (GTK4 handles most; Ghostty content scale must be driven from `gtk4::Widget::scale_factor()`, not hardcoded)
- Configurable numeric workspace shortcuts (already shipped macOS; straightforward port)
- Desktop notifications via GTK4 API
- .deb / .rpm packages

**Defer (v2+):**
- IME / input method support (IBus/Fcitx) — high complexity; defer until GTK4 embedding is stable
- Browser panel (WebKit/GTK4) — explicitly deferred in PROJECT.md; scope risk
- Flatpak distribution — sandboxing vs PTY conflicts need dedicated investigation
- Systemd user service — packaging extra, not core

### Architecture Approach

The architecture is a clean two-layer model: the GTK main thread owns all UI and Ghostty state; a tokio thread pool handles all async I/O. The bridge between them is a `glib::MainContext::channel` mpsc sender. GTK main loop calls `app.run()` on the main thread; tokio runtime is created with `tokio::runtime::Builder::new_multi_thread()` before `app.run()` and its tasks send `AppCommand` enum variants through the channel. All `ghostty_*` C API calls are confined to the GTK main thread via a `!Send` surface wrapper type that provides compiler enforcement.

**Major components:**
1. **GhosttyBridge** — owns `ghostty_app_t`; creates/frees `ghostty_surface_t` handles; routes action callbacks back to GTK main thread via `glib::idle_add`; all unsafe FFI isolated here; surfaces referenced externally by `SurfaceId(u64)` only
2. **GhosttyWidget** — custom `GtkGLArea` subclass per surface; handles `realize`/`render`/`resize` signals, key/mouse/focus event forwarding to GhosttyBridge
3. **WorkspaceManager** — ordered workspace list; create/close/switch/rename; owns the SplitEngine tree per workspace; GTK main thread only; `Rc<RefCell<>>` not `Arc<Mutex<>>`
4. **SplitEngine** — recursive immutable `SplitNode` tree (`Leaf { surface_id }` / `Branch { axis, ratio, left, right }`); pure Rust, no GTK deps; unit-testable in isolation
5. **SocketServer** — tokio `UnixListener`; v1 line protocol + v2 JSON-RPC; socket command policy tags each handler as focus-intent or not; sends `AppCommand` via channel; awaits `oneshot` for synchronous RPC responses
6. **SessionPersistence** — tokio async file I/O; atomic write (write to `.tmp`, then `rename()`); debounced save on state changes
7. **ConfigLoader** — loads `~/.config/cmux/config.toml` (cmux settings) + delegates Ghostty config to C API (`ghostty_config_load_default_files` + overrides); provides keybindings to AppShell
8. **NotificationStore** — per-surface attention/unread state; fed by Ghostty `RING_BELL` / OSC 99 action callbacks via GhosttyBridge

### Critical Pitfalls

1. **Linux platform missing from libghostty C API** — `ghostty.h` has no `ghostty_platform_linux_s`; `ghostty_surface_new()` cannot be called from a Rust/GTK host today. Phase 1 must be a fork investigation spike before any other work. If not resolved, all subsequent phases are blocked.

2. **Ghostty C API called off the GTK main thread** — all `ghostty_*` functions are not thread-safe. Socket command handlers must parse off-thread then send mutations as `AppCommand` messages through the `glib::MainContext::channel`. Make `ghostty_surface_t` wrapper `!Send` at project start for compiler enforcement. Retrofitting this threading model is expensive.

3. **Wakeup callback called on wrong thread** — `wakeup_cb` fires from Ghostty's internal thread; it must call `glib::idle_add_once(|| ghostty_app_tick(app))`, never call `ghostty_*` functions inline. Add a coalescing pending flag (GLib `idle_add` does not coalesce like macOS `DispatchQueue`).

4. **Socket focus steal** — non-focus-intent socket commands (e.g., `new_split`, `send`, `workspace.list`) must never call `gtk_window_present()` or `ghostty_surface_set_focus()`. Port the macOS `socketCommandPolicy` tagging pattern from day one. The macOS audit found 70+ commands that needed this treatment; build the policy in, not onto.

5. **Session persistence corruption on partial write** — never use `std::fs::write()` for the session file; always write atomically (write `.tmp`, then `rename()`). Keep a `.bak` backup and fall back on JSON parse failure. The `rename()` syscall is atomic within the same filesystem on Linux.

6. **GTK4 widget / Ghostty surface lifetime mismatch** — two independent lifetime systems (GObject ref counting vs. manual C memory) must be kept in sync. Wrap `ghostty_surface_t` in a Rust struct with a `Drop` impl; never store raw surface pointers in GLib closures. Free order: `ghostty_surface_request_close()` → wait for `close_surface_cb` → then drop the owning struct.

7. **gtk4-rs reference cycles causing memory leaks** — default to `@weak` captures in all signal closures. RSS grows monotonically with workspace create/close if strong captures create cycles. Verify with Valgrind massif after 50 workspace cycles before any release.

## Implications for Roadmap

Based on the dependency graph from ARCHITECTURE.md and the pitfall-to-phase mapping from PITFALLS.md, the following phase structure is recommended. The build order is hard-constrained: Ghostty embedding must work before any multi-pane work, and multi-pane must work before socket and persistence have meaningful state to operate on.

### Phase 1: Ghostty Fork + Single-Surface Foundation

**Rationale:** The entire project is blocked until the libghostty C API is extended for Linux. This is the highest-risk unknown and must be resolved before any other work begins. Simultaneously, all foundational architectural patterns must be established: threading model, wakeup callback, XDG paths, content scale, input event routing. These cannot be retrofitted cheaply.

**Delivers:** A running GTK4 window with a single working Ghostty terminal surface. Keyboard input works. Wakeup/render cycle works. App scaffolding (tokio + GLib channel) is in place.

**Addresses:** Ghostty surface in GTK4 window (P1); XDG paths (P1); keyboard shortcut config skeleton; clipboard integration (P1)

**Avoids/establishes:**
- `!Send` surface wrapper type for GTK main thread enforcement (Pitfall 1)
- `glib::idle_add_once` wakeup callback pattern (Pitfall 2)
- XDG path compliance via `dirs` crate — never `~/Library`, never hardcoded paths (Pitfall 12)
- Content scale driven from `gtk4::Widget::scale_factor()` — not hardcoded 2.0 (Pitfall 12)
- Keystroke-to-render latency measured at baseline < 10ms (Pitfall 4)

**Research flag:** NEEDS RESEARCH — Ghostty fork investigation is the Phase 1 spike. Must read `src/apprt/gtk` and `src/apprt/embedded.zig` in the Ghostty source to determine exact API extension required (`GtkGLArea*`, `GdkGLContext*`, or `EGLSurface`). Wayland + OpenGL surface embedding on the embedded path also needs validation.

### Phase 2: Workspace Management + Multi-Pane Splitting

**Rationale:** WorkspaceManager and SplitEngine depend on a working GhosttyBridge (Phase 1). Multi-pane splitting is the most user-visible feature after "single terminal works." The Bonsplit Rust port is pure logic (~500 lines) that can be built and unit-tested without GTK; GTK widget layout follows after.

**Delivers:** Multiple workspaces (create/close/switch/rename), horizontal/vertical pane splits, drag-to-resize dividers, keyboard navigation between panes.

**Addresses:** Tab/workspace management (P1); pane splits (P1); resizable dividers (P1); keyboard shortcut navigation (P1)

**Avoids/establishes:**
- Focus routing: explicit "focused surface" reference in AppState; `ghostty_surface_set_focus(old, false)` before `ghostty_surface_set_focus(new, true)` (Pitfall 3)
- GTK reference cycle leak test: RSS stable after 50 workspace create/close cycles (Pitfall 6)
- AddressSanitizer clean after 100 pane open/close cycles (Pitfall 5)
- Immutable SplitNode tree updates for layout operations (Architecture Pattern 4)

**Research flag:** Standard patterns — GTK4 `GtkPaned` + custom split tree is well-understood. No research phase needed.

### Phase 3: Socket Server (v2 JSON-RPC)

**Rationale:** Socket server depends on WorkspaceManager existing (Phase 2). The protocol must be wire-compatible with macOS cmux from the start — drift discovered late is HIGH recovery cost. Building the socket command policy (focus-intent tagging) into the server before the handler list grows is far cheaper than auditing 70+ handlers later (as the macOS team had to do).

**Delivers:** Unix socket server with v2 JSON-RPC protocol; `cmux` CLI works against Linux socket; `tests_v2/` Python suite from macOS passes without modification against Linux server.

**Addresses:** Unix socket server (P1); CLI (P1); socket focus steal prevention (security/correctness)

**Avoids/establishes:**
- Socket command policy: every handler tagged focus-intent or not at authoring time (Pitfall 11)
- `SO_PEERCRED` uid validation on every accept; socket at `$XDG_RUNTIME_DIR/cmux/cmux.sock` with 0600 permissions (Security)
- Protocol compatibility gate: `tests_v2/` suite must pass before phase is complete (Pitfall 9)
- Tokio async handler model — no per-thread spawning (Stack rationale)

**Research flag:** Standard patterns for the tokio socket server itself. The v2 protocol schema derivation (reading macOS handlers + `tests_v2/` assertions) needs careful cross-referencing — budget time for protocol documentation before implementation.

### Phase 4: Session Persistence

**Rationale:** Persistence depends on the workspace/split tree existing (Phase 2) and is meaningfully testable once socket commands can manipulate workspaces (Phase 3). Atomic write pattern must be the success criterion from the start, not a patch added after a data-loss report.

**Delivers:** Workspace + pane layout saved to `~/.local/share/cmux/session.json` on change (debounced); restored on next launch; survives crash mid-save.

**Addresses:** Session persistence (P1)

**Avoids/establishes:**
- Atomic write: write to `.tmp`, then `rename()` — no `std::fs::write()` on session file (Pitfall 8)
- Backup file (`.session.json.bak`) with fallback on parse failure (Pitfall 8)
- `kill -9` during save leaves valid or backup session (test requirement)

**Research flag:** Standard patterns — `serde_json` + atomic file write is well-documented. No research phase needed.

### Phase 5: Notification State + Polish

**Rationale:** Notification state (terminal activity, bell, OSC 99) is a feature that enhances a stable core but is not a dependency for any other system. After core workspace + socket + persistence is solid, notification state and HiDPI polish complete the v1 feature set.

**Delivers:** Per-pane attention/notification state with visual indicator in workspace list; HiDPI/fractional scaling verified; desktop notifications via GTK4 API; `.desktop` file.

**Addresses:** Terminal notification/attention state (P2); HiDPI (P2); desktop notifications (P2); desktop entry (P1)

**Avoids/establishes:**
- `glib::idle_add` delivery of notification events from GhosttyBridge to NotificationStore (thread model consistency)
- Content scale tested on 1.0-scale display; not just Retina/HiDPI (Pitfall 12)

**Research flag:** Standard patterns. GTK4 notification API and `GdkMonitor` HiDPI signals are well-documented.

### Phase 6: CI/CD + Distribution Packaging

**Rationale:** AppImage is the minimum required for Linux users to install and try the app; it also validates that all runtime dependencies are correctly bundled. GitHub Actions on ubuntu-latest runners is the CI target. This phase completes the v1 launch checklist.

**Delivers:** GitHub Actions CI pipeline (build, clippy, unit tests); AppImage artifact on each release; .desktop file integrated; Wayland + X11 testing matrix in CI.

**Addresses:** CI/CD (Active requirement); AppImage packaging (P1); .deb (P2)

**Research flag:** NEEDS RESEARCH for Flatpak specifically — PTY/shell sandboxing conflicts need dedicated investigation before attempting Flatpak. AppImage and .deb are standard patterns.

### Phase Ordering Rationale

- **Phase 1 must come first** because the libghostty Linux embedding API does not exist yet; everything depends on it.
- **Phase 2 before Phase 3** because the socket server needs WorkspaceManager state to be meaningful; protocol testing against empty state is not useful.
- **Phase 3 before Phase 4** because socket commands are the primary way to create the complex workspace state that makes persistence testing meaningful.
- **Phase 5 after Phase 3** because notification state is fed by Ghostty action callbacks that can only be validated once multi-pane + socket are stable.
- **Phase 6 last** because distribution requires a feature-complete binary to package.

### Research Flags

Phases needing deeper research during planning:
- **Phase 1:** Ghostty fork investigation is the critical spike — read `src/apprt/gtk` and `src/apprt/embedded.zig` to determine exact Linux platform struct required for `ghostty_surface_new()`. Also validate GLib + tokio event loop integration pattern against current gtk4-rs docs.
- **Phase 6 (Flatpak):** PTY/subprocess access in Flatpak sandbox needs portal API investigation before committing to Flatpak as a distribution target.

Phases with standard patterns (skip research-phase):
- **Phase 2:** GTK4 `GtkPaned` + custom split tree — well-documented.
- **Phase 3:** tokio `UnixListener` + JSON-RPC — well-documented. Protocol schema derivation is effort, not research.
- **Phase 4:** `serde_json` + atomic file write — well-documented.
- **Phase 5:** GTK4 notification API + HiDPI signals — well-documented.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM | GTK4 is definitively correct (HIGH confidence from ghostty.h analysis); gtk4-rs exact version needs live crates.io verification; GLib+tokio integration pattern needs verification against current gtk4-rs docs |
| Features | MEDIUM | macOS feature set is HIGH confidence (first-party codebase); competitor feature sets (zellij, wezterm) are training-data knowledge capped at ~Aug 2025; XDG spec is stable |
| Architecture | MEDIUM | ghostty.h C API contract is HIGH confidence (first-party); GTK4 custom widget subclass patterns are MEDIUM (training data, not verified against current gtk4-rs book); Ghostty renderer threading on Linux is UNKNOWN until fork investigation |
| Pitfalls | MEDIUM-HIGH | Pitfalls derived from macOS codebase issues (HIGH), ghostty.h structure analysis (HIGH), and GTK4/Rust training knowledge (MEDIUM); Wayland-specific pitfalls need runtime verification |

**Overall confidence:** MEDIUM

### Gaps to Address

- **Ghostty Linux embedded apprt API** (critical): The exact struct/pointer type needed for `GHOSTTY_PLATFORM_GTK4` — whether `GtkGLArea*`, `GdkGLContext*`, or `EGLSurface` — must be determined by reading Ghostty's `src/apprt/embedded.zig` and `src/apprt/gtk` source. This is Phase 1's primary deliverable and the highest-risk unknown in the project. Budget a 1-2 week spike before estimating Phase 1 duration.

- **Alternative embedding path** (medium): Architecture research flagged a potential alternative — use Ghostty's GTK4 app runtime (`apprt/gtk`) as a library with the Rust app acting as a plugin. LOW confidence; needs investigation alongside the C API extension approach to pick the right path.

- **GLib + tokio integration** (low-medium): The `glib::MainContext::channel` pattern as the bridge between tokio and GTK is the recommended approach, but needs validation against current gtk4-rs 0.9.x docs. The `gtk4-rs` book may have updated guidance.

- **Wayland OpenGL surface embedding** (medium): Ghostty already runs on Wayland as a standalone app; the embedded path (accepting an external `GtkGLArea`) needs verification that the same GL context setup works in an embedded scenario, not just self-hosted.

- **IME preedit on Linux** (medium): `ghostty_surface_preedit()` must be wired from GTK4's `InputMethod` API. The current state of IME support in Ghostty's Linux GTK4 build needs verification before committing to IME in any phase.

## Sources

### Primary (HIGH confidence)
- `ghostty.h` in this repo — definitive C API contract; platform union, callback structs, surface lifecycle
- `.planning/codebase/ARCHITECTURE.md` — macOS embedding patterns to mirror
- `.planning/codebase/CONCERNS.md` — macOS tech debt; per-thread socket model flagged for replacement
- `docs/ghostty-fork.md` — fork scope; all current patches are macOS-specific; no Linux embedded apprt work exists
- `docs/socket-focus-steal-audit.todo.md` — complete focus-intent command allowlist; 70+ commands audited
- `docs/v2-api-migration.md` — v2 JSON-RPC wire protocol specification
- `Sources/GhosttyTerminalView.swift` — macOS wakeup_cb, forceRefresh, latency-sensitive path documentation
- `Sources/TerminalController.swift` — macOS socket server patterns to port
- `CLAUDE.md` — typing-latency-sensitive paths, socket threading policy, socket focus policy

### Secondary (MEDIUM confidence)
- gtk-rs project (https://gtk-rs.org) — GTK4 Rust binding patterns; training data, not verified during session
- tokio documentation — async runtime patterns; widely stable API
- XDG Base Directory Specification — config/data/runtime path conventions
- tmux 3.x, zellij 0.39.x, wezterm, kitty feature sets — training knowledge for competitive analysis

### Tertiary (LOW confidence)
- Ghostty `src/apprt/gtk` and `src/apprt/embedded.zig` — not read; submodule not initialized; critical for Phase 1 spike
- Current gtk4-rs 0.9.x book and changelog — not verified; may have updated guidance on GLib+tokio integration
- Flatpak PTY portal API — not researched; needs investigation before Phase 6 Flatpak work

---
*Research completed: 2026-03-23*
*Ready for roadmap: yes*
