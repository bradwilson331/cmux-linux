use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use serde_json::Value;

/// Session name for the agent-browser daemon (one daemon per cmux instance).
const SESSION_NAME: &str = "cmux";

/// Preview pane state tracked by BrowserManager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewState {
    Empty,
    Loading,
    Connected,
    Streaming,
    Error(String),
}

pub struct BrowserManager {
    daemon_process: Option<Child>,
    session_name: String,
    stream_task: Option<tokio::task::JoinHandle<()>>,
    pub frame_tx: Option<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,
    pub preview_state: PreviewState,
}

impl BrowserManager {
    pub fn new() -> Self {
        BrowserManager {
            daemon_process: None,
            session_name: SESSION_NAME.to_string(),
            stream_task: None,
            frame_tx: None,
            preview_state: PreviewState::Empty,
        }
    }

    /// Mirrors agent-browser/cli/src/connection.rs socket dir discovery.
    fn agent_browser_socket_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("AGENT_BROWSER_SOCKET_DIR") {
            if !dir.is_empty() {
                return PathBuf::from(dir);
            }
        }
        if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
            if !dir.is_empty() {
                return PathBuf::from(dir).join("agent-browser");
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".agent-browser");
        }
        std::env::temp_dir().join("agent-browser")
    }

    pub fn daemon_socket_path(&self) -> PathBuf {
        Self::agent_browser_socket_dir().join(format!("{}.sock", self.session_name))
    }

    pub fn stream_port_path(&self) -> PathBuf {
        Self::agent_browser_socket_dir().join(format!("{}.stream", self.session_name))
    }

    fn daemon_ready(&self) -> bool {
        std::os::unix::net::UnixStream::connect(self.daemon_socket_path()).is_ok()
    }

    /// Auto-start the agent-browser daemon (D-05).
    pub fn ensure_daemon(&mut self) -> Result<(), String> {
        if self.daemon_ready() {
            return Ok(());
        }

        // Find agent-browser binary: check PATH, then alongside cmux binary.
        let binary_path = which_agent_browser().ok_or_else(|| {
            "agent-browser not found in PATH. Install it or place it alongside the cmux binary."
                .to_string()
        })?;

        // Create socket dir if needed.
        let socket_dir = Self::agent_browser_socket_dir();
        std::fs::create_dir_all(&socket_dir)
            .map_err(|e| format!("Failed to create socket dir {}: {}", socket_dir.display(), e))?;

        let child = Command::new(&binary_path)
            .env("AGENT_BROWSER_DAEMON", "1")
            .env("AGENT_BROWSER_SESSION", &self.session_name)
            .env("AGENT_BROWSER_STREAM_PORT", "0")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn agent-browser: {}", e))?;

        self.daemon_process = Some(child);

        // Poll daemon_ready() with 200ms intervals, up to 50 retries (10s).
        for _ in 0..50 {
            std::thread::sleep(std::time::Duration::from_millis(200));
            if self.daemon_ready() {
                self.preview_state = PreviewState::Connected;
                return Ok(());
            }
        }

        Err("agent-browser daemon failed to start within 10 seconds".to_string())
    }

    /// Send a newline-delimited JSON command to the daemon socket.
    pub fn send_command(&self, action: &str, params: Value) -> Result<Value, String> {
        let socket_path = self.daemon_socket_path();
        let mut stream = std::os::unix::net::UnixStream::connect(&socket_path)
            .map_err(|e| format!("Failed to connect to daemon socket: {}", e))?;

        let req_id = format!("cmux-{}", rand_u64());
        let mut request = if let Value::Object(map) = params {
            Value::Object(map)
        } else {
            Value::Object(serde_json::Map::new())
        };
        request
            .as_object_mut()
            .unwrap()
            .insert("id".to_string(), Value::String(req_id));
        request
            .as_object_mut()
            .unwrap()
            .insert("action".to_string(), Value::String(action.to_string()));

        let mut json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        json.push('\n');
        stream
            .write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write to daemon socket: {}", e))?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read response: {}", e))?;

        serde_json::from_str(&line)
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Read the stream port from the port file.
    pub fn read_stream_port(&self) -> Result<u16, String> {
        let path = self.stream_port_path();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read stream port file {}: {}", path.display(), e))?;
        content
            .trim()
            .parse::<u16>()
            .map_err(|e| format!("Failed to parse stream port '{}': {}", content.trim(), e))
    }

    /// Shut down the daemon and clean up.
    pub fn shutdown(&mut self) {
        // Try to send close command (best-effort).
        let _ = self.send_command(
            "close",
            serde_json::json!({"id": "cmux-shutdown"}),
        );

        if let Some(ref mut child) = self.daemon_process {
            // Wait up to 2 seconds, then kill.
            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => {
                        if start.elapsed() > std::time::Duration::from_secs(2) {
                            let _ = child.kill();
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(_) => break,
                }
            }
        }
        self.daemon_process = None;

        if let Some(task) = self.stream_task.take() {
            task.abort();
        }
        self.stream_task = None;
    }
}

/// Find agent-browser binary in PATH or alongside the cmux binary.
fn which_agent_browser() -> Option<PathBuf> {
    // Check PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            let candidate = PathBuf::from(dir).join("agent-browser");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    // Check alongside cmux binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("agent-browser");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Simple random u64 for request IDs (no external crate needed).
fn rand_u64() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    hasher.finish()
}
