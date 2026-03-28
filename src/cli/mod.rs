//! cmux CLI — clap-based argument parser and command dispatch.
//!
//! This module is entirely independent of GTK4 and the GUI app.
//! It connects to the cmux-app via Unix socket JSON-RPC.

pub mod discovery;
pub mod socket_client;

pub use socket_client::CliError;

use clap::{Parser, Subcommand};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "cmux", about = "Control cmux terminal multiplexer")]
pub struct Cli {
    /// Path to the cmux socket (overrides discovery)
    #[arg(long, global = true, env = "CMUX_SOCKET")]
    socket: Option<String>,

    /// Output raw JSON responses
    #[arg(long, global = true)]
    json: bool,

    /// Verbose output (connection info to stderr)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Color mode: always, never, auto
    #[arg(long, global = true, default_value = "auto")]
    color: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Ping the running cmux instance
    Ping,
    /// Show cmux instance identity (version, platform, pid)
    Identify,
    /// List supported socket commands
    Capabilities,
    /// List all workspaces
    ListWorkspaces,
    /// Show the current workspace
    CurrentWorkspace,
    /// Send an arbitrary JSON-RPC method
    Raw {
        /// The method name (e.g. "workspace.list")
        method: String,
        /// JSON params string
        #[arg(long, default_value = "{}")]
        params: String,
    },

    // -- Workspace management (stubs for Plan 02) --
    /// Create a new workspace
    NewWorkspace,
    /// Select a workspace by ID
    SelectWorkspace {
        /// Workspace UUID
        id: String,
    },
    /// Close a workspace by ID
    CloseWorkspace {
        /// Workspace UUID
        id: String,
    },
    /// Rename a workspace
    RenameWorkspace {
        /// Workspace UUID
        id: String,
        /// New name
        name: String,
    },
    /// Switch to next workspace
    NextWorkspace,
    /// Switch to previous workspace
    PrevWorkspace,
    /// Switch to last active workspace
    LastWorkspace,
    /// Reorder a workspace
    ReorderWorkspace {
        /// Workspace UUID
        id: String,
        /// Target position (0-indexed)
        position: usize,
    },

    // -- Surface commands (stubs for Plan 02) --
    /// List all surfaces
    ListSurfaces,
    /// Split a surface
    Split {
        /// Split direction: horizontal or vertical
        #[arg(long, default_value = "horizontal")]
        direction: String,
        /// Target surface ID (default: focused)
        #[arg(long)]
        id: Option<String>,
    },
    /// Focus a surface by ID
    FocusSurface {
        /// Surface UUID
        id: String,
    },
    /// Close a surface by ID
    CloseSurface {
        /// Surface UUID
        id: String,
    },
    /// Send text to a surface
    SendText {
        /// Text to send
        text: String,
        /// Target surface ID (default: focused)
        #[arg(long)]
        id: Option<String>,
    },
    /// Send a key event to a surface
    SendKey {
        /// Key descriptor
        key: String,
        /// Target surface ID (default: focused)
        #[arg(long)]
        id: Option<String>,
    },
    /// Read text from a surface
    ReadText {
        /// Target surface ID (default: focused)
        #[arg(long)]
        id: Option<String>,
    },
    /// Check surface health
    Health {
        /// Target surface ID (default: focused)
        #[arg(long)]
        id: Option<String>,
    },
    /// Refresh a surface
    Refresh {
        /// Target surface ID (default: focused)
        #[arg(long)]
        id: Option<String>,
    },

    // -- Pane commands (stubs for Plan 02) --
    /// List all panes
    ListPanes,
    /// Focus a pane
    FocusPane {
        /// Pane ID (default: next)
        id: Option<String>,
    },
    /// Switch to last focused pane
    LastPane,

    // -- Window commands (stubs for Plan 02) --
    /// List all windows
    ListWindows,
    /// Show current window info
    CurrentWindow,

    // -- Debug commands (stubs for Plan 02) --
    /// Show layout tree
    Layout,
    /// Type text into the focused terminal
    Type {
        /// Text to type
        text: String,
    },

    // -- Notification commands (stubs for Plan 02) --
    /// List notifications
    ListNotifications,
    /// Clear a notification
    ClearNotification {
        /// Notification ID
        id: String,
    },

    // -- Browser commands (stubs for Plan 02) --
    /// Open a URL in the browser pane
    BrowserOpen {
        /// URL to open
        url: String,
    },
    /// Close the browser pane
    BrowserClose,
    /// Enable browser streaming
    BrowserStreamEnable,
    /// Disable browser streaming
    BrowserStreamDisable,
    /// Take a browser snapshot
    BrowserSnapshot,
    /// Take a browser screenshot
    BrowserScreenshot,
}

