/// Serializable workspace/pane layout snapshot. Full schema in Plan 05.
#[allow(dead_code)]
pub struct SessionData {
    pub version: u32,
}

/// Load session from ~/.local/share/cmux/session.json (or $XDG_DATA_HOME/cmux/session.json).
/// Returns None if missing or invalid (SESS-04 graceful fallback).
#[allow(dead_code)]
pub fn load_session() -> Option<SessionData> {
    todo!("SESS-02: implement session restore — Plan 05")
}

/// Save session atomically: write to session.json.tmp then rename() to session.json.
#[allow(dead_code)]
pub fn save_session_atomic(_data: &SessionData) -> std::io::Result<()> {
    todo!("SESS-01/SESS-03: implement atomic session save — Plan 05")
}

/// Returns the session file path: $XDG_DATA_HOME/cmux/session.json or
/// ~/.local/share/cmux/session.json.
pub fn session_path() -> std::path::PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| format!("{}/.local/share", std::env::var("HOME").unwrap_or_default()));
    std::path::PathBuf::from(base).join("cmux").join("session.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SESS-01: Save must be triggered after workspace/pane mutations.
    #[test]
    #[ignore = "stub: implement in Plan 05"]
    fn test_save_triggered() {
        todo!("SESS-01")
    }

    /// SESS-02: Full roundtrip — save then load must reproduce the same layout.
    #[test]
    #[ignore = "stub: implement in Plan 05"]
    fn test_restore_roundtrip() {
        todo!("SESS-02")
    }

    /// SESS-03: Atomic write — session.json is only replaced via rename().
    #[test]
    #[ignore = "stub: implement in Plan 05"]
    fn test_atomic_write() {
        todo!("SESS-03")
    }

    /// SESS-04: App must not crash if session.json is missing or invalid JSON.
    #[test]
    fn test_graceful_fallback() {
        unsafe { std::env::set_var("XDG_DATA_HOME", "/tmp/no-such-dir-cmux-test") };
        // load_session() is todo!() in stub; once implemented it must return None here.
        // For now assert the path resolution works correctly.
        let path = session_path();
        assert!(path.to_string_lossy().contains("cmux/session.json"));
    }
}
