use gtk4::prelude::*;
use gtk4::gio;
use std::cell::RefCell;
use std::rc::Rc;

/// Register all GIO actions on the ApplicationWindow.
/// Actions are named "win.{action-name}" and can be invoked by buttons, menus, and shortcuts.
/// Per D-11, D-12: all menu items mirror existing keyboard shortcut actions.
pub fn register_actions(
    window: &gtk4::ApplicationWindow,
    state: Rc<RefCell<crate::app_state::AppState>>,
    sidebar: &gtk4::Box,
    app: &gtk4::Application,
) {
    // --- File section actions ---

    // win.new-workspace (D-01, D-05)
    let action = gio::SimpleAction::new("new-workspace", None);
    action.connect_activate({
        let state = state.clone();
        let app = app.clone();
        move |_, _| {
            crate::shortcuts::handle_new_workspace(&state, &app);
        }
    });
    window.add_action(&action);

    // win.new-ssh-workspace
    let action = gio::SimpleAction::new("new-ssh-workspace", None);
    action.connect_activate({
        let state = state.clone();
        let app = app.clone();
        move |_, _| {
            crate::shortcuts::handle_new_ssh_workspace(&state, &app);
        }
    });
    window.add_action(&action);

    // win.browser-open (D-07)
    let action = gio::SimpleAction::new("browser-open", None);
    action.connect_activate({
        let state = state.clone();
        move |_, _| {
            crate::shortcuts::handle_browser_open(&state);
        }
    });
    window.add_action(&action);

    // win.close-pane
    let action = gio::SimpleAction::new("close-pane", None);
    action.connect_activate({
        let state = state.clone();
        let app = app.clone();
        move |_, _| {
            crate::shortcuts::handle_close_pane(&state, &app);
        }
    });
    window.add_action(&action);

    // win.close-workspace
    let action = gio::SimpleAction::new("close-workspace", None);
    action.connect_activate({
        let state = state.clone();
        let app = app.clone();
        move |_, _| {
            crate::shortcuts::handle_close_workspace(&state, &app);
        }
    });
    window.add_action(&action);

    // --- Edit section actions ---

    // win.copy (Ctrl+Shift+C) -- invoke Ghostty's copy_to_clipboard binding action
    let action = gio::SimpleAction::new("copy", None);
    action.connect_activate({
        let state = state.clone();
        move |_, _| {
            let s = state.borrow();
            if let Some(engine) = s.active_split_engine() {
                if let Some(pane_id) = engine.root.find_active_pane_id() {
                    if let Some(surface) = engine.root.find_surface_for_pane(pane_id) {
                        let action_str = b"copy_to_clipboard";
                        unsafe {
                            crate::ghostty::ffi::ghostty_surface_binding_action(
                                surface,
                                action_str.as_ptr() as *const _,
                                action_str.len(),
                            );
                        }
                    }
                }
            }
        }
    });
    window.add_action(&action);

    // win.paste (Ctrl+Shift+V) -- invoke Ghostty's paste_from_clipboard binding action
    let action = gio::SimpleAction::new("paste", None);
    action.connect_activate({
        let state = state.clone();
        move |_, _| {
            let s = state.borrow();
            if let Some(engine) = s.active_split_engine() {
                if let Some(pane_id) = engine.root.find_active_pane_id() {
                    if let Some(surface) = engine.root.find_surface_for_pane(pane_id) {
                        let action_str = b"paste_from_clipboard";
                        unsafe {
                            crate::ghostty::ffi::ghostty_surface_binding_action(
                                surface,
                                action_str.as_ptr() as *const _,
                                action_str.len(),
                            );
                        }
                    }
                }
            }
        }
    });
    window.add_action(&action);

    // win.find -- stub for now (terminal find not yet implemented)
    let action = gio::SimpleAction::new("find", None);
    action.set_enabled(false);
    window.add_action(&action);

    // win.preferences (D-13) -- open config.toml in $EDITOR
    let action = gio::SimpleAction::new("preferences", None);
    action.connect_activate(move |_, _| {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "xdg-open".to_string());
        let config_path = crate::config::config_path();
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

    // --- View section actions ---

    // win.toggle-sidebar
    let action = gio::SimpleAction::new("toggle-sidebar", None);
    action.connect_activate({
        let state = state.clone();
        let sidebar = sidebar.clone();
        move |_, _| {
            let visible = sidebar.is_visible();
            sidebar.set_visible(!visible);
            if let Some(engine) = state.borrow_mut().active_split_engine_mut() {
                engine.focus_active_surface();
            }
        }
    });
    window.add_action(&action);

    // win.split-right
    let action = gio::SimpleAction::new("split-right", None);
    action.connect_activate({
        let state = state.clone();
        move |_, _| {
            crate::shortcuts::handle_split(&state, false);
        }
    });
    window.add_action(&action);

    // win.split-down
    let action = gio::SimpleAction::new("split-down", None);
    action.connect_activate({
        let state = state.clone();
        move |_, _| {
            crate::shortcuts::handle_split(&state, true);
        }
    });
    window.add_action(&action);

    // win.rename-workspace
    let action = gio::SimpleAction::new("rename-workspace", None);
    action.connect_activate({
        let state = state.clone();
        move |_, _| {
            let (active_index, sidebar_list) = {
                let s = state.borrow();
                (s.active_index, s.sidebar_list.clone())
            };
            crate::sidebar::start_inline_rename(&sidebar_list, active_index, state.clone());
        }
    });
    window.add_action(&action);

    // --- Help section actions ---

    // win.keyboard-shortcuts (D-14)
    let action = gio::SimpleAction::new("keyboard-shortcuts", None);
    action.connect_activate({
        let window_weak = window.downgrade();
        move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                let sw = build_shortcuts_window();
                sw.set_transient_for(Some(&win));
                sw.present();
            }
        }
    });
    window.add_action(&action);

    // win.about (D-15)
    let action = gio::SimpleAction::new("about", None);
    action.connect_activate({
        let window_weak = window.downgrade();
        move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                let about = gtk4::AboutDialog::builder()
                    .program_name("cmux")
                    .version(env!("CARGO_PKG_VERSION"))
                    .comments("GPU-accelerated terminal multiplexer for Linux")
                    .website("https://github.com/manaflow-ai/cmux")
                    .license_type(gtk4::License::MitX11)
                    .transient_for(&win)
                    .modal(true)
                    .build();
                about.present();
            }
        }
    });
    window.add_action(&action);

    // app.quit
    let quit_action = gio::SimpleAction::new("quit", None);
    quit_action.connect_activate({
        let app = app.clone();
        move |_, _| {
            app.quit();
        }
    });
    app.add_action(&quit_action);

    // --- Browser-specific actions (D-09) ---

    // win.open-external-browser -- opens current browser pane URL in xdg-open
    // Disabled until BrowserManager exposes current_url() (wired in Plan 03)
    let action = gio::SimpleAction::new("open-external-browser", None);
    action.set_enabled(false); // TODO: enable when BrowserManager.current_url() is available
    window.add_action(&action);

    // win.copy-url -- copies current browser pane URL to clipboard
    // Disabled until BrowserManager exposes current_url() (wired in Plan 03)
    let action = gio::SimpleAction::new("copy-url", None);
    action.set_enabled(false); // TODO: enable when BrowserManager.current_url() is available
    window.add_action(&action);
}

