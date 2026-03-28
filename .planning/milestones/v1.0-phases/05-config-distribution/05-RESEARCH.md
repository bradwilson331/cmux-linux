# Phase 5: Config + Distribution - Research

**Researched:** 2026-03-26
**Domain:** TOML configuration, GTK4 keyboard accelerator parsing, GitHub Actions Linux CI, AppImage packaging
**Confidence:** HIGH

## Summary

Phase 5 adds a TOML-based keyboard shortcut configuration system and Linux distribution infrastructure. The configuration domain is well-constrained: load `~/.config/cmux/config.toml` at startup, parse a `[shortcuts]` section using `gtk4::accelerator_parse()` to validate accelerator strings, and build a lookup table that replaces the hardcoded match arms in `shortcuts.rs`. The existing `session.rs` XDG path resolution and serde patterns provide a direct template.

The distribution domain requires a new `linux-ci` job in `.github/workflows/ci.yml` (build + clippy + unit tests on ubuntu-latest), a new `linux-appimage` job in `.github/workflows/release.yml` (produce AppImage on tag push), a `.desktop` file for launcher integration, and validation that the app works on both Wayland and X11. The Ghostty submodule build in CI requires Zig installation and the same system dependencies as `setup-linux.sh`.

**Primary recommendation:** Use `toml` crate v0.8 with serde `Deserialize` for config parsing; use `gtk4::accelerator_parse()` for shortcut string validation; use `linuxdeploy` with the GTK plugin for AppImage creation in CI.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Shortcuts expressed as GTK-style accelerator strings in TOML: `new_workspace = "<Ctrl>n"`, `close_workspace = "<Ctrl><Shift>w"`. Parseable with `gtk::accelerator_parse()`.
- **D-02:** Remap only -- users can rebind the 16 existing actions to different key combos. No new custom actions, no action disabling, no socket command bindings.
- **D-03:** Unknown/misspelled action names in config produce a warning to stderr on launch but do not prevent startup.
- **D-04:** Duplicate key combos: last entry in file wins. Simple TOML semantics.
- **D-05:** No reset mechanism -- user removes or comments out the `[shortcuts]` section to restore defaults.
- **D-06:** Config loaded once at startup. No live reload, no file watcher, no socket reload command.
- **D-07:** Config file contains `[shortcuts]` section only in Phase 5. No `[general]`, `[ssh]`, or `[ghostty]` sections.
- **D-08:** Ghostty config (CFG-03) is handled entirely by Ghostty's own config mechanism (`~/.config/ghostty/config`). cmux does not proxy or duplicate Ghostty settings.
- **D-09:** Keep Python CLI (`cmux.py` + bash wrapper). No native Rust CLI binary in Phase 5.
- **D-10:** TOML syntax errors: warn to stderr with file path and line number, then launch with all default shortcuts. App always starts.
- **D-11:** Individual invalid shortcut values (e.g., unparseable accelerator string): skip that entry, warn to stderr with the bad value and the default being used, apply defaults for that action only. Other valid entries still take effect.

### Claude's Discretion
- Config struct design (flat vs. nested serde types)
- Whether to generate a commented example config on first run or only document in README
- AppImage bundling approach (linuxdeploy, appimagetool, etc.)
- CI matrix: Ubuntu version(s), whether to test on multiple distros
- `.desktop` file contents and icon
- Wayland/X11 verification approach in CI

