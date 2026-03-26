use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Build the sidebar widget: GtkScrolledWindow(160px) containing a GtkListBox.
/// Returns the GtkScrolledWindow. The GtkListBox inside is accessible via the AppState
/// after AppState::new() is called with it.
///
/// Per UI-SPEC:
/// - Width: 160px (set_size_request(160, -1))
/// - Background: #242424 (applied via global CssProvider in main.rs)
/// - Row height: 36px min-height (CSS)
/// - Row padding: 8px top/bottom, 16px left/right
/// - Active row: #5b8dd9 background, #ffffff text, font-weight 600
/// - Inactive row: transparent bg, #cccccc text, font-weight 400
/// - Hover (inactive): #2e2e2e
pub fn build_sidebar() -> (gtk4::ScrolledWindow, gtk4::ListBox) {
    let list_box = gtk4::ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    list_box.add_css_class("workspace-list");

    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_size_request(160, -1);
    scrolled.set_hscrollbar_policy(gtk4::PolicyType::Never);
    scrolled.set_vscrollbar_policy(gtk4::PolicyType::Automatic);
    scrolled.set_child(Some(&list_box));
    scrolled.add_css_class("sidebar");

    (scrolled, list_box)
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
            // Restore Phase 4 nested layout: hbox > [vbox > [label], dot]
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

/// Rebuild the Phase 4 sidebar row content: GtkBox(H, 4) > [GtkBox(V, 0) > [GtkLabel(name)], GtkLabel(dot)].
/// Dot is hidden by default (fresh state after rename).
fn rebuild_sidebar_row_content(name: &str) -> gtk4::Box {
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

    hbox
}