/// Register keyboard accelerators for GIO actions so menus show shortcut hints.
/// Per Pitfall 3 from RESEARCH.md: GTK4 shows accels in menus ONLY if registered via set_accels_for_action.
pub fn register_accels(app: &gtk4::Application) {
    app.set_accels_for_action("win.new-workspace", &["<Ctrl>n"]);
    app.set_accels_for_action("win.close-workspace", &["<Ctrl><Shift>w"]);
    app.set_accels_for_action("win.new-ssh-workspace", &["<Ctrl><Shift>s"]);
    app.set_accels_for_action("win.browser-open", &["<Ctrl><Shift>b"]);
    app.set_accels_for_action("win.close-pane", &["<Ctrl><Shift>x"]);
    app.set_accels_for_action("win.toggle-sidebar", &["<Ctrl>b"]);
    app.set_accels_for_action("win.split-right", &["<Ctrl>d"]);
    app.set_accels_for_action("win.split-down", &["<Ctrl><Shift>d"]);
    app.set_accels_for_action("win.copy", &["<Ctrl><Shift>c"]);
    app.set_accels_for_action("win.paste", &["<Ctrl><Shift>v"]);
    app.set_accels_for_action("win.find", &["<Ctrl>f"]);
    app.set_accels_for_action("win.rename-workspace", &["<Ctrl><Shift>r"]);
    app.set_accels_for_action("app.quit", &["<Ctrl>q"]);
}