### Deferred Ideas (OUT OF SCOPE)
- Agent-browser integration and configuration
- SSH host presets in config file (`[ssh.hosts]` section)
- General settings section (`[general]`)
- Config live reload via socket command or file watcher
- Shortcut action disabling (bind to empty string)
- Custom socket command shortcuts
- Native Rust CLI binary
- Ghostty config passthrough section
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CFG-01 | cmux config file loaded from `~/.config/cmux/config.toml` at startup | XDG config path pattern from `session.rs`; `toml` crate for parsing |
| CFG-02 | Keyboard shortcuts are configurable via config file | `gtk4::accelerator_parse()` validates accelerator strings; HashMap lookup replaces match arms |
| CFG-03 | Ghostty config loaded via Ghostty's own config mechanism | Already handled -- `ghostty_config_load_default_files()` in `main.rs` reads `~/.config/ghostty/config`. No cmux work needed. |
| CFG-04 | XDG Base Directory compliance | Config in `$XDG_CONFIG_HOME/cmux/`, data in `$XDG_DATA_HOME/cmux/`, socket in `$XDG_RUNTIME_DIR/cmux/` -- all already implemented except config path |
| DIST-01 | GitHub Actions CI: build, clippy, unit tests on ubuntu-latest | New job in `ci.yml`; requires zig install, system deps, libghostty.a build |
| DIST-02 | AppImage artifact produced on each release tag | `linuxdeploy` + GTK plugin in `release.yml`; new job alongside existing macOS release |
| DIST-03 | `.desktop` file included for application launcher integration | Standard freedesktop.org `.desktop` file in repo root or `resources/` |
| DIST-04 | App runs on Wayland and X11/XWayland | Ghostty built with `-Dgtk-x11=true -Dgtk-wayland=true`; GTK4 handles backend selection via `GDK_BACKEND` |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Test quality policy:** Tests must verify observable runtime behavior, not source code text/grep patterns. No tests that only read metadata files to assert key existence.
- **Never run tests locally.** All tests run via GitHub Actions or on the VM. Unit tests via `cargo test --bin cmux-linux` are safe (no app launch).
- **Surgical changes:** Touch only what is necessary. Match existing style.
- **Simplicity first:** No features beyond what was asked. No abstractions for single-use code.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| toml | 0.8.x | Parse TOML config files with serde | Standard Rust TOML parser; toml 1.x exists but 0.8 is what CONTEXT.md specifies and is stable |
| gtk4 (accelerator_parse) | 0.10.3 (already pinned) | Parse GTK accelerator strings like `<Ctrl>n` | Built-in GTK4 function, no custom parser needed |
| serde | 1 (already present) | Derive Deserialize for config struct | Already in Cargo.toml |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| linuxdeploy | latest AppImage | Bundle GTK4 app into AppImage with dependency resolution | CI release job only |
| linuxdeploy-plugin-gtk | latest | Bundle GTK4 schemas, loaders, themes into AppImage | Required for GTK4 apps in AppImage |
| appimagetool | latest (via linuxdeploy plugin) | Create final AppImage from AppDir | Called by linuxdeploy plugin |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| toml 0.8 | toml 1.1 | toml 1.x supports TOML spec 1.1; 0.8 is simpler and matches CONTEXT.md guidance. Either works. |
| linuxdeploy | cargo-appimage | cargo-appimage is a convenience wrapper but less control over GTK plugin integration |
| linuxdeploy | appimage-builder | appimage-builder uses recipe YAML files; heavier setup, more suited for complex apps |

**Installation:**
```bash
# Cargo.toml addition
toml = "0.8"
```

**Version verification:** `toml` crate latest stable in 0.8.x series is 0.8.19 (verified via `cargo search`). The `toml = "0.8"` semver range will resolve to latest 0.8.x.

## Architecture Patterns

### Recommended Project Structure
```
src/
  config.rs          # Config struct, loading, XDG path, defaults
  shortcuts.rs       # Refactored: HashMap<(Mods, Key), Action> lookup
  main.rs            # Load config before build_ui(), pass to shortcuts
resources/
  cmux.desktop       # Freedesktop .desktop file for AppImage/launcher
  cmux.svg           # App icon (or cmux.png)
```

### Pattern 1: Config Loading (follows session.rs pattern)
**What:** Load TOML config at startup with graceful fallback
**When to use:** App initialization in `main()`, before `build_ui()`
**Example:**
```rust
// Source: Adapted from session.rs XDG pattern + toml crate docs
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    #[serde(default)]
    pub shortcuts: ShortcutConfig,
}

#[derive(Deserialize, Default, Debug)]
pub struct ShortcutConfig {
    pub new_workspace: Option<String>,
    pub close_workspace: Option<String>,
    pub next_workspace: Option<String>,
    pub prev_workspace: Option<String>,
    // ... all 16 actions as Option<String>
}

pub fn config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{home}/.config")
    });
    PathBuf::from(base).join("cmux").join("config.toml")
}

pub fn load_config() -> Config {
    let path = config_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Config::default(),
        Err(e) => {
            eprintln!("cmux: config read error at {}: {e}", path.display());
            return Config::default();
        }
    };
    match toml::from_str::<Config>(&content) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("cmux: config parse error at {}: {e}", path.display());
            Config::default()
        }
    }
}
```

