# Phase 9: UI Buttons and Menus - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Add clickable UI elements (buttons, context menus, header bar, hamburger menu) so operations currently keyboard-shortcut-only become mouse-accessible. The app is currently almost entirely keyboard-driven — this phase makes it discoverable and usable for mouse-oriented users.

</domain>

<decisions>
## Implementation Decisions

### Sidebar controls
- **D-01:** Simple '+' button at the bottom of the sidebar workspace list. Creates a new local workspace (same as Ctrl+N). SSH workspace creation stays keyboard-only or via menu.
- **D-02:** Close button ('×') appears on hover at the right edge of each sidebar workspace row. Matches terminal emulator conventions (iTerm2, Warp, Ghostty tabs).
- **D-03:** Right-click context menu on sidebar workspace rows with: Rename, Close, Split Right, Split Down. Quick access to per-workspace actions with accelerator hints shown.

### Header bar / toolbar
- **D-04:** GTK4 HeaderBar as default, replacing the window titlebar. Configurable via config.toml to use custom toolbar or no header bar.
- **D-05:** Header bar buttons: New Workspace (+), Split Horizontal, Split Vertical, New Browser, Toggle Sidebar.
- **D-06:** Layout: Left side = workspace actions ([+New] [Browser]). Right side = pane/view actions ([Split H] [Split V] [Sidebar]).
- **D-07:** New Browser button triggers the existing BrowserOpen shortcut action (same as Ctrl+Shift+B).

### Right-click context menus
- **D-08:** Terminal pane right-click menu: Copy, Paste, Split Right, Split Down, Close Pane, Open Browser Here. All items show keyboard accelerator hints.
- **D-09:** Browser preview pane right-click menu (adapted): Close Pane, Open in External Browser, Copy URL. No copy/paste since no text selection in preview.
- **D-10:** Dividers have no right-click menu — drag-only, keep simple.

### Hamburger menu
- **D-11:** Hamburger menu ('☰') button in the HeaderBar opens a popover with organized sections. No traditional menu bar. Modern GTK4/GNOME convention.
- **D-12:** Menu sections:
  - File: New Workspace, New SSH Workspace, New Browser, Close Pane, Close Workspace, Quit
  - Edit: Copy, Paste, Find, Preferences
  - View: Toggle Sidebar, Split Right, Split Down
  - Help: Keyboard Shortcuts, About
- **D-13:** Preferences opens config.toml in $EDITOR (no in-app settings UI). Consistent with Ghostty's approach.
- **D-14:** Help > Keyboard Shortcuts opens GtkShortcutsWindow with all shortcuts grouped by category.
- **D-15:** Help > About opens GtkAboutDialog with app name, version, license, website link.

### Config.toml UI section
- **D-16:** Granular config under `[ui.header_bar]` with fields: `style` ('gtk' | 'custom' | 'none', default 'gtk'), `buttons_left` (list of button names), `buttons_right` (list of button names). Requires app restart.

### Tooltips and shortcut discovery
- **D-17:** All clickable buttons show tooltips with shortcut hints on hover (e.g., "New Workspace (Ctrl+N)"). Standard GTK pattern.
- **D-18:** No status bar for inline shortcut hints. Tooltips + GtkShortcutsWindow + menu accelerator hints are sufficient for shortcut discovery.

### Claude's Discretion
- Exact icon choices for header bar buttons (symbolic icons vs text labels vs Unicode)
- PopoverMenu construction details (GMenu model vs programmatic)
- GtkShortcutsWindow section organization and grouping
- CSS styling for header bar and menu buttons (consistent with existing dark theme)
- How to handle "Open in External Browser" (xdg-open or gio)
- Find action implementation (terminal find overlay trigger from menu)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing UI implementation
- `src/sidebar.rs` — Current sidebar (build_sidebar, wire_sidebar_clicks, start_inline_rename, rebuild_sidebar_row_content)
- `src/shortcuts.rs` — Keyboard shortcut system (install_shortcuts, ShortcutAction dispatch)
- `src/config.rs` — Config loading, ShortcutMap, ShortcutAction enum, config.toml structure
- `src/main.rs` — APP_CSS styles, build_ui layout (sidebar + GtkStack), window setup
- `src/browser.rs` — Browser preview pane (create_preview_pane, nav bar buttons, PreviewPaneWidgets)

### Architecture references
- `src/app_state.rs` — AppState with sidebar_list, active_index, workspace management
- `src/split_engine.rs` — SplitNode tree, FocusDirection, pane operations

### Requirements
- `.planning/REQUIREMENTS.md` — All v1 requirements (complete), v2 requirements

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `gtk4::Button` pattern: Browser nav bar buttons (back/forward/reload/go) in `src/browser.rs` — same pattern for header bar and sidebar buttons
- `ShortcutMap` and `ShortcutAction` in `src/config.rs` — reuse for accelerator hints in menus and tooltips
- `APP_CSS` in `src/main.rs` — extend with header bar, menu, and tooltip styles
- `sidebar::rebuild_sidebar_row_content()` — modify to add hover close button

### Established Patterns
- EventControllerKey capture phase for keyboard shortcuts (src/shortcuts.rs)
- Rc<RefCell<AppState>> shared state pattern for all UI callbacks
- SURFACE_REGISTRY for pane-to-surface dispatch
- CSS class-based styling via GtkCssProvider

### Integration Points
- `main.rs::build_ui()` — add HeaderBar to ApplicationWindow
- `sidebar.rs::build_sidebar()` — add '+' button, hover close button, right-click GestureClick
- `shortcuts.rs::install_shortcuts()` — existing match arms provide handler functions to reuse from menu actions
- `config.rs::Config` — add `[ui]` section with header_bar config

</code_context>

<specifics>
## Specific Ideas

- Header bar browser button triggers existing BrowserOpen action (reuse, don't duplicate)
- Preferences opens $EDITOR on config.toml — same philosophy as Ghostty (config file is the UI)
- All menu items mirror existing keyboard shortcuts — no new functionality, just mouse-accessible paths to existing actions
- Sidebar '+' button at bottom, not top

</specifics>

<deferred>
## Deferred Ideas

- In-app settings dialog (beyond opening config.toml in $EDITOR)
- Status bar at bottom of window for shortcut hints
- Sidebar drag-to-reorder workspaces
- Traditional GtkMenuBar (only hamburger menu for now, configurable later)
- Tab bar mode as alternative to sidebar (horizontal tabs)

</deferred>

---

*Phase: 09-ui-buttons-and-menus*
*Context gathered: 2026-03-27*
