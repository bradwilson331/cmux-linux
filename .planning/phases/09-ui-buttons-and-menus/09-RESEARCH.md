# Phase 9: UI Buttons and Menus - Research

**Researched:** 2026-03-27
**Domain:** GTK4 HeaderBar, PopoverMenu, GMenu/GMenuModel, context menus, GtkShortcutsWindow, GtkAboutDialog
**Confidence:** HIGH

## Summary

Phase 9 adds mouse-accessible UI to a currently keyboard-only GTK4 Rust terminal multiplexer. The phase covers: GTK4 HeaderBar with toolbar buttons, a hamburger menu using gio::Menu + PopoverMenu, right-click context menus on sidebar rows and terminal panes, GtkShortcutsWindow for shortcut discovery, and GtkAboutDialog.

All features use standard GTK4 patterns available in gtk4-rs 0.10.3. The key architectural pattern is GIO Actions: define named actions on the ApplicationWindow, then reference them from gio::Menu models. This decouples menu items from handler code, and the same actions can be triggered by buttons, menus, and keyboard shortcuts. The existing `ShortcutAction` enum and handler functions in `shortcuts.rs` provide all the handler logic needed -- menus just need to invoke the same code paths.

**Primary recommendation:** Use GIO Actions as the central dispatch mechanism. Register one `gio::SimpleAction` per operation on the ApplicationWindow. Header bar buttons, hamburger menu items, and context menu items all reference these actions by name. This replaces zero existing code -- it layers on top of the existing EventControllerKey shortcut system.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: Simple '+' button at the bottom of the sidebar workspace list. Creates a new local workspace (same as Ctrl+N). SSH workspace creation stays keyboard-only or via menu.
- D-02: Close button ('x') appears on hover at the right edge of each sidebar workspace row. Matches terminal emulator conventions (iTerm2, Warp, Ghostty tabs).
- D-03: Right-click context menu on sidebar workspace rows with: Rename, Close, Split Right, Split Down. Quick access to per-workspace actions with accelerator hints shown.
- D-04: GTK4 HeaderBar as default, replacing the window titlebar. Configurable via config.toml to use custom toolbar or no header bar.
- D-05: Header bar buttons: New Workspace (+), Split Horizontal, Split Vertical, New Browser, Toggle Sidebar.
- D-06: Layout: Left side = workspace actions ([+New] [Browser]). Right side = pane/view actions ([Split H] [Split V] [Sidebar]).
- D-07: New Browser button triggers the existing BrowserOpen shortcut action (same as Ctrl+Shift+B).
- D-08: Terminal pane right-click menu: Copy, Paste, Split Right, Split Down, Close Pane, Open Browser Here. All items show keyboard accelerator hints.
- D-09: Browser preview pane right-click menu (adapted): Close Pane, Open in External Browser, Copy URL. No copy/paste since no text selection in preview.
- D-10: Dividers have no right-click menu -- drag-only, keep simple.
- D-11: Hamburger menu ('three lines') button in the HeaderBar opens a popover with organized sections. No traditional menu bar. Modern GTK4/GNOME convention.
- D-12: Menu sections: File (New Workspace, New SSH Workspace, New Browser, Close Pane, Close Workspace, Quit), Edit (Copy, Paste, Find, Preferences), View (Toggle Sidebar, Split Right, Split Down), Help (Keyboard Shortcuts, About).
- D-13: Preferences opens config.toml in $EDITOR (no in-app settings UI). Consistent with Ghostty's approach.
- D-14: Help > Keyboard Shortcuts opens GtkShortcutsWindow with all shortcuts grouped by category.
- D-15: Help > About opens GtkAboutDialog with app name, version, license, website link.
- D-16: Granular config under `[ui.header_bar]` with fields: `style` ('gtk' | 'custom' | 'none', default 'gtk'), `buttons_left` (list of button names), `buttons_right` (list of button names). Requires app restart.
- D-17: All clickable buttons show tooltips with shortcut hints on hover (e.g., "New Workspace (Ctrl+N)"). Standard GTK pattern.
- D-18: No status bar for inline shortcut hints. Tooltips + GtkShortcutsWindow + menu accelerator hints are sufficient for shortcut discovery.