### Pattern 2: Shortcut Lookup Table (replaces hardcoded match)
**What:** Build a HashMap from config at startup, look up actions by (modifiers, key)
**When to use:** Refactoring `shortcuts.rs` to be config-driven
**Example:**
```rust
// Source: gtk4-rs docs for accelerator_parse
use gtk4::gdk::{Key, ModifierType};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    NewWorkspace,
    CloseWorkspace,
    NextWorkspace,
    PrevWorkspace,
    // ... all 16 actions
}

pub struct ShortcutMap {
    map: HashMap<(ModifierType, Key), ShortcutAction>,
}

impl ShortcutMap {
    pub fn from_config(config: &ShortcutConfig) -> Self {
        let mut map = HashMap::new();
        let defaults = Self::defaults();

        // For each action, use config value if valid, else use default
        for (action, config_accel, default_accel) in [
            (ShortcutAction::NewWorkspace, &config.new_workspace, "<Ctrl>n"),
            (ShortcutAction::CloseWorkspace, &config.close_workspace, "<Ctrl><Shift>w"),
            // ... etc
        ] {
            let accel_str = config_accel.as_deref().unwrap_or(default_accel);
            match gtk4::accelerator_parse(accel_str) {
                Some((key, mods)) => { map.insert((mods, key), action); }
                None => {
                    eprintln!("cmux: invalid shortcut '{}', using default '{}'", accel_str, default_accel);
                    if let Some((key, mods)) = gtk4::accelerator_parse(default_accel) {
                        map.insert((mods, key), action);
                    }
                }
            }
        }
        Self { map }
    }

    pub fn lookup(&self, mods: ModifierType, key: Key) -> Option<ShortcutAction> {
        self.map.get(&(mods, key)).copied()
    }
}
```

### Pattern 3: CI Linux Job Structure
**What:** GitHub Actions job that builds libghostty.a then runs cargo build/clippy/test
**When to use:** New job in `ci.yml`
**Example:**
```yaml
linux-build:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
      with:
        submodules: recursive
    - name: Install system deps
      run: |
        sudo apt-get update
        sudo apt-get install -y libgtk-4-dev libclang-dev libfontconfig1-dev \
          libfreetype6-dev libonig-dev libgl-dev
    - name: Install Zig
      uses: mlugg/setup-zig@v1
      with:
        version: 0.13.0  # match ghostty's required version
    - name: Build libghostty.a
      run: ./scripts/setup-linux.sh
    - name: Rust toolchain
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - name: Clippy
      run: cargo clippy --all-targets -- -D warnings
    - name: Build
      run: cargo build
    - name: Test
      run: cargo test --bin cmux-linux
```

### Anti-Patterns to Avoid
- **Custom accelerator parser:** Do NOT write a custom parser for shortcut strings. `gtk4::accelerator_parse()` handles all GTK accelerator syntax including abbreviations like `<Ctl>`, case insensitivity, etc.
- **Bundling GTK4 libraries manually in AppImage:** Use `linuxdeploy-plugin-gtk` to handle GTK4 theme/schema/loader bundling. Manual bundling misses GLib schemas, Pango modules, GDK pixbuf loaders, etc.
- **Runtime config validation:** Do NOT validate config on every keystroke. Build the lookup table once at startup, then use it for O(1) dispatch.
- **Blocking on missing config:** The app must ALWAYS launch, even with a completely broken config file (D-10).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Accelerator string parsing | Custom key+modifier parser | `gtk4::accelerator_parse()` | Handles abbreviations, case insensitivity, all GTK key names |
| TOML parsing | Manual string parsing | `toml` crate with serde `Deserialize` | Edge cases in TOML spec (multiline, escape sequences, table syntax) |
| AppImage bundling | Manual library copying | `linuxdeploy` + GTK plugin | GTK4 has dozens of runtime dependencies (schemas, loaders, themes) |
| XDG path resolution | Hardcoded `~/.config` | `$XDG_CONFIG_HOME` with fallback | XDG Base Directory spec compliance |

**Key insight:** The GTK4 ecosystem provides `accelerator_parse()` specifically for this use case. Rolling a custom parser would miss edge cases (key aliases, modifier abbreviations) and create maintenance burden.

