---
phase: 01-ghostty-foundation
plan: 07
subsystem: terminal
tags: [ghostty, gtk4, opengl, glad, rust, ffi]

requires:
  - phase: 01-ghostty-foundation
    provides: "Plans 01-06: Rust scaffold, GTK4 bindings, ghostty FFI, input handling, tokio runtime"

provides:
  - "App starts without crashing — runtime segfault fixed"
  - "Ghostty surface properly initialized via GLArea realize signal"
  - "OpenGL function pointer loading (GLAD) works correctly"
  - "Rendering routed to main thread via must_draw_from_app_thread pattern"

affects:
  - 01-ghostty-foundation
  - 02-tab-management
  - 03-pane-splitting

tech-stack:
  added:
    - "glad.o compiled from ghostty/vendor/glad/src/gl.c — real GLAD loader replacing stub"
    - "Zig 0.15.2 for rebuilding libghostty.a with must_draw_from_app_thread=true"
  patterns:
    - "Defer ghostty_surface_new to GLArea realize callback (GL context must be current first)"
    - "Rc<RefCell<Option<ghostty_surface_t>>> to share surface across GTK callbacks"
    - "must_draw_from_app_thread=true in embedded.zig routes GL draws through action_cb on main thread"
    - "action_cb handles GHOSTTY_ACTION_RENDER by calling queue_render() on GLArea"

key-files:
  created:
    - "glad.o — compiled GLAD OpenGL loader from ghostty/vendor/glad/src/gl.c"
    - "stubs.o — recompiled without gladLoaderLoadGLContext stub"
  modified:
    - "src/ghostty/surface.rs — deferred surface creation to realize, Rc<RefCell> for callbacks"
    - "src/ghostty/callbacks.rs — action_cb handles GHOSTTY_ACTION_RENDER"
    - "build.rs — link glad.o alongside stubs.o"
    - "stubs.c — removed broken gladLoaderLoadGLContext stub"
    - "ghostty/src/apprt/embedded.zig — added must_draw_from_app_thread=true"

key-decisions:
  - "Defer ghostty_surface_new to GLArea realize (not build_ui): GL context must be current before surface creation"
  - "Remove gladLoaderLoadGLContext stub and compile real GLAD loader from vendor source: stub returned version 0.1, failing OpenGL 4.3 version check"
  - "Add must_draw_from_app_thread=true to embedded App in ghostty fork: prevents renderer thread from calling GL functions without current context"
  - "action_cb handles GHOSTTY_ACTION_RENDER to bridge renderer thread wakeup to GTK queue_render on main thread"

patterns-established:
  - "Pattern: GLArea callback sharing via Rc<RefCell<Option<ghostty_surface_t>>> (all run on GLib main thread)"
  - "Pattern: initialization debug logging via eprintln! for tracing sequence during Phase 1"

requirements-completed: [GHOST-02, GHOST-03]

duration: 45min
completed: 2026-03-24
---

# Phase 01 Plan 07: Fix Ghostty Startup Crash Summary

**Three-root-cause fix for startup segfault: deferred surface creation to GLArea realize, replaced stub GLAD loader with real implementation, and added must_draw_from_app_thread to route renderer draws through main thread**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-03-24T01:15:00Z
- **Completed:** 2026-03-24T02:02:17Z
- **Tasks:** 2 (committed together)
- **Files modified:** 6

## Accomplishments

- App starts without segfault — verified running 4+ seconds on headless X display
- Ghostty surface initialization sequence works: ghostty_init → ghostty_app_new → GLArea realize → ghostty_surface_new → set_size/set_scale/set_focus
- OpenGL function pointers properly loaded via real GLAD loader (not stub)
- Renderer thread correctly routes draw requests to main thread via action_cb

## Task Commits

1. **Tasks 1 & 2: Fix initialization timing + defensive checks** - `d577317f` (fix)

## Files Created/Modified

