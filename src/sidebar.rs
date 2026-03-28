use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Build the sidebar widget: outer Box(V) > [ScrolledWindow(ListBox), Button(+)].
/// Returns (sidebar_box, scrolled_window, list_box).
///
/// Per Pitfall 5 from RESEARCH.md: the '+' button is OUTSIDE the ScrolledWindow
/// so it doesn't scroll away.
///
/// Per UI-SPEC:
/// - Width: 160px (set_size_request(160, -1))
/// - Background: #242424 (applied via global CssProvider in main.rs)
/// - Row height: 36px min-height (CSS)
/// - Row padding: 8px top/bottom, 16px left/right
/// - Active row: #5b8dd9 background, #ffffff text, font-weight 600
/// - Inactive row: transparent bg, #cccccc text, font-weight 400
/// - Hover (inactive): #2e2e2e
pub fn build_sidebar() -> (gtk4::Box, gtk4::ScrolledWindow, gtk4::ListBox) {
    let list_box = gtk4::ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    list_box.add_css_class("workspace-list");

    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_size_request(160, -1);
    scrolled.set_hscrollbar_policy(gtk4::PolicyType::Never);
    scrolled.set_vscrollbar_policy(gtk4::PolicyType::Automatic);
    scrolled.set_child(Some(&list_box));
    scrolled.set_vexpand(true);

    // Sidebar container: Box(V) > [ScrolledWindow(ListBox), Button(+)]
    let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sidebar_box.add_css_class("sidebar");
    sidebar_box.append(&scrolled);

    // '+' button at the bottom (D-01)
    let add_btn = gtk4::Button::with_label("+");
    add_btn.add_css_class("sidebar-add-btn");
    add_btn.set_tooltip_text(Some("New Workspace (Ctrl+N)"));
    add_btn.set_action_name(Some("win.new-workspace"));
    sidebar_box.append(&add_btn);

    (sidebar_box, scrolled, list_box)
}

/// Wire sidebar click-to-switch. Called from main.rs after AppState is constructed.
/// Per WS-03: clicking a row calls AppState.switch_to_index.
pub fn wire_sidebar_clicks(
    list_box: &gtk4::ListBox,
    state: Rc<RefCell<crate::app_state::AppState>>,
) {
    list_box.connect_row_activated({
        let state = state.clone();
        move |_list, row| {
            let index = row.index() as usize;
            state.borrow_mut().switch_to_index(index);
            // SPLIT-07: call ghostty_surface_set_focus on the newly active pane.
            // Workspace switches are focus changes — must call set_focus after switch.
            let surface = {
                let mut s = state.borrow_mut();
                s.active_split_engine_mut()
                    .and_then(|engine| engine.root.find_active_pane_id())
                    .and_then(|pane_id| {
                        if let Ok(reg) = crate::ghostty::callbacks::SURFACE_REGISTRY.lock() {
                            reg.iter()
                                .find(|(_, &pid)| pid == pane_id)
                                .map(|(&ptr, _)| ptr as crate::ghostty::ffi::ghostty_surface_t)
                        } else {
                            None
                        }
                    })
            };
            if let Some(surface) = surface {
                unsafe {
                    crate::ghostty::ffi::ghostty_surface_set_focus(surface, true);
                }
            }
        }
    });
}

/// Start inline rename for the active workspace row.
/// Per UI-SPEC: replaces GtkLabel with GtkEntry; Enter commits, Escape cancels.
/// Per D-03: rename triggered by Ctrl+Shift+R (keyboard only).
pub fn start_inline_rename(
    list_box: &gtk4::ListBox,
    active_index: usize,
    state: Rc<RefCell<crate::app_state::AppState>>,
) {
    let row = match list_box.row_at_index(active_index as i32) {
        Some(r) => r,
        None => return,
    };

    // Get current name from the label (Phase 4 nested layout: row > hbox > vbox > label).
    let current_name = row
        .child()
        .and_downcast::<gtk4::Box>()
        .and_then(|hbox| hbox.first_child())
        .and_downcast::<gtk4::Box>()
        .and_then(|vbox| vbox.first_child())
        .and_downcast::<gtk4::Label>()
        .map(|l| l.text().to_string())
        .unwrap_or_default();

    // Replace label with entry.
    let entry = gtk4::Entry::new();
    entry.set_text(&current_name);
    entry.set_placeholder_text(Some("Workspace name"));
    entry.add_css_class("rename-entry");
    row.set_child(Some(&entry));
    entry.grab_focus();

    // Enter key: commit rename.
    entry.connect_activate({
        let state = state.clone();
        let row = row.clone();
        move |e| {
            let new_name = e.text().to_string();
            let trimmed = new_name.trim().to_string();
            if !trimmed.is_empty() {
                state.borrow_mut().rename_active(trimmed.clone());
            }
            // Restore Phase 4 nested layout: hbox > [vbox > [label], dot, close_btn]
            let display = if trimmed.is_empty() { &new_name } else { &trimmed };
            row.set_child(Some(&rebuild_sidebar_row_content(display)));
        }
    });

    // Focus-out: commit rename (same as Enter).
    entry.connect_notify_local(Some("has-focus"), {
        let state = state.clone();
        let row_clone = row.clone();
        move |e, _| {
            if !e.has_focus() && e.parent().is_some() {
                let new_name = e.text().to_string();
                let trimmed = new_name.trim().to_string();
                if !trimmed.is_empty() {
                    state.borrow_mut().rename_active(trimmed.clone());
                }
                let display = if trimmed.is_empty() { &new_name } else { &trimmed };
                row_clone.set_child(Some(&rebuild_sidebar_row_content(display)));
            }
        }
    });

    // Escape key: cancel rename and restore original label.
    let key_ctrl = gtk4::EventControllerKey::new();
    key_ctrl.connect_key_pressed({
        let row_clone = row.clone();
        let current_name_clone = current_name.clone();
        move |_, keyval, _, _| {
            if keyval == gtk4::gdk::Key::Escape {
                row_clone.set_child(Some(&rebuild_sidebar_row_content(&current_name_clone)));
                gtk4::glib::Propagation::Stop
            } else {
                gtk4::glib::Propagation::Proceed
            }
        }
    });
    entry.add_controller(key_ctrl);
}

