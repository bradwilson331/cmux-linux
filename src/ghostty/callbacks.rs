use gtk4::ffi;
use gtk4::prelude::{GLAreaExt, WidgetExt};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};

// A wrapper around a raw pointer to a GtkGLArea to mark it as Send+Sync.
// This is safe because we only ever access the pointer on the GLib main thread,
// inside glib::idle_add_once closures.
#[derive(Copy, Clone)]
struct GtkGLAreaPtr(*mut ffi::GtkGLArea);
unsafe impl Send for GtkGLAreaPtr {}
unsafe impl Sync for GtkGLAreaPtr {}

/// Coalesces burst wakeup calls into a single GLib idle dispatch.
/// GLib's idle_add does not deduplicate — this flag prevents queueing N
/// idle tasks when Ghostty fires N wakeups in a single frame burst.
pub static WAKEUP_PENDING: AtomicBool = AtomicBool::new(false);

/// The GhosttyApp handle — stored as usize to be Send across the idle closure.
/// Safety: ghostty_app_t is opaque void* and only called from the GLib main thread.
pub static APP_PTR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// The GhostttySurface handle — stored so read_clipboard_cb can complete paste requests.
/// Safety: ghostty_surface_t is opaque void* and only accessed from the GLib main thread.
pub static SURFACE_PTR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Registry of all live GLArea instances. wakeup_cb iterates this to queue_render all panes.
/// Stores raw pointers because gtk4::GLArea is not Send/Sync. The pointers are only
/// dereferenced on the main thread inside glib::idle_add_once closures.
pub static GL_AREA_REGISTRY: Mutex<Vec<GtkGLAreaPtr>> = Mutex::new(Vec::new());

/// Maps surface_ptr (as usize) → pane_id for close_surface_cb routing.
pub static SURFACE_REGISTRY: LazyLock<Mutex<HashMap<usize, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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
        // queue_render on ALL registered GLAreas
        if let Ok(areas) = crate::ghostty::callbacks::GL_AREA_REGISTRY.lock() {
            for area_ptr in areas.iter() {
                let area: glib::translate::Borrowed<gtk4::GLArea> =
                    unsafe { glib::translate::from_glib_borrow(area_ptr.0) };
                if area.is_realized() {
                    area.queue_render();
                }
            }
        }
    });
}

/// Called by Ghostty when a surface wants to close (e.g. shell exits).
/// Runs on the GLib main thread (called during ghostty_app_tick).
/// Per D-09: no GUI dialog — exit the process.
/// The bool argument indicates whether the process was still active when closed.
pub unsafe extern "C" fn close_surface_cb(_userdata: *mut std::ffi::c_void, _process_alive: bool) {
    // Do NOT call process::exit — Phase 2 handles per-pane close gracefully.
    // Identify which pane closed via SURFACE_REGISTRY (populated at surface creation).
    // Full AppState.close_pane() dispatch is wired in Plan 04.
    // For now, log the event so the executor can verify routing works.
    eprintln!("cmux: close_surface_cb fired — per-pane close will be handled by AppState");
}

/// Action callback — Ghostty fires actions (e.g. new tab, font size changes).
/// Handles the `.render` action to trigger a GtkGLArea redraw on the main thread.
/// This is required because must_draw_from_app_thread=true in embedded.zig means
/// the renderer thread sends redraw_surface → App.redrawSurface → action_cb(.render).
/// Returns true if handled, false otherwise.
pub unsafe extern "C" fn action_cb(
    _app: crate::ghostty::ffi::ghostty_app_t,
    _target: crate::ghostty::ffi::ghostty_target_s,
    action: crate::ghostty::ffi::ghostty_action_s,
) -> bool {
    use crate::ghostty::ffi;
    if action.tag == ffi::ghostty_action_tag_e_GHOSTTY_ACTION_RENDER {
        // Trigger a render on the GLArea — will call ghostty_surface_draw on main thread.
        if let Ok(areas) = crate::ghostty::callbacks::GL_AREA_REGISTRY.lock() {
            for area_ptr in areas.iter() {
                let area: glib::translate::Borrowed<gtk4::GLArea> =
                    unsafe { glib::translate::from_glib_borrow(area_ptr.0) };
                if area.is_realized() {
                    area.queue_render();
                }
            }
        }
        return true;
    }
    // Phase 1 ignores all other actions — return false (unhandled)
    false
}