## Common Pitfalls

### Pitfall 1: ModifierType Comparison Precision
**What goes wrong:** GTK4 `ModifierType` includes bits for lock keys (Caps Lock, Num Lock). Comparing raw modifier state against accelerator-parsed modifiers fails when lock keys are active.
**Why it happens:** `EventControllerKey` reports all active modifiers including lock keys; `accelerator_parse()` returns only the explicitly specified modifiers.
**How to avoid:** Mask the modifier state to only include Ctrl/Shift/Alt before lookup: `mods & (CONTROL_MASK | SHIFT_MASK | ALT_MASK)`.
**Warning signs:** Shortcuts stop working when Caps Lock or Num Lock is on.

### Pitfall 2: Key vs. Keyval Case Sensitivity
**What goes wrong:** `Key::n` and `Key::N` are different keyvals. When Shift is held, GTK reports the uppercase keyval `Key::N`, not `Key::n`.
**Why it happens:** GTK translates keypress to the post-shift character.
**How to avoid:** `accelerator_parse("<Ctrl><Shift>w")` returns `(Key::W, CONTROL_MASK | SHIFT_MASK)` -- this naturally matches because GTK reports `Key::W` when Shift+W is pressed. The current code already handles this correctly (see existing `Key::W`, `Key::D`, `Key::X` patterns for shifted shortcuts).
**Warning signs:** Shifted shortcuts don't match in the HashMap.

### Pitfall 3: AppImage GTK4 Theme/Schema Missing
**What goes wrong:** AppImage runs but shows no styled widgets, or crashes with "GLib-GIO-ERROR: No GSettings schemas are installed on the system."
**Why it happens:** GTK4 requires GSettings schemas, Pango modules, GDK pixbuf loaders, and Adwaita theme files at runtime. Plain `linuxdeploy` without the GTK plugin doesn't bundle these.
**How to avoid:** Always use `linuxdeploy-plugin-gtk` alongside `linuxdeploy`. Set `DEPLOY_GTK_VERSION=4` environment variable.
**Warning signs:** App launches but looks wrong, or crashes immediately with schema errors.

### Pitfall 4: Zig Version Mismatch in CI
**What goes wrong:** Ghostty build fails with cryptic compile errors.
**Why it happens:** Each Ghostty version requires a specific Zig version. The fork may pin a different version than upstream.
**How to avoid:** Check `ghostty/build.zig.zon` or the ghostty docs for the required Zig version. Pin that exact version in CI.
**Warning signs:** `zig build` fails with syntax errors or type mismatches.

### Pitfall 5: TOML serde Unknown Fields
**What goes wrong:** User adds a misspelled key like `new_workspac = "<Ctrl>n"` and it's silently ignored, or serde rejects the entire file.
**Why it happens:** By default, serde silently ignores unknown fields. With `#[serde(deny_unknown_fields)]`, the entire `[shortcuts]` section fails if any field is unknown.
**How to avoid:** Do NOT use `deny_unknown_fields` on the `ShortcutConfig` struct. Instead, after deserialization, compare parsed keys against known action names and warn about unknown ones (D-03). Use a secondary pass with `toml::Value` to detect unknown keys.
**Warning signs:** Users report that misspelled shortcuts are silently ignored with no feedback.

## Code Examples

### Config TOML Format
```toml
# ~/.config/cmux/config.toml

[shortcuts]
new_workspace = "<Ctrl>n"
close_workspace = "<Ctrl><Shift>w"
next_workspace = "<Ctrl>bracketright"
prev_workspace = "<Ctrl>bracketleft"
rename_workspace = "<Ctrl><Shift>r"
toggle_sidebar = "<Ctrl>b"
split_right = "<Ctrl>d"
split_down = "<Ctrl><Shift>d"
close_pane = "<Ctrl><Shift>x"
focus_left = "<Ctrl><Shift>Left"
focus_right = "<Ctrl><Shift>Right"
focus_up = "<Ctrl><Shift>Up"
focus_down = "<Ctrl><Shift>Down"
workspace_1 = "<Ctrl>1"
workspace_2 = "<Ctrl>2"
# ... workspace_3 through workspace_9
```

