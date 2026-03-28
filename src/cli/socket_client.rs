//! Synchronous Unix socket JSON-RPC client for the cmux CLI.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

/// Errors from CLI socket operations.
#[derive(Debug)]
pub enum CliError {
    /// Could not connect to the socket.
    ConnectionError(String),
    /// The server returned an error response.
    CommandError(String),
    /// Unexpected protocol-level error (malformed response, timeout, etc).
    ProtocolError(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::ConnectionError(msg) => write!(f, "{}", msg),
            CliError::CommandError(msg) => write!(f, "{}", msg),
            CliError::ProtocolError(msg) => write!(f, "{}", msg),
        }
    }
}

/// A synchronous Unix socket JSON-RPC client.
pub struct SocketClient {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
    next_id: u64,
}

impl SocketClient {
    /// Connect to the cmux socket at the given path with the specified timeout.
    pub fn connect(path: &str, timeout: Duration) -> Result<Self, CliError> {
        let stream = UnixStream::connect(path).map_err(|e| {
            CliError::ConnectionError(format!("cannot connect to {}: {}", path, e))
        })?;
        stream.set_read_timeout(Some(timeout)).map_err(|e| {
            CliError::ConnectionError(format!("set_read_timeout: {}", e))
        })?;
        stream.set_write_timeout(Some(timeout)).map_err(|e| {
            CliError::ConnectionError(format!("set_write_timeout: {}", e))
        })?;
        let writer = stream.try_clone().map_err(|e| {
            CliError::ConnectionError(format!("clone stream: {}", e))
        })?;
        Ok(Self {
            reader: BufReader::new(stream),
            writer,
            next_id: 1,
        })
    }

    /// Send a JSON-RPC call and return the result value.
    ///
    /// On success (ok: true), returns the `result` field.
    /// On error (ok: false), returns `Err(CliError::CommandError(...))`.
    pub fn call(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, CliError> {
        let id = self.next_id;
        self.next_id += 1;

        let request = serde_json::json!({
            "id": id,
            "method": method,
            "params": params,
        });

        let mut line = request.to_string();
        line.push('\n');

        self.writer.write_all(line.as_bytes()).map_err(|e| {
            CliError::ProtocolError(format!("write failed: {}", e))
        })?;

        let mut response_line = String::new();
        self.reader.read_line(&mut response_line).map_err(|e| {
            CliError::ProtocolError(format!("read failed: {}", e))
        })?;

        if response_line.is_empty() {
            return Err(CliError::ProtocolError("empty response from server".into()));
        }

        let resp: serde_json::Value = serde_json::from_str(&response_line).map_err(|e| {
            CliError::ProtocolError(format!("invalid JSON response: {}", e))
        })?;

        let ok = resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        if ok {
            Ok(resp.get("result").cloned().unwrap_or(serde_json::Value::Null))
        } else {
            let msg = resp
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            Err(CliError::CommandError(msg.to_string()))
        }
    }
}
