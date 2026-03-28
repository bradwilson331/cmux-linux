---
phase: 01-ghostty-foundation
verified: 2026-03-23T14:20:00Z
status: gaps_found
score: 2/4 success criteria verified
gaps:
  - truth: "User can open the app and type into a working terminal — keystrokes appear with < 10ms latency"
    status: failed
    reason: "App fails to link due to missing libghostty.a symbols"
    artifacts:
      - path: "build.rs"
        issue: "Library build gets -l static=ghostty but binary build doesn't inherit it"
      - path: "stubs.c"
        issue: "Missing ghostty_surface_draw and ghostty_surface_refresh stubs"
    missing:
      - "Fix build.rs to ensure libghostty.a links to final binary"
      - "Add missing function stubs or fix linkage"
  - truth: "User can copy text from the terminal and paste it into another application (and vice versa) on both X11 and Wayland"
    status: failed
    reason: "App doesn't run due to linking errors"
    artifacts:
      - path: "src/ghostty/surface.rs"
        issue: "Clipboard callbacks defined but app won't link"
    missing:
      - "Fix linking issues first to enable runtime testing"
human_verification:
  - test: "Launch the app and type characters"
    expected: "Characters appear with < 10ms latency"
    why_human: "Latency measurement requires human perception"
  - test: "Copy text from terminal, paste elsewhere"
    expected: "Text transfers correctly on X11/Wayland"
    why_human: "Clipboard interaction requires external apps"
  - test: "Run on HiDPI display"
    expected: "Text renders sharp without blur"
    why_human: "Visual quality assessment"
must_haves:
  truths:
    - "User can open the app and type into a working terminal — keystrokes appear with < 10ms latency"
    - "User can copy text from the terminal and paste it into another application (and vice versa) on both X11 and Wayland"
    - "Terminal renders at correct pixel density on HiDPI displays without blurriness or incorrect scale"
    - "App scaffold compiles and runs: tokio runtime started, GLib main loop runs, glib::MainContext::channel bridges the two"
  artifacts:
    - path: "Cargo.toml"
      provides: "Rust project manifest with gtk4/glib/bindgen deps"
    - path: "build.rs"
      provides: "bindgen invocation + static link directives"
    - path: "src/main.rs"
      provides: "GtkApplication entry point skeleton"
    - path: "scripts/setup-linux.sh"
      provides: "Zig build script for libghostty.a"
    - path: "ghostty.h"
      provides: "Extended header with GHOSTTY_PLATFORM_GTK4"
    - path: "src/ghostty/surface.rs"
      provides: "Surface creation and GL rendering"
    - path: "src/ghostty/input.rs"
      provides: "Keyboard/mouse input routing"
    - path: "src/ghostty/callbacks.rs"
      provides: "C callback implementations"
  key_links:
    - from: "build.rs"
      to: "libghostty.a"
      via: "cargo:rustc-link directives"
    - from: "src/ghostty/surface.rs"
      to: "ghostty FFI"
      via: "unsafe extern calls"
    - from: "GtkGLArea signals"
      to: "ghostty_surface_draw"
      via: "render callback"
---

# Phase 1: Ghostty Foundation Verification Report

**Phase Goal:** A GTK4 window runs a single Ghostty terminal surface with correct input, rendering, clipboard, threading, and XDG path compliance
**Verified:** 2026-03-23T14:20:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Success Criteria)

| #   | Truth                                                                                                                     | Status      | Evidence                                               |
| --- | ------------------------------------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------ |
| 1   | User can open the app and type into a working terminal — keystrokes appear with < 10ms latency                          | ✗ FAILED    | App fails to link; undefined ghostty symbols          |
| 2   | User can copy text from the terminal and paste it into another application (and vice versa) on both X11 and Wayland    | ✗ FAILED    | App doesn't run due to linking errors                 |
| 3   | Terminal renders at correct pixel density on HiDPI displays without blurriness or incorrect scale                       | ✗ FAILED    | App doesn't run due to linking errors                 |
| 4   | App scaffold compiles and runs: tokio runtime started, GLib main loop runs, glib::MainContext::channel bridges the two | ⚠️ PARTIAL  | GTK scaffold exists but no tokio runtime in code      |