### Unknown Key Detection (D-03)
```rust
// Source: toml crate docs + serde
use toml::Value;

fn warn_unknown_shortcuts(content: &str) {
    let known_keys: HashSet<&str> = [
        "new_workspace", "close_workspace", "next_workspace", "prev_workspace",
        "rename_workspace", "toggle_sidebar", "split_right", "split_down",
        "close_pane", "focus_left", "focus_right", "focus_up", "focus_down",
        "workspace_1", "workspace_2", "workspace_3", "workspace_4",
        "workspace_5", "workspace_6", "workspace_7", "workspace_8", "workspace_9",
    ].into_iter().collect();

    if let Ok(value) = content.parse::<Value>() {
        if let Some(shortcuts) = value.get("shortcuts").and_then(|v| v.as_table()) {
            for key in shortcuts.keys() {
                if !known_keys.contains(key.as_str()) {
                    eprintln!("cmux: unknown shortcut action '{}' in config, ignoring", key);
                }
            }
        }
    }
}
```

### .desktop File
```ini
[Desktop Entry]
Type=Application
Name=cmux
Comment=GPU-accelerated terminal multiplexer
Exec=cmux
Icon=cmux
Terminal=false
Categories=System;TerminalEmulator;
Keywords=terminal;multiplexer;tabs;splits;
StartupWMClass=cmux
```

### AppImage CI Job
```yaml
linux-appimage:
  runs-on: ubuntu-22.04  # Not ubuntu-latest; pin for reproducibility
  needs: [linux-build]
  steps:
    - uses: actions/checkout@v4
      with:
        submodules: recursive
    - name: Install system deps
      run: |
        sudo apt-get update
        sudo apt-get install -y libgtk-4-dev libclang-dev libfontconfig1-dev \
          libfreetype6-dev libonig-dev libgl-dev
    - name: Install Zig
      # Pin to ghostty's required version
      uses: mlugg/setup-zig@v1
    - name: Build libghostty.a
      run: ./scripts/setup-linux.sh
    - name: Build release binary
      run: cargo build --release
    - name: Download linuxdeploy
      run: |
        wget -q https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
        wget -q https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh
        chmod +x linuxdeploy-x86_64.AppImage linuxdeploy-plugin-gtk.sh
    - name: Create AppImage
      run: |
        DEPLOY_GTK_VERSION=4 ./linuxdeploy-x86_64.AppImage \
          --appdir AppDir \
          --executable target/release/cmux-linux \
          --desktop-file resources/cmux.desktop \
          --icon-file resources/cmux.svg \
          --plugin gtk \
          --output appimage
    - name: Upload AppImage artifact
      uses: actions/upload-artifact@v4
      with:
        name: cmux-linux-appimage
        path: cmux-*.AppImage
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| appimagetool only | linuxdeploy + plugins | 2022+ | linuxdeploy handles dependency resolution, plugin architecture for GTK/Qt |
| Ubuntu 20.04 for CI | Ubuntu 22.04+ | 2024 | GTK4 dev packages available; glib >= 2.72 required by gtk4-rs 0.10 |
| `toml-rs` crate | `toml` crate 0.8+/1.x | 2023 | Old `toml-rs` renamed to `toml`; 0.8+ rewrote parser for spec compliance |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test framework (cargo test) |
| Config file | None needed -- `#[cfg(test)] mod tests` inline |
| Quick run command | `cargo test --bin cmux-linux` |
| Full suite command | `cargo test --bin cmux-linux` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CFG-01 | Config loads from XDG path, returns default on missing/error | unit | `cargo test --bin cmux-linux config::tests` | Wave 0 |
| CFG-02 | Valid accelerator strings parsed into shortcut map; invalid strings fall back to defaults | unit | `cargo test --bin cmux-linux config::tests` | Wave 0 |
| CFG-03 | Ghostty config loaded by ghostty_config_load_default_files | N/A (already works) | N/A | N/A |
| CFG-04 | XDG_CONFIG_HOME respected for config path | unit | `cargo test --bin cmux-linux config::tests` | Wave 0 |
| DIST-01 | CI builds, runs clippy, runs tests | CI workflow | `gh workflow run ci.yml` | Wave 0 (workflow file) |
| DIST-02 | AppImage artifact produced | CI workflow | `gh workflow run release.yml` | Wave 0 (workflow file) |
| DIST-03 | .desktop file present and valid | manual (desktop-file-validate) | N/A | Wave 0 |
| DIST-04 | App runs on Wayland and X11 | manual | N/A | N/A |

