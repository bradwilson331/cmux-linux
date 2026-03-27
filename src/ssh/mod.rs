pub mod bridge;
pub mod deploy;
pub mod tunnel;

use crate::workspace::ConnectionState;
use tokio::sync::mpsc;

/// Message from SSH task to GTK main thread.
pub enum SshEvent {
    /// Connection state changed for a workspace.
    StateChanged {
        workspace_id: u64,
        state: ConnectionState,
    },
    /// Output data from remote shell -- dispatch to surface on GTK main thread.
    RemoteOutput { pane_id: u64, data: Vec<u8> },
    /// Remote shell exited (proxy.stream.eof).
    RemoteEof { pane_id: u64 },
    /// Stream opened successfully -- pane can start receiving I/O.
    StreamOpened {
        pane_id: u64,
        stream_id: String,
    },
}

/// Sender for SSH events (cloned into tokio tasks).
pub type SshEventTx = mpsc::UnboundedSender<SshEvent>;
/// Receiver for SSH events on the GTK main thread.
pub type SshEventRx = mpsc::UnboundedReceiver<SshEvent>;
