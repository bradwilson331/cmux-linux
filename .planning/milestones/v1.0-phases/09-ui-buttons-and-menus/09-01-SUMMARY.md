---
phase: 09-ui-buttons-and-menus
plan: 01
subsystem: ui
tags: [gtk4, gio-actions, menus, shortcuts, config]

# Dependency graph
requires:
  - phase: 05-config-distribution
    provides: Config struct with shortcuts section
  - phase: 08-add-agent-browser
    provides: BrowserManager for browser actions
provides:
  - GIO action system (register_actions) for all menu/button dispatch
  - Hamburger menu model with File/Edit/View/Help sections
  - Sidebar, terminal, and browser context menu models
  - Keyboard shortcut hints via register_accels
  - UiConfig/HeaderBarConfig for header bar customization
  - ShortcutsWindow with all keyboard shortcuts
  - AboutDialog
affects: [09-02, 09-03]

# Tech tracking
tech-stack:
  added: [gtk4 v4_14 feature]
  patterns: [GIO actions on ApplicationWindow, menu models via gio::Menu]

key-files:
  created: [src/menus.rs]
  modified: [src/config.rs, Cargo.toml, src/shortcuts.rs, src/main.rs, src/app_state.rs]

key-decisions:
  - "Browser open-external and copy-url actions disabled until BrowserManager exposes current_url()"
  - "Added active_split_engine (immutable) method to AppState for copy/paste action closures"
  - "Config param prefixed with underscore (_config) in build_ui since header bar style is wired in Plan 02"

patterns-established:
  - "GIO actions: register on ApplicationWindow, invoke via win.action-name from menus/buttons"
  - "Menu accelerator hints: must call set_accels_for_action to show shortcuts in menus"

requirements-completed: [D-16, D-04, D-13]

# Metrics
duration: 6min
completed: 2026-03-28
---

# Phase 9 Plan 01: GIO Actions and Menu Infrastructure Summary

**GIO action system with 16 actions, hamburger/context menu models, shortcuts window, and UiConfig for header bar customization**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-28T01:56:36Z
- **Completed:** 2026-03-28T02:02:47Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Extended Config with UiConfig/HeaderBarConfig for [ui.header_bar] section with style, buttons_left, buttons_right fields
- Created menus.rs with 16 GIO actions covering File/Edit/View/Help/Browser operations
- Built hamburger menu model with named sections (File, Edit, View, Help) per D-12
- Built sidebar, terminal, and browser context menu models
- Implemented copy/paste via ghostty_surface_binding_action FFI
- Added ShortcutsWindow with grouped keyboard shortcuts using GTK 4.14 API
- Added AboutDialog with app metadata

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend config.rs with UiConfig and update Cargo.toml** - `b30ae4f3` (feat)
2. **Task 2: Create menus.rs with GIO actions and menu models** - `cb30d323` (feat)

## Files Created/Modified
- `src/menus.rs` - GIO action registration, menu model builders, shortcuts window, about dialog
- `src/config.rs` - UiConfig and HeaderBarConfig structs with serde deserialization
- `Cargo.toml` - gtk4 v4_14 feature flag
- `src/shortcuts.rs` - Handler functions made public for menu reuse
- `src/main.rs` - Module declaration, register_actions/register_accels calls, config param to build_ui
- `src/app_state.rs` - Added active_split_engine (immutable) accessor

## Decisions Made
- Browser open-external and copy-url actions registered as disabled stubs since BrowserManager lacks current_url() method; to be wired when available
- Added immutable active_split_engine() to AppState so copy/paste closures can use borrow() instead of borrow_mut()
- Used ShortcutsWindow::builder().build() since ShortcutsWindow::new() doesn't exist in gtk4-rs 0.10

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added active_split_engine (immutable) to AppState**
- **Found during:** Task 2 (copy/paste action closures)
- **Issue:** Copy/paste closures need immutable access to find active pane/surface, but only active_split_engine_mut existed
- **Fix:** Added `pub fn active_split_engine(&self) -> Option<&SplitEngine>` method
- **Files modified:** src/app_state.rs
- **Verification:** cargo check passes
- **Committed in:** cb30d323

**2. [Rule 1 - Bug] ShortcutsWindow::new() doesn't exist**
- **Found during:** Task 2 (shortcuts window construction)
- **Issue:** gtk4-rs 0.10 ShortcutsWindow has no new() constructor
- **Fix:** Used ShortcutsWindow::builder().build() instead
- **Files modified:** src/menus.rs
- **Verification:** cargo check passes
- **Committed in:** cb30d323

**3. [Rule 2 - Missing Critical] Browser actions disabled without current_url()**
- **Found during:** Task 2 (browser context menu actions)
- **Issue:** BrowserManager has no current_url() method, so open-external-browser and copy-url cannot function
- **Fix:** Registered actions as disabled with TODO comments; Plan 03 can wire them
- **Files modified:** src/menus.rs
- **Verification:** cargo check passes
- **Committed in:** cb30d323

---

**Total deviations:** 3 auto-fixed (1 blocking, 1 bug, 1 missing critical)
**Impact on plan:** All auto-fixes necessary for correctness. No scope creep.

## Issues Encountered
None

## Known Stubs
- `win.open-external-browser` action registered but disabled (no BrowserManager.current_url())
- `win.copy-url` action registered but disabled (no BrowserManager.current_url())
- `win.find` action registered but disabled (terminal find not yet implemented)

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- GIO actions ready for header bar buttons (Plan 02) and context menu attachment (Plan 03)
- Menu models can be set on MenuButton widgets
- register_accels ensures shortcut hints display in menus

## Self-Check: PASSED

All 6 files verified present. Both commit hashes (b30ae4f3, cb30d323) verified in git log.

---
*Phase: 09-ui-buttons-and-menus*
*Completed: 2026-03-28*
