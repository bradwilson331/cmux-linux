---
phase: 09-ui-buttons-and-menus
verified: 2026-03-28T03:00:00Z
status: gaps_found
score: 10/11 must-haves verified
gaps:
  - truth: "Browser context menu 'Open in External Browser' and 'Copy URL' actions are functional (D-09)"
    status: partial
    reason: "Actions registered but disabled (set_enabled(false)) because BrowserManager lacks current_url() method. Menu items appear but do nothing when clicked."
    artifacts:
      - path: "src/menus.rs"
        issue: "Lines 245-253: open-external-browser and copy-url actions are disabled stubs with TODO comments"
    missing:
      - "BrowserManager.current_url() method (Phase 8 dependency gap)"
      - "Wire open-external-browser action to call xdg-open with current URL"
      - "Wire copy-url action to copy current URL to clipboard"
---

# Phase 9: UI Buttons and Menus Verification Report

**Phase Goal:** Add GTK4 HeaderBar with toolbar buttons, hamburger menu, sidebar controls (+/close/context menu), and terminal/browser pane context menus
**Verified:** 2026-03-28T03:00:00Z
**Status:** gaps_found
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Config file [ui.header_bar] section parses correctly with style, buttons_left, buttons_right fields | VERIFIED | `src/config.rs` lines 48-75: UiConfig and HeaderBarConfig structs with serde deserialization, default_header_style() |
| 2 | Clicking a menu item dispatches the same operation as the corresponding keyboard shortcut | VERIFIED | `src/menus.rs` register_actions(): all GIO action callbacks invoke the same `handle_*` functions from shortcuts.rs |
| 3 | Menu items display keyboard shortcut hints | VERIFIED | `src/menus.rs` register_accels(): 13 accelerator registrations via set_accels_for_action |
| 4 | Hamburger menu shows File, Edit, View, Help section headings per D-12 | VERIFIED | `src/menus.rs` lines 277-311: build_hamburger_menu() uses `Some("File")`, `Some("Edit")`, `Some("View")`, `Some("Help")` section labels |
| 5 | GTK4 HeaderBar replaces the default window titlebar | VERIFIED | `src/main.rs` line 259-260: `window.set_titlebar(Some(&header))` |
| 6 | Header bar has correct button layout (left: New Workspace, Browser; right: Split H, Split V, Sidebar, Hamburger) | VERIFIED | `src/header_bar.rs`: pack_start for workspace actions, pack_end for pane/view actions |
| 7 | All header bar buttons show tooltips with shortcut hints | VERIFIED | `src/header_bar.rs`: all 6 buttons have set_tooltip_text with shortcut hints (e.g., "New Workspace (Ctrl+N)") |
| 8 | Header bar is hidden when config style is 'none' | VERIFIED | `src/header_bar.rs` line 6: returns None when style == "none" |
| 9 | Sidebar has a '+' button at the bottom that creates a new workspace | VERIFIED | `src/sidebar.rs` lines 36-41: Button with label "+", CSS class "sidebar-add-btn", action_name "win.new-workspace", outside ScrolledWindow |
| 10 | Right-clicking a sidebar row/terminal pane/browser pane shows appropriate context menus | VERIFIED | `src/sidebar.rs` attach_sidebar_context_menu with GestureClick button-3; `src/split_engine.rs` attach_terminal_context_menu on GLArea; browser context menu on preview container |
| 11 | Browser context menu 'Open in External Browser' and 'Copy URL' are functional | FAILED | `src/menus.rs` lines 245-253: both actions are disabled stubs (set_enabled(false)) due to missing BrowserManager.current_url() |

