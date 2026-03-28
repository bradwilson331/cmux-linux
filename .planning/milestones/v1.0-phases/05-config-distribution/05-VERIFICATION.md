---
phase: 05-config-distribution
verified: 2026-03-26T15:00:00Z
status: passed
score: 7/7 must-haves verified
---

# Phase 05: Config & Distribution Verification Report

**Phase Goal:** Keyboard shortcuts are configurable via a TOML config file; GitHub Actions CI validates every commit; AppImage artifact ships on release tags
**Verified:** 2026-03-26T15:00:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | App loads config from $XDG_CONFIG_HOME/cmux/config.toml at startup | VERIFIED | `src/config.rs:91-97` config_path() uses XDG_CONFIG_HOME with ~/.config fallback; `src/main.rs:112` calls load_config() before build_ui |
| 2 | Keyboard shortcuts are configurable via [shortcuts] section in config file | VERIFIED | `src/config.rs:16-39` ShortcutConfig with 22 Option<String> fields; `src/config.rs:142-189` ShortcutMap::from_config uses config values with defaults fallback |
| 3 | Invalid config syntax warns to stderr and falls back to defaults | VERIFIED | `src/config.rs:118-121` catches toml parse errors, prints to stderr, returns Config::default() |
| 4 | Invalid individual shortcut warns to stderr and falls back to default for that action | VERIFIED | `src/config.rs:176-184` catches accelerator_parse failure, warns, uses default |
| 5 | Unknown action names in config produce a warning to stderr | VERIFIED | `src/config.rs:126-138` warn_unknown_shortcuts compares against KNOWN_SHORTCUTS |
| 6 | GitHub Actions CI builds, lints, and tests the Linux Rust crate on every push and PR | VERIFIED | `.github/workflows/ci.yml:77-114` linux-build job: apt deps, Zig 0.15.2, libghostty.a build, clippy, cargo build, cargo test |
| 7 | An AppImage artifact is produced on each release tag | VERIFIED | `.github/workflows/release.yml:353-411` linux-appimage job: ubuntu-22.04, linuxdeploy with GTK4 plugin, gh release upload |

**Score:** 7/7 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/config.rs` | Config struct, TOML loading, XDG path, shortcut validation | VERIFIED | 300 lines, Config/ShortcutConfig/ShortcutAction/ShortcutMap with HashMap lookup, load_config, config_path, warn_unknown_shortcuts, 7 unit tests |
| `src/shortcuts.rs` | Config-driven shortcut dispatch via ShortcutMap lookup | VERIFIED | 212 lines, uses shortcut_map.lookup(mods, keyval) dispatch; old hardcoded match removed (grep confirmed) |
| `Cargo.toml` | toml dependency | VERIFIED | Line 12: `toml = "0.8"` |
| `.github/workflows/ci.yml` | linux-build job for CI | VERIFIED | Job present with ubuntu-latest, Zig 0.15.2, clippy, build, test, GTK-x11+Wayland flags |
| `.github/workflows/release.yml` | linux-appimage job for release | VERIFIED | Job present with ubuntu-22.04, linuxdeploy, DEPLOY_GTK_VERSION=4, gh release upload --clobber |
| `resources/cmux.desktop` | Freedesktop .desktop file | VERIFIED | 10 lines, Type=Application, Categories=System;TerminalEmulator;, Exec=cmux-linux |
| `resources/cmux.svg` | App icon for .desktop and AppImage | VERIFIED | Valid SVG, terminal-themed icon with app colors |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/main.rs` | `src/config.rs` | `config::load_config()` call before build_ui | WIRED | Line 112: `let config = crate::config::load_config();` before line 128 build_ui call |
| `src/shortcuts.rs` | `src/config.rs` | ShortcutMap::from_config builds lookup table | WIRED | main.rs:113 calls `ShortcutMap::from_config(&config.shortcuts)`, passes to install_shortcuts at line 329 |
| `src/shortcuts.rs` | `gtk4::accelerator_parse` | Validates accelerator strings at startup | WIRED | config.rs:174 and 182 call `gtk4::accelerator_parse(accel_str)` |
| `.github/workflows/ci.yml` | `ghostty zig build` | CI builds libghostty.a with X11+Wayland flags | WIRED | Lines 97-100: cd ghostty, zig build with -Dgtk-x11=true -Dgtk-wayland=true |
| `.github/workflows/release.yml` | `resources/cmux.desktop` | linuxdeploy bundles .desktop into AppImage | WIRED | Line 395: `--desktop-file resources/cmux.desktop` |