- `src/ghostty/surface.rs` — Deferred surface creation to realize callback, Rc<RefCell<Option>> surface sharing, null checks and debug logging throughout
- `src/ghostty/callbacks.rs` — action_cb handles GHOSTTY_ACTION_RENDER action to trigger queue_render
- `build.rs` — Link glad.o to provide real GLAD loader symbols
- `stubs.c` — Removed stub gladLoaderLoadGLContext that returned version 0 (causing OpenGL version check failure)
- `glad.o` — Compiled from ghostty/vendor/glad/src/gl.c
- `stubs.o` — Recompiled without glad stubs
- `ghostty/src/apprt/embedded.zig` — Added `pub const must_draw_from_app_thread = true` to App struct

## Decisions Made

- Defer `ghostty_surface_new` to the `realize` callback: confirmed via GDB that the previous code called `ghostty_surface_set_content_scale` before any GL context existed, causing a null dereference inside GLAD's threadlocal context.
- Remove the gladLoaderLoadGLContext stub: the stub returned `1` (parsed as OpenGL version 0.1), which failed ghostty's OpenGL 4.3 minimum version check. Replacing with the real GLAD loader from `ghostty/vendor/glad/src/gl.c` fixed the version check.
- Add `must_draw_from_app_thread = true`: ghostty uses thread-local GLAD context structs. The renderer thread's thread-local was uninitialized, causing null function pointer crash in `drawFrame`. Adding this flag routes all draws through `action_cb(.render)` → `queue_render()` → `render` signal on main thread where GL context is current.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Real GLAD loader required — stub caused OpenGL version check failure**
- **Found during:** Task 1 (Fix Ghostty initialization timing)
- **Issue:** `stubs.c` had `int gladLoaderLoadGLContext(void* context) { return 1; }` — returned version integer 1, parsed as OpenGL 0.1, failing ghostty's 4.3 minimum check. ghostty_surface_new returned null silently.
- **Fix:** Removed stub from stubs.c, compiled real `ghostty/vendor/glad/src/gl.c` as `glad.o`, linked it in build.rs
- **Files modified:** stubs.c, build.rs, glad.o
- **Verification:** ghostty_surface_new now returns non-null pointer
- **Committed in:** d577317f

**2. [Rule 1 - Bug] Renderer thread GL context crash — must_draw_from_app_thread needed**
- **Found during:** Task 1 (after fixing surface creation)
- **Issue:** Ghostty spawns a renderer thread that calls `drawFrame` directly. GLAD's `gl.glad.context` is threadlocal in Zig — main thread loaded it but renderer thread's copy was uninitialized (null function pointers). Crash at `drawFrame` in renderer thread.
- **Fix:** Added `pub const must_draw_from_app_thread = true` to `ghostty/src/apprt/embedded.zig` App struct. Rebuilt libghostty.a with Zig 0.15.2. Added GHOSTTY_ACTION_RENDER handling in action_cb to call queue_render().
- **Files modified:** ghostty/src/apprt/embedded.zig, src/ghostty/callbacks.rs
- **Verification:** App runs 4+ seconds without segfault; renderer thread no longer crashes
- **Committed in:** d577317f (embedded.zig in ghostty submodule commit 39cc6e0)

---

**Total deviations:** 2 auto-fixed (both Rule 1 - Bug)
**Impact on plan:** Both auto-fixes necessary for correct operation. No scope creep.

## Issues Encountered

- The Zig version in PATH (0.16.0-dev) was incompatible with ghostty's minimum_zig_version=0.15.2. Downloaded and used Zig 0.15.2 to rebuild libghostty.a.
- Ghostty log output to stderr was silenced by default (Zig build sets `stderr: bool = build_config.app_runtime != .none` but app_runtime is `.none` for the C library build). Diagnosis relied on adding eprintln! to Rust code and GDB backtrace.

## Known Stubs

None — no intentional stubs remain in the initialization path. The previous gladLoaderLoadGLContext stub has been replaced with the real implementation.

## Next Phase Readiness

- App starts and creates a functional Ghostty terminal surface
- The terminal renders on the main thread via GLArea render signal
- Input routing (keyboard, mouse) is wired up and ready for testing
- Clipboard callbacks are implemented (basic, Phase 1 level)
- Ready for Phase 1 verification: visual terminal display, keyboard input latency test

---
*Phase: 01-ghostty-foundation*
*Completed: 2026-03-24*
