# Phase 1: Ghostty Foundation - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Embed a single Ghostty terminal surface into a GTK4 window with correct input routing, GPU rendering, clipboard integration, threading, DPI scaling, and XDG path compliance. No workspace management, no pane splits, no socket server, no UI chrome — just the foundational embedding layer that all other phases build on.

</domain>

<decisions>
## Implementation Decisions

### Repo Layout
- **D-01:** Root-level `Cargo.toml` at repo root, with `src/main.rs` as the Rust entry point. The existing macOS Swift files (`Sources/`, etc.) remain at the top level alongside the Rust project. `cargo build` works from the repo root.

### Phase 1 Window Scope
- **D-02:** A bare `GtkApplicationWindow` containing a single `GtkGLArea` hosting the Ghostty surface. No sidebar, no tab bar, no header bar chrome — just a working terminal window. Phase 2 adds the multiplexer UI structure on top of this foundation.

### Ghostty Fork Extension Strategy
- **D-03:** Add `GHOSTTY_PLATFORM_GTK4` to `ghostty.h` as a **thin embedded variant**: extend the existing `embedded.zig` platform struct mechanism with a new `ghostty_platform_gtk4_s` struct that holds only `GtkWidget *gl_area` (as `void*`, to avoid GTK4 headers in ghostty.h — consistent with the macOS `void* nsview` pattern). `GdkGLContext` is NOT passed in this struct; GTK4 manages it internally when the GtkGLArea is realized, and the Rust layer calls `gl_area.make_current()` in the realize signal handler. The `wakeup_cb` and `userdata` callback pair are registered globally via `ghostty_runtime_config_s`, not per-surface via `ghostty_platform_gtk4_s`. Minimal fork diff; reuses existing `apprt/embedded.zig` infrastructure. (Note: initial sketch of D-03 mentioned GdkGLContext and per-platform wakeup fields — revised after research confirmed the simpler approach is correct and consistent with how the embedded API works.)
- **D-04:** `wakeup_cb` dispatches to GLib main loop via `glib::idle_add_once()` — never calls any `ghostty_*` API inline from Ghostty's thread (per GHOST-07).

### libghostty Linkage
- **D-05:** `scripts/setup.sh` pre-builds or downloads `libghostty.a` (static archive) and places it at a known location (e.g. `ghostty/zig-out/lib/libghostty.a`). `build.rs` then links it statically via `cargo:rustc-link-lib=static=ghostty`. Zig is NOT a Cargo build-time dependency — developers run `./scripts/setup.sh` once before `cargo build`. This mirrors the existing macOS `scripts/setup.sh` workflow.

### Bindgen
- **D-06:** `bindgen` runs at build time in `build.rs` against `ghostty.h` to auto-generate `src/ghostty_sys.rs`. `libclang` must be available in CI. Bindings regenerate automatically when `ghostty.h` changes as the fork evolves.

### Tokio / GLib Threading
- **D-07:** **Defer tokio to Phase 3.** Phase 1 has no socket server and no inter-thread async work. GLib main loop handles all async via `glib::spawn_future_local`. A `tokio` runtime is introduced in Phase 3 alongside the socket server, then bridged to GTK via `glib::MainContext::channel`. This avoids adding a tokio/GLib bridge that can't be tested in Phase 1.

### Clipboard
- **D-08:** Clipboard callbacks from Ghostty (GHOST-05) bridge to GTK4 via `gdk4::Clipboard::read_text_future()` / `set_text()`, invoked from `glib::spawn_future_local`. GDK handles X11/Wayland divergence transparently — no manual protocol branching needed in Phase 1.

### Error Handling
- **D-09:** If Ghostty surface initialization fails (GL context failure, PTY spawn error, bad config): print a clear diagnostic to stderr and exit with code 1. No GUI error dialog in Phase 1. Phase 1 targets developers, not end-users. `close_surface_cb` also uses `std::process::exit(0)` — the `gtk4::Application::default()` API does not exist in gtk4-rs 0.11 in a form suitable for quitting the app from a C callback context.

### Socket API Architectural Constraint (Non-Functional)
- **D-10:** **The Phase 1 architecture must not block Phase 3 socket integration.** The socket API for agent automation is a core project value prop (alongside the in-terminal browser). Phase 1 code organization should make it straightforward to add a tokio-based socket server in Phase 3 — avoid tightly coupling app state to GLib in a way that makes off-main-thread command dispatch impossible.

### Claude's Discretion
- Exact Zig build target flags for producing `libghostty.a` on Linux (researcher to confirm)
- DPI/scale factor update mechanism when window moves between monitors — researcher verifies GTK4 signal name (`notify::scale-factor` on `GtkWidget`) and whether Ghostty requires an explicit `ghostty_surface_set_display_scale()` call
- Whether GTK4 requires a custom GDK surface backend or `GtkGLArea` is sufficient for Ghostty's OpenGL renderer

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Ghostty Embedding API
- `ghostty.h` — Full C API for ghostty_app_t, ghostty_surface_t, input, clipboard, and platform structs. The Linux platform extension adds a new entry to `ghostty_platform_e` and `ghostty_platform_u`.
- `ghostty/src/apprt/embedded.zig` — **Critical spike target.** Defines the embedded platform implementation that Phase 1 must extend. Researcher must read to determine: how platform structs are consumed, what init sequence GTK4 needs, and whether any new Zig files are required.

