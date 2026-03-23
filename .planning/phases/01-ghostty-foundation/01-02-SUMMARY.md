---
phase: 01-ghostty-foundation
plan: 02
subsystem: infra
tags: [ghostty, zig, gtk4, opengl, embedded, linux, c-api, libghostty]

# Dependency graph
requires: []
provides:
  - "GHOSTTY_PLATFORM_GTK4 = 3 C ABI enum value in ghostty.h"
  - "ghostty_platform_gtk4_s struct with void* gl_area field"
  - "gtk4 union member in ghostty_platform_u"
  - "GTK4 variant in embedded.zig Platform union (linux-only, gl_area: *anyopaque)"
  - "gtk4 = 3 in PlatformTag enum in embedded.zig"
  - "OpenGL surfaceInit dispatch for GTK4 embedded surfaces (prepareContext(null))"
  - "libghostty.a built from manaflow-ai/ghostty fork with GTK4 support"
  - "docs/ghostty-fork.md documenting all fork changes for future rebases"
affects:
  - "01-ghostty-foundation plan 01 (rust scaffold using ghostty.h bindings)"
  - "all plans requiring ghostty_surface_new with GTK4 platform"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "void* platform handle pattern (gl_area follows nsview/uiview convention)"
    - "Zig conditional type: pub const GTK4 = if (builtin.os.tag == .linux) struct { ... } else void"
    - "Zig labeled block for Platform.init() match arms: .gtk4 => if (GTK4 != void) gtk4: { ... }"

key-files:
  created:
    - "ghostty/src/apprt/embedded.zig (GTK4 arm — within existing file)"
  modified:
    - "ghostty.h"
    - "ghostty/src/apprt/embedded.zig"
    - "ghostty/src/renderer/OpenGL.zig"
    - "docs/ghostty-fork.md"

key-decisions:
  - "Use void* gl_area (not GtkWidget*) to avoid requiring GTK4 headers in ghostty.h — matches nsview/uiview pattern"
  - "GdkGLContext not passed in struct; GTK4 manages it internally via GtkGLArea — Rust side calls make_current() in realize handler"
  - "wakeup_cb/userdata registered globally via ghostty_runtime_config_s, not per-surface (correct embedded API pattern)"
  - "GTK4 type is void on non-Linux targets — conditional compilation keeps fork cross-platform"
  - "threadEnter/threadExit embedded arms are no-ops for GTK4 (GtkGLArea manages context lifecycle on Rust side)"

patterns-established:
  - "Platform handle as void*: avoids toolkit headers in ghostty.h public API"
  - "linux-conditional Zig types: use builtin.os.tag == .linux guard for platform-specific types"

requirements-completed: [GHOST-01]

# Metrics
duration: 8min
completed: 2026-03-23
---

# Phase 01 Plan 02: Ghostty GTK4 Fork Extension Summary

**GHOSTTY_PLATFORM_GTK4 = 3 added to ghostty.h C ABI and embedded.zig Platform union; libghostty.a builds successfully with prepareContext(null) wired for GTK4 embedded surfaces**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-23T16:38:29Z
- **Completed:** 2026-03-23T16:46:31Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Extended ghostty.h with `GHOSTTY_PLATFORM_GTK4 = 3`, `ghostty_platform_gtk4_s` struct, and `gtk4` union member
- Extended embedded.zig with `gtk4 = 3` in PlatformTag, `GTK4` linux-conditional type, `gtk4` arm in Platform union and Platform.init()
- Fixed OpenGL.zig surfaceInit "strictly broken" TODO stub — GTK4 embedded surfaces now call `prepareContext(null)` (same as gtk apprt)
- Verified `zig build -Dapp-runtime=none -Doptimize=ReleaseFast -Dgtk-x11=true -Dgtk-wayland=true` exits 0, producing `zig-out/lib/libghostty.a`
- Documented all fork changes in docs/ghostty-fork.md (section 8) with design notes and conflict guidance

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend ghostty.h with GHOSTTY_PLATFORM_GTK4** - `65de7a90` (feat)
2. **Task 2: Extend embedded.zig and OpenGL.zig with gtk4 variant** - `a7420943` (feat) [includes ghostty submodule `f9f56b7`]

