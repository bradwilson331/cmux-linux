---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: Ready to execute
stopped_at: Completed 04-03-PLAN.md
last_updated: "2026-03-26T13:02:21.940Z"
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 31
  completed_plans: 28
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-23)

**Core value:** A Linux user should get the same cmux experience as a Mac user: tabs, splits, workspaces, and socket CLI control — powered by Ghostty's GPU-accelerated terminal.
**Current focus:** Phase 04 — notifications-hidpi-ssh

## Current Position

Phase: 04 (notifications-hidpi-ssh) — EXECUTING
Plan: 4 of 5

## Performance Metrics

**Velocity:**

- Total plans completed: 0
- Average duration: -
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**

- Last 5 plans: none yet
- Trend: -

*Updated after each plan completion*
| Phase 01-ghostty-foundation P02 | 8 | 2 tasks | 4 files |
| Phase 01-ghostty-foundation P01 | 3 | 2 tasks | 6 files |
| Phase 01-ghostty-foundation P03 | 76 | 2 tasks | 8 files |
| Phase 01-ghostty-foundation P04 | 640 | 3 tasks | 6 files |
| Phase 01-ghostty-foundation P05 | 207 | 3 tasks | 4 files |
| Phase 01-ghostty-foundation P06 | 152 | 3 tasks | 3 files |
| Phase 01-ghostty-foundation P07 | 45 | 2 tasks | 6 files |
| Phase 01-ghostty-foundation P08 | 15 | 2 tasks | 2 files |
| Phase 01-ghostty-foundation P09 | 15 | 2 tasks | 3 files |
| Phase 02-workspaces-pane-splits P00 | 15m | 3 tasks | 4 files |
| Phase 02-workspaces-pane-splits P03 | 0.33 | 1 tasks | 1 files |
| Phase 02-workspaces-pane-splits P04 | 15 | 2 tasks | 2 files |
| Phase 02-workspaces-pane-splits P07 | 2 | 3 tasks | 3 files |
| Phase 02-workspaces-pane-splits PP08 | 10 | 4 tasks | 3 files |
| Phase 03-socket-api-session-persistence P00 | 116 | 2 tasks | 6 files |
| Phase 03-socket-api-session-persistence P01 | 18 | 2 tasks | 5 files |
| Phase 03 P02 | 3 | 2 tasks | 4 files |
| Phase 03 P03 | 4 | 2 tasks | 4 files |
| Phase 03 P04 | 5 | 2 tasks | 2 files |
| Phase 03 P05 | 4 | 2 tasks | 3 files |
| Phase 03 P07 | 2 | 1 tasks | 2 files |
| Phase 04 P01 | 7 | 2 tasks | 6 files |
| Phase 04 P02 | 218 | 1 tasks | 3 files |
| Phase 04 P03 | 1 | 1 tasks | 4 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Init]: Rust + GTK4 (gtk4-rs) is mandatory — iced/egui/slint eliminated because Ghostty surfaces require GtkGLArea
- [Init]: tokio for socket I/O; glib::MainContext::channel bridges tokio to GTK main thread
- [Init]: ghostty.h has no Linux platform variant — Phase 1 must extend manaflow-ai/ghostty fork with GHOSTTY_PLATFORM_GTK4 before any surface embedding
- [Phase 01-ghostty-foundation]: void* gl_area pattern in ghostty_platform_gtk4_s avoids GTK4 headers in public C ABI — matches nsview/uiview convention
- [Phase 01-ghostty-foundation]: GTK4 Zig type is conditional (void on non-Linux) so fork compiles cross-platform
- [Phase 01-ghostty-foundation]: gtk4 crate pinned to 0.10 (not 0.11) for rustc 1.91.1 compatibility; APIs identical for Phase 1 usage
- [Phase 01-ghostty-foundation]: setup-linux.sh installs libgtk-4-dev + libclang-dev system deps (apt/dnf/pacman) before building libghostty.a
- [Phase 01-ghostty-foundation]: Use RefCell for GL_AREA_FOR_RENDER since gtk4::GLArea is not Copy
- [Phase 01-ghostty-foundation]: Remove lib.rs and apply build.rs directly to binary crate
- [Phase 01-ghostty-foundation]: Use stack-allocated text buffer to avoid heap allocations in typing hot path
- [Phase 01-ghostty-foundation]: Map X11 hardware keycodes directly to ghostty_input_key_e for layout-independent input
- [Phase 01-ghostty-foundation]: Defer ghostty_surface_new to GLArea realize callback — GL context must be current before surface creation
- [Phase 01-ghostty-foundation]: must_draw_from_app_thread=true in embedded.zig routes renderer draws through action_cb on main thread — avoids GLAD threadlocal crash
- [Phase 01-ghostty-foundation]: Replace gladLoaderLoadGLContext stub with real GLAD loader from vendor/glad/src/gl.c — stub returned version 0 causing OpenGL version check failure
- [Phase 01-ghostty-foundation]: ApplicationFlags::NON_UNIQUE required for GTK4 app in cross-namespace DBus sessions (NX/containers) — prevents deadlock in GApplication singleton registration
- [Phase 01-ghostty-foundation]: Remove lib.rs: binary-only crate required for build.rs rustc-link-lib=static=ghostty to apply to binary link step
- [Phase 01-ghostty-foundation]: SURFACE_PTR global for clipboard: read_clipboard_cb has no surface arg, requires AtomicUsize global set at realize time
- [Phase 01-ghostty-foundation]: set_focusable(true) on GTK4 GLArea is required for EventControllerKey keyboard events
- [Phase 02-workspaces-pane-splits]: Commented out non-compiling AlertDialog code, preserving the intended 'proceed on close' behavior. This is a temporary fix until the correct API usage is determined.
- [Phase 02-workspaces-pane-splits]: Use notify::position on GtkPaned to detect drag end and restore active pane focus
- [Phase 02-workspaces-pane-splits]: Change Ctrl+Alt+Arrow to Ctrl+Shift+Arrow to avoid Linux compositor interception (GNOME/KDE)
- [Phase 02-workspaces-pane-splits]: EventControllerFocus on GLArea keeps Ghostty focused state in sync with GTK focus routing after any widget-tree operation
- [Phase 02-workspaces-pane-splits]: focus_active_surface() replaces grab_active_focus() for Ctrl+B to also sync Ghostty internal focus state
- [Phase 03-socket-api-session-persistence]: cargo test --bin cmux-linux used for binary crate unit tests; cargo test --lib not applicable
- [Phase 03-socket-api-session-persistence]: glib::MainContext::channel removed in glib 0.18+; use tokio::sync::mpsc::unbounded_channel + glib::MainContext::default().spawn_local() as equivalent bridge pattern
- [Phase 03-socket-api-session-persistence]: Mutex<Option<UnboundedReceiver<...>>> wraps cmd_rx so it can be moved into a Fn connect_activate closure (receiver is not Clone)
- [Phase 03]: Used tokio::sync::mpsc::UnboundedSender instead of glib::MainContext::channel (removed in glib 0.18+) -- pass cmd_tx into start_socket_server from main.rs
- [Phase 03]: Used ghostty_surface_text FFI (not ghostty_surface_input_text) for debug.type text injection
- [Phase 03]: workspace.rename saves/restores active_index to avoid SOCK-05 focus side effect
- [Phase 03]: Used ghostty_surface_text for send_text/send_key — matches existing debug.type pattern
- [Phase 03]: surface.close adapts close_active() by setting target as active first — no direct close-by-uuid API
- [Phase 03]: surface.refresh uses GTK4 queue_render() on GLArea instead of direct ghostty_surface_draw
- [Phase 03]: Snapshot SessionData on GTK main thread in trigger_session_save() and send via mpsc to tokio debounce task -- avoids Rc Send problem
- [Phase 03]: Phase 3 restores workspace names only; full split layout restore deferred to Phase 4
- [Phase 03]: Thin bash wrapper (6 lines) exec-ing cmux.py for CLI entry point -- no duplication of CLI logic per D-04
- [Phase 04]: Bell processing via glib::timeout_add_local(100ms) polling BELL_PENDING atomic -- action_cb fires on main thread but AppState is Rc<RefCell> not accessible from wakeup_cb
- [Phase 04]: Nested sidebar row layout (GtkBox(H) > GtkBox(V) > Label + dot) established in Plan 01 to avoid double-refactor in Plan 04
- [Phase 04]: Enabled gdk4 v4_12 feature for GdkSurface::scale() fractional scaling on Wayland
- [Phase 04]: notification.list returns workspace-level attention matching macOS socket API

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 1 spike]: GHOST-01 is a fork investigation spike with MEDIUM confidence — exact API surface (GtkGLArea* vs GdkGLContext* vs EGLSurface) unknown until ghostty/src/apprt/embedded.zig is read. This blocks all terminal rendering.
- [Phase 1]: GLib + tokio integration pattern needs validation against current gtk4-rs 0.9.x docs.
- Missing system libraries (glslang-dev, oniguruma-dev, ImGui) prevent full build - created stub implementations as workaround

## Session Continuity

Last session: 2026-03-26T13:02:21.938Z
Stopped at: Completed 04-03-PLAN.md
Resume file: None