**Score:** 0/4 truths fully verified (app won't link/run)

### Required Artifacts

| Artifact                    | Expected                                    | Status      | Details                                                         |
| --------------------------- | ------------------------------------------- | ----------- | --------------------------------------------------------------- |
| `Cargo.toml`                | Rust manifest with gtk4/glib/bindgen       | ✓ VERIFIED  | Has gtk4 = "0.10", glib = "0.20", bindgen = "0.72"            |
| `build.rs`                  | bindgen + static link directives           | ⚠️ PARTIAL  | Has bindgen, but linking fails for binary                      |
| `src/main.rs`               | GtkApplication entry point                  | ✓ VERIFIED  | Creates Application and ApplicationWindow                      |
| `scripts/setup-linux.sh`    | Zig build script for libghostty.a          | ✓ VERIFIED  | Builds libghostty.a with ReleaseFast                          |
| `ghostty.h`                 | Extended with GHOSTTY_PLATFORM_GTK4        | ✓ VERIFIED  | Has GHOSTTY_PLATFORM_GTK4 = 3 and ghostty_platform_gtk4_s     |
| `ghostty/zig-out/lib/libghostty.a` | Static library built                | ✓ VERIFIED  | 23MB file exists, built Mar 23 12:45                          |
| `src/ghostty/surface.rs`    | Surface creation and GL rendering          | ✓ VERIFIED  | 381 lines, has create_surface, render callback, clipboard     |
| `src/ghostty/input.rs`      | Keyboard/mouse input routing               | ✓ VERIFIED  | 215 lines, maps keycodes and handles mouse events             |
| `src/ghostty/callbacks.rs`  | C callback implementations                 | ✓ VERIFIED  | Has wakeup_cb, action_cb, close_surface_cb                    |
| `src/ghostty/ffi.rs`        | FFI bindings include                       | ✓ VERIFIED  | Includes generated ghostty_sys.rs from OUT_DIR                |

### Key Link Verification

| From                   | To                      | Via                          | Status       | Details                                                 |
| ---------------------- | ----------------------- | ---------------------------- | ------------ | ------------------------------------------------------- |
| build.rs               | libghostty.a            | cargo:rustc-link-search     | ⚠️ PARTIAL   | Library build gets it, binary build doesn't inherit    |
| build.rs               | ghostty.h               | bindgen::Builder            | ✓ WIRED      | Line 89: .header("ghostty.h")                         |
| src/ghostty/surface.rs | ghostty FFI             | unsafe extern calls         | ✓ WIRED      | Many calls: ghostty_init, ghostty_surface_new, etc.    |
| GtkGLArea::render      | ghostty_surface_draw    | closure at line 114         | ✗ NOT_WIRED  | Symbol undefined at link time                          |
| keyboard controller    | ghostty_input_key       | send_key at input.rs:171    | ✓ WIRED      | Properly routes through FFI                            |

### Data-Flow Trace (Level 4)

Not applicable — app doesn't run due to linking errors. Cannot verify actual data flow.

### Behavioral Spot-Checks

| Behavior                    | Command        | Result                                    | Status  |
| --------------------------- | -------------- | ----------------------------------------- | ------- |
| Cargo build succeeds        | `cargo build`  | Linking fails with undefined symbols     | ✗ FAIL  |
| libghostty.a exists         | `ls ghostty/zig-out/lib/libghostty.a` | File exists (23MB) | ✓ PASS  |
| GTK4 headers available      | `pkg-config --exists gtk4` | Returns 0 | ✓ PASS  |

### Requirements Coverage

| Requirement | Description                                           | Status      | Evidence                                          |
| ----------- | ----------------------------------------------------- | ----------- | ------------------------------------------------- |
| GHOST-01    | Fork extended with GHOSTTY_PLATFORM_GTK4             | ✓ SATISFIED | ghostty.h has platform enum value 3              |
| GHOST-02    | Single Ghostty surface renders in GtkGLArea          | ✗ BLOCKED   | Code exists but app won't link                   |
| GHOST-03    | Keyboard input routes with < 10ms latency            | ✗ BLOCKED   | Input module exists but app won't run            |
| GHOST-04    | Mouse input routed to correct surface                | ✗ BLOCKED   | Mouse handlers exist but app won't run           |
| GHOST-05    | Clipboard works on X11 and Wayland                   | ✗ BLOCKED   | Callbacks implemented but app won't run          |
| GHOST-06    | Terminal renders at correct DPI                      | ✗ BLOCKED   | Scale factor code exists but app won't run       |
| GHOST-07    | wakeup_cb dispatches to GLib main loop               | ✓ SATISFIED | Uses glib::idle_add_once_local (callbacks.rs:24) |

### Anti-Patterns Found

| File        | Line | Pattern                          | Severity   | Impact                                        |
| ----------- | ---- | -------------------------------- | ---------- | --------------------------------------------- |
| build.rs    | N/A  | Missing -l static=ghostty propagation | 🛑 Blocker | Binary doesn't link libghostty.a             |
| stubs.c     | N/A  | Missing function stubs           | 🛑 Blocker | ghostty_surface_draw/refresh undefined       |
| src/main.rs | N/A  | No tokio runtime                 | ⚠️ Warning | Success criteria mentions tokio but not used |

### Human Verification Required

Once linking is fixed:

### 1. Typing Latency Test

**Test:** Open the app and type rapidly in the terminal
**Expected:** Characters appear with imperceptible delay (< 10ms)
**Why human:** Latency measurement requires human perception and specialized tools

### 2. Clipboard Integration Test

**Test:** Select text in terminal, copy (Ctrl+C), paste in gedit. Then copy from gedit and paste in terminal.
**Expected:** Text transfers correctly in both directions on both X11 and Wayland
**Why human:** Requires interaction with external applications

### 3. HiDPI Rendering Test

**Test:** Run app on a HiDPI display (scale factor > 1.0)
**Expected:** Text renders sharp without blur or pixelation
**Why human:** Visual quality assessment requires human judgment

### Gaps Summary

The phase has created most of the required artifacts and code structure, but the application fails to link and run due to build configuration issues. The main problems are:

1. **Linking failure:** The build.rs file correctly specifies libghostty.a for the library crate but this doesn't propagate to the binary crate, causing undefined symbol errors
2. **Missing stubs:** Some ghostty functions (ghostty_surface_draw, ghostty_surface_refresh) are called but not defined
3. **No tokio runtime:** Success criteria mentions tokio+GLib integration but only GLib is present

Until the linking issues are resolved, none of the user-facing success criteria can be verified. The code structure appears complete but cannot be tested.

---

_Verified: 2026-03-23T14:20:00Z_
_Verifier: gsd-verifier_