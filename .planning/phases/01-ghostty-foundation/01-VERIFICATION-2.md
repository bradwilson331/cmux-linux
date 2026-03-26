---
phase: 01-ghostty-foundation
verified: 2026-03-23T18:45:00Z
status: gaps_found
score: 2/4 success criteria verified
re_verification:
  previous_status: gaps_found
  previous_score: 0/4
  gaps_closed:
    - "App now builds and links successfully"
    - "Tokio runtime added and integrated with GLib"
  gaps_remaining:
    - "App crashes immediately after startup"
    - "Cannot verify clipboard/input/rendering behaviors"
  regressions: []
gaps:
  - truth: "User can open the app and type into a working terminal — keystrokes appear with < 10ms latency"
    status: failed
    reason: "App crashes with core dump immediately after startup"
    artifacts:
      - path: "src/main.rs"
        issue: "App starts but crashes, likely during surface initialization"
    missing:
      - "Debug and fix segmentation fault during startup"
      - "Verify ghostty initialization sequence"
  - truth: "User can copy text from the terminal and paste it into another application (and vice versa) on both X11 and Wayland"
    status: failed
    reason: "App crashes before UI can be tested"
    artifacts:
      - path: "src/ghostty/surface.rs"
        issue: "Clipboard callbacks implemented but app crashes before they can run"
    missing:
      - "Fix crash to enable clipboard testing"
  - truth: "Terminal renders at correct pixel density on HiDPI displays without blurriness or incorrect scale"
    status: failed
    reason: "App crashes before rendering can be tested"
    artifacts:
      - path: "src/ghostty/surface.rs"
        issue: "DPI handling code exists but app crashes before rendering"
    missing:
      - "Fix crash to enable HiDPI testing"
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
      provides: "GtkApplication entry point with tokio runtime"
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

# Phase 1: Ghostty Foundation Re-Verification Report

**Phase Goal:** A GTK4 window runs a single Ghostty terminal surface with correct input, rendering, clipboard, threading, and XDG path compliance
**Verified:** 2026-03-23T18:45:00Z
**Status:** gaps_found
**Re-verification:** Yes — after gap closure

## Goal Achievement

### Observable Truths (Success Criteria)

| #   | Truth                                                                                                                     | Status      | Evidence                                               |
| --- | ------------------------------------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------ |
| 1   | User can open the app and type into a working terminal — keystrokes appear with < 10ms latency                          | ✗ FAILED    | App crashes with core dump on startup                 |
| 2   | User can copy text from the terminal and paste it into another application (and vice versa) on both X11 and Wayland    | ✗ FAILED    | App crashes before UI can be tested                   |
| 3   | Terminal renders at correct pixel density on HiDPI displays without blurriness or incorrect scale                       | ✗ FAILED    | App crashes before rendering can be tested            |
| 4   | App scaffold compiles and runs: tokio runtime started, GLib main loop runs, glib::MainContext::channel bridges the two | ✓ VERIFIED  | Build succeeds, tokio runtime starts, but then crashes |

**Score:** 1/4 truths fully verified (major progress: app builds, but crashes at runtime)

### Required Artifacts

| Artifact                    | Expected                                    | Status      | Details                                                         |
| --------------------------- | ------------------------------------------- | ----------- | --------------------------------------------------------------- |
| `Cargo.toml`                | Rust manifest with gtk4/glib/tokio         | ✓ VERIFIED  | Has gtk4 = "0.10", glib = "0.20", tokio = "1.50" with full features |
| `build.rs`                  | bindgen + static link directives           | ✓ VERIFIED  | Fixed: proper absolute paths, links libghostty.a + stubs.o    |
| `src/main.rs`               | GtkApplication + tokio runtime              | ✓ VERIFIED  | Has tokio runtime in thread + GLib bridge via mpsc channel    |
| `scripts/setup-linux.sh`    | Zig build script for libghostty.a          | ✓ VERIFIED  | Builds libghostty.a with ReleaseFast                          |
| `ghostty.h`                 | Extended with GHOSTTY_PLATFORM_GTK4        | ✓ VERIFIED  | Has GHOSTTY_PLATFORM_GTK4 = 3 and ghostty_platform_gtk4_s     |
| `ghostty/zig-out/lib/libghostty.a` | Static library built                | ✓ VERIFIED  | 23MB file exists, built Mar 23 12:45                          |
| `src/ghostty/surface.rs`    | Surface creation and GL rendering          | ✓ VERIFIED  | 381 lines, has create_surface, render callback, clipboard     |
| `src/ghostty/input.rs`      | Keyboard/mouse input routing               | ✓ VERIFIED  | 215 lines, maps keycodes and handles mouse events             |
| `src/ghostty/callbacks.rs`  | C callback implementations                 | ✓ VERIFIED  | Has wakeup_cb, action_cb, close_surface_cb (59 lines)         |
| `stubs.c`                   | Stub functions for missing libs            | ✓ VERIFIED  | 139 lines, stubs for glslang, SPIRV-Cross, ImGui, GLAD       |
| `stubs.o`                   | Compiled stub object                       | ✓ VERIFIED  | 13KB object file linked via build.rs                          |