### Claude's Discretion
- Exact icon choices for header bar buttons (symbolic icons vs text labels vs Unicode)
- PopoverMenu construction details (GMenu model vs programmatic)
- GtkShortcutsWindow section organization and grouping
- CSS styling for header bar and menu buttons (consistent with existing dark theme)
- How to handle "Open in External Browser" (xdg-open or gio)
- Find action implementation (terminal find overlay trigger from menu)

### Deferred Ideas (OUT OF SCOPE)
- In-app settings dialog (beyond opening config.toml in $EDITOR)
- Status bar at bottom of window for shortcut hints
- Sidebar drag-to-reorder workspaces
- Traditional GtkMenuBar (only hamburger menu for now, configurable later)
- Tab bar mode as alternative to sidebar (horizontal tabs)

</user_constraints>

## Project Constraints (from CLAUDE.md)

- All user-facing strings must be localized (but this is a Linux-only Rust app with no localization infrastructure yet -- use standard Rust string constants with comments marking them for future i18n)
- Test quality policy: no tests that only verify source text, method signatures, or AST fragments. Tests must verify observable runtime behavior.
- Never run tests locally -- CI only.
- Socket focus policy: non-focus commands must not steal focus.
- Typing-latency-sensitive paths must not be degraded.

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| gtk4 | 0.10.3 | All UI widgets (HeaderBar, PopoverMenu, etc.) | Already the project's UI framework |
| gio (via gtk4) | 0.21.5 | GIO Actions, Menu, MenuModel | GTK4's action/menu system |
| glib (via gtk4) | 0.21.5 | GLib types, MainContext | Already used throughout |
| toml | 0.8 | Config parsing for new `[ui]` section | Already a dependency |

### New Feature Flag
| Feature | Version Req | Purpose | Why Needed |
|---------|------------|---------|------------|
| gtk4 `v4_14` | GTK 4.14+ | `ShortcutsWindow::add_section()` programmatic API | System has GTK 4.14.5; enables building ShortcutsWindow without XML/Builder |

**No new crate dependencies required.** Everything needed is in gtk4-rs and gio.

## Architecture Patterns

### Recommended Additions to Project Structure
```
src/
  header_bar.rs       # HeaderBar construction, button wiring
  menus.rs            # GIO actions, hamburger menu, context menus
  shortcuts_window.rs # GtkShortcutsWindow builder
  sidebar.rs          # (modified) add '+' button, hover close, right-click
  config.rs           # (modified) add [ui.header_bar] config section
  main.rs             # (modified) add HeaderBar to window, register GIO actions
  shortcuts.rs        # (unchanged) existing keyboard shortcut system stays
```

### Pattern 1: GIO Actions as Central Dispatch

**What:** Define named actions (e.g., `win.new-workspace`, `win.split-right`) on the ApplicationWindow. Menu items, buttons, and shortcuts all invoke these actions by name.

**When to use:** Every menu item and toolbar button.

**Example:**
```rust
use gtk4::prelude::*;
use gtk4::gio;

fn register_actions(window: &gtk4::ApplicationWindow, state: Rc<RefCell<AppState>>) {
    // Simple action with no parameter
    let action = gio::SimpleAction::new("new-workspace", None);
    action.connect_activate({
        let state = state.clone();
        move |_, _| {
            state.borrow_mut().create_workspace();
        }
    });
    window.add_action(&action);

    // Action name is "win.new-workspace" when referenced from menus
}
```

### Pattern 2: gio::Menu for Hamburger and Context Menus

**What:** Build menu models programmatically with `gio::Menu`, attach to `PopoverMenu` or `MenuButton`.

**When to use:** Hamburger menu (D-11/D-12), sidebar context menu (D-03), terminal context menu (D-08).

