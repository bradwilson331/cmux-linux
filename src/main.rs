
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, gio, CssProvider, StyleContext};
use std::ffi::CString;

mod ghostty;
mod workspace;
mod split_engine;
mod app_state;
mod sidebar;
mod shortcuts;
mod socket;
mod session;

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
/* Phase 4: Attention dot for bell notifications (NOTF-02) */
.attention-dot {
    background-color: #e8a444;
    border-radius: 50%;
    min-width: 8px;
    min-height: 8px;
    max-width: 8px;
    max-height: 8px;
    margin: 0 4px;
}
";

fn main() {
    // Tokio runtime for socket I/O (kept alive for app lifetime).
    let runtime = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime");
    let runtime_handle = runtime.handle().clone();

    // glib::MainContext::channel pattern: event-driven bridge from tokio to GTK main thread.
    // NOTE: glib::MainContext::channel was removed in glib 0.18+. We replicate its semantics
    // using tokio::sync::mpsc::unbounded_channel + glib::MainContext::default().spawn_local()
    // in build_ui. The Sender is Send+Clone — tokio tasks hold it. The Receiver is consumed by
    // a spawn_local future that processes commands on the GTK main thread.
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel::<crate::socket::commands::SocketCommand>();

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    eprintln!("cmux: GtkApplication created, connecting activate signal");

    // Try to restore session from previous run (SESS-02, SESS-04).
    // load_session() returns None if file is missing or invalid -- that's fine.
    let saved_session = crate::session::load_session();
    if let Some(ref s) = saved_session {
        eprintln!("cmux: restoring session ({} workspace(s))", s.workspaces.len());
    }

    // Session save infrastructure: Notify for debounce, channel for session snapshots.
    let save_notify = std::sync::Arc::new(tokio::sync::Notify::new());
    let (session_tx, session_rx) = tokio::sync::mpsc::unbounded_channel::<crate::session::SessionData>();

    // Spawn debounce task in tokio. Waits for notify, debounces 500ms, then writes
    // the latest session snapshot to disk atomically (SESS-01, SESS-03).
    {
        let notify = save_notify.clone();
        let mut session_rx = session_rx;
        runtime_handle.spawn(async move {
            loop {
                notify.notified().await;
                // Debounce: 500ms window -- drain extra notifications that arrive.
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                // Drain all queued snapshots, keep only the latest.
                let mut latest = None;
                while let Ok(data) = session_rx.try_recv() {
                    latest = Some(data);
                }
                if let Some(session) = latest {
                    if let Err(e) = crate::session::save_session_atomic(&session) {
                        eprintln!("cmux: session save failed: {e}");
                    }
                }
            }
        });
    }

    // Move runtime_handle, cmd_tx, cmd_rx into the activate closure.
    // cmd_rx is wrapped in Mutex<Option<...>> so it can be taken once from a Fn closure.
    let cmd_rx = std::sync::Mutex::new(Some(cmd_rx));
    let saved_session = std::sync::Mutex::new(Some(saved_session));
    app.connect_activate({
        let runtime_handle = runtime_handle.clone();
        let save_notify = save_notify.clone();
        let session_tx = session_tx.clone();
        move |app| {
            let rx = cmd_rx.lock().unwrap().take().expect("activate called more than once");
            let session = saved_session.lock().unwrap().take().flatten();
            build_ui(app, runtime_handle.clone(), cmd_tx.clone(), rx, save_notify.clone(), session_tx.clone(), session);
        }
    });

    eprintln!("cmux: calling app.run()");
    let _exit_code = app.run();
    eprintln!("cmux: app.run() returned");

    // Runtime drops here — tokio tasks are cancelled.
    drop(runtime);
}

fn build_ui(
    app: &Application,
    runtime_handle: tokio::runtime::Handle,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<crate::socket::commands::SocketCommand>,
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<crate::socket::commands::SocketCommand>,
    save_notify: std::sync::Arc<tokio::sync::Notify>,
    session_tx: tokio::sync::mpsc::UnboundedSender<crate::session::SessionData>,
    saved_session: Option<crate::session::SessionData>,
) {
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

    // Set save_notify and session_tx on AppState so trigger_session_save() works.
    {
        let mut s = state.borrow_mut();
        s.save_notify = Some(save_notify);
        s.session_tx = Some(session_tx);
    }

    // Restore session if available (SESS-02), otherwise create default workspace.
    // For Phase 3, we restore workspace names. Full layout (pane splits) restore
    // requires Ghostty surface reconstruction -- deferred to Phase 4.
    {
        let has_session = saved_session.as_ref().map(|s| !s.workspaces.is_empty()).unwrap_or(false);
        if has_session {
            let session = saved_session.unwrap();
            for ws_session in &session.workspaces {
                state.borrow_mut().create_workspace();
                state.borrow_mut().rename_active(ws_session.name.clone());
            }
            // Restore active workspace index.
            let active = session.active_index.min(session.workspaces.len().saturating_sub(1));
            state.borrow_mut().switch_to_index(active);
        } else {
            // No session -- create the default first workspace.
            state.borrow_mut().create_workspace();
        }
    }

    // Attach command receiver to GTK main loop via glib::MainContext::default().spawn_local.
    // This replaces the old glib::MainContext::channel pattern (removed in glib 0.18+).
    // The spawn_local future runs on the GTK main thread, receiving SocketCommands sent from
    // tokio tasks via the UnboundedSender (cmd_tx). All AppState mutations happen here.
    // Full handler dispatch is wired in Plan 03. For now, just attach with stub dispatch.
    {
        let state = state.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                crate::socket::handlers::handle_socket_command(cmd, &state);
            }
        });
    }

    // Phase 4: Process pending bell notifications on the GTK main thread.
    // action_cb sets BELL_PENDING from within ghostty_app_tick (already on main thread).
    // This idle-driven check runs after each tick to dispatch to AppState::set_pane_attention.
    {
        let state = state.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if crate::ghostty::callbacks::BELL_PENDING.swap(false, std::sync::atomic::Ordering::SeqCst) {
                let pane_id = crate::ghostty::callbacks::BELL_PANE_ID.load(std::sync::atomic::Ordering::SeqCst);
                if pane_id != 0 {
                    state.borrow_mut().set_pane_attention(pane_id);
                }
            }
            glib::ControlFlow::Continue
        });
    }

    // Start socket server (tokio accept loop + XDG path setup).
    // cmd_tx is passed in so the socket server dispatches commands through the
    // existing tokio mpsc bridge to the GTK main thread (spawn_local above).
    crate::socket::start_socket_server(&runtime_handle, state.clone(), cmd_tx);

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