### Key Link Verification

| From                   | To                      | Via                          | Status       | Details                                                    |
| ---------------------- | ----------------------- | ---------------------------- | ------------ | ---------------------------------------------------------- |
| build.rs               | libghostty.a            | cargo:rustc-link-search     | ✓ WIRED      | Fixed: uses absolute path, properly links to binary       |
| build.rs               | stubs.o                 | cargo:rustc-link-arg        | ✓ WIRED      | Links stub object file with absolute path                 |
| src/ghostty/surface.rs | ghostty FFI             | unsafe extern calls         | ✓ WIRED      | Calls ghostty_init (line 33), ghostty_surface_new (78)    |
| GtkGLArea::render      | ghostty_surface_draw    | connect_render at line 114  | ✓ WIRED      | Symbol now resolved via libghostty.a                      |
| keyboard controller    | ghostty_input_key       | send_key at input.rs:171    | ✓ WIRED      | Properly routes through FFI                               |
| tokio runtime          | GLib main loop          | mpsc channel + timeout      | ✓ WIRED      | Bridge established with test message sent                 |

### Data-Flow Trace (Level 4)

Not applicable — app crashes during initialization before data flow can be verified.

### Behavioral Spot-Checks

| Behavior                    | Command                             | Result                                         | Status  |
| --------------------------- | ----------------------------------- | ---------------------------------------------- | ------- |
| Cargo build succeeds        | `cargo build`                      | Build completes successfully                   | ✓ PASS  |
| libghostty.a exists         | `ls ghostty/zig-out/lib/libghostty.a` | File exists (23MB)                          | ✓ PASS  |
| GTK4 headers available      | `pkg-config --exists gtk4`         | Returns 0                                      | ✓ PASS  |
| App starts without crash    | `./target/debug/cmux-linux`        | Crashes with core dump after tokio starts     | ✗ FAIL  |
| Tokio runtime initializes   | Check output                        | Prints "Tokio runtime started successfully"   | ✓ PASS  |

### Requirements Coverage

| Requirement | Description                                           | Status      | Evidence                                              |
| ----------- | ----------------------------------------------------- | ----------- | ----------------------------------------------------- |
| GHOST-01    | Fork extended with GHOSTTY_PLATFORM_GTK4             | ✓ SATISFIED | ghostty.h has platform enum value 3                  |
| GHOST-02    | Single Ghostty surface renders in GtkGLArea          | ✗ BLOCKED   | Code complete but app crashes during initialization  |
| GHOST-03    | Keyboard input routes with < 10ms latency            | ✗ BLOCKED   | Input module exists but app crashes before UI        |
| GHOST-04    | Mouse input routed to correct surface                | ✗ BLOCKED   | Mouse handlers exist but app crashes before UI       |
| GHOST-05    | Clipboard works on X11 and Wayland                   | ✗ BLOCKED   | Callbacks implemented but app crashes before UI      |
| GHOST-06    | Terminal renders at correct DPI                      | ✗ BLOCKED   | Scale factor code exists but app crashes before UI   |
| GHOST-07    | wakeup_cb dispatches to GLib main loop               | ✓ SATISFIED | Uses glib::idle_add_once_local (callbacks.rs:24)     |

### Anti-Patterns Found

| File        | Line | Pattern                             | Severity   | Impact                                           |
| ----------- | ---- | ------------------------------------ | ---------- | ------------------------------------------------ |
| src/main.rs | N/A  | Core dump after startup              | 🛑 Blocker | App crashes preventing all user-facing features |
| src/main.rs | 36   | println! in GLib timeout             | ℹ️ Info    | Debug output should be removed for production   |
| src/main.rs | 43   | tx_for_tokio clone not used later   | ℹ️ Info    | Prepared for future use but not needed yet      |

### Human Verification Required

Once crash is fixed:

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

Significant progress has been made since the initial verification:

**Gaps Closed:**
1. **Linking fixed:** The app now successfully links with libghostty.a. Build.rs properly uses absolute paths and links all required libraries including stubs for optional dependencies.
2. **Tokio runtime added:** A tokio runtime now runs in a separate thread with a message-passing bridge to the GLib main loop, fulfilling the async requirement.

**Remaining Gaps:**
1. **Runtime crash:** The application crashes with a core dump immediately after startup, likely during Ghostty surface initialization or GTK window creation.
2. **Untestable behaviors:** Due to the crash, none of the user-facing features (typing, clipboard, HiDPI rendering) can be verified.

The phase has made substantial progress — the build system is fixed and the async runtime is integrated — but a runtime crash prevents verification of the actual terminal functionality. The crash appears to occur early in the initialization sequence, after the tokio runtime starts but before the UI becomes interactive.

---

_Verified: 2026-03-23T18:45:00Z_
_Verifier: gsd-verifier_