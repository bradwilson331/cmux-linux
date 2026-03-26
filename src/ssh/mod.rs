pub mod tunnel;
pub mod deploy;

use crate::workspace::ConnectionState;
use tokio::sync::mpsc;

/// Message from SSH task to GTK main thread.
pub enum SshEvent {
    /// Connection state changed for a workspace.
    StateChanged { workspace_id: u64, state: ConnectionState },
}

/// Sender for SSH events (cloned into tokio tasks).
pub type SshEventTx = mpsc::UnboundedSender<SshEvent>;
/// Receiver for SSH events on the GTK main thread.
pub type SshEventRx = mpsc::UnboundedReceiver<SshEvent>;
