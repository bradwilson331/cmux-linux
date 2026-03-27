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
            let (_write, mut read) = ws_stream.split();
            while let Some(msg_result) = read.next().await {
                let msg = match msg_result {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("cmux: browser stream error: {}", e);
                        break;
                    }
                };
                if let tokio_tungstenite::tungstenite::Message::Text(text) = &msg {
                    if let Ok(frame) = serde_json::from_str::<serde_json::Value>(text) {
                        if frame.get("type").and_then(|t| t.as_str()) == Some("frame") {
                            if let Some(data_b64) = frame.get("data").and_then(|d| d.as_str()) {
                                if let Ok(jpeg_bytes) = base64::engine::general_purpose::STANDARD.decode(data_b64) {
                                    let _ = frame_tx.send(jpeg_bytes);
                                }
                            }
                        }
                    }
                }
            }
        });
        self.stream_task = Some(stream_task);

        // Spawn GTK-side receiver: poll mpsc and update Picture widget
        let picture_clone = picture.clone();
        glib::MainContext::default().spawn_local(async move {
            let mut first_frame = true;
            while let Some(jpeg_bytes) = frame_rx.recv().await {
                let bytes = glib::Bytes::from(&jpeg_bytes);
                match gtk4::gdk::Texture::from_bytes(&bytes) {
                    Ok(texture) => {
                        picture_clone.set_paintable(Some(&texture));
                        // Hide the "No browser preview" overlay label on first frame
                        if first_frame {
                            first_frame = false;
                            if let Some(overlay) = picture_clone.parent().and_then(|p| p.downcast::<gtk4::Overlay>().ok()) {
                                if let Some(child) = overlay.first_child() {
                                    let mut sibling = child.next_sibling();
                                    while let Some(widget) = sibling {
                                        let next = widget.next_sibling();
                                        if widget.has_css_class("preview-empty") {
                                            widget.set_visible(false);
                                        }
                                        sibling = next;
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        });

        self.preview_state = PreviewState::Streaming;
        Ok(())
    }
}

/// Widgets returned by create_preview_pane for callers to connect signals.
pub struct PreviewPaneWidgets {
    pub container: gtk4::Box,
    pub picture: gtk4::Picture,
    pub url_entry: gtk4::Entry,
    pub back_btn: gtk4::Button,
    pub forward_btn: gtk4::Button,
    pub reload_btn: gtk4::Button,
    pub go_btn: gtk4::Button,
    pub devtools_btn: gtk4::ToggleButton,
    pub pane_id: u64,
    pub uuid: Uuid,
}

/// Create a browser preview pane widget (nav bar + Picture + status overlay).
/// Returns PreviewPaneWidgets so callers can connect button signals.
pub fn create_preview_pane(next_pane_id: u64) -> PreviewPaneWidgets {
    let uuid = Uuid::new_v4();
    let picture = gtk4::Picture::new();
    picture.add_css_class("browser-preview");
    picture.set_can_shrink(true);
    picture.set_hexpand(true);
    picture.set_vexpand(true);

    let overlay = gtk4::Overlay::new();
    overlay.add_css_class("preview-container");
    overlay.set_child(Some(&picture));
    overlay.set_vexpand(true);

    // Empty state label (shown when no stream is active)
    let empty_label = gtk4::Label::new(Some("No browser preview"));
    empty_label.add_css_class("preview-empty");
    empty_label.set_halign(gtk4::Align::Center);
    empty_label.set_valign(gtk4::Align::Center);
    overlay.add_overlay(&empty_label);

    // Navigation bar buttons
    let back_btn = gtk4::Button::with_label("\u{25C0}");
    back_btn.add_css_class("browser-nav-btn");
    back_btn.set_tooltip_text(Some("Back"));

    let forward_btn = gtk4::Button::with_label("\u{25B6}");
    forward_btn.add_css_class("browser-nav-btn");
    forward_btn.set_tooltip_text(Some("Forward"));

    let reload_btn = gtk4::Button::with_label("\u{21BB}");
    reload_btn.add_css_class("browser-nav-btn");
    reload_btn.set_tooltip_text(Some("Reload"));

    // URL entry inside the nav bar
    let url_entry = gtk4::Entry::new();
    url_entry.set_placeholder_text(Some("Enter URL..."));
    url_entry.add_css_class("browser-url-bar");
    url_entry.set_hexpand(true);

    let go_btn = gtk4::Button::with_label("\u{2192}");
    go_btn.add_css_class("browser-nav-btn");
    go_btn.add_css_class("browser-nav-go");
    go_btn.set_tooltip_text(Some("Go"));

    let devtools_btn = gtk4::ToggleButton::with_label("{ }");
    devtools_btn.add_css_class("browser-nav-btn");
    devtools_btn.add_css_class("browser-nav-devtools");
    devtools_btn.set_tooltip_text(Some("Developer Tools"));

    // Navigation bar: horizontal box with buttons + URL entry
    let nav_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    nav_bar.add_css_class("browser-nav-bar");
    nav_bar.append(&back_btn);
    nav_bar.append(&forward_btn);
    nav_bar.append(&reload_btn);
    nav_bar.append(&url_entry);
    nav_bar.append(&go_btn);
    nav_bar.append(&devtools_btn);

    // Vertical box: nav bar on top, picture overlay below
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    vbox.append(&nav_bar);
    vbox.append(&overlay);

    PreviewPaneWidgets {
        container: vbox,
        picture,
        url_entry,
        back_btn,
        forward_btn,
        reload_btn,
        go_btn,
        devtools_btn,
        pane_id: next_pane_id,
        uuid,
    }
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

/// Spawn a tokio task that forwards mouse motion events to the agent-browser daemon.
/// Events are throttled to ~16fps (60ms) to avoid flooding the daemon (D-08).
/// The returned sender can be cloned into the GTK motion controller closure.
pub fn spawn_motion_forwarder(
    runtime: &tokio::runtime::Handle,
    daemon_socket_path: std::path::PathBuf,
) -> tokio::sync::mpsc::UnboundedSender<(i64, i64)> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(i64, i64)>();
    runtime.spawn(async move {
        let mut last_sent = std::time::Instant::now();
        while let Some((x, y)) = rx.recv().await {
            let now = std::time::Instant::now();
            if now.duration_since(last_sent) < std::time::Duration::from_millis(60) {
                continue;
            }
            last_sent = now;
            let path = daemon_socket_path.clone();
            let _ = tokio::task::spawn_blocking(move || {
                use std::io::Write;
                if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(&path) {
                    let req = serde_json::json!({
                        "id": "motion",
                        "action": "input_mouse",
                        "type": "mouseMoved",
                        "x": x,
                        "y": y,
                    });
                    let mut msg = serde_json::to_string(&req).unwrap_or_default();
                    msg.push('\n');
                    let _ = stream.write_all(msg.as_bytes());
                }
            }).await;
        }
    });
    tx
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