/// Rebuild the Phase 4 sidebar row content:
/// GtkBox(H, 4) > [GtkBox(V, 0) > [GtkLabel(name)], GtkLabel(dot), Button(close)].
/// Dot is hidden by default (fresh state after rename).
/// Close button is hidden by default, shown on row hover via CSS (D-02).
pub fn rebuild_sidebar_row_content(name: &str) -> gtk4::Box {
    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let label = gtk4::Label::new(Some(name));
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);
    vbox.append(&label);
    vbox.set_hexpand(true);
    hbox.append(&vbox);

    let dot = gtk4::Label::new(None);
    dot.add_css_class("attention-dot");
    dot.set_visible(false);
    hbox.append(&dot);

    // Close button (D-02) -- hidden by default, shown on row hover via CSS
    let close_btn = gtk4::Button::with_label("\u{00D7}"); // Unicode multiplication sign
    close_btn.add_css_class("sidebar-close-btn");
    close_btn.set_tooltip_text(Some("Close Workspace"));
    hbox.append(&close_btn);

    hbox
}

/// Wire the close button for a specific sidebar row.
/// Called when a row is created (in app_state::create_workspace or after rename rebuild).
pub fn wire_row_close_button(
    row: &gtk4::ListBoxRow,
    state: Rc<RefCell<crate::app_state::AppState>>,
    app: &gtk4::Application,
) {
    let close_btn = row
        .child()
        .and_downcast::<gtk4::Box>()
        .and_then(|hbox| hbox.last_child())
        .and_downcast::<gtk4::Button>();

    if let Some(btn) = close_btn {
        btn.connect_clicked({
            let state = state.clone();
            let app = app.clone();
            let row = row.clone();
            move |_| {
                let index = row.index() as usize;
                let ws_count = state.borrow().workspaces.len();
                if ws_count <= 1 {
                    return; // Cannot close last workspace
                }
                // Switch to this workspace first (so close_workspace operates on the right one)
                state.borrow_mut().switch_to_index(index);
                crate::shortcuts::handle_close_workspace(&state, &app);
            }
        });
    }
}

/// Attach right-click context menu to a sidebar row (D-03).
pub fn attach_sidebar_context_menu(
    row: &gtk4::ListBoxRow,
    state: Rc<RefCell<crate::app_state::AppState>>,
) {
    let menu_model = crate::menus::build_sidebar_context_menu();
    let popover = gtk4::PopoverMenu::from_model(Some(&menu_model));
    popover.set_parent(row);
    popover.set_has_arrow(false);

    let gesture = gtk4::GestureClick::new();
    gesture.set_button(3); // Right-click
    gesture.connect_released({
        let popover = popover.clone();
        let state = state.clone();
        let row = row.clone();
        move |_, _, x, y| {
            // Switch to this workspace first so context menu actions apply to it
            let index = row.index() as usize;
            state.borrow_mut().switch_to_index(index);
            popover.set_pointing_to(Some(&gtk4::gdk::Rectangle::new(
                x as i32, y as i32, 1, 1,
            )));
            popover.popup();
        }
    });
    row.add_controller(gesture);
}

/// Wire close button + context menu to the most recently added sidebar row.
pub fn wire_latest_row(
    sidebar_list: &gtk4::ListBox,
    state: Rc<RefCell<crate::app_state::AppState>>,
    app: &gtk4::Application,
) {
    let n = sidebar_list.observe_children().n_items();
    if n == 0 {
        return;
    }
    if let Some(row) = sidebar_list.row_at_index((n - 1) as i32) {
        wire_row_close_button(&row, state.clone(), app);
        attach_sidebar_context_menu(&row, state);
    }
}
