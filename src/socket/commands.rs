use serde_json::Value;

/// Response channel for commands that return data to the socket client.
pub type RespTx = tokio::sync::oneshot::Sender<Value>;

/// Commands dispatched from tokio accept loop to GTK main thread via glib::MainContext::channel.
/// Each variant carries a `resp_tx` oneshot sender so the tokio connection handler can await
/// the result produced on the main thread.
#[allow(dead_code)]
pub enum SocketCommand {
    Ping { req_id: Value, resp_tx: RespTx },
    NotImplemented { req_id: Value, method: String, resp_tx: RespTx },
}