**Example:**
```rust
fn build_hamburger_menu() -> gio::Menu {
    let menu = gio::Menu::new();

    // File section
    let file_section = gio::Menu::new();
    file_section.append(Some("New Workspace"), Some("win.new-workspace"));
    file_section.append(Some("New SSH Workspace"), Some("win.new-ssh-workspace"));
    file_section.append(Some("New Browser"), Some("win.browser-open"));
    file_section.append(Some("Close Pane"), Some("win.close-pane"));
    file_section.append(Some("Close Workspace"), Some("win.close-workspace"));
    file_section.append(Some("Quit"), Some("app.quit"));
    menu.append_section(Some("File"), &file_section);

    // Edit section
    let edit_section = gio::Menu::new();
    edit_section.append(Some("Copy"), Some("win.copy"));
    edit_section.append(Some("Paste"), Some("win.paste"));
    edit_section.append(Some("Find"), Some("win.find"));
    edit_section.append(Some("Preferences"), Some("win.preferences"));
    menu.append_section(Some("Edit"), &edit_section);

    // ... View, Help sections similarly
    menu
}
```

### Pattern 3: HeaderBar with MenuButton

**What:** GTK4 HeaderBar replaces the window titlebar. Contains toolbar buttons and a hamburger MenuButton.

**Example:**
```rust
fn build_header_bar(menu_model: &gio::MenuModel) -> gtk4::HeaderBar {
    let header = gtk4::HeaderBar::new();

    // Left side: workspace actions
    let new_ws_btn = gtk4::Button::from_icon_name("list-add-symbolic");
    new_ws_btn.set_tooltip_text(Some("New Workspace (Ctrl+N)"));
    new_ws_btn.set_action_name(Some("win.new-workspace"));
    header.pack_start(&new_ws_btn);

    let browser_btn = gtk4::Button::from_icon_name("web-browser-symbolic");
    browser_btn.set_tooltip_text(Some("New Browser (Ctrl+Shift+B)"));
    browser_btn.set_action_name(Some("win.browser-open"));
    header.pack_start(&browser_btn);

    // Right side: pane/view actions
    let sidebar_btn = gtk4::Button::from_icon_name("sidebar-show-symbolic");
    sidebar_btn.set_tooltip_text(Some("Toggle Sidebar (Ctrl+B)"));
    sidebar_btn.set_action_name(Some("win.toggle-sidebar"));
    header.pack_end(&sidebar_btn);

    // Hamburger menu (rightmost)
    let menu_btn = gtk4::MenuButton::new();
    menu_btn.set_icon_name("open-menu-symbolic");
    menu_btn.set_menu_model(Some(menu_model));
    header.pack_end(&menu_btn);

    header
}
```

### Pattern 4: Right-Click Context Menu on Widget

**What:** Attach a `GestureClick` listening for button 3 (secondary), create a `PopoverMenu` positioned at the click point.

**Example:**
```rust
fn attach_context_menu(widget: &impl IsA<gtk4::Widget>, menu_model: &gio::MenuModel) {
    let popover = gtk4::PopoverMenu::from_model(Some(menu_model));
    popover.set_parent(widget);
    popover.set_has_arrow(false);

    let gesture = gtk4::GestureClick::new();
    gesture.set_button(3); // Secondary button (right-click)
    gesture.connect_released({
        let popover = popover.clone();
        move |_, _, x, y| {
            popover.set_pointing_to(Some(&gtk4::gdk::Rectangle::new(
                x as i32, y as i32, 1, 1,
            )));
            popover.popup();
        }
    });
    widget.add_controller(gesture);
}
```

### Pattern 5: Sidebar Hover Close Button

**What:** Add a close button to each sidebar row that appears on hover via CSS `:hover` pseudo-class.

**Example:**
```rust
// In sidebar row construction:
let close_btn = gtk4::Button::from_icon_name("window-close-symbolic");
close_btn.add_css_class("sidebar-close-btn");
close_btn.set_tooltip_text(Some("Close Workspace"));
// Initially hidden, shown on row hover via CSS
hbox.append(&close_btn);

// CSS:
// .sidebar-close-btn { opacity: 0; transition: opacity 150ms; }
// .workspace-list row:hover .sidebar-close-btn { opacity: 1; }
```

### Pattern 6: Config Extension for UI Section

**What:** Extend `Config` struct with `[ui.header_bar]` section.

**Example:**
```rust
#[derive(serde::Deserialize, Default, Debug)]
pub struct Config {
    #[serde(default)]
    pub shortcuts: ShortcutConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(serde::Deserialize, Default, Debug)]
pub struct UiConfig {
    #[serde(default)]
    pub header_bar: HeaderBarConfig,
}

#[derive(serde::Deserialize, Debug)]
pub struct HeaderBarConfig {
    #[serde(default = "default_style")]
    pub style: String,  // "gtk" | "custom" | "none"
    pub buttons_left: Option<Vec<String>>,
    pub buttons_right: Option<Vec<String>>,
}

fn default_style() -> String { "gtk".to_string() }
```

