use base64::Engine;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Per-pane stream state tracking.
pub struct PaneStream {
    pub stream_id: String,
    pub subscribed: bool,
}

/// Request to write data through the SSH tunnel to a specific stream.
pub struct WriteRequest {
    pub stream_id: String,
    pub data_base64: String,
}

/// Output data from remote shell to be dispatched to GTK main thread.
pub struct OutputEvent {
    pub pane_id: u64,
    pub data: Vec<u8>,
}

/// Manages the mapping between local panes and remote proxy streams.
pub struct SshBridge {
    /// Maps pane_id -> stream state
    pub streams: Arc<Mutex<HashMap<u64, PaneStream>>>,
    /// Maps stream_id -> pane_id (reverse lookup for incoming events)
    pub stream_to_pane: Arc<Mutex<HashMap<String, u64>>>,
    /// Channel to send write requests to the SSH tunnel task (swappable for reconnect)
    pub write_tx: Arc<Mutex<mpsc::UnboundedSender<WriteRequest>>>,
    /// Receiver side of the write channel (taken by run_proxy_routing)
    write_rx: Mutex<Option<mpsc::UnboundedReceiver<WriteRequest>>>,
    /// Atomic counter for JSON-RPC request IDs
    pub next_rpc_id: Arc<AtomicU64>,
    /// Channel for output events to GTK main thread
    pub output_tx: mpsc::UnboundedSender<OutputEvent>,
}

impl SshBridge {
    pub fn new(
        write_tx: mpsc::UnboundedSender<WriteRequest>,
        write_rx: mpsc::UnboundedReceiver<WriteRequest>,
        output_tx: mpsc::UnboundedSender<OutputEvent>,
    ) -> Self {
        Self {
            streams: Arc::new(Mutex::new(HashMap::new())),
            stream_to_pane: Arc::new(Mutex::new(HashMap::new())),
            write_tx: Arc::new(Mutex::new(write_tx)),
            write_rx: Mutex::new(Some(write_rx)),
            next_rpc_id: Arc::new(AtomicU64::new(10)), // Start after handshake IDs
            output_tx,
        }
    }

    /// Take the write receiver for use in the proxy routing loop.
    /// On reconnect, creates a fresh channel pair and swaps the sender.
    pub fn take_or_recreate_write_rx(&self) -> mpsc::UnboundedReceiver<WriteRequest> {
        let mut rx_guard = self.write_rx.lock().unwrap();
        if let Some(rx) = rx_guard.take() {
            return rx;
        }
        // Reconnect case: old rx was consumed. Create fresh channel.
        let (new_tx, new_rx) = mpsc::unbounded_channel();
        *self.write_tx.lock().unwrap() = new_tx;
        new_rx
    }

    /// Clear all stream state (for reconnect -- old streams are stale).
    pub fn clear_stream_ids(&self) {
        if let Ok(mut streams) = self.streams.lock() {
            for ps in streams.values_mut() {
                ps.stream_id.clear();
                ps.subscribed = false;
            }
        }
        if let Ok(mut s2p) = self.stream_to_pane.lock() {
            s2p.clear();
        }
    }

    /// Clone the current write sender (for IoWriteContext creation).
    pub fn clone_write_tx(&self) -> mpsc::UnboundedSender<WriteRequest> {
        self.write_tx.lock().unwrap().clone()
    }

    /// Register a new pane with its stream_id after proxy.open succeeds.
    pub fn register_pane(&self, pane_id: u64, stream_id: String) {
        if let Ok(mut streams) = self.streams.lock() {
            streams.insert(
                pane_id,
                PaneStream {
                    stream_id: stream_id.clone(),
                    subscribed: false,
                },
            );
        }
        if let Ok(mut s2p) = self.stream_to_pane.lock() {
            s2p.insert(stream_id, pane_id);
        }
    }

    /// Mark a pane's stream as subscribed.
    pub fn mark_subscribed(&self, pane_id: u64) {
        if let Ok(mut streams) = self.streams.lock() {
            if let Some(ps) = streams.get_mut(&pane_id) {
                ps.subscribed = true;
            }
        }
    }

    /// Remove a pane's stream mapping (on close or EOF).
    pub fn remove_pane(&self, pane_id: u64) {
        let stream_id = if let Ok(mut streams) = self.streams.lock() {
            streams.remove(&pane_id).map(|ps| ps.stream_id)
        } else {
            None
        };
        if let Some(sid) = stream_id {
            if let Ok(mut s2p) = self.stream_to_pane.lock() {
                s2p.remove(&sid);
            }
        }
    }

    /// Get the next JSON-RPC request ID.
    pub fn next_id(&self) -> u64 {
        self.next_rpc_id.fetch_add(1, Ordering::SeqCst)
    }
}

/// Context passed as userdata to the Ghostty io_write_cb callback.
/// Must be allocated with Arc and leaked via Arc::into_raw for the C callback.
pub struct IoWriteContext {
    pub pane_id: u64,
    pub write_tx: mpsc::UnboundedSender<WriteRequest>,
    /// Set after proxy.open returns the stream_id.
    pub stream_id: Mutex<Option<String>>,
}

/// C-compatible callback invoked by Ghostty when user types in a manual-mode surface.
/// Signature: void(*)(void* userdata, const char* data, uintptr_t len)
///
/// SAFETY: userdata must be a valid Arc<IoWriteContext> pointer created via Arc::into_raw.
/// This callback runs on the GTK main thread (same thread as key events).
pub unsafe extern "C" fn ssh_io_write_cb(
    userdata: *mut std::ffi::c_void,
    data: *const std::ffi::c_char,
    len: usize,
) {
    if userdata.is_null() || data.is_null() || len == 0 {
        return;
    }
    // SAFETY: userdata is an Arc<IoWriteContext> pointer -- we borrow without taking ownership.
    let ctx = &*(userdata as *const IoWriteContext);
    let bytes = std::slice::from_raw_parts(data as *const u8, len);
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    if let Some(ref stream_id) = *ctx.stream_id.lock().unwrap() {
        let _ = ctx.write_tx.send(WriteRequest {
            stream_id: stream_id.clone(),
            data_base64: b64,
        });
    }
}
