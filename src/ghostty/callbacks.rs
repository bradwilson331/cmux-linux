use std::sync::atomic::{AtomicBool, Ordering};
use gtk4::prelude::{GLAreaExt, WidgetExt};

/// Coalesces burst wakeup calls into a single GLib idle dispatch.
/// GLib's idle_add does not deduplicate — this flag prevents queueing N
/// idle tasks when Ghostty fires N wakeups in a single frame burst.
pub static WAKEUP_PENDING: AtomicBool = AtomicBool::new(false);

/// The GhosttyApp handle — stored as usize to be Send across the idle closure.
/// Safety: ghostty_app_t is opaque void* and only called from the GLib main thread.
pub static APP_PTR: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Called by Ghostty from its renderer thread. Must not call any ghostty_* API inline.
/// Instead, schedules ghostty_app_tick() on the GLib main loop (per D-04, GHOST-07).
pub unsafe extern "C" fn wakeup_cb(_userdata: *mut std::ffi::c_void) {
    // Swap: if already pending, another idle task is queued — skip.
    if WAKEUP_PENDING.swap(true, Ordering::SeqCst) {
        return;
    }
    glib::idle_add_once(|| {
        WAKEUP_PENDING.store(false, Ordering::SeqCst);
        let app_ptr = APP_PTR.load(Ordering::SeqCst);
        if app_ptr != 0 {
            unsafe {
                let app = app_ptr as crate::ghostty::ffi::ghostty_app_t;
                crate::ghostty::ffi::ghostty_app_tick(app);
            }
        }
        // queue_render on the GtkGLArea is handled via GL_AREA_FOR_RENDER in surface.rs
        crate::ghostty::surface::GL_AREA_FOR_RENDER.with(|cell| {
            if let Some(area) = cell.borrow().as_ref() {
                if area.is_realized() {
                    area.queue_render();
                }
            }
        });
    });
}

/// Called by Ghostty when a surface wants to close (e.g. shell exits).
/// Runs on the GLib main thread (called during ghostty_app_tick).
/// Per D-09: no GUI dialog — exit the process.
/// The bool argument indicates whether the process was still active when closed.
pub unsafe extern "C" fn close_surface_cb(
    _userdata: *mut std::ffi::c_void,
    _process_alive: bool,
) {
    // Phase 1: single surface. Exit the process — per D-09 (no GUI error dialog).
    std::process::exit(0);
}

/// Action callback — Ghostty fires actions (e.g. new tab, font size changes).
/// Phase 1 ignores all actions. Phase 2 will route workspace/split actions.
/// Returns false to indicate the action was not handled.
pub unsafe extern "C" fn action_cb(
    _app: crate::ghostty::ffi::ghostty_app_t,
    _target: crate::ghostty::ffi::ghostty_target_s,
    _action: crate::ghostty::ffi::ghostty_action_s,
) -> bool {
    // No-op in Phase 1 — return false (unhandled)
    false
}
