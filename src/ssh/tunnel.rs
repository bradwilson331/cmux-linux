use base64::Engine;
use crate::ssh::bridge::{SshBridge, WriteRequest};
use crate::ssh::{SshEvent, SshEventTx};
use crate::workspace::ConnectionState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, Command};
use tokio::sync::oneshot;

/// Maximum reconnection backoff delay.
const MAX_BACKOFF_SECS: u64 = 30;

/// Maximum number of reconnection attempts before giving up.
const MAX_RETRIES: u32 = 10;

/// Whether a failure is permanent (no point retrying) or transient (retry with backoff).
enum FailureKind {
    Permanent,
    Transient,
}

/// Pending RPC responses awaiting completion.
type PendingMap = Arc<Mutex<HashMap<u64, oneshot::Sender<serde_json::Value>>>>;

/// Manage an SSH workspace connection lifecycle.
/// Runs as a tokio task. Reports state changes via ssh_tx.
pub async fn run_ssh_lifecycle(
    workspace_id: u64,
    target: String,
    ssh_tx: SshEventTx,
    bridge: Arc<SshBridge>,
) {
    let mut attempt: u32 = 0;

    loop {
        // Update state to reconnecting
        let _ = ssh_tx.send(SshEvent::StateChanged {
            workspace_id,
            state: ConnectionState::Reconnecting(attempt),
        });

        // Deploy if first attempt
        if attempt == 0 {
            if let Err(e) = crate::ssh::deploy::deploy_remote(&target).await {
                eprintln!("cmux: SSH deploy failed: {e}");

                // Classify: binary-not-found is permanent, everything else is transient
                let kind = if e.contains("not found at") {
                    FailureKind::Permanent
                } else {
                    FailureKind::Transient
                };

                if matches!(kind, FailureKind::Permanent) {
                    let _ = ssh_tx.send(SshEvent::StateChanged {
                        workspace_id,
                        state: ConnectionState::Disconnected,
                    });
                    eprintln!("cmux: SSH permanent failure, giving up: {e}");
                    break;
                }

                let _ = ssh_tx.send(SshEvent::StateChanged {
                    workspace_id,
                    state: ConnectionState::Disconnected,
                });
                let backoff = backoff_duration(attempt);
                tokio::time::sleep(backoff).await;
                attempt += 1;
                continue;
            }
        }

        // Start SSH connection with cmuxd-remote in stdio mode
        match start_ssh(&target).await {
            Ok(mut child) => {
                let was_reconnect = attempt > 0;
                attempt = 0; // Reset on successful connection
                let _ = ssh_tx.send(SshEvent::StateChanged {
                    workspace_id,
                    state: ConnectionState::Connected,
                });

                // D-07: inject reconnect message if this was a reconnection
                if was_reconnect {
                    if let Ok(streams) = bridge.streams.lock() {
                        for (&pane_id, _) in streams.iter() {
                            let msg = b"\r\n\x1b[32m[Reconnected \xe2\x80\x94 new session]\x1b[0m\r\n";
                            let _ = ssh_tx.send(SshEvent::RemoteOutput {
                                pane_id,
                                data: msg.to_vec(),
                            });
                        }
                    }
                }

                let stdin = child.stdin.take();
                let stdout = child.stdout.take();

                if let (Some(writer), Some(reader)) = (stdin, stdout) {
                    let mut buf_writer = BufWriter::new(writer);

                    // Send hello/handshake to verify cmuxd-remote is running
                    let hello = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"hello","params":{}});
                    let hello_line = format!("{}\n", hello);
                    if let Err(e) = buf_writer.write_all(hello_line.as_bytes()).await {
                        eprintln!("cmux: SSH handshake write failed: {e}");
                    } else if let Err(e) = buf_writer.flush().await {
                        eprintln!("cmux: SSH handshake flush failed: {e}");
                    } else {
                        // Run bidirectional proxy routing
                        run_proxy_routing(
                            buf_writer,
                            reader,
                            &bridge,
                            &ssh_tx,
                        )
                        .await;
                    }
                }

                // Wait for SSH process to exit
                let exit_status = child.wait().await;
                eprintln!("cmux: SSH to {target} exited: {exit_status:?}");

                // D-06: inject disconnect message into all active panes
                if let Ok(streams) = bridge.streams.lock() {
                    for (&pane_id, _) in streams.iter() {
                        let msg = b"\r\n\x1b[33m[SSH disconnected \xe2\x80\x94 reconnecting...]\x1b[0m\r\n";
                        let _ = ssh_tx.send(SshEvent::RemoteOutput {
                            pane_id,
                            data: msg.to_vec(),
                        });
                    }
                }

                let _ = ssh_tx.send(SshEvent::StateChanged {
                    workspace_id,
                    state: ConnectionState::Disconnected,
                });
            }
            Err(e) => {
                eprintln!("cmux: SSH connection to {target} failed: {e}");
                let _ = ssh_tx.send(SshEvent::StateChanged {
                    workspace_id,
                    state: ConnectionState::Disconnected,
                });
            }
        }

        if attempt >= MAX_RETRIES {
            eprintln!("cmux: SSH max retries ({MAX_RETRIES}) exceeded for {target}, giving up");
            let _ = ssh_tx.send(SshEvent::StateChanged {
                workspace_id,
                state: ConnectionState::Disconnected,
            });
            break;
        }

        // Exponential backoff before reconnect (per D-14)
        let backoff = backoff_duration(attempt);
        eprintln!(
            "cmux: SSH reconnecting to {target} in {}s (attempt {})",
            backoff.as_secs(),
            attempt + 1
        );
        tokio::time::sleep(backoff).await;
        attempt += 1;
    }
}

