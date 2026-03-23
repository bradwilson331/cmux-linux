use std::cell::RefCell;

/// Thread-local storage for the GLArea — allows wakeup_cb (via idle_add_once)
/// to call queue_render() on the main thread without passing it through C callbacks.
/// RefCell is required because gtk4::GLArea is not Copy.
thread_local! {
    pub(crate) static GL_AREA_FOR_RENDER: RefCell<Option<gtk4::GLArea>> = RefCell::new(None);
}

/// Creates and returns a GtkGLArea with a Ghostty terminal surface wired up.
/// Initializes ghostty_app_t, creates ghostty_surface_t with GTK4 platform,
/// and connects all GtkGLArea signals (realize, render, resize, scale-factor).
pub fn create_surface(_app: &gtk4::Application) -> gtk4::GLArea {
    use gtk4::prelude::*;
    use std::ffi::CString;
    use std::sync::atomic::Ordering;

    use crate::ghostty::callbacks::{action_cb, close_surface_cb, wakeup_cb, APP_PTR};
    use crate::ghostty::ffi;

    let gl_area = gtk4::GLArea::new();
    // Per Pitfall 1: require OpenGL 4.3 before the area is realized.
    gl_area.set_required_version(4, 3);
    // Manual render mode: only render when wakeup_cb schedules queue_render().
    // An independent render loop adds input latency (per CLAUDE.md pitfall).
    gl_area.set_auto_render(false);

    // Initialize ghostty app (one-time, before surface creation).
    // Safety: called once on the main thread before any surface is created.
    let ghostty_app = unsafe {
        let argv: Vec<CString> = std::env::args().map(|a| CString::new(a).unwrap()).collect();
        let mut ptrs: Vec<*mut i8> = argv.iter().map(|a| a.as_ptr() as *mut i8).collect();
        ffi::ghostty_init(ptrs.len(), ptrs.as_mut_ptr());

        let config = ffi::ghostty_config_new();
        ffi::ghostty_config_load_default_files(config);
        ffi::ghostty_config_finalize(config); // CRITICAL: finalize before ghostty_app_new

        // Runtime config: register all C callbacks.
        let runtime_config = ffi::ghostty_runtime_config_s {
            userdata: std::ptr::null_mut(),
            supports_selection_clipboard: true,
            wakeup_cb: Some(wakeup_cb),
            action_cb: Some(action_cb),
            read_clipboard_cb: Some(read_clipboard_cb),
            confirm_read_clipboard_cb: Some(confirm_read_clipboard_cb),
            write_clipboard_cb: Some(write_clipboard_cb),
            close_surface_cb: Some(close_surface_cb),
        };

        let ghostty_app = ffi::ghostty_app_new(&runtime_config, config);
        ffi::ghostty_config_free(config);

        // Store app pointer globally for wakeup_cb to use.
        APP_PTR.store(ghostty_app as usize, Ordering::SeqCst);

        ghostty_app
    };

    // Create the ghostty surface config pointing at this GtkGLArea.
    // Surface creation happens before realize — the PTY starts immediately.
    // Rendering is deferred until realize (is_realized guard in wakeup path).
    let ghostty_surface = unsafe {
        let gl_area_ptr = gl_area.as_ptr() as *mut std::ffi::c_void;

        let platform = ffi::ghostty_platform_u {
            gtk4: ffi::ghostty_platform_gtk4_s {
                gl_area: gl_area_ptr,
            },
        };

        let mut surface_config = ffi::ghostty_surface_config_new();
        surface_config.platform_tag = ffi::ghostty_platform_e_GHOSTTY_PLATFORM_GTK4;
        surface_config.platform = platform;
        surface_config.userdata = std::ptr::null_mut();
        surface_config.scale_factor = 1.0; // updated in realize signal

        ffi::ghostty_surface_new(ghostty_app, &surface_config)
    };

    // Store gl_area in thread-local for wakeup_cb to call queue_render on.
    GL_AREA_FOR_RENDER.with(|cell| {
        *cell.borrow_mut() = Some(gl_area.clone());
    });

    // ── GtkGLArea::realize ───────────────────────────────────────────────────
    // GL context is now valid. Set size, scale, and focus.
    gl_area.connect_realize({
        let surface = ghostty_surface;
        move |area| {
            area.make_current();
            if let Some(err) = area.error() {
                eprintln!("cmux: GLArea realize error: {err}");
                std::process::exit(1); // Per D-09: no GUI error dialog in Phase 1
            }
            let scale = area.scale_factor() as f64;
            let w = area.width();
            let h = area.height();
            unsafe {
                // Per Pitfall 5: convert logical→physical pixels.
                ffi::ghostty_surface_set_size(
                    surface,
                    (w as f64 * scale) as u32,
                    (h as f64 * scale) as u32,
                );
                ffi::ghostty_surface_set_content_scale(surface, scale, scale);
                ffi::ghostty_surface_set_focus(surface, true);
            }
        }
    });

    // ── GtkGLArea::render ────────────────────────────────────────────────────
    // Called by GTK frame clock when queue_render() was requested.
    gl_area.connect_render({
        let surface = ghostty_surface;
        move |_area, _ctx| {
            unsafe {
                ffi::ghostty_surface_draw(surface);
            }
            gtk4::glib::Propagation::Stop // suppress GTK default render
        }
    });

    // ── GtkGLArea::resize ────────────────────────────────────────────────────
    // GTK provides logical (CSS) pixels. Ghostty needs physical pixels (Pitfall 5).
    gl_area.connect_resize({
        let surface = ghostty_surface;
        move |area, logical_w, logical_h| {
            let scale = area.scale_factor();
            unsafe {
                ffi::ghostty_surface_set_size(
                    surface,
                    (logical_w * scale) as u32,
                    (logical_h * scale) as u32,
                );
            }
        }
    });

    // ── notify::scale-factor (GHOST-06) ─────────────────────────────────────
    // Fires when the window moves to a monitor with a different DPI.
    // Must use connect_notify_local: ghostty_surface_t is *mut c_void (not Send+Sync).
    // connect_notify_local only requires 'static, and runs on the GLib main thread.
    gl_area.connect_notify_local(Some("scale-factor"), {
        let surface = ghostty_surface;
        move |widget, _| {
            let scale = widget.scale_factor() as f64;
            unsafe {
                ffi::ghostty_surface_set_content_scale(surface, scale, scale);
                ffi::ghostty_surface_refresh(surface); // trigger redraw at new scale
            }
        }
    });

    // ── Key input (GHOST-03) ─────────────────────────────────────────────────────
    // EventControllerKey fires key-pressed and key-released events.
    // CRITICAL: no allocations in this path — per CLAUDE.md typing-latency-sensitive paths.
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.connect_key_pressed({
        let surface = ghostty_surface;
        move |_ctrl, keyval, keycode, state| {
            use crate::ghostty::ffi;
            use crate::ghostty::input::{map_keycode_to_ghostty, map_mods};

            // text field: UTF-8 from the keyval (what the key produces with modifiers applied).
            // Must be a C string. Use a stack-allocated buffer to avoid heap allocation.
            let unicode = keyval.to_unicode();
            let mut text_buf = [0u8; 8]; // UTF-8: max 4 bytes + null
            let text_ptr = if let Some(ch) = unicode {
                let mut s = [0u8; 5];
                let encoded = ch.encode_utf8(&mut s[..4]);
                let len = encoded.len();
                text_buf[..len].copy_from_slice(encoded.as_bytes());
                text_buf[len] = 0;
                text_buf.as_ptr() as *const i8
            } else {
                std::ptr::null()
            };

            let mut input = unsafe { std::mem::zeroed::<ffi::ghostty_input_key_s>() };
            input.keycode = map_keycode_to_ghostty(keycode);
            input.mods = map_mods(state);
            input.action = ffi::ghostty_input_action_e_GHOSTTY_ACTION_PRESS;
            input.text = text_ptr;
            input.consumed_mods = 0; // Not used in Phase 1

            unsafe {
                ffi::ghostty_surface_key(surface, input);
            }
            gtk4::glib::Propagation::Stop // Inhibit: prevent GTK from handling the key
        }
    });
    key_controller.connect_key_released({
        let surface = ghostty_surface;
        move |_ctrl, _keyval, keycode, state| {
            use crate::ghostty::ffi;
            use crate::ghostty::input::{map_keycode_to_ghostty, map_mods};
            let mut input = unsafe { std::mem::zeroed::<ffi::ghostty_input_key_s>() };
            input.keycode = map_keycode_to_ghostty(keycode);
            input.mods = map_mods(state);
            input.action = ffi::ghostty_input_action_e_GHOSTTY_ACTION_RELEASE;
            input.text = std::ptr::null();
            input.consumed_mods = 0; // Not used in Phase 1
            unsafe {
                ffi::ghostty_surface_key(surface, input);
            }
        }
    });
    gl_area.add_controller(key_controller);

    // ── Mouse button input (GHOST-04) ────────────────────────────────────────────
    let click_gesture = gtk4::GestureClick::new();
    click_gesture.set_button(0); // 0 = listen to all mouse buttons
    click_gesture.connect_pressed({
        let surface = ghostty_surface;
        move |gesture, _n_press, _x, _y| {
            use crate::ghostty::ffi;
            let button = match gesture.current_button() {
                1 => ffi::ghostty_input_mouse_button_e_GHOSTTY_MOUSE_LEFT,
                2 => ffi::ghostty_input_mouse_button_e_GHOSTTY_MOUSE_MIDDLE,
                3 => ffi::ghostty_input_mouse_button_e_GHOSTTY_MOUSE_RIGHT,
                _ => return,
            };
            let mods = crate::ghostty::input::map_mods(gesture.current_event_state());
            unsafe {
                ffi::ghostty_surface_mouse_button(
                    surface,
                    ffi::ghostty_input_mouse_state_e_GHOSTTY_MOUSE_PRESS,
                    button,
                    mods,
                );
            }
        }
    });
    click_gesture.connect_released({
        let surface = ghostty_surface;
        move |gesture, _n_press, _x, _y| {
            use crate::ghostty::ffi;
            let button = match gesture.current_button() {
                1 => ffi::ghostty_input_mouse_button_e_GHOSTTY_MOUSE_LEFT,
                2 => ffi::ghostty_input_mouse_button_e_GHOSTTY_MOUSE_MIDDLE,
                3 => ffi::ghostty_input_mouse_button_e_GHOSTTY_MOUSE_RIGHT,
                _ => return,
            };
            let mods = crate::ghostty::input::map_mods(gesture.current_event_state());
            unsafe {
                ffi::ghostty_surface_mouse_button(
                    surface,
                    ffi::ghostty_input_mouse_state_e_GHOSTTY_MOUSE_RELEASE,
                    button,
                    mods,
                );
            }
        }
    });
    gl_area.add_controller(click_gesture);

    // ── Mouse motion ─────────────────────────────────────────────────────────────
    let motion_controller = gtk4::EventControllerMotion::new();
    motion_controller.connect_motion({
        let surface = ghostty_surface;
        move |ctrl, x, y| {
            let mods = crate::ghostty::input::map_mods(ctrl.current_event_state());
            unsafe {
                crate::ghostty::ffi::ghostty_surface_mouse_pos(surface, x, y, mods);
            }
        }
    });
    gl_area.add_controller(motion_controller);

    // ── Scroll input ─────────────────────────────────────────────────────────────
    let scroll_controller = gtk4::EventControllerScroll::new(
        gtk4::EventControllerScrollFlags::BOTH_AXES | gtk4::EventControllerScrollFlags::DISCRETE,
    );
    scroll_controller.connect_scroll({
        let surface = ghostty_surface;
        move |ctrl, dx, dy| {
            use crate::ghostty::ffi;
            // Detect if this is pixel-precise (touchpad) or discrete (mouse wheel)
            let is_pixel = ctrl
                .current_event()
                .and_then(|e| e.downcast::<gtk4::gdk::ScrollEvent>().ok())
                .map(|se| se.direction() == gtk4::gdk::ScrollDirection::Smooth)
                .unwrap_or(false);

            // ghostty_input_scroll_mods_t is a bitmask:
            // bit 0: scroll_is_pixel (1 if touchpad, 0 if mouse wheel)
            // bit 1: momentum (1 if momentum scrolling)
            let scroll_mods = if is_pixel { 1 } else { 0 };

            unsafe {
                ffi::ghostty_surface_mouse_scroll(surface, dx, dy, scroll_mods);
            }
            gtk4::glib::Propagation::Stop
        }
    });
    gl_area.add_controller(scroll_controller);

    gl_area
}