/// Build the hamburger menu model (D-11, D-12).
/// Returns a gio::Menu that can be set on a MenuButton.
/// Per D-12: sections use File/Edit/View/Help labels.
pub fn build_hamburger_menu() -> gio::Menu {
    let menu = gio::Menu::new();

    // File section (D-12)
    let file_section = gio::Menu::new();
    file_section.append(Some("New Workspace"), Some("win.new-workspace"));
    file_section.append(Some("New SSH Workspace"), Some("win.new-ssh-workspace"));
    file_section.append(Some("New Browser"), Some("win.browser-open"));
    file_section.append(Some("Close Pane"), Some("win.close-pane"));
    file_section.append(Some("Close Workspace"), Some("win.close-workspace"));
    file_section.append(Some("Quit"), Some("app.quit"));
    menu.append_section(Some("File"), &file_section);

    // Edit section (D-12)
    let edit_section = gio::Menu::new();
    edit_section.append(Some("Copy"), Some("win.copy"));
    edit_section.append(Some("Paste"), Some("win.paste"));
    edit_section.append(Some("Find"), Some("win.find"));
    edit_section.append(Some("Preferences"), Some("win.preferences"));
    menu.append_section(Some("Edit"), &edit_section);

    // View section (D-12)
    let view_section = gio::Menu::new();
    view_section.append(Some("Toggle Sidebar"), Some("win.toggle-sidebar"));
    view_section.append(Some("Split Right"), Some("win.split-right"));
    view_section.append(Some("Split Down"), Some("win.split-down"));
    menu.append_section(Some("View"), &view_section);

    // Help section (D-12)
    let help_section = gio::Menu::new();
    help_section.append(Some("Keyboard Shortcuts"), Some("win.keyboard-shortcuts"));
    help_section.append(Some("About cmux"), Some("win.about"));
    menu.append_section(Some("Help"), &help_section);

    menu
}

/// Build sidebar workspace row context menu (D-03).
pub fn build_sidebar_context_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("Rename"), Some("win.rename-workspace"));
    menu.append(Some("Close"), Some("win.close-workspace"));
    menu.append(Some("Split Right"), Some("win.split-right"));
    menu.append(Some("Split Down"), Some("win.split-down"));
    menu
}

/// Build terminal pane context menu (D-08).
pub fn build_terminal_context_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    let edit_section = gio::Menu::new();
    edit_section.append(Some("Copy"), Some("win.copy"));
    edit_section.append(Some("Paste"), Some("win.paste"));
    menu.append_section(None, &edit_section);

    let pane_section = gio::Menu::new();
    pane_section.append(Some("Split Right"), Some("win.split-right"));
    pane_section.append(Some("Split Down"), Some("win.split-down"));
    pane_section.append(Some("Close Pane"), Some("win.close-pane"));
    menu.append_section(None, &pane_section);

    let browser_section = gio::Menu::new();
    browser_section.append(Some("Open Browser Here"), Some("win.browser-open"));
    menu.append_section(None, &browser_section);

    menu
}

/// Build browser preview pane context menu (D-09).
/// Includes Open in External Browser and Copy URL actions.
pub fn build_browser_context_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("Open in External Browser"), Some("win.open-external-browser"));
    menu.append(Some("Copy URL"), Some("win.copy-url"));
    menu.append(Some("Close Pane"), Some("win.close-pane"));
    menu
}

