use crate::ssh::{SshEvent, SshEventTx};
use crate::workspace::ConnectionState;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Maximum reconnection backoff delay.
const MAX_BACKOFF_SECS: u64 = 30;

/// Manage an SSH workspace connection lifecycle.
/// Runs as a tokio task. Reports state changes via ssh_tx.
pub async fn run_ssh_lifecycle(
    workspace_id: u64,
    target: String,
    ssh_tx: SshEventTx,
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
                attempt = 0; // Reset on successful connection
                let _ = ssh_tx.send(SshEvent::StateChanged {
                    workspace_id,
                    state: ConnectionState::Connected,
                });

                // Wire JSON-RPC protocol over SSH stdin/stdout to cmuxd-remote.
                // The cmuxd-remote daemon speaks JSON-RPC over stdio:
                //   - proxy.open: opens a terminal session on the remote host
                //   - proxy.stream: bidirectional data for terminal I/O
                let stdin = child.stdin.take();
                let stdout = child.stdout.take();

                if let (Some(mut writer), Some(reader)) = (stdin, stdout) {
                    // Send hello/handshake to verify cmuxd-remote is running
                    let hello = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"system.hello","params":{}});
                    let hello_line = format!("{}\n", hello);
                    if let Err(e) = writer.write_all(hello_line.as_bytes()).await {
                        eprintln!("cmux: SSH handshake write failed: {e}");
                    } else {
                        // Read responses from cmuxd-remote
                        let mut buf_reader = BufReader::new(reader);
                        let mut line = String::new();
                        loop {
                            line.clear();
                            match buf_reader.read_line(&mut line).await {
                                Ok(0) => break, // EOF — SSH connection closed
                                Ok(_) => {
                                    // Parse JSON-RPC response/notification from cmuxd-remote.
                                    // TODO: Route proxy.stream data to terminal surfaces
                                    // and proxy.open responses to create remote shells.
                                    // For Phase 4 MVP, log and continue.
                                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                                        eprintln!("cmux: SSH RPC recv: {}", msg);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("cmux: SSH stdout read error: {e}");
                                    break;
                                }
                            }
                        }
                    }
                }

                // Wait for SSH process to exit
                let exit_status = child.wait().await;
                eprintln!("cmux: SSH to {target} exited: {exit_status:?}");

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

        // Exponential backoff before reconnect (per D-14)
        let backoff = backoff_duration(attempt);
        eprintln!("cmux: SSH reconnecting to {target} in {}s (attempt {})", backoff.as_secs(), attempt + 1);
        tokio::time::sleep(backoff).await;
        attempt += 1;
    }
}

/// Start an SSH process with cmuxd-remote in stdio mode.
async fn start_ssh(target: &str) -> Result<Child, String> {
    let child = Command::new("ssh")
        .args([
            "-o", "ServerAliveInterval=15",
            "-o", "ServerAliveCountMax=3",
            "-o", "ConnectTimeout=10",
            "-o", "BatchMode=yes",
            target,
            ".local/bin/cmuxd-remote", "serve", "--stdio",
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
