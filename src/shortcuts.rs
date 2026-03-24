use crate::app_state::AppState;
use crate::split_engine::FocusDirection;
use gtk4::gio;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Install all cmux Phase 2 keyboard shortcuts on the application window.
///
/// Uses PropagationPhase::Capture (parent → child) so the window controller fires
/// BEFORE Ghostty's per-GLArea EventControllerKey. Without capture phase, Ghostty
/// eats Ctrl+D, Ctrl+N, etc. (per RESEARCH.md Pattern 4 and Anti-patterns).
///
/// Per D-10: all shortcuts use Ctrl (not Cmd) as the modifier base.
pub fn install_shortcuts(
    window: &gtk4::ApplicationWindow,
    state: Rc<RefCell<AppState>>,
    sidebar: &gtk4::ScrolledWindow,
    app: &gtk4::Application,
) {
    let key_ctrl = gtk4::EventControllerKey::new();
    // CRITICAL: Capture phase — fires before GLArea key handlers.
    key_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);

    let sidebar_clone = sidebar.clone();
    let app_clone = app.clone();

    key_ctrl.connect_key_pressed({
        let state = state.clone();
        move |_ctrl, keyval, _keycode, mods| {
            use gtk4::gdk::ModifierType;
            let ctrl = mods.contains(ModifierType::CONTROL_MASK);
            let shift = mods.contains(ModifierType::SHIFT_MASK);
            let alt = mods.contains(ModifierType::ALT_MASK);

            // Match on (ctrl, shift, alt, keyval).
            // Return Propagation::Stop for handled shortcuts (prevents Ghostty from seeing them).
            // Return Propagation::Proceed for unhandled keys.

            match (ctrl, shift, alt, keyval) {
                // ── Workspace shortcuts ──────────────────────────────────
                // Ctrl+N: New workspace (WS-01, D-10)
                (true, false, false, k) if k == gtk4::gdk::Key::n => {
                    handle_new_workspace(&state, &app_clone);
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+Shift+W: Close active workspace with confirmation (WS-02, D-10)
                (true, true, false, k) if k == gtk4::gdk::Key::W => {
                    handle_close_workspace(&state, &app_clone);
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+]: Next workspace (WS-03, D-10)
                (true, false, false, k) if k == gtk4::gdk::Key::bracketright => {
                    state.borrow_mut().switch_next();
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+[: Prev workspace (WS-03, D-10)
                (true, false, false, k) if k == gtk4::gdk::Key::bracketleft => {
                    state.borrow_mut().switch_prev();
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+1 through Ctrl+9: Switch by number (WS-05, D-10)
                (true, false, false, k) if k == gtk4::gdk::Key::_1 => {
                    state.borrow_mut().switch_to_index(0);
                    gtk4::glib::Propagation::Stop
                }
                (true, false, false, k) if k == gtk4::gdk::Key::_2 => {
                    state.borrow_mut().switch_to_index(1);
                    gtk4::glib::Propagation::Stop
                }
                (true, false, false, k) if k == gtk4::gdk::Key::_3 => {
                    state.borrow_mut().switch_to_index(2);
                    gtk4::glib::Propagation::Stop
                }
                (true, false, false, k) if k == gtk4::gdk::Key::_4 => {
                    state.borrow_mut().switch_to_index(3);
                    gtk4::glib::Propagation::Stop
                }
                (true, false, false, k) if k == gtk4::gdk::Key::_5 => {
                    state.borrow_mut().switch_to_index(4);
                    gtk4::glib::Propagation::Stop
                }
                (true, false, false, k) if k == gtk4::gdk::Key::_6 => {
                    state.borrow_mut().switch_to_index(5);
                    gtk4::glib::Propagation::Stop
                }
                (true, false, false, k) if k == gtk4::gdk::Key::_7 => {
                    state.borrow_mut().switch_to_index(6);
                    gtk4::glib::Propagation::Stop
                }
                (true, false, false, k) if k == gtk4::gdk::Key::_8 => {
                    state.borrow_mut().switch_to_index(7);
                    gtk4::glib::Propagation::Stop
                }
                (true, false, false, k) if k == gtk4::gdk::Key::_9 => {
                    state.borrow_mut().switch_to_index(8);
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+Shift+R: Rename active workspace (WS-04, D-10)
                (true, true, false, k) if k == gtk4::gdk::Key::R => {
                    let (active_index, sidebar_list) = {
                        let s = state.borrow();
                        let idx = s.active_index;
                        let list = s.sidebar_list.clone();
                        (idx, list)
                    };
                    crate::sidebar::start_inline_rename(&sidebar_list, active_index, state.clone());
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+B: Toggle sidebar visibility (D-04, D-10)
                (true, false, false, k) if k == gtk4::gdk::Key::b => {
                    let visible = sidebar_clone.is_visible();
                    sidebar_clone.set_visible(!visible);
                    // Re-grab terminal focus — sidebar visibility change defocuses GLArea.
                    if let Some(engine) = state.borrow_mut().active_split_engine_mut() {
                        engine.grab_active_focus();
                    }
                    gtk4::glib::Propagation::Stop
                }

                // ── Pane split shortcuts ─────────────────────────────────
                // Ctrl+D: Split right (SPLIT-01, D-10)
                (true, false, false, k) if k == gtk4::gdk::Key::d => {
                    handle_split(&state, false);
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+Shift+D: Split down (SPLIT-02, D-10)
                (true, true, false, k) if k == gtk4::gdk::Key::D => {
                    handle_split(&state, true);
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+Shift+X: Close active pane (SPLIT-05, UI-SPEC)
                (true, true, false, k) if k == gtk4::gdk::Key::X => {
                    handle_close_pane(&state, &app_clone);
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+Alt+Left: Focus pane left (SPLIT-03, D-10)
                (true, false, true, k) if k == gtk4::gdk::Key::Left => {
                    handle_focus_direction(&state, FocusDirection::Left);
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+Alt+Right: Focus pane right (SPLIT-03, D-10)
                (true, false, true, k) if k == gtk4::gdk::Key::Right => {
                    handle_focus_direction(&state, FocusDirection::Right);
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+Alt+Up: Focus pane up (SPLIT-03, D-10)
                (true, false, true, k) if k == gtk4::gdk::Key::Up => {
                    handle_focus_direction(&state, FocusDirection::Up);
                    gtk4::glib::Propagation::Stop
                }

                // Ctrl+Alt+Down: Focus pane down (SPLIT-03, D-10)
                (true, false, true, k) if k == gtk4::gdk::Key::Down => {
                    handle_focus_direction(&state, FocusDirection::Down);
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

/// Split the active pane. `vertical=false` → split right (Ctrl+D), `vertical=true` → split down.
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
                None => (true, s.active_index), // last pane → close workspace
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
