---
phase: 05-config-distribution
plan: 01
subsystem: config
tags: [toml, keyboard-shortcuts, xdg, gtk4-accelerator]

# Dependency graph
requires:
  - phase: 02-workspaces-pane-splits
    provides: "16 hardcoded shortcut actions in shortcuts.rs"
provides:
  - "TOML config loading from $XDG_CONFIG_HOME/cmux/config.toml"
  - "Config-driven keyboard shortcut dispatch via HashMap lookup"
  - "ShortcutAction enum and ShortcutMap for all 22 actions"
  - "Graceful config error handling (app always starts)"
affects: [05-02]

# Tech tracking
tech-stack:
  added: [toml 0.8]
  patterns: [config-driven-shortcuts, xdg-config-home]

key-files:
  created: [src/config.rs]
  modified: [src/shortcuts.rs, src/main.rs, Cargo.toml]

key-decisions:
  - "GTK4 accelerator_parse validates shortcut strings natively -- no custom parser"
  - "ModifierType mask (CONTROL|SHIFT|ALT) ignores Caps Lock/Num Lock bits in lookup"
  - "ShortcutMap built at startup, moved into key_pressed closure (no Rc/RefCell needed)"

patterns-established:
  - "XDG_CONFIG_HOME config loading pattern mirroring session.rs XDG_DATA_HOME"
  - "Config-driven dispatch: HashMap<(ModifierType, Key), ShortcutAction> replaces hardcoded match"

requirements-completed: [CFG-01, CFG-02, CFG-03, CFG-04]

# Metrics
duration: 4min
completed: 2026-03-26
---

# Phase 05 Plan 01: Config System Summary

**TOML-based keyboard shortcut configuration with GTK4 accelerator parsing and graceful fallback to defaults**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-26T14:28:20Z
- **Completed:** 2026-03-26T14:32:12Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Config loading from $XDG_CONFIG_HOME/cmux/config.toml with graceful error handling
- All 22 keyboard shortcuts configurable via [shortcuts] section in TOML
- HashMap-based shortcut dispatch replacing hardcoded match arms
- Unknown action name detection with stderr warnings (D-03)
- Invalid accelerator string fallback to per-action defaults (D-11)
- Unit tests for config loading, XDG paths, and shortcut map construction

## Task Commits

Each task was committed atomically:

1. **Task 1: Create config.rs with TOML loading and shortcut validation** - `b3bad226` (feat)
2. **Task 2: Refactor shortcuts.rs to use ShortcutMap and wire config in main.rs** - `f4b76a45` (feat)

## Files Created/Modified
- `src/config.rs` - Config/ShortcutConfig structs, ShortcutAction enum, ShortcutMap with HashMap lookup, load_config with fallback, warn_unknown_shortcuts, unit tests
- `src/shortcuts.rs` - Refactored from hardcoded match to shortcut_map.lookup dispatch
- `src/main.rs` - Added mod config, config loading at startup, ShortcutMap passed through build_ui to install_shortcuts
- `Cargo.toml` - Added toml = "0.8" dependency

## Decisions Made
- GTK4 accelerator_parse used for validation (no custom parser needed)
- ModifierType masked to CONTROL|SHIFT|ALT to ignore Caps Lock/Num Lock
- ShortcutMap moved (not cloned) into key_pressed closure since it's used only once
- CFG-03 confirmed already handled by ghostty_config_load_default_files (no cmux proxying)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed &&str IntoGStr trait bound in accelerator_parse call**
- **Found during:** Task 2 (cargo check)
- **Issue:** default_accel was &&str from tuple reference, accelerator_parse requires &str
- **Fix:** Dereferenced with *default_accel
- **Files modified:** src/config.rs
- **Verification:** cargo check passes
- **Committed in:** f4b76a45 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Trivial type dereference fix. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Config system complete, ready for CI/distribution work in Plan 02
- All 22 shortcuts configurable via TOML
- App starts gracefully with missing/broken config

## Self-Check: PASSED

All files exist. All commits verified.

---
*Phase: 05-config-distribution*
*Completed: 2026-03-26*
