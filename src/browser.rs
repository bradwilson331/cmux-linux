use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use serde_json::Value;
use base64::Engine as _;
use futures_util::StreamExt;
use gtk4::prelude::*;
use uuid::Uuid;

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

    /// Connect to the agent-browser stream WebSocket and start forwarding
    /// decoded JPEG frames to the GTK main thread via mpsc channel.
    /// The Picture widget is updated in idle callbacks per D-02.
    pub fn start_stream(
        &mut self,
        runtime: &tokio::runtime::Handle,
        picture: gtk4::Picture,
    ) -> Result<(), String> {
        let port = self.read_stream_port()?;
        let url = format!("ws://127.0.0.1:{}", port);

        // Create mpsc channel for frame delivery (tokio -> GTK)
        let (frame_tx, mut frame_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        self.frame_tx = Some(frame_tx.clone());

        // Spawn tokio task: WebSocket client that reads frames
        let stream_task = runtime.spawn(async move {
            let ws_result = tokio_tungstenite::connect_async(&url).await;
            let (ws_stream, _) = match ws_result {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("cmux: browser stream WS connect failed: {}", e);
                    return;
                }
            };
            eprintln!("cmux: browser stream connected to {}", url);

            let (_write, mut read) = ws_stream.split();
            while let Some(msg_result) = read.next().await {
                let msg = match msg_result {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("cmux: browser stream WS error: {}", e);
                        break;
                    }
                };
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    // Parse frame JSON
                    if let Ok(frame) = serde_json::from_str::<serde_json::Value>(&text) {
                        if frame.get("type").and_then(|t| t.as_str()) == Some("frame") {
                            if let Some(data_b64) = frame.get("data").and_then(|d| d.as_str()) {
                                // Decode base64 to JPEG bytes
                                if let Ok(jpeg_bytes) = base64::engine::general_purpose::STANDARD.decode(data_b64) {
                                    // Send to GTK thread (ignore error if receiver dropped)
                                    let _ = frame_tx.send(jpeg_bytes);
                                }
                            }
                        }
                    }
                }
            }
            eprintln!("cmux: browser stream ended");
        });
        self.stream_task = Some(stream_task);

        // Spawn GTK-side receiver: poll mpsc and update Picture widget
        // Use glib::MainContext to process frames on GTK main thread
        let picture_clone = picture.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Some(jpeg_bytes) = frame_rx.recv().await {
                let bytes = glib::Bytes::from(&jpeg_bytes);
                match gtk4::gdk::Texture::from_bytes(&bytes) {
                    Ok(texture) => {
                        picture_clone.set_paintable(Some(&texture));
                    }
                    Err(e) => {
                        // Per UI-SPEC: frame decode failure is silent (keep last valid frame)
                        eprintln!("cmux: browser frame decode error: {}", e);
                    }
                }
            }
        });

        self.preview_state = PreviewState::Streaming;
        Ok(())
    }
}

/// Create a browser preview pane widget (Overlay with Picture + status label).
/// Returns (container, picture, pane_id, uuid) for insertion into SplitNode::Preview.
pub fn create_preview_pane(next_pane_id: u64) -> (gtk4::Overlay, gtk4::Picture, u64, Uuid) {
    let uuid = Uuid::new_v4();
    let picture = gtk4::Picture::new();
    picture.add_css_class("browser-preview");
    picture.set_can_shrink(true);
    picture.set_hexpand(true);
    picture.set_vexpand(true);

    let overlay = gtk4::Overlay::new();
    overlay.add_css_class("preview-container");
    overlay.set_child(Some(&picture));

    // Empty state label (shown when no stream is active)
    let empty_label = gtk4::Label::new(Some("No browser preview"));
    empty_label.add_css_class("preview-empty");
    empty_label.set_halign(gtk4::Align::Center);
    empty_label.set_valign(gtk4::Align::Center);
    overlay.add_overlay(&empty_label);

    (overlay, picture, next_pane_id, uuid)
}

/// Update the preview pane overlay to show the given state.
/// Removes existing status overlays and adds the appropriate label.
pub fn update_preview_overlay(overlay: &gtk4::Overlay, state: &PreviewState) {
    // Remove existing overlay children (status labels).
    // Walk siblings after the main child (Picture) and remove any status labels.
    if let Some(child) = overlay.first_child() {
        let mut sibling = child.next_sibling();
        while let Some(widget) = sibling {
            let next = widget.next_sibling();
            if widget.has_css_class("preview-empty") || widget.has_css_class("preview-error") {
                overlay.remove_overlay(&widget);
            }
            sibling = next;
        }
    }

    match state {
        PreviewState::Empty => {
            let label = gtk4::Label::new(Some("No browser preview"));
            label.add_css_class("preview-empty");
            label.set_halign(gtk4::Align::Center);
            label.set_valign(gtk4::Align::Center);
            overlay.add_overlay(&label);
        }
        PreviewState::Loading => {
            let label = gtk4::Label::new(Some("Starting browser..."));
            label.add_css_class("preview-empty");
            label.set_halign(gtk4::Align::Center);
            label.set_valign(gtk4::Align::Center);
            overlay.add_overlay(&label);
        }
        PreviewState::Connected | PreviewState::Streaming => {
            // No overlay needed -- Picture shows frames or empty background
        }
        PreviewState::Error(msg) => {
            let label = gtk4::Label::new(Some(msg.as_str()));
            label.add_css_class("preview-error");
            label.set_halign(gtk4::Align::Center);
            label.set_valign(gtk4::Align::Center);
            overlay.add_overlay(&label);
        }
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
