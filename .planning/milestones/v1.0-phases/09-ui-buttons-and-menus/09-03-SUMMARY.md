---
phase: 09-ui-buttons-and-menus
plan: 03
subsystem: ui
tags: [gtk4, sidebar, context-menu, popover, css]

# Dependency graph
requires:
  - phase: 09-ui-buttons-and-menus
    provides: GIO actions and menu models (menus.rs)
provides:
  - Sidebar '+' button for new workspace creation (D-01)
  - Sidebar hover close button per row (D-02)
  - Sidebar right-click context menu with Rename/Close/Split (D-03)
  - Terminal pane right-click context menu (D-08)
  - Browser preview right-click context menu (D-09)
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: [PopoverMenu from gio::Menu model for context menus, GestureClick button-3 for right-click]

key-files:
  created: []
  modified: [src/sidebar.rs, src/main.rs, src/shortcuts.rs, src/menus.rs, src/app_state.rs, src/split_engine.rs, src/socket/commands.rs]

key-decisions:
  - "Sidebar '+' button outside ScrolledWindow to prevent scrolling away (Pitfall 5)"
  - "Reuse rebuild_sidebar_row_content in create_workspace/restore_workspace for consistent row layout"
  - "Terminal context menu on GLArea via GestureClick button-3 (does not interfere with Ghostty mouse handling)"

patterns-established:
  - "attach_terminal_context_menu: standalone function for right-click on any GLArea"
  - "wire_latest_row: convenience to wire close + context menu on newly added sidebar row"

requirements-completed: [D-01, D-02, D-03, D-08, D-09, D-10]

# Metrics
duration: 6min
completed: 2026-03-28
---

# Phase 9 Plan 03: Sidebar Controls and Pane Context Menus Summary

**Sidebar '+' button, hover close buttons, right-click context menus on sidebar rows, terminal panes, and browser preview panes**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-28T02:05:52Z
- **Completed:** 2026-03-28T02:12:11Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Restructured sidebar to Box(V) > [ScrolledWindow(ListBox), Button('+')] so '+' button stays pinned at bottom
- Added hover-visible close button to every sidebar row via CSS opacity transition
- Wired right-click context menus (PopoverMenu) on sidebar rows, terminal GLAreas, and browser preview containers
- Updated sidebar toggle (Ctrl+B and win.toggle-sidebar) to control outer Box instead of ScrolledWindow

## Task Commits

Each task was committed atomically:

1. **Task 1: Add sidebar '+' button, hover close button, and CSS** - `7cfdc91d` (feat)
2. **Task 2: Wire terminal and browser pane context menus** - `d30d492b` (feat)

## Files Created/Modified
- `src/sidebar.rs` - Restructured build_sidebar, added rebuild_sidebar_row_content with close button, wire_row_close_button, attach_sidebar_context_menu, wire_latest_row
- `src/main.rs` - Updated to use sidebar_box, added CSS for sidebar buttons and context menus
- `src/shortcuts.rs` - Changed sidebar param from ScrolledWindow to Box, wire_latest_row after new workspace
- `src/menus.rs` - Changed sidebar param from ScrolledWindow to Box
- `src/app_state.rs` - Replaced inline row building with rebuild_sidebar_row_content
- `src/split_engine.rs` - Added attach_terminal_context_menu, wired on all GLArea creation points, browser context menu on preview
- `src/socket/commands.rs` - Added missing BrowserAction enum variant (Rule 3 fix)

## Decisions Made
- Sidebar '+' button placed outside ScrolledWindow per RESEARCH.md Pitfall 5, preventing it from scrolling out of view
- Used rebuild_sidebar_row_content in create_workspace/restore_workspace instead of inline row building for consistency
- Terminal context menu attached directly to GLArea with GestureClick button-3; Ghostty doesn't use right-click for its own handling

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing BrowserAction enum variant in socket/commands.rs**
- **Found during:** Task 1 (cargo check)
- **Issue:** Another agent added BrowserAction handler in handlers.rs but didn't add the corresponding enum variant in commands.rs, causing compile failure
- **Fix:** Added `BrowserAction { req_id, action, params, resp_tx }` variant to SocketCommand enum
- **Files modified:** src/socket/commands.rs
- **Verification:** cargo check passes
- **Committed in:** 7cfdc91d

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Fix necessary for compilation. No scope creep.

## Issues Encountered
None

## Known Stubs
None -- all sidebar controls and context menus are fully wired to existing GIO actions.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All phase 9 UI controls are now complete
- Sidebar has full mouse interaction: click to switch, '+' to create, 'x' to close, right-click for context menu
- Terminal and browser panes have right-click context menus dispatching to GIO actions

## Self-Check: PASSED

All 7 files verified present. Both commit hashes (7cfdc91d, d30d492b) verified in git log.

---
*Phase: 09-ui-buttons-and-menus*
*Completed: 2026-03-28*