### Sampling Rate
- **Per task commit:** `cargo test --bin cmux-linux`
- **Per wave merge:** Full `cargo test --bin cmux-linux` + `cargo clippy`
- **Phase gate:** CI workflow green on push

### Wave 0 Gaps
- [ ] `src/config.rs` tests -- covers CFG-01, CFG-02, CFG-04
- [ ] `.github/workflows/ci.yml` linux job -- covers DIST-01
- [ ] `resources/cmux.desktop` -- covers DIST-03

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo/rustc | Rust build | Yes | 1.91.1 | -- |
| zig | libghostty.a build | Yes | 0.16.0-dev | Pin to ghostty's required version in CI |
| pkg-config + gtk4 | Build linking | Yes | System | -- |
| linuxdeploy | AppImage creation | No | -- | Download in CI; not needed locally |
| appimagetool | AppImage finalization | No | -- | Bundled in linuxdeploy plugin |
| desktop-file-validate | .desktop validation | No | -- | Skip local validation; CI can install |

**Missing dependencies with no fallback:**
- None blocking

**Missing dependencies with fallback:**
- linuxdeploy/appimagetool: downloaded on-demand in CI workflow steps

## Open Questions

1. **Exact Zig version for Ghostty fork**
   - What we know: Ghostty pins specific Zig versions. The local machine has 0.16.0-dev but setup-linux.sh may need a different version.
   - What's unclear: Which exact Zig version the manaflow-ai/ghostty fork requires.
   - Recommendation: Check `ghostty/build.zig.zon` at implementation time. Use `mlugg/setup-zig@v1` action in CI with the pinned version.

2. **Ubuntu version for CI**
   - What we know: ubuntu-latest currently resolves to Ubuntu 22.04 or 24.04. GTK4 dev packages are available on 22.04+.
   - What's unclear: Whether ubuntu-latest (possibly 24.04) has all needed deps or if we should pin ubuntu-22.04.
   - Recommendation: Start with ubuntu-latest. If GTK4 version issues arise, pin to ubuntu-22.04.

3. **toml 0.8 vs 1.x**
   - What we know: CONTEXT.md says `toml = "0.8"`. Current latest is toml 1.1.0.
   - What's unclear: Whether the CONTEXT.md specification of 0.8 was intentional or based on older info.
   - Recommendation: Use `toml = "0.8"` as specified. The API is identical for our use case (`from_str` with serde).

## Sources

### Primary (HIGH confidence)
- [gtk4-rs accelerator_parse docs](https://docs.rs/gtk4/latest/gtk4/fn.accelerator_parse.html) -- function signature, return type, string format
- [toml crate docs](https://docs.rs/toml/latest/toml/) -- serde deserialization API, version info
- Existing codebase: `src/shortcuts.rs` (16 hardcoded match arms), `src/session.rs` (XDG path pattern, serde load/save)
- Existing codebase: `src/main.rs` (app initialization flow, ghostty config loading)
- Existing codebase: `build.rs` (system library dependencies for linking)
- Existing codebase: `scripts/setup-linux.sh` (system deps, zig build flags)
- Existing CI: `.github/workflows/ci.yml` (macOS CI structure), `.github/workflows/release.yml` (macOS release structure)

### Secondary (MEDIUM confidence)
- [linuxdeploy](https://github.com/linuxdeploy/linuxdeploy) -- AppDir creation, plugin architecture
- [linuxdeploy-plugin-gtk](https://github.com/linuxdeploy/linuxdeploy-plugin-gtk) -- GTK bundling for AppImage
- [AppImage docs](https://docs.appimage.org/packaging-guide/from-source/linuxdeploy-user-guide.html) -- linuxdeploy user guide
- [Ghostty build docs](https://ghostty.org/docs/install/build) -- build from source, Zig version requirements

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- toml crate and gtk4::accelerator_parse are well-documented, stable APIs
- Architecture: HIGH -- config loading pattern directly mirrors existing session.rs; shortcut refactor is mechanical
- Pitfalls: HIGH -- GTK modifier masking and AppImage GTK bundling are well-documented issues
- CI/Distribution: MEDIUM -- AppImage + GTK4 has documented challenges; exact Zig version needs verification at impl time

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (stable domain, 30-day validity)