### Anti-Patterns to Avoid
- **Don't duplicate handler logic:** Menu actions must call the same functions as keyboard shortcuts (e.g., `handle_new_workspace`, `handle_split`). Extract shared handler functions from `shortcuts.rs` into a public API that both keyboard handlers and GIO action handlers can call.
- **Don't use GtkBuilder/XML:** The project builds all UI programmatically in Rust. Continue this pattern for menus and HeaderBar. Exception: GtkShortcutsWindow sections may need Builder if `v4_14` feature is not enabled, but since GTK 4.14.5 is installed, prefer `add_section()`.
- **Don't use deprecated GtkDialog for About:** Use `GtkAboutDialog` directly (it has its own builder).
- **Don't add actions to GtkApplication for window-specific operations:** Use `window.add_action()` (prefix `win.`) for operations that need AppState. Only `app.quit` goes on the Application.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Menu models | Custom menu widget tree | `gio::Menu` + `PopoverMenu::from_model()` | GTK4's menu model system handles accelerator labels, sections, keyboard navigation automatically |
| Action dispatch | Manual match on button clicks | `gio::SimpleAction` + `set_action_name()` | Actions are the GTK4 standard; buttons/menus auto-disable when action is disabled |
| Keyboard shortcut display in menus | Manual string formatting | GTK4 automatic accelerator rendering from action accel | Set accels on actions, GTK renders them in menus |
| Context menu positioning | Manual coordinate math | `PopoverMenu::set_pointing_to()` with GdkRectangle | GTK handles screen edge clamping, arrow positioning |
| About dialog | Custom window with labels | `gtk4::AboutDialog::builder()` | Standard GNOME pattern, handles credits, license, links |
| Open in external browser | Manual process spawning | `gtk4::gio::AppInfo::launch_default_for_uri()` or `xdg-open` subprocess | Respects user's default browser setting |

## Common Pitfalls

### Pitfall 1: Action Scope Confusion
**What goes wrong:** Menu items reference `win.action-name` but action is registered on the Application, or vice versa.
**Why it happens:** GTK4 has three action scopes: `app.` (Application), `win.` (ApplicationWindow), and bare names (widget-level).
**How to avoid:** Register workspace/pane operations on the window (`win.`), app-level operations (quit) on the application (`app.`).
**Warning signs:** Menu items appear greyed out / insensitive despite being added.

### Pitfall 2: PopoverMenu Parent Lifecycle
**What goes wrong:** PopoverMenu for context menus outlives or loses its parent widget.
**Why it happens:** Context menus are created once but sidebar rows can be rebuilt (rename, session restore).
**How to avoid:** Create context menu PopoverMenu fresh per right-click, or ensure it is properly re-parented when rows are rebuilt. For sidebar rows that get rebuilt (rename), attach the GestureClick to the ListBoxRow (which persists) not the inner content (which gets replaced).
**Warning signs:** GTK critical warnings about widget hierarchy, popover appearing at wrong position.

### Pitfall 3: Accelerator Hints Not Showing in Menus
**What goes wrong:** Menu items don't show keyboard shortcut hints (e.g., "Ctrl+N" next to "New Workspace").
**Why it happens:** GTK4 shows accels in menus only if they are registered via `gtk4::Application::set_accels_for_action()`.
**How to avoid:** After registering GIO actions, call `app.set_accels_for_action("win.new-workspace", &["<Ctrl>n"])` for each action. This is separate from the EventControllerKey system.
**Warning signs:** Menu items work but show no shortcut hint text.

### Pitfall 4: HeaderBar Replacing Titlebar
**What goes wrong:** Window has both a system titlebar AND a HeaderBar, or HeaderBar doesn't show window controls.
**Why it happens:** Must use `window.set_titlebar(Some(&header_bar))` not `window.set_child()`.
**How to avoid:** Call `set_titlebar()` which tells the window manager to use the HeaderBar as the title bar. The HeaderBar automatically shows close/minimize/maximize buttons.
**Warning signs:** Double title bars, missing window controls.

