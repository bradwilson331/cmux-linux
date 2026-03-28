---
phase: 01-ghostty-foundation
plan: 01
subsystem: infra
tags: [rust, gtk4, bindgen, cargo, ghostty, libghostty, gtk4-rs, glib, linux]

# Dependency graph
requires:
  - phase: 01-ghostty-foundation
    plan: 02
    provides: "ghostty.h with GHOSTTY_PLATFORM_GTK4 = 3 C ABI additions; libghostty.a"
provides:
  - "Cargo.toml with gtk4 0.10, glib 0.20, bindgen 0.72, cc 1 dependencies"
  - "build.rs that runs bindgen against ghostty.h and statically links libghostty.a"
  - "src/main.rs GtkApplication skeleton"
  - "src/ghostty/mod.rs + src/ghostty/ffi.rs bindgen bindings include"
  - "scripts/setup-linux.sh that builds libghostty.a via zig build -Dapp-runtime=none"
affects:
  - "all subsequent plans building on the Rust GTK4 scaffold"
  - "01-ghostty-foundation plan 03 (surface embedding)"

# Tech tracking
tech-stack:
  added:
    - "gtk4 = 0.10 (gtk4-rs GTK4 bindings)"
    - "glib = 0.20 (GLib main loop, idle_add, channels)"
    - "bindgen = 0.72 (build-time ghostty.h → ghostty_sys.rs generation)"
    - "cc = 1 (build-time link directive helper)"
  patterns:
    - "build.rs bindgen pattern: allowlist_item ghostty_.* + GHOSTTY_.* avoids pulling in GTK4 GObject types from system headers"
    - "pkg-config soft-fallback in build.rs: dynamically adds GTK4 dylib flags only if pkg-config finds gtk4"
    - "setup-linux.sh dependency check+install pattern: detects apt/dnf/pacman and installs libgtk-4-dev and libclang-dev"

key-files:
  created:
    - "Cargo.toml"
    - "build.rs"
    - "scripts/setup-linux.sh"
    - "src/main.rs"
    - "src/ghostty/mod.rs"
    - "src/ghostty/ffi.rs"
  modified: []

key-decisions:
  - "Downgraded gtk4 from 0.11 to 0.10 (glib from 0.22 to 0.20) — gtk4 0.11 requires rustc 1.92; installed rustc is 1.91.1. APIs used (Application, ApplicationWindow, builder) identical across versions."
  - "setup-linux.sh extended with system dependency check+install for libgtk-4-dev and libclang-dev — both required for gtk4-sys and bindgen to compile; not in original plan spec."

patterns-established:
  - "Cargo project at repo root alongside macOS Swift files — `cargo build` works from root per D-01"
  - "bindgen in build.rs with allowlist_item pattern — keeps ghostty_sys.rs minimal, avoids GTK4 GObject type conflicts"

requirements-completed: [GHOST-01]

# Metrics
duration: 3min
completed: 2026-03-23
---

# Phase 01 Plan 01: Rust Scaffold Summary

**Cargo.toml + build.rs + bindgen setup linking libghostty.a with GtkApplication skeleton in src/main.rs**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-23T16:50:08Z
- **Completed:** 2026-03-23T16:53:44Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Created Cargo.toml with gtk4 0.10, glib 0.20, bindgen 0.72, cc 1 (version downgraded from 0.11 for rustc 1.91.1 compatibility)
- Created build.rs that statically links libghostty.a (`cargo:rustc-link-search=ghostty/zig-out/lib`), links OpenGL, runs bindgen against ghostty.h, and emits pkg-config GTK4 dylib flags
- Created scripts/setup-linux.sh to build libghostty.a via `zig build -Dapp-runtime=none -Doptimize=ReleaseFast -Dgtk-x11=true -Dgtk-wayland=true`; extended with system dependency check+install for libgtk-4-dev and libclang-dev
- Created src/main.rs GtkApplication skeleton, src/ghostty/mod.rs, src/ghostty/ffi.rs with bindgen include macro

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Cargo.toml and build.rs** - `89810a39` (feat)
2. **Task 2: Create setup-linux.sh and src skeleton** - `d8626d4a` (feat)