/// Build GtkShortcutsWindow (D-14) with all shortcuts grouped by category.
/// Uses GTK 4.14 add_section/add_group API.
fn build_shortcuts_window() -> gtk4::ShortcutsWindow {
    let window = gtk4::ShortcutsWindow::builder().build();

    // Workspaces section
    let ws_section = gtk4::ShortcutsSection::builder()
        .section_name("workspaces")
        .title("Workspaces")
        .build();

    let ws_group = gtk4::ShortcutsGroup::builder()
        .title("Workspaces")
        .build();
    ws_group.add_shortcut(&shortcut("<Ctrl>n", "New Workspace"));
    ws_group.add_shortcut(&shortcut("<Ctrl><Shift>w", "Close Workspace"));
    ws_group.add_shortcut(&shortcut("<Ctrl>bracketright", "Next Workspace"));
    ws_group.add_shortcut(&shortcut("<Ctrl>bracketleft", "Previous Workspace"));
    ws_group.add_shortcut(&shortcut("<Ctrl><Shift>r", "Rename Workspace"));
    ws_group.add_shortcut(&shortcut("<Ctrl>1..9", "Switch to Workspace 1-9"));
    ws_section.add_group(&ws_group);
    window.add_section(&ws_section);

    // Panes section
    let pane_section = gtk4::ShortcutsSection::builder()
        .section_name("panes")
        .title("Panes")
        .build();

    let pane_group = gtk4::ShortcutsGroup::builder()
        .title("Panes")
        .build();
    pane_group.add_shortcut(&shortcut("<Ctrl>d", "Split Right"));
    pane_group.add_shortcut(&shortcut("<Ctrl><Shift>d", "Split Down"));
    pane_group.add_shortcut(&shortcut("<Ctrl><Shift>x", "Close Pane"));
    pane_group.add_shortcut(&shortcut("<Ctrl><Shift>Left", "Focus Left"));
    pane_group.add_shortcut(&shortcut("<Ctrl><Shift>Right", "Focus Right"));
    pane_group.add_shortcut(&shortcut("<Ctrl><Shift>Up", "Focus Up"));
    pane_group.add_shortcut(&shortcut("<Ctrl><Shift>Down", "Focus Down"));
    pane_section.add_group(&pane_group);
    window.add_section(&pane_section);

    // Edit section
    let edit_section = gtk4::ShortcutsSection::builder()
        .section_name("edit")
        .title("Edit")
        .build();

    let edit_group = gtk4::ShortcutsGroup::builder()
        .title("Edit")
        .build();
    edit_group.add_shortcut(&shortcut("<Ctrl><Shift>c", "Copy"));
    edit_group.add_shortcut(&shortcut("<Ctrl><Shift>v", "Paste"));
    edit_group.add_shortcut(&shortcut("<Ctrl>f", "Find"));
    edit_section.add_group(&edit_group);
    window.add_section(&edit_section);

    // View section
    let view_section = gtk4::ShortcutsSection::builder()
        .section_name("view")
        .title("View")
        .build();

    let view_group = gtk4::ShortcutsGroup::builder()
        .title("View")
        .build();
    view_group.add_shortcut(&shortcut("<Ctrl>b", "Toggle Sidebar"));
    view_group.add_shortcut(&shortcut("<Ctrl><Shift>b", "Open Browser"));
    view_group.add_shortcut(&shortcut("<Ctrl><Shift>s", "New SSH Workspace"));
    view_section.add_group(&view_group);
    window.add_section(&view_section);

    // General section
    let general_section = gtk4::ShortcutsSection::builder()
        .section_name("general")
        .title("General")
        .build();

    let general_group = gtk4::ShortcutsGroup::builder()
        .title("General")
        .build();
    general_group.add_shortcut(&shortcut("<Ctrl>q", "Quit"));
    general_section.add_group(&general_group);
    window.add_section(&general_section);

    window
}

/// Helper to create a ShortcutsShortcut widget.
fn shortcut(accel: &str, title: &str) -> gtk4::ShortcutsShortcut {
    gtk4::ShortcutsShortcut::builder()
        .accelerator(accel)
        .title(title)
        .build()
}