/// Bidirectional proxy routing between bridge channels and SSH stdin/stdout.
async fn run_proxy_routing(
    buf_writer: BufWriter<tokio::process::ChildStdin>,
    reader: tokio::process::ChildStdout,
    bridge: &Arc<SshBridge>,
    ssh_tx: &SshEventTx,
) {
    let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
    let writer = Arc::new(tokio::sync::Mutex::new(buf_writer));

    // Take (or recreate on reconnect) the write channel receiver from bridge
    let mut local_write_rx = bridge.take_or_recreate_write_rx();

    // Clear stale stream state from any prior connection
    bridge.clear_stream_ids();

    // Open remote streams for all registered panes
    {
        let pane_ids: Vec<u64> = bridge.streams.lock()
            .map(|s| s.keys().copied().collect())
            .unwrap_or_default();
        for pane_id in pane_ids {
            match open_remote_stream(&writer, bridge, pane_id, &pending, ssh_tx, 80, 24).await {
                Ok(stream_id) => {
                    eprintln!("cmux: opened remote stream {stream_id} for pane {pane_id}");
                }
                Err(e) => {
                    eprintln!("cmux: failed to open remote stream for pane {pane_id}: {e}");
                }
            }
        }
    }

    // Read path: parse JSON lines from SSH stdout
    let read_bridge = bridge.clone();
    let read_ssh_tx = ssh_tx.clone();
    let read_pending = pending.clone();
    let read_handle = tokio::spawn(async move {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            match buf_reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                        handle_incoming_message(&msg, &read_bridge, &read_ssh_tx, &read_pending);
                    }
                }
                Err(e) => {
                    eprintln!("cmux: SSH stdout read error: {e}");
                    break;
                }
            }
        }
    });

    // Write path: consume WriteRequests and send as JSON-RPC to SSH stdin
    let write_writer = writer.clone();
    let write_bridge = bridge.clone();
    let write_handle = tokio::spawn(async move {
        while let Some(req) = local_write_rx.recv().await {
            let rpc_id = write_bridge.next_id();
            let rpc = serde_json::json!({
                "jsonrpc": "2.0",
                "id": rpc_id,
                "method": "proxy.write",
                "params": {
                    "stream_id": req.stream_id,
                    "data_base64": req.data_base64,
                }
            });
            let line = format!("{}\n", rpc);
            let mut w = write_writer.lock().await;
            if w.write_all(line.as_bytes()).await.is_err() {
                break;
            }
            if w.flush().await.is_err() {
                break;
            }
        }
    });

    // Wait for read path to finish (SSH connection closed)
    let _ = read_handle.await;
    // Cancel write path
    write_handle.abort();
}

/// Handle an incoming JSON message from cmuxd-remote.
fn handle_incoming_message(
    msg: &serde_json::Value,
    bridge: &SshBridge,
    ssh_tx: &SshEventTx,
    pending: &PendingMap,
) {
    // Check if it's an async event (has "event" field)
    if let Some(event_name) = msg.get("event").and_then(|v| v.as_str()) {
        let stream_id = msg
            .get("stream_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match event_name {
            "proxy.stream.data" => {
                if let Some(data_b64) = msg.get("data_base64").and_then(|v| v.as_str()) {
                    if let Ok(data) = base64::engine::general_purpose::STANDARD.decode(data_b64) {
                        // Look up pane_id from stream_id
                        if let Ok(s2p) = bridge.stream_to_pane.lock() {
                            if let Some(&pane_id) = s2p.get(stream_id) {
                                let _ = ssh_tx.send(SshEvent::RemoteOutput { pane_id, data });
                            }
                        }
                    }
                }
            }
            "proxy.stream.eof" => {
                if let Ok(s2p) = bridge.stream_to_pane.lock() {
                    if let Some(&pane_id) = s2p.get(stream_id) {
                        let _ = ssh_tx.send(SshEvent::RemoteEof { pane_id });
                        drop(s2p);
                        bridge.remove_pane(pane_id);
                    }
                }
            }
            "proxy.stream.error" => {
                let error = msg
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                eprintln!("cmux: proxy stream error for {stream_id}: {error}");
                // Treat like EOF
                if let Ok(s2p) = bridge.stream_to_pane.lock() {
                    if let Some(&pane_id) = s2p.get(stream_id) {
                        let _ = ssh_tx.send(SshEvent::RemoteEof { pane_id });
                        drop(s2p);
                        bridge.remove_pane(pane_id);
                    }
                }
            }
            _ => {
                eprintln!("cmux: unknown SSH event: {event_name}");
            }
        }
        return;
    }

    // Check if it's an RPC response (has "id" field)
    if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
        if let Ok(mut map) = pending.lock() {
            if let Some(tx) = map.remove(&id) {
                let _ = tx.send(msg.clone());
            }
        }
    }
}