### Requirements
- `.planning/REQUIREMENTS.md` §Ghostty Embedding — GHOST-01 through GHOST-07 are the acceptance criteria for this phase. Every requirement must map to a plan task.

### Project Constraints
- `.planning/PROJECT.md` §Constraints — Rust primary, GTK4 mandatory, libghostty C API integration point, tokio for socket I/O (Phase 3+).
- `.planning/STATE.md` §Decisions — Lists locked decisions (Rust + GTK4, glib::MainContext::channel bridge, GHOST-01 spike status).

### macOS Reference Implementation
- `Sources/GhosttyTerminalView.swift` — macOS reference for Ghostty surface lifecycle (init, wakeup callback, input feed, surface close). The Rust/GTK4 port mirrors this data flow.
- `Sources/AppDelegate.swift` — macOS keyboard event routing reference (performKeyEquivalent → ghostty_surface_feed_input). Phase 1 Rust code implements equivalent GTK4 key-press-event routing.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ghostty.h` (repo root): The full Ghostty C embedding API — 1168 lines covering app/surface lifecycle, input, clipboard, config, and action dispatch. Phase 1 uses a subset: `ghostty_app_new`, `ghostty_surface_new`, `ghostty_surface_feed_input`, `ghostty_surface_mouse_*`, `ghostty_surface_set_focus`, `ghostty_surface_close`.
- `scripts/setup.sh`: Existing macOS setup script pattern — Phase 1 extends (or creates a parallel `scripts/setup-linux.sh`) to pre-build `libghostty.a` via `zig build`.
- `tests_v2/`: Python v2 JSON-RPC test suite — platform-agnostic, will run unmodified against Phase 3 Linux socket server. No changes needed here in Phase 1, but architecture must not preclude it.
- `daemon/remote/` (Go): SSH remote daemon — platform-agnostic, potentially reusable in Phase 4 without changes.

### Established Patterns
- **Ghostty surface lifecycle (macOS reference):** `ghostty_app_new` → `ghostty_surface_new` (with platform struct) → render loop driven by `wakeup_cb` → `ghostty_surface_close` on teardown.
- **Input routing (macOS reference):** Key events → `ghostty_surface_feed_input()`; mouse events → `ghostty_surface_mouse_*()`. Phase 1 Rust code routes GTK4 `key-pressed` / `button-press-event` signals to these functions.
- **Socket protocol:** v2 JSON-RPC wire format defined in `tests_v2/` — Phase 1 must not break this format expectation for Phase 3.

### Integration Points
- `ghostty/` submodule: Fork target for GTK4 platform extension (GHOST-01). The Zig build system produces `libghostty.a`.
- `ghostty.h`: bindgen input — generates `src/ghostty_sys.rs` in the Rust crate.
- `scripts/setup.sh`: Extended or parallel script for Linux pre-build of `libghostty.a`.

</code_context>

<specifics>
## Specific Ideas

- **Socket API is a core value prop**: The user emphasized that agent automation via socket API is a primary reason for the Linux port (alongside the in-terminal browser). Phase 1 architecture decisions (especially state ownership and async model) must not accidentally foreclose Phase 3 socket integration. The researcher should flag any design that would make `DispatchQueue.main.async`-equivalent patterns difficult in GTK4/Rust.
- **In-terminal browser**: Mentioned as a key future capability (v2 BROW-01..03). Currently Out of Scope for v1. The Phase 1 GTK4 pane abstraction should not hardcode "only terminals" — but browser panel implementation is deferred.
- **Thin fork diff**: The Ghostty fork extension should be as minimal as possible to make upstream rebasing tractable. Prefer extending `embedded.zig` over adding a new `apprt/gtk4.zig`.
- **Pre-built libghostty.a via setup.sh**: Mirrors the existing macOS developer experience (`scripts/setup.sh` already handles GhosttyKit). Linux equivalent should follow the same pattern so the workflow is familiar.

</specifics>

<deferred>
## Deferred Ideas

- **In-terminal browser (BROW-01..03)**: User confirmed this is a key value prop. Currently v2 scope; GTK4/Rust WebKit embedding (webkit2gtk-rs) is non-trivial. Phase 1 pane abstraction should not hardcode "terminal-only" but browser panel implementation is deferred to v2.
- **Socket API (SOCK-01..06)**: Core project value prop, explicitly Phase 3 scope. Phase 1 must not architecturally block it but does not implement it.
- **Tokio/GLib bridge validation**: Deferred to Phase 3 when tokio is actually needed for the socket server. Phase 1 uses GLib async only.
- **GTK4 error dialogs**: Phase 1 uses stderr + exit code. User-facing error dialogs are a future phase concern.

</deferred>

---

*Phase: 01-ghostty-foundation*
*Context gathered: 2026-03-23*