**Score:** 10/11 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/config.rs` | UiConfig and HeaderBarConfig structs | VERIFIED | Structs with serde deserialization, default values |
| `src/menus.rs` | GIO actions, menu models, shortcuts window, about dialog | VERIFIED | 16 GIO actions, 4 menu builders, ShortcutsWindow, AboutDialog. 450 lines. |
| `src/header_bar.rs` | HeaderBar with buttons and hamburger menu | VERIFIED | 63 lines, 5 toolbar buttons + 1 hamburger MenuButton, all wired to GIO actions |
| `src/sidebar.rs` | Sidebar with '+' button, hover close, context menu | VERIFIED | build_sidebar returns (Box, ScrolledWindow, ListBox), '+' button outside scroll, close button per row |
| `Cargo.toml` | gtk4 v4_14 feature flag | VERIFIED | `features = ["v4_12", "v4_14"]` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| src/menus.rs | src/shortcuts.rs | GIO action callbacks call handle_* functions | WIRED | handle_new_workspace, handle_split, handle_close_pane, etc. all called from action closures |
| src/config.rs | src/main.rs | Config struct used in build_ui for header bar style | WIRED | `crate::header_bar::build_header_bar(config)` at line 259 |
| src/header_bar.rs | src/menus.rs | hamburger menu model from build_hamburger_menu() | WIRED | `crate::menus::build_hamburger_menu()` called at line 33 |
| src/header_bar.rs | GIO actions | set_action_name on buttons | WIRED | All 5 buttons use set_action_name("win.*") |
| src/main.rs | src/header_bar.rs | build_header_bar called, set as titlebar | WIRED | Lines 259-260: build_header_bar + set_titlebar |
| src/sidebar.rs | src/menus.rs | build_sidebar_context_menu() for right-click | WIRED | Line 229 in attach_sidebar_context_menu |
| src/sidebar.rs | GIO actions | '+' button set_action_name(win.new-workspace) | WIRED | Line 40 |
| src/split_engine.rs | src/menus.rs | build_terminal_context_menu() for right-click | WIRED | Lines 272, 319, 405, 609 |
| src/split_engine.rs | src/menus.rs | build_browser_context_menu() for browser preview | WIRED | Line 678 |

### Data-Flow Trace (Level 4)

Not applicable -- this phase creates UI controls (buttons, menus) that dispatch actions rather than rendering dynamic data.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Project compiles | `cargo check` | 19 warnings, 0 errors | PASS |
| menus.rs exports register_actions | `grep "pub fn register_actions" src/menus.rs` | Found | PASS |
| menus.rs exports build_hamburger_menu | `grep "pub fn build_hamburger_menu" src/menus.rs` | Found | PASS |
| shortcuts.rs handlers are public | `grep "pub fn handle_" src/shortcuts.rs` | 8 public handlers found | PASS |
| Header bar module wired | `grep "mod header_bar" src/main.rs` | Found | PASS |
| All 6 commits exist in git | `git log --oneline <hash>` | All 6 verified | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-----------|-------------|--------|----------|
| D-01 | 09-03 | Sidebar '+' button creates new workspace | SATISFIED | sidebar.rs: Button("+") with action_name "win.new-workspace" |
| D-02 | 09-03 | Sidebar close button on hover | SATISFIED | sidebar.rs: rebuild_sidebar_row_content adds close_btn with CSS opacity transition |
| D-03 | 09-03 | Sidebar right-click context menu (Rename, Close, Split, Split) | SATISFIED | sidebar.rs: attach_sidebar_context_menu, menus.rs: build_sidebar_context_menu |
| D-04 | 09-01, 09-02 | GTK4 HeaderBar replacing titlebar, configurable | SATISFIED | header_bar.rs + main.rs set_titlebar + config.rs UiConfig |
| D-05 | 09-02 | Header bar buttons (New WS, Split H, Split V, Browser, Sidebar) | SATISFIED | header_bar.rs: 5 buttons with correct icons and actions |
| D-06 | 09-02 | Header bar layout (left=workspace, right=pane/view) | SATISFIED | header_bar.rs: pack_start for left, pack_end for right |
| D-07 | 09-02 | New Browser button triggers BrowserOpen action | SATISFIED | header_bar.rs: browser_btn.set_action_name("win.browser-open") |
| D-08 | 09-03 | Terminal pane right-click menu (Copy, Paste, Split, Close, Browser) | SATISFIED | split_engine.rs: attach_terminal_context_menu, menus.rs: build_terminal_context_menu |
| D-09 | 09-03 | Browser preview right-click menu (Open External, Copy URL, Close) | PARTIAL | Menu model exists with correct items, but open-external-browser and copy-url actions are disabled stubs |
| D-10 | 09-03 | Dividers have no right-click menu | SATISFIED | No context menu attached to GtkPaned separators |
| D-11 | 09-02 | Hamburger menu button in HeaderBar | SATISFIED | header_bar.rs: MenuButton with open-menu-symbolic icon |
| D-12 | 09-01, 09-02 | Hamburger menu sections: File, Edit, View, Help | SATISFIED | menus.rs: build_hamburger_menu with named sections |
| D-13 | 09-01 | Preferences opens config.toml in $EDITOR | SATISFIED | menus.rs: win.preferences action spawns $EDITOR with config path |
| D-14 | 09-02 | Help > Keyboard Shortcuts opens GtkShortcutsWindow | SATISFIED | menus.rs: build_shortcuts_window with 5 sections, 20+ shortcuts |
| D-15 | 09-02 | Help > About opens GtkAboutDialog | SATISFIED | menus.rs: AboutDialog with program_name, version, comments, website, license |
| D-16 | 09-01 | [ui.header_bar] config section with style/buttons_left/buttons_right | SATISFIED | config.rs: UiConfig + HeaderBarConfig with serde deserialization |
| D-17 | 09-02 | All buttons show tooltips with shortcut hints | SATISFIED | header_bar.rs: all buttons have set_tooltip_text with "(Ctrl+...)" hints |
| D-18 | 09-02 | No status bar; tooltips + ShortcutsWindow + menu accels sufficient | SATISFIED | No status bar code; shortcut discovery via 3 mechanisms |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| src/menus.rs | 121 | `win.find` action disabled stub (terminal find not implemented) | Info | Out of scope for this phase; Find shown in menu but greyed out |
| src/menus.rs | 246 | `win.open-external-browser` disabled stub with TODO | Warning | D-09 browser actions non-functional; dependency on BrowserManager.current_url() |
| src/menus.rs | 252 | `win.copy-url` disabled stub with TODO | Warning | D-09 browser actions non-functional; dependency on BrowserManager.current_url() |

### Human Verification Required

### 1. HeaderBar Visual Appearance

**Test:** Launch the app and verify the HeaderBar replaces the default titlebar with correct button layout
**Expected:** Left side shows New Workspace (+) and Browser icons. Right side shows Split Right, Split Down, Toggle Sidebar, and Hamburger menu icons. Dark theme styling matches.
**Why human:** Visual appearance cannot be verified programmatically

### 2. Hamburger Menu Popover

**Test:** Click the hamburger menu button and verify the popover opens with File, Edit, View, Help sections
**Expected:** Menu sections are visually separated with section headings. Each item shows keyboard shortcut hints (e.g., Ctrl+N next to New Workspace).
**Why human:** GTK4 popover rendering and accelerator hint display needs visual confirmation

### 3. Sidebar '+' Button and Hover Close

**Test:** Click the '+' button at the bottom of the sidebar. Hover over a workspace row.
**Expected:** '+' creates a new workspace. Hovering reveals an 'x' close button that closes the workspace when clicked.
**Why human:** Hover CSS transitions and click behavior need runtime testing

### 4. Right-Click Context Menus

**Test:** Right-click on a sidebar workspace row, a terminal pane, and a browser preview pane
**Expected:** Sidebar: Rename, Close, Split Right, Split Down. Terminal: Copy, Paste, Split Right, Split Down, Close Pane, Open Browser Here. Browser: Open in External Browser (greyed), Copy URL (greyed), Close Pane.
**Why human:** Context menu positioning and content need visual confirmation

### 5. Config Style "none" Hides Header Bar

**Test:** Set `[ui.header_bar] style = "none"` in config.toml, restart the app
**Expected:** No header bar is shown; window uses default window manager decorations
**Why human:** Requires app restart and visual verification

### Gaps Summary

One partial gap found: the browser context menu (D-09) exists with the correct menu items ("Open in External Browser", "Copy URL", "Close Pane"), but two of the three actions are disabled stubs because BrowserManager does not expose a `current_url()` method. This is a cross-phase dependency -- the BrowserManager API from Phase 8 needs to be extended. The menu structure itself is complete and correct; only the action wiring is missing.

All other 17 requirements (D-01 through D-08, D-10 through D-18) are fully satisfied with substantive, wired implementations.

The `win.find` action is also disabled, but this is explicitly out of scope for Phase 9 (terminal find is a separate feature). It appears in the Edit menu as a greyed-out item, which is acceptable UX.

---

_Verified: 2026-03-28T03:00:00Z_
_Verifier: Claude (gsd-verifier)_
