
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, gio, CssProvider, StyleContext};
use std::ffi::CString;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

mod ghostty;
mod workspace;
mod split_engine;
mod app_state;
mod sidebar;
mod shortcuts;

const APP_ID: &str = "io.cmux.App";

const APP_CSS: &str = "
/* cmux Phase 2 styles — per UI-SPEC.md */
window { background-color: #1a1a1a; }
.sidebar { background-color: #242424; }
.workspace-list { background-color: #242424; }
.workspace-list row { min-height: 36px; padding: 8px 16px; }
.workspace-list row label { color: #cccccc; font-size: 14px; font-weight: 400; }
.workspace-list row:hover:not(.active-workspace) { background-color: #2e2e2e; }
.workspace-list row.active-workspace { background-color: #5b8dd9; }
.workspace-list row.active-workspace label { color: #ffffff; font-weight: 600; }
.active-pane { border: 1px solid #5b8dd9; }
.rename-entry { font-size: 14px; padding: 2px 4px; }
/* GtkPaned separator styling — makes divider visible on dark backgrounds.
   wide-handle is set programmatically; separator gets min-width/height for draggability. */
paned > separator { background-color: #3a3a3a; min-width: 4px; min-height: 4px; }
paned > separator:hover { background-color: #5b8dd9; }
";

fn main() {
    // Create tokio runtime before GTK app initialization
    let runtime = Arc::new(
        tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime")
    );
    
    // Spawn the runtime in a separate thread to avoid blocking GTK
    let runtime_handle = runtime.handle().clone();
    thread::spawn(move || {
        // Keep the runtime alive for the duration of the application
        // This thread will just park itself after creating the runtime
        thread::park();
    });
    
    // Create a channel to bridge tokio to GTK main thread
    // Using std::sync::mpsc for now, but in production we'd use glib::MainContext::channel
    let (tx, rx) = mpsc::channel::<String>();
    
    // Use glib::idle_add to process messages in GTK main thread
    // This is the pattern for bridging async tasks to the main thread
    let rx = Arc::new(std::sync::Mutex::new(rx));
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if let Ok(rx) = rx.lock() {
            if let Ok(msg) = rx.try_recv() {
                println!("Received from tokio: {}", msg);
            }
        }
        glib::ControlFlow::Continue
    });
    
    // Store the sender for tokio tasks to use
    let tx_for_tokio = tx.clone();
    
    // Test the bridge with a simple message
    runtime_handle.spawn(async move {
        println!("Tokio runtime started successfully");
        
        // Simulate async work
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Send message to GTK main thread
        let _ = tx_for_tokio.send("Tokio runtime successfully bridged to GLib main loop".to_string());
        
        // Log that both runtimes are working
        println!("Tokio task completed - message sent to GLib main thread");
    });

    // NON_UNIQUE bypasses DBus singleton check — required in environments where
    // cross-namespace DBus EXTERNAL auth deadlocks (e.g. NX/container sessions).
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    eprintln!("cmux: GtkApplication created, connecting activate signal");
    app.connect_activate(build_ui);
    eprintln!("cmux: calling app.run()");
    let _exit_code = app.run();
    eprintln!("cmux: app.run() returned");
    
    // Shutdown the runtime when the app exits
    // Note: In production, we'd want a more graceful shutdown mechanism
}

fn build_ui(app: &Application) {
    // 1. Initialize Ghostty once
    let ghostty_app = unsafe {
        use crate::ghostty::ffi;
        use crate::ghostty::callbacks::APP_PTR;
        use std::sync::atomic::Ordering;

        let argv: Vec<CString> = std::env::args().map(|a| CString::new(a).unwrap()).collect();
        let mut ptrs: Vec<*mut i8> = argv.iter().map(|a| a.as_ptr() as *mut i8).collect();
        ffi::ghostty_init(ptrs.len(), ptrs.as_mut_ptr());

        let config = ffi::ghostty_config_new();
        ffi::ghostty_config_load_default_files(config);
        ffi::ghostty_config_finalize(config);

        let runtime_config = ffi::ghostty_runtime_config_s {
            userdata: std::ptr::null_mut(),
            supports_selection_clipboard: true,
            wakeup_cb: Some(crate::ghostty::callbacks::wakeup_cb),
            action_cb: Some(crate::ghostty::callbacks::action_cb),
            read_clipboard_cb: Some(crate::ghostty::surface::read_clipboard_cb),
            confirm_read_clipboard_cb: Some(crate::ghostty::surface::confirm_read_clipboard_cb),
            write_clipboard_cb: Some(crate::ghostty::surface::write_clipboard_cb),
            close_surface_cb: Some(crate::ghostty::callbacks::close_surface_cb),
        };

        let ghostty_app = ffi::ghostty_app_new(&runtime_config, config);
        ffi::ghostty_config_free(config);
        if ghostty_app.is_null() {
            eprintln!("cmux: FATAL — ghostty_app_new returned null");
            std::process::exit(1);
        }
        APP_PTR.store(ghostty_app as usize, Ordering::SeqCst);
        ghostty_app
    };

    // 2. Load CSS
    let provider = CssProvider::new();
    provider.load_from_data(APP_CSS);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("no display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // 3. Build the window layout
    let window = ApplicationWindow::builder()
        .application(app)
        .title("cmux")
        .default_width(800)
        .default_height(600)
        .build();

    let (sidebar_scroll, sidebar_list) = crate::sidebar::build_sidebar();
    let stack = gtk4::Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::None);

    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    hbox.append(&sidebar_scroll);
    hbox.append(&stack);
    // Make the stack expand to fill remaining width.
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    window.set_child(Some(&hbox));

    // 4. Create AppState and initial workspace
    let state = crate::app_state::AppState::new(
        stack.clone(),
        sidebar_list.clone(),
        ghostty_app,
        app.clone(),
    );

    // Wire sidebar click-to-switch.
    crate::sidebar::wire_sidebar_clicks(&sidebar_list, state.clone());

    // Create the first workspace.
    {
        state.borrow_mut().create_workspace();
    }

    // 5. Handle delete-event for close confirmation
    window.connect_close_request({
        let state = state.clone();
        move |_win| {
            let count = state.borrow().workspaces.len();
            if count == 0 {
                return gtk4::glib::Propagation::Proceed;
            }
            // Show close confirmation dialog.
            // let dialog = gtk4::AlertDialog::builder()
            //     .message("Close Workspace?")
            //     .detail("All panes in this workspace will be closed. This cannot be undone.")
            //     .modal(true)
            //     .build();
            // dialog.set_buttons(&["Keep Workspace", "Close Workspace"]);
            // dialog.set_default_button(0);
            // dialog.set_cancel_button(0);

            // For window close, just allow it — full per-workspace dialog wired in shortcuts.rs.
            // This dialog is for the window X button — proceed to close.
            gtk4::glib::Propagation::Proceed
        }
    });

    // 6. Sidebar toggle state (D-04, Ctrl+B — full shortcut wired in Plan 05):
    // Storing sidebar_scroll on the stack is enough for now. Plan 05 will pass it to shortcuts.

    // 7. Install keyboard shortcuts
    crate::shortcuts::install_shortcuts(&window, state.clone(), &sidebar_scroll, app);

    // 8. Present the window
    window.present();
}
