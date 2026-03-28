# Phase 5: Config + Distribution - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-26
**Phase:** 05-config-distribution
**Areas discussed:** Shortcut config syntax, Config file scope, Config error handling

---

## Shortcut Config Syntax

| Option | Description | Selected |
|--------|-------------|----------|
| GTK-style strings | e.g. "<Ctrl>n", "<Ctrl><Shift>w" — matches GTK accelerator format, parseable with gtk::accelerator_parse() | ✓ |
| Human-friendly strings | e.g. "Ctrl+N", "Ctrl+Shift+W" — more readable but requires custom parser | |
| You decide | Claude picks based on GTK4 ecosystem conventions | |

**User's choice:** GTK-style strings
**Notes:** Avoids custom parser; native GTK4 validation

| Option | Description | Selected |
|--------|-------------|----------|
| Remap only | Users can rebind the 16 existing actions. No new actions. | ✓ |
| Remap + disable | Users can rebind or disable shortcuts | |
| Remap + socket commands | Bind shortcuts to arbitrary socket commands | |

**User's choice:** Remap only

| Option | Description | Selected |
|--------|-------------|----------|
| Warn to stderr | Print warning for unknown action names, continue with defaults | ✓ |
| Strict reject | Refuse to launch on unrecognized action names | |
| You decide | Claude picks | |

**User's choice:** Warn to stderr

| Option | Description | Selected |
|--------|-------------|----------|
| Last wins | Duplicate key combos: last entry in file takes effect | ✓ |
| Error on conflict | Warn or reject on duplicate key combos | |
| You decide | Claude picks | |

**User's choice:** Last wins

| Option | Description | Selected |
|--------|-------------|----------|
| No reset mechanism | Delete/comment out [shortcuts] section to restore defaults | ✓ |
| Special 'defaults' value | e.g. new_workspace = "default" | |
| You decide | Claude picks | |

**User's choice:** No reset mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Startup only | Config read once at launch. Restart to pick up changes. | ✓ |
| Socket command reload | Add config.reload socket command | |
| File watcher | Watch config.toml with inotify and auto-reload | |

**User's choice:** Startup only

---

## Config File Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Shortcuts only | Just [shortcuts] section. Matches Phase 5 requirements. | ✓ |
| Shortcuts + SSH presets | Add [ssh] section for named remote hosts | |
| Shortcuts + general settings | Add [general] for sidebar, default shell, etc. | |

**User's choice:** Shortcuts only

| Option | Description | Selected |
|--------|-------------|----------|
| Ghostty handles its own config | Users edit ~/.config/ghostty/config. cmux doesn't proxy. | ✓ |
| Passthrough section | Add [ghostty] section forwarding settings | |
| You decide | Claude picks | |

**User's choice:** Ghostty handles its own config

| Option | Description | Selected |
|--------|-------------|----------|
| Keep Python CLI | cmux.py + bash wrapper is working | ✓ |
| Add Rust CLI | Native cmux binary in Rust | |
| You decide | Claude assesses | |

**User's choice:** Keep Python CLI

---

## Config Error Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Warn + use all defaults | Print error to stderr, launch with default shortcuts | ✓ |
| Refuse to launch | Exit with error code | |
| You decide | Claude picks | |

**User's choice:** Warn + use all defaults

| Option | Description | Selected |
|--------|-------------|----------|
| Skip bad entry, warn | Use default for that action, warn to stderr | ✓ |
| Reject entire [shortcuts] | Fall back to all defaults if any entry is invalid | |
| You decide | Claude picks | |

**User's choice:** Skip bad entry, warn

---

## Claude's Discretion

- Config struct design (flat vs. nested serde types)
- AppImage bundling approach
- CI matrix details
- .desktop file contents
- Whether to generate example config on first run

## Deferred Ideas

- Agent-browser integration and configuration (user mentioned; redirected as scope creep)
- SSH host presets in config file
- General settings section
- Config live reload
- Shortcut disabling
- Custom socket command shortcuts
- Native Rust CLI binary
- Ghostty config passthrough