### Pitfall 5: Sidebar '+' Button Scrolling
**What goes wrong:** The '+' button at the bottom of the sidebar scrolls out of view with the workspace list.
**Why it happens:** Button is inside the ScrolledWindow.
**How to avoid:** Place the '+' button OUTSIDE the ScrolledWindow, below it in a vertical Box. The sidebar becomes: `Box(V) > [ScrolledWindow(ListBox), Button(+)]`.
**Warning signs:** Button disappears when list is long enough to scroll.

### Pitfall 6: GIO Actions and Rc<RefCell<AppState>>
**What goes wrong:** Borrow panics when a GIO action callback borrows AppState while it's already borrowed.
**Why it happens:** GIO action callbacks fire synchronously during GTK event processing. If another callback already holds a borrow, RefCell panics.
**How to avoid:** Keep borrows short. Borrow, extract needed data, drop borrow, then do GTK operations. Same pattern used throughout existing codebase.
**Warning signs:** `BorrowMutError` panics at runtime.

### Pitfall 7: Hover Close Button Stealing Row Clicks
**What goes wrong:** Clicking the close button also triggers the row's `row-activated` signal, switching to the workspace before closing it.
**Why it happens:** Click events propagate from button up to the ListBoxRow.
**How to avoid:** The close button click handler should call `propagation_stop()` or use `set_propagation_phase(Capture)`. Alternatively, connect to `clicked` signal on the Button (which doesn't propagate to the row activation).
**Warning signs:** Workspace switches briefly before closing.

## Code Examples

### Registering GIO Actions That Call Existing Handlers
```rust
// Source: Derived from existing shortcuts.rs handler pattern
fn register_window_actions(
    window: &gtk4::ApplicationWindow,
    state: Rc<RefCell<AppState>>,
    sidebar: &gtk4::ScrolledWindow,
    app: &gtk4::Application,
) {
    // New Workspace
    let action = gio::SimpleAction::new("new-workspace", None);
    action.connect_activate({
        let state = state.clone();
        let app = app.clone();
        move |_, _| {
            state.borrow_mut().create_workspace();
        }
    });
    window.add_action(&action);

    // Split Right
    let action = gio::SimpleAction::new("split-right", None);
    action.connect_activate({
        let state = state.clone();
        move |_, _| {
            let mut s = state.borrow_mut();
            if let Some(engine) = s.active_split_engine_mut() {
                engine.split_right();
            }
        }
    });
    window.add_action(&action);

    // Preferences: open config.toml in $EDITOR
    let action = gio::SimpleAction::new("preferences", None);
    action.connect_activate(move |_, _| {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "xdg-open".to_string());
        let config_path = crate::config::config_path();
        // Ensure config directory and file exist
        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if !config_path.exists() {
            let _ = std::fs::write(&config_path, "# cmux configuration\n# See documentation for options\n");
        }
        let _ = std::process::Command::new(&editor)
            .arg(&config_path)
            .spawn();
    });
    window.add_action(&action);
}
```

### Building GtkShortcutsWindow Programmatically (with v4_14 feature)
```rust
fn build_shortcuts_window() -> gtk4::ShortcutsWindow {
    let window = gtk4::ShortcutsWindow::builder().build();

    let section = gtk4::ShortcutsSection::builder()
        .section_name("shortcuts")
        .title("Keyboard Shortcuts")
        .build();

    // Workspace group
    let ws_group = gtk4::ShortcutsGroup::builder()
        .title("Workspaces")
        .build();
    ws_group.add_shortcut(&gtk4::ShortcutsShortcut::builder()
        .shortcut_type(gtk4::ShortcutType::Accelerator)
        .accelerator("<Ctrl>n")
        .title("New Workspace")
        .build());
    ws_group.add_shortcut(&gtk4::ShortcutsShortcut::builder()
        .shortcut_type(gtk4::ShortcutType::Accelerator)
        .accelerator("<Ctrl><Shift>w")
        .title("Close Workspace")
        .build());
    section.add_group(&ws_group);

    // Panes group
    let pane_group = gtk4::ShortcutsGroup::builder()
        .title("Panes")
        .build();
    pane_group.add_shortcut(&gtk4::ShortcutsShortcut::builder()
        .shortcut_type(gtk4::ShortcutType::Accelerator)
        .accelerator("<Ctrl>d")
        .title("Split Right")
        .build());
    section.add_group(&pane_group);

    window.add_section(&section);
    window
}
```

### Sidebar Row with Hover Close Button
```rust
fn build_sidebar_row(name: &str, workspace_id: u64) -> gtk4::ListBoxRow {
    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let label = gtk4::Label::new(Some(name));
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);
    vbox.append(&label);
    vbox.set_hexpand(true);
    hbox.append(&vbox);

    // Attention dot (existing)
    let dot = gtk4::Label::new(None);
    dot.add_css_class("attention-dot");
    dot.set_visible(false);
    hbox.append(&dot);

    // Close button (new, hover-visible via CSS)
    let close_btn = gtk4::Button::from_icon_name("window-close-symbolic");
    close_btn.add_css_class("sidebar-close-btn");
    close_btn.set_tooltip_text(Some("Close Workspace"));
    hbox.append(&close_btn);

    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&hbox));
    row
}
```

## Discretion Recommendations

### Icon Choices (Claude's Discretion)
Use GNOME symbolic icons (available in all GTK4 installations):
- New Workspace: `list-add-symbolic`
- Browser: `web-browser-symbolic`
- Split Horizontal: `view-dual-symbolic` or `object-flip-horizontal-symbolic`
- Split Vertical: `view-dual-symbolic` rotated, or `object-flip-vertical-symbolic`
- Toggle Sidebar: `sidebar-show-symbolic`
- Hamburger: `open-menu-symbolic`
- Close (sidebar row): `window-close-symbolic`

Fallback: If symbolic icons are not available on the target system, use Unicode text labels as fallback ("+", "x", etc.).

### PopoverMenu Construction (Claude's Discretion)
**Recommendation:** Use programmatic `gio::Menu` construction (not GtkBuilder XML). Consistent with the project's all-programmatic-Rust pattern. XML would be the only non-Rust UI definition in the codebase.

### Open in External Browser (Claude's Discretion)
**Recommendation:** Use `xdg-open` subprocess. Simpler than `gio::AppInfo::launch_default_for_uri()` which requires async handling. `xdg-open` is universally available on Linux desktops.

```rust
let _ = std::process::Command::new("xdg-open")
    .arg(&url)
    .spawn();
```

### Find Action (Claude's Discretion)
**Recommendation:** For now, the Find action in the menu should be a no-op or disabled. Terminal find overlay is not yet implemented (it would need Ghostty's find API). Mark the action as insensitive with a tooltip "Not yet implemented" or omit it from the initial implementation. Add it as a stub action that can be wired later.

### CSS Styling (Claude's Discretion)
Extend APP_CSS in main.rs with header bar and menu button styles:
```css
/* Phase 9: Header bar */
headerbar { background-color: #242424; }
headerbar button { color: #cccccc; }
headerbar button:hover { background-color: rgba(255, 255, 255, 0.08); }
/* Phase 9: Sidebar close button (hover-reveal) */
.sidebar-close-btn { opacity: 0; transition: opacity 150ms; min-width: 20px; min-height: 20px; padding: 0; margin: 0; }
.workspace-list row:hover .sidebar-close-btn { opacity: 1; }
/* Phase 9: Sidebar '+' button */
.sidebar-add-btn { margin: 4px 8px; }
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| GtkMenuBar | PopoverMenu from GMenuModel | GTK4 (2020) | No traditional menu bar; use HeaderBar + MenuButton |
| GtkAction | gio::SimpleAction | GTK3.10+ | All actions go through GIO action system |
| Manual accel labels | `set_accels_for_action()` | GTK4 | Automatic accelerator display in menus |
| GtkShortcutsWindow (XML only) | `add_section()` programmatic | GTK 4.14 | Can build ShortcutsWindow in pure Rust code |

## Open Questions

1. **ShortcutsWindow Shortcut Strings**
   - What we know: The ShortcutsShortcut `accelerator` property takes GTK accelerator format strings.
   - What's unclear: Whether the project's existing custom shortcut overrides (from config.toml) should be reflected in the ShortcutsWindow, or just show defaults.
   - Recommendation: Show the actual configured shortcuts by reading from ShortcutMap. If user has remapped Ctrl+N to Ctrl+T, the ShortcutsWindow should show Ctrl+T.

2. **Copy/Paste Actions for Terminal**
   - What we know: Ghostty handles its own clipboard operations via `read_clipboard_cb`/`write_clipboard_cb`.
   - What's unclear: Whether we can trigger Ghostty's copy/paste from a GIO action, or need to use the clipboard API directly.
   - Recommendation: For Copy, read selection from Ghostty surface clipboard. For Paste, write system clipboard to Ghostty surface via `ghostty_surface_text`. Check existing clipboard callback flow.

3. **Header Bar Config "custom" Mode**
   - What we know: D-16 specifies `buttons_left` and `buttons_right` lists for custom mode.
   - What's unclear: Exact UX of "custom" mode vs "gtk" mode.
   - Recommendation: Implement "gtk" (default with all buttons) and "none" (no header bar) first. "custom" mode can be a stretch goal that reads button names from config and only adds those buttons.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| GTK4 | All UI | Yes | 4.14.5 | -- |
| GNOME symbolic icons | Header bar buttons | Yes (via gtk4) | -- | Unicode text labels |
| xdg-open | Open in External Browser (D-09) | Yes (standard on Linux desktops) | -- | gio::AppInfo |
| $EDITOR | Preferences (D-13) | User-dependent | -- | xdg-open on config.toml |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust unit tests) |
| Config file | Cargo.toml |
| Quick run command | `cargo test --bin cmux-linux` |
| Full suite command | `cargo test --bin cmux-linux` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-04 | HeaderBar config parsing (style, buttons) | unit | `cargo test --bin cmux-linux config::tests` | Wave 0 |
| D-16 | Config `[ui.header_bar]` deserialization | unit | `cargo test --bin cmux-linux config::tests` | Wave 0 |
| D-01-D-18 | UI widget construction and behavior | manual-only | Visual verification | N/A (GTK4 requires display) |

### Sampling Rate
- **Per task commit:** `cargo test --bin cmux-linux`
- **Per wave merge:** `cargo test --bin cmux-linux` + `cargo clippy`
- **Phase gate:** Full suite green, visual verification of all UI elements

### Wave 0 Gaps
- [ ] Config tests for `[ui.header_bar]` deserialization (new fields)
- [ ] Config tests for default values and edge cases

## Sources

### Primary (HIGH confidence)
- gtk4-rs stable docs: [HeaderBar](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.HeaderBar.html) - pack_start/pack_end/set_titlebar API
- gtk4-rs stable docs: [PopoverMenu](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.PopoverMenu.html) - from_model, set_pointing_to
- gtk4-rs stable docs: [ShortcutsWindow](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.ShortcutsWindow.html) - add_section (v4_14+)
- gio docs: [Menu](https://gtk-rs.org/gtk-rs-core/stable/latest/docs/gio/struct.Menu.html) - append, append_section, append_submenu
- gtk-rs book: [Actions](https://gtk-rs.org/gtk4-rs/stable/latest/book/actions.html) - ActionEntry, SimpleAction, set_accels_for_action
- gtk4-rs docs: [AboutDialog](https://gtk-rs.org/gtk4-rs/git/docs/gtk4/struct.AboutDialog.html) - builder pattern

### Secondary (MEDIUM confidence)
- GNOME Discourse: [Context menu on ListView](https://discourse.gnome.org/t/adding-a-context-menu-to-a-listview-using-gtk4-rs/19995) - GestureClick button 3 + PopoverMenu pattern
- GTK docs: [ShortcutsWindow.add_section](https://docs.gtk.org/gtk4/method.ShortcutsWindow.add_section.html) - available since 4.14

### Tertiary (LOW confidence)
- None -- all patterns verified against official documentation.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all libraries already in use, only adding a feature flag
- Architecture: HIGH - GIO Actions + gio::Menu is the canonical GTK4 pattern
- Pitfalls: HIGH - derived from official docs and known GTK4 behavior
- Code examples: MEDIUM - patterns are standard but exact API signatures verified against docs, not compiled

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable GTK4 APIs, no rapid changes expected)