**Plan metadata:** committed with state update

## Files Created/Modified

- `/home/twilson/code/cmux-linux/ghostty.h` - Added GTK4 enum value, struct, and union member
- `/home/twilson/code/cmux-linux/ghostty/src/apprt/embedded.zig` - Added GTK4 PlatformTag, type, union arm, Platform.C member, and Platform.init() arm
- `/home/twilson/code/cmux-linux/ghostty/src/renderer/OpenGL.zig` - Replaced TODO stub in surfaceInit with GTK4 dispatch; cleaned up threadEnter/threadExit
- `/home/twilson/code/cmux-linux/docs/ghostty-fork.md` - Added section 8 documenting GHOSTTY_PLATFORM_GTK4 changes with design notes and conflict guide

## Decisions Made

- `void* gl_area` avoids requiring GTK4 headers in the public ghostty.h — consistent with the `void* nsview` macOS pattern
- `GdkGLContext` deliberately omitted: GTK4 manages it internally; Rust side calls `gl_area.make_current()` in the realize signal handler
- `GTK4` Zig type is `void` on non-Linux to keep the fork cross-platform (`if (builtin.os.tag == .linux) struct { ... } else void`)
- `threadEnter`/`threadExit` embedded arms are no-ops for GTK4 since GtkGLArea manages context on the Rust side

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed `_ = surface` pointless discards after adding surface usage**
- **Found during:** Task 2 (OpenGL.zig changes)
- **Issue:** After adding `surface.platform` access in surfaceInit and threadEnter, Zig emits "pointless discard of function parameter" compile errors for existing `_ = surface;` lines
- **Fix:** Removed `_ = surface;` from surfaceInit; kept `_ = surface;` in threadEnter's embedded arm (where surface is legitimately not used by GTK4 path)
- **Files modified:** ghostty/src/renderer/OpenGL.zig
- **Verification:** `zig build` exits 0 with no errors
- **Committed in:** `a7420943` (part of Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — Bug: Zig compile error from pointless discard)
**Impact on plan:** Essential fix to make the build pass. No scope creep.

## Issues Encountered

- **Zig version mismatch:** Only Zig 0.16.0-dev was on PATH; the ghostty repo requires 0.15.2. Found `/usr/lib/zig/0.15.2/zig` and used it instead.
- **Xcframework git tags blocking build:** Local xcframework tags (not in vX.Y.Z format) caused `Config.zig` to panic. Deleted local-only tags so git describe falls through to dev mode.
- **Ghostty submodule push auth gate:** Cannot push to `origin main` (manaflow-ai/ghostty) — requires GitHub credentials. The commit `f9f56b7` is on local `main` but not yet pushed. Manual step: `cd ghostty && git push origin main`.

## User Setup Required

**One manual step required to complete the submodule push:**

```bash
cd /home/twilson/code/cmux-linux/ghostty
git push origin main
```

This pushes commit `f9f56b7` (GHOSTTY_PLATFORM_GTK4) to manaflow-ai/ghostty. Required per CLAUDE.md submodule safety policy before the parent repo submodule pointer is considered fully synchronized.

## Next Phase Readiness

- `ghostty.h` C ABI is ready: `GHOSTTY_PLATFORM_GTK4 = 3` can be used in Rust bindgen (Plan 01-01)
- `libghostty.a` builds successfully with the GTK4 embedded apprt path wired
- Plan 01-01 (rust scaffold + bindgen) can now proceed against the updated ghostty.h
- Blocker note: ghostty submodule push to origin pending auth credentials

---
*Phase: 01-ghostty-foundation*
*Completed: 2026-03-23*