/// Open a remote PTY stream for a pane via session.spawn + proxy.stream.subscribe.
pub async fn open_remote_stream(
    writer: &Arc<tokio::sync::Mutex<BufWriter<tokio::process::ChildStdin>>>,
    bridge: &SshBridge,
    pane_id: u64,
    pending: &PendingMap,
    ssh_tx: &SshEventTx,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    // Send session.spawn RPC
    let spawn_id = bridge.next_id();
    let spawn_rpc = serde_json::json!({
        "jsonrpc": "2.0",
        "id": spawn_id,
        "method": "session.spawn",
        "params": {
            "cols": cols,
            "rows": rows,
        }
    });

    // Register oneshot for response
    let (resp_tx, resp_rx) = oneshot::channel();
    if let Ok(mut map) = pending.lock() {
        map.insert(spawn_id, resp_tx);
    }

    // Write RPC
    {
        let line = format!("{}\n", spawn_rpc);
        let mut w = writer.lock().await;
        w.write_all(line.as_bytes())
            .await
            .map_err(|e| format!("write session.spawn failed: {e}"))?;
        w.flush()
            .await
            .map_err(|e| format!("flush session.spawn failed: {e}"))?;
    }

    // Await response
    let resp = resp_rx
        .await
        .map_err(|_| "session.spawn response channel dropped".to_string())?;

    let stream_id = resp
        .get("result")
        .and_then(|r| r.get("stream_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            let err_msg = resp
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            format!("session.spawn failed: {err_msg}")
        })?
        .to_string();

    // Subscribe to the stream
    let sub_id = bridge.next_id();
    let sub_rpc = serde_json::json!({
        "jsonrpc": "2.0",
        "id": sub_id,
        "method": "proxy.stream.subscribe",
        "params": {
            "stream_id": &stream_id,
        }
    });

    let (sub_tx, sub_rx) = oneshot::channel();
    if let Ok(mut map) = pending.lock() {
        map.insert(sub_id, sub_tx);
    }

    {
        let line = format!("{}\n", sub_rpc);
        let mut w = writer.lock().await;
        w.write_all(line.as_bytes())
            .await
            .map_err(|e| format!("write proxy.stream.subscribe failed: {e}"))?;
        w.flush()
            .await
            .map_err(|e| format!("flush proxy.stream.subscribe failed: {e}"))?;
    }

    // Await subscribe response
    let _sub_resp = sub_rx
        .await
        .map_err(|_| "proxy.stream.subscribe response channel dropped".to_string())?;

    // Register in bridge
    bridge.register_pane(pane_id, stream_id.clone());
    bridge.mark_subscribed(pane_id);

    // Notify via SSH event
    let _ = ssh_tx.send(SshEvent::StreamOpened {
        pane_id,
        stream_id: stream_id.clone(),
    });

    Ok(stream_id)
}

/// Start an SSH process with cmuxd-remote in stdio mode.
async fn start_ssh(target: &str) -> Result<Child, String> {
    let child = Command::new("ssh")
        .args([
            "-o",
            "ServerAliveInterval=15",
            "-o",
            "ServerAliveCountMax=3",
            "-o",
            "ConnectTimeout=10",
            "-o",
            "BatchMode=yes",
            target,
            ".local/bin/cmuxd-remote",
            "serve",
            "--stdio",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn ssh: {e}"))?;

    Ok(child)
}

/// Calculate exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s cap (per D-14).
fn backoff_duration(attempt: u32) -> Duration {
    let secs = (1u64 << attempt.min(5)).min(MAX_BACKOFF_SECS);
    Duration::from_secs(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_retries_is_reasonable() {
        assert!(MAX_RETRIES >= 5, "too few retries");
        assert!(MAX_RETRIES <= 20, "too many retries");
    }

    #[test]
    fn test_backoff_duration() {
        assert_eq!(backoff_duration(0), Duration::from_secs(1));
        assert_eq!(backoff_duration(1), Duration::from_secs(2));
        assert_eq!(backoff_duration(2), Duration::from_secs(4));
        assert_eq!(backoff_duration(3), Duration::from_secs(8));
        assert_eq!(backoff_duration(4), Duration::from_secs(16));
        assert_eq!(backoff_duration(5), Duration::from_secs(30)); // capped
        assert_eq!(backoff_duration(10), Duration::from_secs(30)); // still capped
    }
}
