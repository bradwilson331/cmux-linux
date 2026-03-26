# Phase 5: Config + Distribution - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Make keyboard shortcuts configurable via a TOML config file at `~/.config/cmux/config.toml`;
add GitHub Actions CI for Linux (build, clippy, unit tests on ubuntu-latest); produce an
AppImage artifact on release tags; verify Wayland and X11/XWayland compatibility.

No new terminal features. No browser panel. No Rust CLI binary (Python cmux.py remains).
No Ghostty config passthrough (Ghostty uses its own native config mechanism).

</domain>

<decisions>
## Implementation Decisions

### Shortcut Config Syntax

- **D-01:** Shortcuts expressed as GTK-style accelerator strings in TOML:
  `new_workspace = "<Ctrl>n"`, `close_workspace = "<Ctrl><Shift>w"`.
  Parseable with `gtk::accelerator_parse()` — no custom parser needed.
- **D-02:** Remap only — users can rebind the 16 existing actions to different key combos.
  No new custom actions, no action disabling, no socket command bindings.
- **D-03:** Unknown/misspelled action names in config produce a warning to stderr on launch
  but do not prevent startup.
- **D-04:** Duplicate key combos: last entry in file wins. Simple TOML semantics.
- **D-05:** No reset mechanism — user removes or comments out the `[shortcuts]` section
  to restore defaults.
- **D-06:** Config loaded once at startup. No live reload, no file watcher, no socket
  reload command.

### Config File Scope

- **D-07:** Config file contains `[shortcuts]` section only in Phase 5. No `[general]`,
  `[ssh]`, or `[ghostty]` sections. Future phases can add sections.
- **D-08:** Ghostty config (CFG-03) is handled entirely by Ghostty's own config mechanism
  (`~/.config/ghostty/config`). cmux does not proxy or duplicate Ghostty settings.
- **D-09:** Keep Python CLI (`cmux.py` + bash wrapper). No native Rust CLI binary in Phase 5.

### Config Error Handling

- **D-10:** TOML syntax errors: warn to stderr with file path and line number, then launch
  with all default shortcuts. App always starts.
- **D-11:** Individual invalid shortcut values (e.g., unparseable accelerator string): skip
  that entry, warn to stderr with the bad value and the default being used, apply defaults
  for that action only. Other valid entries still take effect.

### Claude's Discretion

- Config struct design (flat vs. nested serde types)
- Whether to generate a commented example config on first run or only document in README
- AppImage bundling approach (linuxdeploy, appimagetool, etc.)
- CI matrix: Ubuntu version(s), whether to test on multiple distros
- `.desktop` file contents and icon
- Wayland/X11 verification approach in CI

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Configuration — CFG-01, CFG-02, CFG-03, CFG-04
- `.planning/REQUIREMENTS.md` §Distribution — DIST-01, DIST-02, DIST-03, DIST-04

### Roadmap
- `.planning/ROADMAP.md` §Phase 5 — success criteria, phase goal, requirement IDs

### Prior Phase Context
- `.planning/phases/03-socket-api-session-persistence/03-CONTEXT.md` — XDG path patterns
  (D-06, D-12), Python CLI decision (D-04, D-05)
- `.planning/phases/04-notifications-hidpi-ssh/04-CONTEXT.md` — SSH config deferred to
  Phase 5 (D-13), notification socket commands (D-16)

### Existing Implementation (read before modifying)
- `src/shortcuts.rs` — 16 hardcoded shortcut match arms; **primary modification target**
- `src/session.rs` — XDG path pattern + serde template to follow for config loading
- `src/main.rs` — App initialization; config loading hooks in here
- `src/socket/mod.rs` — XDG runtime dir path function (reusable pattern)
- `Cargo.toml` — Dependencies; needs `toml` crate addition
- `.github/workflows/ci.yml` — Existing macOS CI; add Linux job
- `.github/workflows/release.yml` — Existing macOS release; add AppImage job

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `session.rs` XDG path resolution: `$XDG_DATA_HOME` with `~/.local/share` fallback —
  copy pattern for `$XDG_CONFIG_HOME` with `~/.config` fallback
- `session.rs` serde load/save pattern: `serde_json::from_str` + atomic write — adapt
  for TOML with `toml::from_str`
- `shortcuts.rs` action dispatch: `handle_shortcut()` match arms — refactor to look up
  from a `HashMap<(bool, bool, bool, gdk::Key), ShortcutAction>` populated from config

### Established Patterns
- XDG Base Directory compliance throughout (socket, session, SSH deploy)
- serde derive on all data models
- Warnings to stderr for recoverable errors (existing convention in socket/session code)
- GTK4 accelerator format used internally in Phase 2 shortcut design

### Integration Points
- `main.rs` app initialization: load config before `build_ui()` call
- `shortcuts.rs`: replace hardcoded match with config-driven lookup table
- `Cargo.toml`: add `toml = "0.8"` dependency
- `.github/workflows/`: add ubuntu-latest jobs

</code_context>

<specifics>
## Specific Ideas

- GTK-style accelerator strings (`<Ctrl>n`) chosen specifically because `gtk::accelerator_parse()`
  validates them natively — no custom parsing code needed
- Config file is shortcuts-only in Phase 5 to keep scope minimal; SSH host presets and
  general settings are future phase work
- App must always launch even with broken config — terminal apps should never refuse to start
  over a config error

</specifics>

<deferred>
## Deferred Ideas

- Agent-browser integration and configuration — browser panel embedding for Linux; separate
  future phase (deferred since Phase 3)
- SSH host presets in config file (`[ssh.hosts]` section) — Phase 4 deferred config file
  integration; add when SSH config matures
- General settings section (`[general]` — default shell, sidebar visibility, session restore)
- Config live reload via socket command or file watcher
- Shortcut action disabling (bind to empty string)
- Custom socket command shortcuts (bind key to arbitrary socket command)
- Native Rust CLI binary (`src/bin/cmux.rs`) — Python CLI sufficient for now
- Ghostty config passthrough section

</deferred>

---

*Phase: 05-config-distribution*
*Context gathered: 2026-03-26 via discuss-phase*
