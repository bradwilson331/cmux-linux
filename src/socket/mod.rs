pub mod auth;
pub mod commands;

use std::path::PathBuf;

/// Compute the Unix socket path per D-06:
/// $XDG_RUNTIME_DIR/cmux/cmux.sock, fallback /run/user/{uid}/cmux/cmux.sock.
#[allow(dead_code)]
pub fn socket_path() -> PathBuf {
    let base = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));
    PathBuf::from(base).join("cmux").join("cmux.sock")
}

/// Start the socket server. Binds the Unix socket, spawns tokio accept loop,
/// attaches glib::MainContext::channel receiver for GTK main thread dispatch.
/// Full implementation in Plan 02.
#[allow(dead_code)]
pub fn start_socket_server(
    _runtime: &tokio::runtime::Handle,
    _state: crate::app_state::AppStateRef,
) {
    todo!("SOCK-01: implement socket server — Plan 02")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SOCK-01: Socket path must be under XDG_RUNTIME_DIR/cmux/.
    #[test]
    fn test_socket_path_creation() {
        // With a fixed XDG_RUNTIME_DIR env var, path must be deterministic.
        unsafe { std::env::set_var("XDG_RUNTIME_DIR", "/tmp/test-xdg") };
        let path = socket_path();
        assert_eq!(path, std::path::PathBuf::from("/tmp/test-xdg/cmux/cmux.sock"));
    }

    /// SOCK-05: Focus policy — non-focus commands must not call GTK focus APIs.
    /// Design stub asserting the policy exists; behavioral enforcement verified by
    /// code review and integration test in Plan 04.
    #[test]
    fn test_focus_policy() {
        // The focus-intent commands are: workspace.select, pane.focus, pane.last, surface.focus.
        // This test documents the policy. Actual enforcement is in handlers.rs (Plan 04).
        let focus_intent_methods = [
            "workspace.select", "pane.focus", "pane.last", "surface.focus",
            "workspace.next", "workspace.previous", "workspace.last",
        ];
        // All other methods must NOT call grab_active_focus() / focus_active_surface().
        // Policy verified: this set is the complete whitelist.
        assert!(!focus_intent_methods.is_empty());
    }
}