/// Run the CLI with the parsed arguments.
pub fn run(cli: Cli) -> Result<(), CliError> {
    // Resolve socket path: --socket flag > discovery > error
    let socket_path = if let Some(ref path) = cli.socket {
        path.clone()
    } else {
        discovery::discover_socket().ok_or_else(|| {
            CliError::ConnectionError(
                "no cmux socket found (is cmux-app running?)".into(),
            )
        })?
    };

    let mut client =
        socket_client::SocketClient::connect(&socket_path, Duration::from_secs(5))?;

    if cli.verbose {
        eprintln!("Connected to {}", socket_path);
    }

    // Handle Raw command separately (dynamic method name)
    let result = if let Commands::Raw { ref method, ref params } = cli.command {
        let params_val: serde_json::Value = serde_json::from_str(params).map_err(|e| {
            CliError::ProtocolError(format!("invalid JSON params: {}", e))
        })?;
        client.call(method, params_val)?
    } else {
        let (method, params) = command_to_rpc(&cli.command);
        client.call(method, params)?
    };

    // Output result
    if cli.json {
        println!("{}", serde_json::to_string(&result).unwrap_or_default());
    } else {
        // For now, pretty-print JSON. Human formatting added in Plan 02.
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );
    }

    Ok(())
}

/// Convert a CLI command to a JSON-RPC method and params.
/// Raw is handled separately in run() — panics if called with Raw.
fn command_to_rpc(cmd: &Commands) -> (&'static str, serde_json::Value) {
    use serde_json::json;
    match cmd {
        Commands::Ping => ("system.ping", json!({})),
        Commands::Identify => ("system.identify", json!({})),
        Commands::Capabilities => ("system.capabilities", json!({})),
        Commands::ListWorkspaces => ("workspace.list", json!({})),
        Commands::CurrentWorkspace => ("workspace.current", json!({})),

        Commands::Raw { .. } => unreachable!("Raw handled separately"),

        Commands::NewWorkspace => ("workspace.create", json!({})),
        Commands::SelectWorkspace { id } => ("workspace.select", json!({"id": id})),
        Commands::CloseWorkspace { id } => ("workspace.close", json!({"id": id})),
        Commands::RenameWorkspace { id, name } => {
            ("workspace.rename", json!({"id": id, "name": name}))
        }
        Commands::NextWorkspace => ("workspace.next", json!({})),
        Commands::PrevWorkspace => ("workspace.previous", json!({})),
        Commands::LastWorkspace => ("workspace.last", json!({})),
        Commands::ReorderWorkspace { id, position } => {
            ("workspace.reorder", json!({"id": id, "position": position}))
        }

        Commands::ListSurfaces => ("surface.list", json!({})),
        Commands::Split { direction, id } => {
            ("surface.split", json!({"direction": direction, "id": id}))
        }
        Commands::FocusSurface { id } => ("surface.focus", json!({"id": id})),
        Commands::CloseSurface { id } => ("surface.close", json!({"id": id})),
        Commands::SendText { text, id } => {
            ("surface.send_text", json!({"text": text, "id": id}))
        }
        Commands::SendKey { key, id } => {
            ("surface.send_key", json!({"key": key, "id": id}))
        }
        Commands::ReadText { id } => ("surface.read_text", json!({"id": id})),
        Commands::Health { id } => ("surface.health", json!({"id": id})),
        Commands::Refresh { id } => ("surface.refresh", json!({"id": id})),

        Commands::ListPanes => ("pane.list", json!({})),
        Commands::FocusPane { id } => ("pane.focus", json!({"id": id})),
        Commands::LastPane => ("pane.last", json!({})),

        Commands::ListWindows => ("window.list", json!({})),
        Commands::CurrentWindow => ("window.current", json!({})),

        Commands::Layout => ("debug.layout", json!({})),
        Commands::Type { text } => ("debug.type", json!({"text": text})),

        Commands::ListNotifications => ("notification.list", json!({})),
        Commands::ClearNotification { id } => {
            ("notification.clear", json!({"id": id}))
        }

        Commands::BrowserOpen { url } => ("browser.open", json!({"url": url})),
        Commands::BrowserClose => ("browser.close", json!({})),
        Commands::BrowserStreamEnable => ("browser.stream.enable", json!({})),
        Commands::BrowserStreamDisable => ("browser.stream.disable", json!({})),
        Commands::BrowserSnapshot => ("browser.snapshot", json!({})),
        Commands::BrowserScreenshot => ("browser.screenshot", json!({})),
    }
}
