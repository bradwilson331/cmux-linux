
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
mod ssh;
mod config;
mod browser;
mod ssh_hosts;
mod ssh_dialog;

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
/* Phase 4: SSH connection state subtitle (SSH-01, SSH-04) */
.connection-state {
    font-size: 11px;
    font-weight: 400;
    color: #888888;
}
.connection-state.connected { color: #5b8dd9; }
.connection-state.disconnected { color: #888888; }
.connection-state.reconnecting { color: #e8a444; }
/* Phase 7.1: SSH connect dialog (SSH-01) */
.ssh-dialog { background-color: #242424; }
.ssh-dialog entry { font-size: 14px; padding: 8px 16px; }
/* Phase 8: Browser preview pane */
.browser-preview { background-color: #1a1a1a; }
.browser-url-bar { font-size: 13px; padding: 4px 8px; background-color: #2a2a2a; color: #e0e0e0; border-bottom: 1px solid #444; }
.preview-container { background-color: #1a1a1a; }
.preview-empty, .preview-error { color: #888888; font-size: 14px; font-weight: 400; padding: 32px; }
.preview-error { color: #cc4444; }
.stream-indicator { font-size: 9px; font-weight: 600; color: #5b8dd9; margin: 0 4px; padding: 1px 4px; border-radius: 2px; background-color: rgba(91, 141, 217, 0.15); }
/* Phase 8 Plan 05: Navigation bar */
.browser-nav-bar { background-color: #242424; padding: 4px 8px; border-bottom: 1px solid #3a3a3a; }
.browser-nav-btn { min-width: 28px; min-height: 28px; padding: 4px; margin: 0 2px; border-radius: 4px; background-color: transparent; color: #cccccc; border: none; font-size: 14px; }
.browser-nav-btn:hover { background-color: rgba(255, 255, 255, 0.08); }
.browser-nav-btn:active { background-color: rgba(255, 255, 255, 0.12); }
.browser-nav-btn:disabled { color: #555555; }
.browser-nav-go { color: #5b8dd9; font-weight: 600; }
.browser-nav-devtools { margin-left: auto; }
.browser-nav-devtools:checked { background-color: rgba(91, 141, 217, 0.2); color: #5b8dd9; }
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

    // Load config once at startup (D-06). ShortcutMap must be built inside
    // activate (after GTK init) because accelerator_parse requires GTK.
    let config = crate::config::load_config();

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
            let smap = crate::config::ShortcutMap::from_config(&config.shortcuts);
            build_ui(app, runtime_handle.clone(), cmd_tx.clone(), rx, save_notify.clone(), session_tx.clone(), session, smap);
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
    shortcut_map: crate::config::ShortcutMap,
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
        // CFG-03: Ghostty loads its own config from ~/.config/ghostty/config
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

    // Set save_notify, session_tx, and SSH event channel on AppState.
    let (ssh_event_tx, mut ssh_event_rx) = tokio::sync::mpsc::unbounded_channel::<crate::ssh::SshEvent>();
    {
        let mut s = state.borrow_mut();
        s.save_notify = Some(save_notify);
        s.session_tx = Some(session_tx);
        s.ssh_event_tx = Some(ssh_event_tx);
        s.runtime_handle = Some(runtime_handle.clone());
    }

    // Restore session if available (SESS-02), otherwise create default workspace.
    {
        let has_session = saved_session.as_ref().map(|s| !s.workspaces.is_empty()).unwrap_or(false);
        if has_session {
            let session = saved_session.unwrap();
            if session.version >= 2 {
                // Version 2: full tree restore (D-05)
                let mut restored_count = 0;
                for ws_session in &session.workspaces {
                    if state.borrow_mut().restore_workspace(ws_session).is_some() {
                        restored_count += 1;
                    } else {
                        // D-15: tree invalid or too deep, fall back to single pane
                        eprintln!("cmux: workspace '{}' tree invalid, creating default", ws_session.name);
                        state.borrow_mut().create_workspace();
                        state.borrow_mut().rename_active(ws_session.name.clone());
                    }
                }
                eprintln!("cmux: restored {} workspaces from v2 session", restored_count);
            } else {
                // Version 1: name-only restore (auto-upgrade on next save per D-01)
                for ws_session in &session.workspaces {
                    state.borrow_mut().create_workspace();
                    state.borrow_mut().rename_active(ws_session.name.clone());
                }
            }
            // Restore active workspace index.
            let ws_count = state.borrow().workspaces.len();
            let active = session.active_index.min(ws_count.saturating_sub(1));
            state.borrow_mut().switch_to_index(active);

            // D-10: After GLAreas realize, sync surface pointers from registry.
            if session.version >= 2 {
                let state_for_sync = state.clone();
                gtk4::glib::idle_add_local_once(move || {
                    let mut s = state_for_sync.borrow_mut();
                    for engine in &mut s.split_engines {
                        engine.sync_surfaces_from_registry();
                    }
                    eprintln!("cmux: synced surface pointers from registry after restore");
                });
            }
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

    // Phase 4: Process pending bell notifications and SSH events on the GTK main thread.
    // action_cb sets BELL_PENDING from within ghostty_app_tick (already on main thread).
    // SSH events arrive via ssh_event_rx from tokio tasks.
    {
        let state = state.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            // Process bell notifications
            if crate::ghostty::callbacks::BELL_PENDING.swap(false, std::sync::atomic::Ordering::SeqCst) {
                let pane_id = crate::ghostty::callbacks::BELL_PANE_ID.load(std::sync::atomic::Ordering::SeqCst);
                if pane_id != 0 {
                    state.borrow_mut().set_pane_attention(pane_id);
                }
            }
            // Process SSH events
            while let Ok(event) = ssh_event_rx.try_recv() {
                match event {
                    crate::ssh::SshEvent::StateChanged { workspace_id, state: conn_state } => {
                        // Auto-save host on successful connection (D-04)
                        if conn_state == crate::workspace::ConnectionState::Connected {
                            let remote_target = state.borrow().workspaces.iter()
                                .find(|ws| ws.id == workspace_id)
                                .and_then(|ws| ws.remote_target.clone());
                            if let Some(target) = remote_target {
                                crate::ssh_hosts::save_host(&target);
                            }
                        }
                        state.borrow_mut().update_connection_state(workspace_id, conn_state);
                    }
                    crate::ssh::SshEvent::RemoteOutput { pane_id, data } => {
                        // Dispatch remote output to the Ghostty surface via process_output.
                        if let Ok(registry) = crate::ghostty::callbacks::SURFACE_REGISTRY.lock() {
                            let surface_ptr = registry.iter()
                                .find(|(_, &pid)| pid == pane_id)
                                .map(|(&sptr, _)| sptr as crate::ghostty::ffi::ghostty_surface_t);
                            if let Some(surface) = surface_ptr {
                                unsafe {
                                    crate::ghostty::ffi::ghostty_surface_process_output(
                                        surface,
                                        data.as_ptr() as *const _,
                                        data.len(),
                                    );
                                }
                                // Queue render for the GLArea associated with this surface
                                if let Ok(gl_areas) = crate::ghostty::callbacks::GL_TO_SURFACE.lock() {
                                    for (&gl_ptr, &s_ptr) in gl_areas.iter() {
                                        if s_ptr == surface as usize {
                                            let area: glib::translate::Borrowed<gtk4::GLArea> =
                                                unsafe { glib::translate::from_glib_borrow(gl_ptr as *mut gtk4::ffi::GtkGLArea) };
                                            area.queue_render();
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    crate::ssh::SshEvent::RemoteEof { pane_id } => {
                        // D-08: write exit message to surface, keep pane open for user to close
                        if let Ok(registry) = crate::ghostty::callbacks::SURFACE_REGISTRY.lock() {
                            let surface_ptr = registry.iter()
                                .find(|(_, &pid)| pid == pane_id)
                                .map(|(&sptr, _)| sptr as crate::ghostty::ffi::ghostty_surface_t);
                            if let Some(surface) = surface_ptr {
                                let msg = b"\r\n\x1b[90m[Remote shell exited. Press any key to close]\x1b[0m\r\n";
                                unsafe {
                                    crate::ghostty::ffi::ghostty_surface_process_output(
                                        surface,
                                        msg.as_ptr() as *const _,
                                        msg.len(),
                                    );
                                }
                            }
                        }
                    }
                    crate::ssh::SshEvent::StreamOpened { pane_id, stream_id } => {
                        // Set the stream_id on the IoWriteContext so keystrokes start flowing
                        if let Some(ctx) = state.borrow().remote_pane_contexts.get(&pane_id) {
                            if let Ok(mut sid) = ctx.stream_id.lock() {
                                *sid = Some(stream_id);
                            }
                        }
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    }

    // Phase 8: Clean up browser daemon on app shutdown.
    // connect_shutdown fires while GTK is still alive and runtime handle is valid,
    // ensuring the daemon gets a clean shutdown command before tokio tasks are cancelled.
    {
        let state_for_shutdown = state.clone();
        app.connect_shutdown(move |_| {
            state_for_shutdown.borrow_mut().shutdown_browser();
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

    // 7. Install keyboard shortcuts (config-driven, D-06)
    crate::shortcuts::install_shortcuts(&window, state.clone(), &sidebar_scroll, app, shortcut_map);

    // 8. Present the window
    window.present();
}
