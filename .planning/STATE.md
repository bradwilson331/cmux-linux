---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: Ready to execute
stopped_at: Completed 02-workspaces-pane-splits-03-PLAN.md
last_updated: "2026-03-24T04:47:43.243Z"
progress:
  total_phases: 5
  completed_phases: 1
  total_plans: 16
  completed_plans: 13
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-23)

**Core value:** A Linux user should get the same cmux experience as a Mac user: tabs, splits, workspaces, and socket CLI control — powered by Ghostty's GPU-accelerated terminal.
**Current focus:** Phase 02 — workspaces-pane-splits

## Current Position

Phase: 02 (workspaces-pane-splits) — EXECUTING
Plan: 5 of 7

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

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 1 spike]: GHOST-01 is a fork investigation spike with MEDIUM confidence — exact API surface (GtkGLArea* vs GdkGLContext* vs EGLSurface) unknown until ghostty/src/apprt/embedded.zig is read. This blocks all terminal rendering.
- [Phase 1]: GLib + tokio integration pattern needs validation against current gtk4-rs 0.9.x docs.
- Missing system libraries (glslang-dev, oniguruma-dev, ImGui) prevent full build - created stub implementations as workaround

## Session Continuity

Last session: 2026-03-24T04:47:43.240Z
Stopped at: Completed 02-workspaces-pane-splits-03-PLAN.md
Resume file: None
