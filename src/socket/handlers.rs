// src/socket/handlers.rs — GTK main thread command dispatch (stub, expanded in Plans 03-04)

/// Dispatch a SocketCommand on the GTK main thread.
/// All GTK/AppState mutations happen here — never in tokio tasks.
#[allow(dead_code, unused_variables)]
pub fn handle_socket_command(
    cmd: crate::socket::commands::SocketCommand,
    state: &crate::app_state::AppStateRef,
) {
    // Plan 03 implements full dispatch table.
    match cmd {
        crate::socket::commands::SocketCommand::Ping { req_id: _, resp_tx } => {
            let _ = resp_tx.send(serde_json::json!({"ok": true, "result": {}}));
        }
        crate::socket::commands::SocketCommand::NotImplemented { req_id: _, method: _, resp_tx } => {
            let _ = resp_tx.send(serde_json::json!({"ok": false, "error": "not_implemented"}));
        }
    }
}