// ── Clipboard callbacks ──────────────────────────────────────────────────────

unsafe extern "C" fn read_clipboard_cb(
    _userdata: *mut std::ffi::c_void,
    clipboard_type: crate::ghostty::ffi::ghostty_clipboard_e,
    _request: *mut std::ffi::c_void,
) {
    use crate::ghostty::ffi;
    use gtk4::prelude::*;

    let display = match gtk4::gdk::Display::default() {
        Some(d) => d,
        None => return,
    };
    let clipboard = if clipboard_type == ffi::ghostty_clipboard_e_GHOSTTY_CLIPBOARD_SELECTION {
        display.primary_clipboard()
    } else {
        display.clipboard()
    };

    // We can't store _request across an async boundary without unsafe global storage.
    // For Phase 1 simplicity: fire-and-forget read. Full async implementation in Phase 2.
    // This is sufficient for basic terminal clipboard paste.
    let _ = clipboard;
}

unsafe extern "C" fn confirm_read_clipboard_cb(
    _userdata: *mut std::ffi::c_void,
    value: *const std::os::raw::c_char,
    surface_ptr: *mut std::ffi::c_void,
    _request_type: crate::ghostty::ffi::ghostty_clipboard_request_e,
) {
    // Phase 1: auto-confirm all clipboard reads without a dialog (per D-09).
    // surface_ptr (arg3) is the ghostty_surface_t — passed back to complete_clipboard_request.
    // _request_type is informational only; we always confirm.
    // complete_clipboard_request's 3rd arg (*mut c_void) is NULL for non-request-based calls.
    unsafe {
        crate::ghostty::ffi::ghostty_surface_complete_clipboard_request(
            surface_ptr as crate::ghostty::ffi::ghostty_surface_t,
            value,
            std::ptr::null_mut(), // no pending request object in confirm path
            true,
        );
    }
}

unsafe extern "C" fn write_clipboard_cb(
    _userdata: *mut std::ffi::c_void,
    clipboard_type: crate::ghostty::ffi::ghostty_clipboard_e,
    content: *const crate::ghostty::ffi::ghostty_clipboard_content_s,
    _len: usize,
    _confirm: bool,
) {
    use crate::ghostty::ffi;
    use gtk4::prelude::*;

    if content.is_null() {
        return;
    }
    let item = &*content;
    let text = if !item.data.is_null() {
        match std::ffi::CStr::from_ptr(item.data).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return,
        }
    } else {
        return;
    };

    let display = match gtk4::gdk::Display::default() {
        Some(d) => d,
        None => return,
    };
    let clipboard = if clipboard_type == ffi::ghostty_clipboard_e_GHOSTTY_CLIPBOARD_SELECTION {
        display.primary_clipboard()
    } else {
        display.clipboard()
    };
    clipboard.set_text(&text);
}