### Behavioral Spot-Checks

Step 7b: SKIPPED (project requires GTK4 display for runtime; no runnable entry points in headless verification environment)

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-----------|-------------|--------|----------|
| CFG-01 | 05-01 | Config file loaded from ~/.config/cmux/config.toml at startup | SATISFIED | config_path() + load_config() in main.rs:112 |
| CFG-02 | 05-01 | Keyboard shortcuts configurable via config file | SATISFIED | ShortcutConfig with 22 fields, ShortcutMap::from_config |
| CFG-03 | 05-01 | Ghostty config loaded via Ghostty own mechanism | SATISFIED | main.rs:161-162: ghostty_config_load_default_files(config) with CFG-03 comment |
| CFG-04 | 05-01 | XDG Base Directory compliance for config | SATISFIED | config_path() uses $XDG_CONFIG_HOME with ~/.config fallback |
| DIST-01 | 05-02 | GitHub Actions CI pipeline on ubuntu-latest | SATISFIED | linux-build job in ci.yml with build+clippy+test |
| DIST-02 | 05-02 | AppImage artifact on release tag | SATISFIED | linux-appimage job in release.yml with linuxdeploy bundling |
| DIST-03 | 05-02 | .desktop file for launcher integration | SATISFIED | resources/cmux.desktop with TerminalEmulator category |
| DIST-04 | 05-02 | App runs on Wayland and X11 | SATISFIED | -Dgtk-x11=true -Dgtk-wayland=true in both CI and release builds |

No orphaned requirements found. All 8 requirement IDs from plans match REQUIREMENTS.md Phase 5 mapping.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None | - | - | - | No anti-patterns detected |

No TODO/FIXME/placeholder patterns found in config.rs or shortcuts.rs. No empty return stubs. No hardcoded empty data patterns.

### Human Verification Required

### 1. Config File Override Test

**Test:** Create `~/.config/cmux/config.toml` with `[shortcuts]\nnew_workspace = "<Ctrl>t"`, launch app, press Ctrl+T (should create workspace) and Ctrl+N (should NOT create workspace)
**Expected:** Ctrl+T creates a new workspace; Ctrl+N passes through to terminal
**Why human:** Requires running GTK4 app with display, keyboard interaction

### 2. Invalid Config Resilience Test

**Test:** Write syntactically broken TOML to config.toml, launch app, check stderr output
**Expected:** App starts normally with default shortcuts; stderr shows parse error warning
**Why human:** Requires launching app and observing stderr output

### 3. CI Pipeline Validation

**Test:** Push a commit and verify linux-build job runs on GitHub Actions
**Expected:** linux-build job succeeds: apt install, zig build, clippy, cargo build, cargo test all green
**Why human:** Requires actual GitHub Actions execution

### 4. AppImage Release Test

**Test:** Create a release tag and verify linux-appimage job produces an AppImage
**Expected:** AppImage artifact attached to the GitHub release
**Why human:** Requires actual tag push and GitHub Actions execution

### Gaps Summary

No gaps found. All 7 observable truths verified. All 8 requirements satisfied. All artifacts exist, are substantive, and are properly wired. Config loading is wired into the startup path before UI construction. Shortcut dispatch uses HashMap lookup instead of hardcoded match arms. CI and release workflows have valid YAML with correct build flags. Both YAML files validated syntactically. All 4 commits verified in git history.

---

_Verified: 2026-03-26T15:00:00Z_
_Verifier: Claude (gsd-verifier)_
