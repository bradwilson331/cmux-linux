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
                // -- SSH workspace shortcut --
                Some(ShortcutAction::NewSshWorkspace) => {
                    handle_new_ssh_workspace(&state, &app_clone);
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
                // -- Browser shortcuts --
                Some(ShortcutAction::BrowserOpen) => {
                    handle_browser_open(&state);
                    gtk4::glib::Propagation::Stop
                }
                Some(ShortcutAction::BrowserClose) => {
                    handle_browser_close(&state);
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

    let dialog = gtk4::AlertDialog::builder()
        .message("Close Workspace?")
        .detail("All panes in this workspace will be closed. This cannot be undone.")
        .modal(true)
        .build();
    dialog.set_buttons(&["Keep Workspace", "Close Workspace"]);
    dialog.set_default_button(0);
    dialog.set_cancel_button(0);

    let window = app.windows().into_iter().next();

    dialog.choose(window.as_ref(), None::<&gtk4::gio::Cancellable>, {
        let state = state.clone();
        move |result| {
            // Button index 1 = "Close Workspace" (destructive)
            if let Ok(1) = result {
                state.borrow_mut().close_workspace(active_index);
            }
        }
    });
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

/// Open the SSH connect dialog (Ctrl+Shift+S).
fn handle_new_ssh_workspace(state: &Rc<RefCell<AppState>>, app: &gtk4::Application) {
    crate::ssh_dialog::show_ssh_dialog(app, state.clone());
}

/// Move focus to adjacent pane in `direction`.
fn handle_focus_direction(state: &Rc<RefCell<AppState>>, direction: FocusDirection) {
    let mut s = state.borrow_mut();
    if let Some(engine) = s.active_split_engine_mut() {
        engine.focus_next_in_direction(direction);
    }
}

/// Open a browser preview pane (Ctrl+Shift+B).
/// Launches the agent-browser daemon, navigates to about:blank, creates a preview pane,
/// and starts the CDP screencast stream so frames render in the Picture widget.
fn handle_browser_open(state: &Rc<RefCell<AppState>>) {
    // Step 1: Ensure daemon is running, launch browser, enable streaming, navigate
    {
        let mut s = state.borrow_mut();
        if s.browser_manager.is_none() {
            s.browser_manager = Some(crate::browser::BrowserManager::new());
        }
        let bm = s.browser_manager.as_mut().unwrap();
        if let Err(e) = bm.ensure_daemon() {
            eprintln!("cmux: browser.open failed to start daemon: {e}");
            return;
        }
        // Enable WebSocket stream server, launch Chrome, navigate, start screencast
        let _ = bm.send_command("stream_enable", serde_json::json!({}));
        let _ = bm.send_command("launch", serde_json::json!({"headless": true}));
        if let Err(e) = bm.send_command("navigate", serde_json::json!({"url": "about:blank"})) {
            eprintln!("cmux: browser.open navigate failed: {e}");
            return;
        }
        let _ = bm.send_command("screencast_start", serde_json::json!({"format": "jpeg", "quality": 80}));
    } // drop borrow

    // Step 2: Create preview pane
    let pane_result = {
        let mut s = state.borrow_mut();
        if let Some(engine) = s.active_split_engine_mut() {
            engine.split_active_with_preview()
        } else {
            None
        }
    }; // drop borrow

    let Some(widgets) = pane_result else {
        return;
    };
    let picture = widgets.picture.clone();
    let url_entry = widgets.url_entry.clone();
    let picture_ref = picture.clone();

    // Step 3: Start WebSocket stream to pipe frames to Picture widget
    {
        let mut s = state.borrow_mut();
        let runtime = s.runtime_handle.clone();
        let bm = s.browser_manager.as_mut().unwrap();
        if let Some(ref rt) = runtime {
            if let Err(e) = bm.start_stream(rt, picture) {
                eprintln!("cmux: browser start_stream failed: {e}");
            }
        }
    } // drop borrow

    // Step 4: Set viewport to match pane size (deferred until after GTK layout)
    {
        let state_for_viewport = state.clone();
        let picture_for_viewport = picture_ref.clone();
        glib::idle_add_local_once(move || {
            let pic_w = picture_for_viewport.width();
            let pic_h = picture_for_viewport.height();
            if pic_w > 0 && pic_h > 0 {
                let s = state_for_viewport.borrow();
                if let Some(ref bm) = s.browser_manager {
                    let _ = bm.send_command("viewport", serde_json::json!({"width": pic_w, "height": pic_h}));
                }
            }
        });
    }

    // Attach mouse click controller to the Picture for browser interaction
    {
        let click_ctrl = gtk4::GestureClick::new();
        let state_for_click = state.clone();
        let picture_for_click = picture_ref.clone();
        click_ctrl.connect_released(move |_gesture, _n_press, x, y| {
            // Scale widget coordinates to viewport coordinates
            let pic_w = picture_for_click.width() as f64;
            let pic_h = picture_for_click.height() as f64;
            if pic_w <= 0.0 || pic_h <= 0.0 {
                return;
            }
            // Get the current viewport size from the texture paintable
            let (vp_w, vp_h) = picture_for_click
                .paintable()
                .map(|p| (p.intrinsic_width() as f64, p.intrinsic_height() as f64))
                .unwrap_or((pic_w, pic_h));
            let scale_x = vp_w / pic_w;
            let scale_y = vp_h / pic_h;
            let cx = (x * scale_x) as i64;
            let cy = (y * scale_y) as i64;

            let s = state_for_click.borrow();
            if let Some(ref bm) = s.browser_manager {
                // mousePressed + mouseReleased = click
                let _ = bm.send_command("input_mouse", serde_json::json!({
                    "type": "mousePressed", "x": cx, "y": cy,
                    "button": "left", "clickCount": 1
                }));
                let _ = bm.send_command("input_mouse", serde_json::json!({
                    "type": "mouseReleased", "x": cx, "y": cy,
                    "button": "left", "clickCount": 1
                }));
            }
        });
        picture_ref.add_controller(click_ctrl);

        // Attach mouse motion controller for hover effects
        let motion_ctrl = gtk4::EventControllerMotion::new();
        let state_for_motion = state.clone();
        let picture_for_motion = picture_ref.clone();
        motion_ctrl.connect_motion(move |_ctrl, x, y| {
            let pic_w = picture_for_motion.width() as f64;
            let pic_h = picture_for_motion.height() as f64;
            if pic_w <= 0.0 || pic_h <= 0.0 {
                return;
            }
            let (vp_w, vp_h) = picture_for_motion
                .paintable()
                .map(|p| (p.intrinsic_width() as f64, p.intrinsic_height() as f64))
                .unwrap_or((pic_w, pic_h));
            let scale_x = vp_w / pic_w;
            let scale_y = vp_h / pic_h;
            let mx = (x * scale_x) as i64;
            let my = (y * scale_y) as i64;

            let s = state_for_motion.borrow();
            if let Some(ref bm) = s.browser_manager {
                let _ = bm.send_command("input_mouse", serde_json::json!({
                    "type": "mouseMoved", "x": mx, "y": my
                }));
            }
        });
        picture_ref.add_controller(motion_ctrl);

        // Attach scroll controller for scroll wheel forwarding
        let scroll_ctrl = gtk4::EventControllerScroll::new(
            gtk4::EventControllerScrollFlags::VERTICAL | gtk4::EventControllerScrollFlags::DISCRETE,
        );
        let state_for_scroll = state.clone();
        let picture_for_scroll = picture_ref.clone();
        scroll_ctrl.connect_scroll(move |_ctrl, _dx, dy| {
            let pic_w = picture_for_scroll.width() as f64;
            let pic_h = picture_for_scroll.height() as f64;
            if pic_w <= 0.0 || pic_h <= 0.0 {
                return gtk4::glib::Propagation::Proceed;
            }
            // Scroll at center of viewport; dy is in discrete scroll units
            let (vp_w, vp_h) = picture_for_scroll
                .paintable()
                .map(|p| (p.intrinsic_width() as f64, p.intrinsic_height() as f64))
                .unwrap_or((pic_w, pic_h));
            let cx = (vp_w / 2.0) as i64;
            let cy = (vp_h / 2.0) as i64;
            // CDP mouseWheel uses pixel delta; ~120px per scroll tick
            let delta_y = (dy * 120.0) as i64;

            let s = state_for_scroll.borrow();
            if let Some(ref bm) = s.browser_manager {
                let _ = bm.send_command("input_mouse", serde_json::json!({
                    "type": "mouseWheel", "x": cx, "y": cy,
                    "deltaX": 0, "deltaY": delta_y
                }));
            }
            gtk4::glib::Propagation::Stop
        });
        picture_ref.add_controller(scroll_ctrl);

        // Attach keyboard controller for key forwarding to Chrome
        let key_ctrl = gtk4::EventControllerKey::new();
        // Bubble phase so cmux capture-phase shortcuts (Ctrl+Shift+B etc) take priority
        key_ctrl.set_propagation_phase(gtk4::PropagationPhase::Bubble);
        let state_for_key = state.clone();
        key_ctrl.connect_key_pressed(move |_ctrl, keyval, _keycode, mods| {
            let s = state_for_key.borrow();
            let bm = match s.browser_manager.as_ref() {
                Some(bm) => bm,
                None => return gtk4::glib::Propagation::Proceed,
            };
            let (key_str, code_str) = gdk_keyval_to_cdp(keyval);
            if key_str.is_empty() {
                return gtk4::glib::Propagation::Proceed;
            }
            let text = if key_str.len() == 1 && !mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                key_str.clone()
            } else {
                String::new()
            };
            let modifiers = cdp_modifiers(mods);
            let mut params = serde_json::json!({
                "type": "keyDown", "key": key_str, "code": code_str,
                "modifiers": modifiers
            });
            if !text.is_empty() {
                params.as_object_mut().unwrap().insert("text".to_string(), serde_json::json!(text));
            }
            let _ = bm.send_command("input_keyboard", params);
            gtk4::glib::Propagation::Stop
        });
        let state_for_keyup = state.clone();
        key_ctrl.connect_key_released(move |_ctrl, keyval, _keycode, mods| {
            let s = state_for_keyup.borrow();
            if let Some(ref bm) = s.browser_manager {
                let (key_str, code_str) = gdk_keyval_to_cdp(keyval);
                if !key_str.is_empty() {
                    let modifiers = cdp_modifiers(mods);
                    let _ = bm.send_command("input_keyboard", serde_json::json!({
                        "type": "keyUp", "key": key_str, "code": code_str,
                        "modifiers": modifiers
                    }));
                }
            }
        });
        // Attach to container (the focusable Box) so it receives key events when preview is focused
        if let Some(parent_box) = picture_ref.parent()
            .and_then(|o| o.parent()) // overlay -> container box
        {
            parent_box.set_focusable(true);
            parent_box.add_controller(key_ctrl);
        }
    }

    // Step 5: Connect URL entry — Enter navigates the browser
    let state_for_entry = state.clone();
    let picture_for_nav = picture_ref.clone();
    url_entry.connect_activate(move |entry| {
        let raw_url = entry.text().to_string();
        if raw_url.is_empty() {
            return;
        }
        // Auto-prepend https:// if no scheme is present
        let url = if raw_url.contains("://") {
            raw_url
        } else {
            format!("https://{raw_url}")
        };
        entry.set_text(&url);
        let s = state_for_entry.borrow();
        if let Some(ref bm) = s.browser_manager {
            // Resize viewport to match current pane size before navigating
            let w = picture_for_nav.width();
            let h = picture_for_nav.height();
            if w > 0 && h > 0 {
                let _ = bm.send_command("viewport", serde_json::json!({"width": w, "height": h}));
            }
            let params = serde_json::json!({"url": url});
            let _ = bm.send_command("navigate", params);
        }
    });
}

/// Map GDK keyval to (CDP key name, CDP code name).
/// Returns empty strings for unmapped keys.
fn gdk_keyval_to_cdp(keyval: gtk4::gdk::Key) -> (String, String) {
    use gtk4::gdk::Key;
    match keyval {
        Key::Return | Key::KP_Enter => ("Enter".into(), "Enter".into()),
        Key::Tab => ("Tab".into(), "Tab".into()),
        Key::Escape => ("Escape".into(), "Escape".into()),
        Key::BackSpace => ("Backspace".into(), "Backspace".into()),
        Key::Delete => ("Delete".into(), "Delete".into()),
        Key::Home => ("Home".into(), "Home".into()),
        Key::End => ("End".into(), "End".into()),
        Key::Page_Up => ("PageUp".into(), "PageUp".into()),
        Key::Page_Down => ("PageDown".into(), "PageDown".into()),
        Key::Left => ("ArrowLeft".into(), "ArrowLeft".into()),
        Key::Right => ("ArrowRight".into(), "ArrowRight".into()),
        Key::Up => ("ArrowUp".into(), "ArrowUp".into()),
        Key::Down => ("ArrowDown".into(), "ArrowDown".into()),
        Key::space => (" ".into(), "Space".into()),
        Key::F1 => ("F1".into(), "F1".into()),
        Key::F2 => ("F2".into(), "F2".into()),
        Key::F3 => ("F3".into(), "F3".into()),
        Key::F4 => ("F4".into(), "F4".into()),
        Key::F5 => ("F5".into(), "F5".into()),
        Key::F6 => ("F6".into(), "F6".into()),
        Key::F7 => ("F7".into(), "F7".into()),
        Key::F8 => ("F8".into(), "F8".into()),
        Key::F9 => ("F9".into(), "F9".into()),
        Key::F10 => ("F10".into(), "F10".into()),
        Key::F11 => ("F11".into(), "F11".into()),
        Key::F12 => ("F12".into(), "F12".into()),
        other => {
            // For printable characters, use the unicode value
            if let Some(ch) = other.to_unicode() {
                let s = ch.to_string();
                let code = if ch.is_ascii_alphabetic() {
                    format!("Key{}", ch.to_ascii_uppercase())
                } else if ch.is_ascii_digit() {
                    format!("Digit{}", ch)
                } else {
                    s.clone()
                };
                (s, code)
            } else {
                (String::new(), String::new())
            }
        }
    }
}

/// Convert GDK modifier flags to CDP modifier bitmask.
/// CDP: Alt=1, Ctrl=2, Meta=4, Shift=8
fn cdp_modifiers(mods: gtk4::gdk::ModifierType) -> i32 {
    let mut m = 0;
    if mods.contains(gtk4::gdk::ModifierType::ALT_MASK) { m |= 1; }
    if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK) { m |= 2; }
    if mods.contains(gtk4::gdk::ModifierType::SHIFT_MASK) { m |= 8; }
    m
}

/// Close the browser preview and shut down the daemon (Ctrl+Shift+Q).
fn handle_browser_close(state: &Rc<RefCell<AppState>>) {
    let mut s = state.borrow_mut();
    if let Some(ref mut bm) = s.browser_manager {
        bm.shutdown();
        s.browser_manager = None;
    }
}
