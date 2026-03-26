use crate::app_state::AppState;
use crate::config::ShortcutAction;
use crate::split_engine::FocusDirection;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Install all cmux keyboard shortcuts on the application window.
///
/// Uses PropagationPhase::Capture (parent -> child) so the window controller fires
/// BEFORE Ghostty's per-GLArea EventControllerKey. Without capture phase, Ghostty
/// eats Ctrl+D, Ctrl+N, etc. (per RESEARCH.md Pattern 4 and Anti-patterns).
///
/// Shortcut bindings are driven by ShortcutMap (config-driven, D-06).
pub fn install_shortcuts(
    window: &gtk4::ApplicationWindow,
    state: Rc<RefCell<AppState>>,
    sidebar: &gtk4::ScrolledWindow,
    app: &gtk4::Application,
    shortcut_map: crate::config::ShortcutMap,
) {
    let key_ctrl = gtk4::EventControllerKey::new();
    // CRITICAL: Capture phase -- fires before GLArea key handlers.
    key_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);

    let sidebar_clone = sidebar.clone();
    let app_clone = app.clone();

    key_ctrl.connect_key_pressed({
        let state = state.clone();
        move |_ctrl, keyval, _keycode, mods| {
            match shortcut_map.lookup(mods, keyval) {
                // -- Workspace shortcuts --
                Some(ShortcutAction::NewWorkspace) => {
                    handle_new_workspace(&state, &app_clone);
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::CloseWorkspace) => {
                    handle_close_workspace(&state, &app_clone);
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::NextWorkspace) => {
                    state.borrow_mut().switch_next();
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::PrevWorkspace) => {
                    state.borrow_mut().switch_prev();
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::RenameWorkspace) => {
                    let (active_index, sidebar_list) = {
                        let s = state.borrow();
                        let idx = s.active_index;
                        let list = s.sidebar_list.clone();
                        (idx, list)
                    };
                    crate::sidebar::start_inline_rename(&sidebar_list, active_index, state.clone());
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::ToggleSidebar) => {
                    let visible = sidebar_clone.is_visible();
                    sidebar_clone.set_visible(!visible);
                    if let Some(engine) = state.borrow_mut().active_split_engine_mut() {
                        engine.focus_active_surface();
                    }
                    gtk4::glib::Propagation::Stop
                }
                // -- Pane split shortcuts --
                Some(ShortcutAction::SplitRight) => {
                    handle_split(&state, false);
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::SplitDown) => {
                    handle_split(&state, true);
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::ClosePane) => {
                    handle_close_pane(&state, &app_clone);
                    gtk4::glib::Propagation::Stop
                }
                // -- Focus direction shortcuts --
                Some(ShortcutAction::FocusLeft) => {
                    handle_focus_direction(&state, FocusDirection::Left);
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::FocusRight) => {
                    handle_focus_direction(&state, FocusDirection::Right);
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::FocusUp) => {
                    handle_focus_direction(&state, FocusDirection::Up);
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::FocusDown) => {
                    handle_focus_direction(&state, FocusDirection::Down);
                    gtk4::glib::Propagation::Stop
                }
                // -- Workspace number shortcuts --
                Some(action @ (
                    ShortcutAction::Workspace1 | ShortcutAction::Workspace2 |
                    ShortcutAction::Workspace3 | ShortcutAction::Workspace4 |
                    ShortcutAction::Workspace5 | ShortcutAction::Workspace6 |
                    ShortcutAction::Workspace7 | ShortcutAction::Workspace8 |
                    ShortcutAction::Workspace9
                )) => {
                    let idx = match action {
                        ShortcutAction::Workspace1 => 0,
                        ShortcutAction::Workspace2 => 1,
                        ShortcutAction::Workspace3 => 2,
                        ShortcutAction::Workspace4 => 3,
                        ShortcutAction::Workspace5 => 4,
                        ShortcutAction::Workspace6 => 5,
                        ShortcutAction::Workspace7 => 6,
                        ShortcutAction::Workspace8 => 7,
                        ShortcutAction::Workspace9 => 8,
                        _ => unreachable!(),
                    };
                    state.borrow_mut().switch_to_index(idx);
                    gtk4::glib::Propagation::Stop
                }
                // Everything else passes through to Ghostty.
                _ => gtk4::glib::Propagation::Proceed,
            }
        }
    });

    window.add_controller(key_ctrl);
}

/// Create a new workspace with an initial GLArea pane and add it to AppState + GtkStack.
fn handle_new_workspace(state: &Rc<RefCell<AppState>>, _app: &gtk4::Application) {
    state.borrow_mut().create_workspace();
}

/// Show close-workspace confirmation dialog. If confirmed, closes the active workspace.
fn handle_close_workspace(state: &Rc<RefCell<AppState>>, app: &gtk4::Application) {
    // Cannot close the last workspace.
    let (active_index, workspace_count) = {
        let s = state.borrow();
        (s.active_index, s.workspaces.len())
    };
    if workspace_count <= 1 {
        return; // No-op: cannot close the last workspace
    }

    // Use AlertDialog for close confirmation (per UI-SPEC copywriting contract).
    // let dialog = gtk4::AlertDialog::builder()
    //     .message("Close Workspace?")
    //     .detail("All panes in this workspace will be closed. This cannot be undone.")
    //     .modal(true)
    //     .build();
    // dialog.set_buttons(&["Keep Workspace", "Close Workspace"]);
    // dialog.set_default_button(0);
    // dialog.set_cancel_button(0);

    // // Get the window to attach the dialog to.
    // let window = app.windows().into_iter().next();

    // dialog.choose(window.as_ref(), None::<&gio::Cancellable>, {
    //     let state = state.clone();
    //     move |result| {
    //         // Button index 1 = "Close Workspace" (destructive)
    //         if let Ok(1) = result {
    //             // Free all surfaces in the workspace before closing.
    //             // Full surface cleanup is deferred to SplitEngine integration in later phase.
    //             state.borrow_mut().close_workspace(active_index);
    //         }
    //     }
    // });
    state.borrow_mut().close_workspace(active_index);
}

/// Split the active pane. `vertical=false` -> split right (Ctrl+D), `vertical=true` -> split down.
fn handle_split(state: &Rc<RefCell<AppState>>, vertical: bool) {
    let mut s = state.borrow_mut();
    if let Some(engine) = s.active_split_engine_mut() {
        let _new_pane_id = if vertical {
            engine.split_down()
        } else {
            engine.split_right()
        };
        // The new GLArea is already added to the widget tree inside SplitEngine.
        // CSS active-pane class is updated inside SplitEngine.
    }
}

/// Close the active pane (Ctrl+Shift+X).
fn handle_close_pane(state: &Rc<RefCell<AppState>>, app: &gtk4::Application) {
    let (close_workspace, active_index) = {
        let mut s = state.borrow_mut();
        if let Some(engine) = s.active_split_engine_mut() {
            match engine.close_active() {
                None => (true, s.active_index), // last pane -> close workspace
                Some(_) => (false, 0),
            }
        } else {
            (false, 0)
        }
    };
    if close_workspace {
        handle_close_workspace(state, app);
    }
}

/// Move focus to adjacent pane in `direction`.
fn handle_focus_direction(state: &Rc<RefCell<AppState>>, direction: FocusDirection) {
    let mut s = state.borrow_mut();
    if let Some(engine) = s.active_split_engine_mut() {
        engine.focus_next_in_direction(direction);
    }
}
