---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: Ready to execute
stopped_at: Completed 01-ghostty-foundation plan 07 - startup crash fixed
last_updated: "2026-03-24T02:03:38.579Z"
progress:
  total_phases: 5
  completed_phases: 0
  total_plans: 9
  completed_plans: 7
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-23)

**Core value:** A Linux user should get the same cmux experience as a Mac user: tabs, splits, workspaces, and socket CLI control — powered by Ghostty's GPU-accelerated terminal.
**Current focus:** Phase 01 — ghostty-foundation

## Current Position

Phase: 01 (ghostty-foundation) — EXECUTING
Plan: 2 of 9

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

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 1 spike]: GHOST-01 is a fork investigation spike with MEDIUM confidence — exact API surface (GtkGLArea* vs GdkGLContext* vs EGLSurface) unknown until ghostty/src/apprt/embedded.zig is read. This blocks all terminal rendering.
- [Phase 1]: GLib + tokio integration pattern needs validation against current gtk4-rs 0.9.x docs.
- Missing system libraries (glslang-dev, oniguruma-dev, ImGui) prevent full build - created stub implementations as workaround

## Session Continuity

Last session: 2026-03-24T02:03:38.577Z
Stopped at: Completed 01-ghostty-foundation plan 07 - startup crash fixed
Resume file: None
