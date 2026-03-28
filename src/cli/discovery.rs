//! Socket path discovery chain for the cmux CLI.
//!
//! Ports the Python `_default_socket_path()` logic from tests_v2/cmux.py to Rust.
//! Discovery order matches D-11.

use std::path::Path;

/// Discover the cmux socket path using the standard search chain.
///
/// Returns `Some(path)` if a candidate is found, `None` otherwise.
/// The caller should check whether the path actually exists if needed;
/// this function returns the best candidate even if the socket file
/// is not yet present (for env-var overrides and fallback paths).
pub fn discover_socket() -> Option<String> {
    // 1. CMUX_SOCKET env var
    if let Ok(val) = std::env::var("CMUX_SOCKET") {
        if !val.is_empty() {
            return Some(val);
        }
    }

    // Also check CMUX_SOCKET_PATH for backwards compat with Python client
    if let Ok(val) = std::env::var("CMUX_SOCKET_PATH") {
        if !val.is_empty() {
            return Some(val);
        }
    }

    // 2. $XDG_RUNTIME_DIR/cmux/cmux.sock (fallback /run/user/{uid}/cmux/cmux.sock)
    let xdg_base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        format!("/run/user/{}", unsafe { libc::getuid() })
    });
    let xdg_socket = format!("{}/cmux/cmux.sock", xdg_base);
    if Path::new(&xdg_socket).exists() {
        return Some(xdg_socket);
    }

    // 3. $XDG_RUNTIME_DIR/cmux/last-socket-path marker file
    let marker = format!("{}/cmux/last-socket-path", xdg_base);
    if let Ok(contents) = std::fs::read_to_string(&marker) {
        let path = contents.trim().to_string();
        if !path.is_empty() && Path::new(&path).exists() {
            return Some(path);
        }
    }

    // 4. /tmp/cmux-debug.sock
    let debug_sock = "/tmp/cmux-debug.sock";
    if Path::new(debug_sock).exists() {
        return Some(debug_sock.to_string());
    }

    // 5. Glob /tmp/cmux-debug-*.sock — sort by mtime descending, return newest
    if let Ok(entries) = std::fs::read_dir("/tmp") {
        let mut candidates: Vec<(String, std::time::SystemTime)> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("cmux-debug-") && name_str.ends_with(".sock") {
                if let Ok(meta) = entry.metadata() {
                    let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                    candidates.push((entry.path().to_string_lossy().to_string(), mtime));
                }
            }
        }
        if !candidates.is_empty() {
            candidates.sort_by(|a, b| b.1.cmp(&a.1));
            return Some(candidates[0].0.clone());
        }
    }

    None
}