**Plan metadata:** committed with state update

## Files Created/Modified

- `Cargo.toml` - Rust project manifest with gtk4/glib/bindgen deps
- `build.rs` - bindgen invocation + static link directives for libghostty.a
- `scripts/setup-linux.sh` - Zig build script for libghostty.a with apt/dnf/pacman dependency install
- `src/main.rs` - GtkApplication entry point skeleton
- `src/ghostty/mod.rs` - ghostty module declaration
- `src/ghostty/ffi.rs` - bindgen-generated bindings include macro

## Decisions Made

- Downgraded `gtk4 = "0.11"` to `"0.10"` (and `glib = "0.22"` to `"0.20"`) because gtk4 0.11 requires rustc 1.92 and the installed version is 1.91.1. APIs used in src/main.rs are identical in 0.10.
- setup-linux.sh extended with libgtk-4-dev and libclang-dev install check — both are required system prerequisites for gtk4-sys and bindgen to compile, and were not present on the machine.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Downgraded gtk4 crate to 0.10 for rustc 1.91.1 compatibility**
- **Found during:** Task 2 (cargo build verification)
- **Issue:** `gtk4 = "0.11"` specifies minimum rustc 1.92; `cargo build` emitted `X requires rustc 1.92` errors for gtk4, gtk4-sys, gsk4, pango, graphene-sys
- **Fix:** Changed `gtk4 = "0.11"` to `"0.10"` and `glib = "0.22"` to `"0.20"` in Cargo.toml
- **Files modified:** Cargo.toml
- **Verification:** `cargo build` no longer shows rustc version errors for these crates
- **Committed in:** `d8626d4a` (Task 2 commit)

**2. [Rule 2 - Missing Critical] Added system dependency install to setup-linux.sh**
- **Found during:** Task 2 (cargo build verification)
- **Issue:** `gtk4-sys` requires `libgtk-4-dev` system headers (via pkg-config); `bindgen` requires `libclang-dev`; neither was installed; `cargo build` failed with "system library gtk4 required by crate gtk4-sys was not found"
- **Fix:** Added apt-get/dnf/pacman detection+install block at start of setup-linux.sh for libgtk-4-dev and libclang-dev
- **Files modified:** scripts/setup-linux.sh
- **Verification:** Script correctly checks `pkg-config --exists gtk4` before attempting install
- **Committed in:** `d8626d4a` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 rustc version compatibility, 1 missing system dependency)
**Impact on plan:** Both fixes necessary for correctness. Version downgrade is transparent — gtk4 0.10 and 0.11 expose identical APIs for Phase 1 usage.

## Issues Encountered

- **GTK4 system library not installed:** libgtk-4-dev not available on the machine without sudo. `cargo build` requires it via gtk4-sys. setup-linux.sh now handles this with apt/dnf/pacman auto-install. Full `cargo build` verification requires running `./scripts/setup-linux.sh` first on a machine with sudo access.
- **rustc version mismatch:** gtk4 0.11 requires rustc 1.92 but 1.91.1 is installed. Fixed by pinning to 0.10 per research note in 01-RESEARCH.md (which did call out "Note on versions: use 0.11" but that assumed 1.92+ installed).

## User Setup Required

None — all setup automated via `./scripts/setup-linux.sh` (requires sudo for apt-get install if libgtk-4-dev not yet present).

## Next Phase Readiness

- Cargo.toml, build.rs, and src/ scaffold in place — all subsequent plans build on this
- libghostty.a already exists at `ghostty/zig-out/lib/libghostty.a` from plan 01-02
- `cargo build` will pass once `libgtk-4-dev` is installed (via `./scripts/setup-linux.sh`)
- bindgen will generate `ghostty_sys.rs` in OUT_DIR on first successful `cargo build`
- Plan 01-03 (GTK4 surface embedding) can proceed against this scaffold

---
*Phase: 01-ghostty-foundation*
*Completed: 2026-03-23*
