use std::cell::RefCell;
use std::rc::Rc;
use gtk4::glib;

/// Call glGetError() via FFI to check for GL errors after ghostty calls.
/// Returns 0 if no error, or the GL error code otherwise.
fn gl_get_error() -> u32 {
    extern "C" {
        fn glGetError() -> u32;
    }
    unsafe { glGetError() }
}

/// Thread-local storage for the GLArea — allows wakeup_cb (via idle_add_once)
/// to call queue_render() on the main thread without passing it through C callbacks.
/// RefCell is required because gtk4::GLArea is not Copy.
thread_local! {
    pub(crate) static GL_AREA_FOR_RENDER: RefCell<Option<gtk4::GLArea>> = RefCell::new(None);
}

/// Creates and returns a GtkGLArea with a Ghostty terminal surface wired up.
/// Initializes ghostty_app_t, then defers ghostty_surface_t creation to the
/// GtkGLArea realize signal — when the GL context is guaranteed to exist.
pub fn create_surface(_app: &gtk4::Application) -> gtk4::GLArea {
    use gtk4::prelude::*;
    use std::ffi::CString;
    use std::sync::atomic::Ordering;

    use crate::ghostty::callbacks::{action_cb, close_surface_cb, wakeup_cb, APP_PTR, SURFACE_PTR};
    use crate::ghostty::ffi;

    let gl_area = gtk4::GLArea::new();
    // Per Pitfall 1: require OpenGL 4.3 before the area is realized.
    gl_area.set_required_version(4, 3);
    // Manual render mode: only render when wakeup_cb schedules queue_render().
    // An independent render loop adds input latency (per CLAUDE.md pitfall).
    gl_area.set_auto_render(false);
    // Must be focusable to receive keyboard events via EventControllerKey.
    gl_area.set_focusable(true);
    // Grab keyboard focus when the user clicks inside the terminal.
    gl_area.set_focus_on_click(true);

    // Initialize ghostty app (one-time, before surface creation).
    // ghostty_init + ghostty_app_new do NOT require a GL context.
    // Safety: called once on the main thread before any surface is created.
    let ghostty_app = unsafe {
        let argv: Vec<CString> = std::env::args().map(|a| CString::new(a).unwrap()).collect();
        let mut ptrs: Vec<*mut i8> = argv.iter().map(|a| a.as_ptr() as *mut i8).collect();
        ffi::ghostty_init(ptrs.len(), ptrs.as_mut_ptr());
        eprintln!("cmux: ghostty_init complete");

        let config = ffi::ghostty_config_new();
        ffi::ghostty_config_load_default_files(config);
        ffi::ghostty_config_finalize(config); // CRITICAL: finalize before ghostty_app_new
        eprintln!("cmux: ghostty_config finalized");

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

        if ghostty_app.is_null() {
            eprintln!("cmux: FATAL — ghostty_app_new returned null");
            std::process::exit(1);
        }
        eprintln!("cmux: ghostty_app_new succeeded: {:p}", ghostty_app);

        // Store app pointer globally for wakeup_cb to use.
        APP_PTR.store(ghostty_app as usize, Ordering::SeqCst);

        ghostty_app
    };

    // Shared cell for the surface pointer — created in realize (after GL context exists),
    // then used in render, resize, input, and scale-factor callbacks.
    // Rc<RefCell<...>> is safe here: all callbacks run on the GLib main thread.
    let surface_cell: Rc<RefCell<Option<ffi::ghostty_surface_t>>> =
        Rc::new(RefCell::new(None));

    // Store gl_area in thread-local for wakeup_cb to call queue_render on.
    GL_AREA_FOR_RENDER.with(|cell| {
        *cell.borrow_mut() = Some(gl_area.clone());
    });

    // ── GtkGLArea::realize ───────────────────────────────────────────────────
    // GL context is now valid. Create the surface HERE so ghostty can access
    // the GL context immediately after creation (fixes segfault in set_content_scale).
    gl_area.connect_realize({
        let cell = surface_cell.clone();
        move |area| {
            eprintln!("cmux: GLArea realize — making GL context current");
            area.make_current();
            if let Some(err) = area.error() {
                eprintln!("cmux: GLArea realize error: {err}");
                std::process::exit(1); // Per D-09: no GUI error dialog in Phase 1
            }
            eprintln!("cmux: GL context made current, no error");
            eprintln!("cmux: GL area size at realize: {}x{}", area.width(), area.height());
            eprintln!("cmux: GL scale factor at realize: {}", area.scale_factor());

            // Log GL version and renderer info for diagnostics.
            if let Some(ctx) = area.context() {
                let (major, minor) = ctx.version();
                eprintln!("cmux: GL context version: {major}.{minor}");
                eprintln!("cmux: GL context is_legacy: {}", ctx.is_legacy());
            }

            // Create the ghostty surface now that GL context is current.
            let surface = unsafe {
                let gl_area_ptr = area.as_ptr() as *mut std::ffi::c_void;

                let platform = ffi::ghostty_platform_u {
                    gtk4: ffi::ghostty_platform_gtk4_s {
                        gl_area: gl_area_ptr,
                    },
                };

                let mut surface_config = ffi::ghostty_surface_config_new();
                surface_config.platform_tag = ffi::ghostty_platform_e_GHOSTTY_PLATFORM_GTK4;
                surface_config.platform = platform;
                surface_config.userdata = std::ptr::null_mut();
                surface_config.scale_factor = area.scale_factor() as f64;

                eprintln!("cmux: calling ghostty_surface_new");
                let s = ffi::ghostty_surface_new(ghostty_app, &surface_config);
                if s.is_null() {
                    eprintln!("cmux: FATAL — ghostty_surface_new returned null");
                    std::process::exit(1);
                }
                eprintln!("cmux: ghostty_surface_new succeeded: {:p}", s);
                // Check GL error state after surface creation.
                let gl_err = gl_get_error();
                if gl_err != 0 {
                    eprintln!("cmux: GL error after ghostty_surface_new: 0x{gl_err:x}");
                } else {
                    eprintln!("cmux: GL error state after ghostty_surface_new: OK");
                }
                s
            };

            // Set initial size and scale after surface creation.
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
                eprintln!("cmux: ghostty_surface_set_size({}, {})", w, h);
                ffi::ghostty_surface_set_content_scale(surface, scale, scale);
                eprintln!("cmux: ghostty_surface_set_content_scale({scale})");
                ffi::ghostty_surface_set_focus(surface, true);
                eprintln!("cmux: ghostty_surface_set_focus(true)");
            }

            // Store the surface pointer for other callbacks.
            *cell.borrow_mut() = Some(surface);
            // Also store in global for read_clipboard_cb (which has no surface arg).
            SURFACE_PTR.store(surface as usize, Ordering::SeqCst);

            // Request first render.
            area.queue_render();
        }
    });

    // ── GtkGLArea::render ────────────────────────────────────────────────────
    // Called by GTK frame clock when queue_render() was requested.
    gl_area.connect_render({
        let cell = surface_cell.clone();
        move |_area, _ctx| {
            eprintln!("cmux: render callback fired");
            if let Some(surface) = *cell.borrow() {
                eprintln!("cmux: calling ghostty_surface_draw({:p})", surface);
                unsafe {
                    ffi::ghostty_surface_draw(surface);
                }
                eprintln!("cmux: ghostty_surface_draw complete");
            } else {
                eprintln!("cmux: render callback — surface not yet initialized, skipping draw");
            }
            gtk4::glib::Propagation::Stop // suppress GTK default render
        }
    });

    // ── GtkGLArea::resize ────────────────────────────────────────────────────
    // GTK provides logical (CSS) pixels. Ghostty needs physical pixels (Pitfall 5).
    gl_area.connect_resize({
        let cell = surface_cell.clone();
        move |area, logical_w, logical_h| {
            if let Some(surface) = *cell.borrow() {
                let scale = area.scale_factor();
                unsafe {
                    ffi::ghostty_surface_set_size(
                        surface,
                        (logical_w * scale) as u32,
                        (logical_h * scale) as u32,
                    );
                }
            }
        }
    });

    // ── notify::scale-factor (GHOST-06) ─────────────────────────────────────
    // Fires when the window moves to a monitor with a different DPI.
    // Must use connect_notify_local: ghostty_surface_t is *mut c_void (not Send+Sync).
    // connect_notify_local only requires 'static, and runs on the GLib main thread.
    gl_area.connect_notify_local(Some("scale-factor"), {
        let cell = surface_cell.clone();
        move |widget, _| {
            if let Some(surface) = *cell.borrow() {
                let scale = widget.scale_factor() as f64;
                unsafe {
                    ffi::ghostty_surface_set_content_scale(surface, scale, scale);
                    ffi::ghostty_surface_refresh(surface); // trigger redraw at new scale
                }
            }
        }
    });

    // ── Key input (GHOST-03) ─────────────────────────────────────────────────────
    // EventControllerKey fires key-pressed and key-released events.
    // CRITICAL: no allocations in this path — per CLAUDE.md typing-latency-sensitive paths.
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.connect_key_pressed({
        let cell = surface_cell.clone();
        move |_ctrl, keyval, keycode, state| {
            use crate::ghostty::ffi;
            use crate::ghostty::input::map_mods;

            let surface = match *cell.borrow() {
                Some(s) => s,
                None => return gtk4::glib::Propagation::Proceed,
            };

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
            // keycode must be the raw GTK hardware keycode (XKB scancode).
            // Ghostty looks this up in its own native keycodes table to resolve the physical key.
            // Do NOT translate to ghostty_input_key_e here — that is an entirely different type.
            input.keycode = keycode;
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
        let cell = surface_cell.clone();
        move |_ctrl, _keyval, keycode, state| {
            use crate::ghostty::ffi;
            use crate::ghostty::input::map_mods;

            let surface = match *cell.borrow() {
                Some(s) => s,
                None => return,
            };

            let mut input = unsafe { std::mem::zeroed::<ffi::ghostty_input_key_s>() };
            input.keycode = keycode;
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
        let cell = surface_cell.clone();
        move |gesture, _n_press, _x, _y| {
            use crate::ghostty::ffi;
            let surface = match *cell.borrow() {
                Some(s) => s,
                None => return,
            };
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
        let cell = surface_cell.clone();
        move |gesture, _n_press, _x, _y| {
            use crate::ghostty::ffi;
            let surface = match *cell.borrow() {
                Some(s) => s,
                None => return,
            };
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
        let cell = surface_cell.clone();
        move |ctrl, x, y| {
            let surface = match *cell.borrow() {
                Some(s) => s,
                None => return,
            };
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
        let cell = surface_cell.clone();
        move |ctrl, dx, dy| {
            use crate::ghostty::ffi;
            let surface = match *cell.borrow() {
                Some(s) => s,
                None => return gtk4::glib::Propagation::Proceed,
            };
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
    request: *mut std::ffi::c_void,
) {
    use crate::ghostty::ffi;
    use gtk4::prelude::*;
    use std::sync::atomic::Ordering;

    let surface_ptr = crate::ghostty::callbacks::SURFACE_PTR.load(Ordering::SeqCst);
    if surface_ptr == 0 {
        return;
    }
    let surface = surface_ptr as ffi::ghostty_surface_t;

    let display = match gtk4::gdk::Display::default() {
        Some(d) => d,
        None => return,
    };
    let clipboard = if clipboard_type == ffi::ghostty_clipboard_e_GHOSTTY_CLIPBOARD_SELECTION {
        display.primary_clipboard()
    } else {
        display.clipboard()
    };

    // Read clipboard text synchronously using GLib event loop.
    // gtk4::glib::MainContext::block_on runs the async future on the current (main) thread.
    // This is safe here because read_clipboard_cb is called from the GLib main thread.
    let text_result = glib::MainContext::default().block_on(clipboard.read_text_future());

    let c_text = match text_result {
        Ok(Some(ref s)) => std::ffi::CString::new(s.as_str()).ok(),
        _ => None,
    };
    let text_ptr = c_text.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());

    unsafe {
        ffi::ghostty_surface_complete_clipboard_request(surface, text_ptr, request, true);
    }
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
