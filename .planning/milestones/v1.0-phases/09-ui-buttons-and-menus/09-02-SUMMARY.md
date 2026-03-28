---
phase: 09-ui-buttons-and-menus
plan: 02
subsystem: ui
tags: [gtk4, headerbar, toolbar, hamburger-menu]

# Dependency graph
requires:
  - phase: 09-ui-buttons-and-menus
    plan: 01
    provides: GIO actions, hamburger menu model, UiConfig/HeaderBarConfig
provides:
  - GTK4 HeaderBar with 5 toolbar buttons + hamburger menu
  - Config-driven header bar visibility (style "none" hides it)
  - CSS styles for header bar buttons matching dark theme
affects: [09-03]

# Tech tracking
tech-stack:
  added: []
  patterns: [GTK4 HeaderBar as window titlebar, MenuButton with GIO menu model]

key-files:
  created: [src/header_bar.rs]
  modified: [src/main.rs]

key-decisions:
  - "All buttons use set_action_name for GIO action dispatch -- no manual click handlers"
  - "pack_end order reversed from visual order (hamburger rightmost, split-right leftmost of right group)"

patterns-established:
  - "HeaderBar buttons: icon + tooltip with shortcut hint + GIO action name + headerbar-btn CSS class"

requirements-completed: [D-04, D-05, D-06, D-07, D-11, D-12, D-14, D-15, D-17, D-18]

# Metrics
duration: 3min
completed: 2026-03-28
---

# Phase 9 Plan 02: HeaderBar with Toolbar Buttons and Hamburger Menu Summary

**GTK4 HeaderBar with 6 buttons (New Workspace, Browser, Split Right, Split Down, Toggle Sidebar, Hamburger) replacing default titlebar**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-28T02:05:33Z
- **Completed:** 2026-03-28T02:08:23Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Created header_bar.rs with build_header_bar() returning a fully wired GTK4 HeaderBar
- 5 toolbar buttons with GIO action dispatch + tooltips with shortcut hints
- Hamburger MenuButton using menu model from menus::build_hamburger_menu()
- Config-driven: returns None when style is "none" to hide header bar
- CSS styles for dark theme consistency (transparent bg, hover/active states)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create header_bar.rs module** - `77cf1cd1` (feat)
2. **Task 2: Integrate HeaderBar into main.rs window layout and add CSS** - `33bd710e` (feat)

## Files Created/Modified
- `src/header_bar.rs` - HeaderBar construction with all buttons and hamburger menu
- `src/main.rs` - Module declaration, CSS rules, set_titlebar call, config param rename

## Decisions Made
- All buttons use set_action_name for GIO action dispatch -- no manual click handlers needed, GTK4 handles routing
- pack_end adds right-to-left, so hamburger (rightmost) is pack_end'd first, then sidebar, split-v, split-h

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Pre-existing compilation errors from parallel agents (BrowserAction variant, sidebar tuple change) -- not caused by this plan's changes, documented as out-of-scope

## Known Stubs
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- HeaderBar fully wired to GIO actions from Plan 01
- Ready for Plan 03 context menu attachment
- CSS class pattern established for future button styling

## Self-Check: PASSED

All 2 files verified present. Both commit hashes (77cf1cd1, 33bd710e) verified in git log.
