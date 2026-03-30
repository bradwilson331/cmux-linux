//! cmux CLI — clap-based argument parser and command dispatch.
//!
//! This module is entirely independent of GTK4 and the GUI app.
//! It connects to the cmux-app via Unix socket JSON-RPC.

pub mod discovery;
pub mod format;
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

    /// Suppress JSON output for browser commands (browser defaults to JSON)
    #[arg(long, global = true)]
    no_json: bool,

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

    // -- Workspace management --
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

    // -- Surface commands --
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

    // -- Pane commands --
    /// List all panes
    ListPanes,
    /// Focus a pane
    FocusPane {
        /// Pane ID (default: next)
        id: Option<String>,
    },
    /// Switch to last focused pane
    LastPane,

    // -- Window commands --
    /// List all windows
    ListWindows,
    /// Show current window info
    CurrentWindow,

    // -- Debug commands --
    /// Show layout tree
    Layout,
    /// Type text into the focused terminal
    Type {
        /// Text to type
        text: String,
    },

    // -- Notification commands --
    /// List notifications
    ListNotifications,
    /// Clear a notification
    ClearNotification {
        /// Notification ID
        id: String,
    },

    // -- Browser subcommand group (agent primary interface) --
    /// Browser automation (agent primary interface)
    #[command(subcommand)]
    Browser(BrowserCommand),

    // -- Legacy flat browser commands (hidden, backward compat) --
    /// Open a URL in the browser pane
    #[command(hide = true)]
    BrowserOpen {
        /// URL to open
        url: String,
    },
    /// Close the browser pane
    #[command(hide = true)]
    BrowserClose,
    /// Enable browser streaming
    #[command(hide = true)]
    BrowserStreamEnable,
    /// Disable browser streaming
    #[command(hide = true)]
    BrowserStreamDisable,
    /// Take a browser snapshot (accessibility tree / DOM text)
    #[command(hide = true)]
    BrowserSnapshot,
    /// Take a browser screenshot (base64 PNG)
    #[command(hide = true)]
    BrowserScreenshot,

    // -- Legacy browser navigation (hidden) --
    /// Navigate to a URL
    #[command(hide = true)]
    BrowserGoto {
        /// URL to navigate to
        url: String,
    },
    /// Go back in browser history
    #[command(hide = true)]
    BrowserBack,
    /// Go forward in browser history
    #[command(hide = true)]
    BrowserForward,
    /// Reload the current page
    #[command(hide = true)]
    BrowserReload,

    // -- Legacy browser interaction (hidden) --
    /// Click an element by CSS selector
    #[command(hide = true)]
    BrowserClick {
        /// CSS selector of the element to click
        selector: String,
    },
    /// Type text into an element
    #[command(hide = true)]
    BrowserType {
        /// CSS selector of the element
        selector: String,
        /// Text to type
        text: String,
    },
    /// Fill an input field (clears first, then types)
    #[command(hide = true)]
    BrowserFill {
        /// CSS selector of the element
        selector: String,
        /// Value to fill
        value: String,
    },
    /// Press a key (e.g. "Enter", "Tab", "Escape")
    #[command(hide = true)]
    BrowserPress {
        /// Key name
        key: String,
    },
    /// Hover over an element
    #[command(hide = true)]
    BrowserHover {
        /// CSS selector of the element
        selector: String,
    },
    /// Scroll the page
    #[command(hide = true)]
    BrowserScroll {
        /// Direction: up, down, left, right
        direction: String,
        /// Amount in pixels (default: 300)
        #[arg(long, default_value = "300")]
        amount: i32,
    },
    /// Select an option from a dropdown
    #[command(hide = true)]
    BrowserSelect {
        /// CSS selector of the select element
        selector: String,
        /// Value to select
        value: String,
    },
    /// Evaluate JavaScript in the browser
    #[command(hide = true)]
    BrowserEval {
        /// JavaScript expression to evaluate
        expression: String,
    },
    /// Wait for a selector to appear
    #[command(hide = true)]
    BrowserWait {
        /// CSS selector to wait for
        selector: String,
        /// Timeout in milliseconds
        #[arg(long, default_value = "5000")]
        timeout: u64,
    },

    // -- Legacy browser getters (hidden) --
    /// Get the current page URL
    #[command(hide = true)]
    BrowserGetUrl,
    /// Get the current page title
    #[command(hide = true)]
    BrowserGetTitle,
    /// Get text content of an element
    #[command(hide = true)]
    BrowserGetText {
        /// CSS selector of the element
        selector: String,
    },
    /// Get HTML content of an element
    #[command(hide = true)]
    BrowserGetHtml {
        /// CSS selector of the element
        selector: String,
    },
}

/// Browser subcommands for `cmux browser <action>` / `cmux browser <surface> <action>`.
#[derive(Subcommand)]
pub enum BrowserCommand {
    /// Open a URL in the browser pane
    Open {
        /// URL to open
        url: String,
        /// Target workspace ID
        #[arg(long)]
        workspace: Option<String>,
    },
    /// List browser surfaces
    List,
    /// Close browser surface(s)
    Close {
        /// Surface reference (surface:N or UUID); closes all if omitted
        #[arg(long)]
        surface: Option<String>,
    },
    /// Take a browser snapshot (accessibility tree / DOM text)
    Snapshot {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// Include interactive element annotations
        #[arg(long)]
        interactive: bool,
        /// Compact output
        #[arg(long)]
        compact: bool,
        /// Maximum depth
        #[arg(long)]
        max_depth: Option<u32>,
    },
    /// Click an element
    Click {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// Target element (e1 or CSS selector)
        target: String,
        /// Take snapshot after action
        #[arg(long)]
        snapshot_after: bool,
    },
    /// Fill an input field (clears first, then types)
    Fill {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// Target element (CSS selector)
        target: String,
        /// Value to fill
        text: String,
        /// Take snapshot after action
        #[arg(long)]
        snapshot_after: bool,
    },
    /// Type text into an element
    #[command(name = "type")]
    BrowserType {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// CSS selector of the element
        selector: String,
        /// Text to type
        text: String,
    },
    /// Press a key (e.g. "Enter", "Tab", "Escape")
    Press {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// Key name
        key: String,
    },
    /// Hover over an element
    Hover {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// CSS selector of the element
        selector: String,
    },
    /// Scroll the page
    Scroll {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// Direction: up, down, left, right
        direction: String,
        /// Amount in pixels
        #[arg(long, default_value = "300")]
        amount: i32,
    },
    /// Select an option from a dropdown
    #[command(name = "select")]
    Select {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// CSS selector of the select element
        selector: String,
        /// Value to select
        value: String,
    },
    /// Evaluate JavaScript in the browser
    Eval {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// JavaScript expression to evaluate
        expression: String,
    },
    /// Wait for a condition
    Wait {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// CSS selector to wait for
        #[arg(long)]
        selector: Option<String>,
        /// Text to wait for
        #[arg(long)]
        text: Option<String>,
        /// URL substring to wait for
        #[arg(long)]
        url_contains: Option<String>,
        /// Load state to wait for
        #[arg(long)]
        load_state: Option<String>,
        /// JavaScript function to wait for
        #[arg(long)]
        function: Option<String>,
        /// Timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout_ms: u64,
    },
    /// Navigate to a URL
    Goto {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// URL to navigate to
        url: String,
    },
    /// Go back in browser history
    Back {
        /// Surface reference (surface:N or UUID)
        surface: String,
    },
    /// Go forward in browser history
    Forward {
        /// Surface reference (surface:N or UUID)
        surface: String,
    },
    /// Reload the current page
    Reload {
        /// Surface reference (surface:N or UUID)
        surface: String,
    },
    /// Get the current page URL
    #[command(name = "get-url")]
    GetUrl {
        /// Surface reference (surface:N or UUID)
        surface: String,
    },
    /// Get the current page title
    #[command(name = "get-title")]
    GetTitle {
        /// Surface reference (surface:N or UUID)
        surface: String,
    },
    /// Get text content of an element
    #[command(name = "get-text")]
    GetText {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// CSS selector of the element
        selector: String,
    },
    /// Get HTML content of an element
    #[command(name = "get-html")]
    GetHtml {
        /// Surface reference (surface:N or UUID)
        surface: String,
        /// CSS selector of the element
        selector: String,
    },
    /// Take a browser screenshot (base64 PNG)
    Screenshot {
        /// Surface reference (surface:N or UUID)
        surface: String,
    },
    /// Enable browser streaming
    #[command(name = "stream-enable")]
    StreamEnable,
    /// Disable browser streaming
    #[command(name = "stream-disable")]
    StreamDisable,
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

    // Use longer timeout for browser wait commands
    let timeout = match &cli.command {
        Commands::Browser(BrowserCommand::Wait { timeout_ms, .. }) => {
            Duration::from_millis(timeout_ms + 5000)
        }
        _ => Duration::from_secs(5),
    };

    let mut client =
        socket_client::SocketClient::connect(&socket_path, timeout)?;

    if cli.verbose {
        eprintln!("Connected to {}", socket_path);
    }

    let use_color = format::use_color(&cli.color);

    // Handle Raw command separately (dynamic method name)
    let (method_name, result) = if let Commands::Raw { ref method, ref params } = cli.command {
        let params_val: serde_json::Value = serde_json::from_str(params).map_err(|e| {
            CliError::ProtocolError(format!("invalid JSON params: {}", e))
        })?;
        let result = client.call(method, params_val)?;
        (method.clone(), result)
    } else {
        let (method, params) = command_to_rpc(&cli.command);
        let result = client.call(method, params)?;
        (method.to_string(), result)
    };

    // Browser commands default to JSON; everything else defaults to human-readable
    let json_mode = match &cli.command {
        Commands::Browser(_) => !cli.no_json,
        _ => cli.json,
    };

    // Output formatted result
    let output = format::format_response(&method_name, &result, json_mode, use_color);
    if !output.is_empty() {
        println!("{}", output);
    }

    Ok(())
}

/// Map a BrowserCommand variant to its JSON-RPC method and params.
fn browser_command_to_rpc(cmd: &BrowserCommand) -> (&'static str, serde_json::Value) {
    use serde_json::json;
    match cmd {
        BrowserCommand::Open { url, workspace } => {
            ("browser.open", json!({"url": url, "workspace": workspace}))
        }
        BrowserCommand::List => ("browser.list", json!({})),
        BrowserCommand::Close { surface } => {
            ("browser.close", json!({"surface_ref": surface}))
        }
        BrowserCommand::Snapshot { surface, interactive, compact, max_depth } => {
            ("browser.snapshot", json!({
                "surface_ref": surface,
                "interactive": interactive,
                "compact": compact,
                "max_depth": max_depth
            }))
        }
        BrowserCommand::Click { surface, target, snapshot_after } => {
            ("browser.click", json!({
                "surface_ref": surface,
                "target": target,
                "snapshot_after": snapshot_after
            }))
        }
        BrowserCommand::Fill { surface, target, text, snapshot_after } => {
            ("browser.fill", json!({
                "surface_ref": surface,
                "target": target,
                "text": text,
                "snapshot_after": snapshot_after
            }))
        }
        BrowserCommand::BrowserType { surface, selector, text } => {
            ("browser.type", json!({
                "surface_ref": surface,
                "selector": selector,
                "text": text
            }))
        }
        BrowserCommand::Press { surface, key } => {
            ("browser.press", json!({"surface_ref": surface, "key": key}))
        }
        BrowserCommand::Hover { surface, selector } => {
            ("browser.hover", json!({"surface_ref": surface, "selector": selector}))
        }
        BrowserCommand::Scroll { surface, direction, amount } => {
            ("browser.scroll", json!({
                "surface_ref": surface,
                "direction": direction,
                "amount": amount
            }))
        }
        BrowserCommand::Select { surface, selector, value } => {
            ("browser.select", json!({
                "surface_ref": surface,
                "selector": selector,
                "value": value
            }))
        }
        BrowserCommand::Eval { surface, expression } => {
            ("browser.eval", json!({"surface_ref": surface, "expression": expression}))
        }
        BrowserCommand::Wait { surface, selector, text, url_contains, load_state, function, timeout_ms } => {
            ("browser.wait", json!({
                "surface_ref": surface,
                "selector": selector,
                "text": text,
                "url_contains": url_contains,
                "load_state": load_state,
                "function": function,
                "timeout_ms": timeout_ms
            }))
        }
        BrowserCommand::Goto { surface, url } => {
            ("browser.goto", json!({"surface_ref": surface, "url": url}))
        }
        BrowserCommand::Back { surface } => {
            ("browser.back", json!({"surface_ref": surface}))
        }
        BrowserCommand::Forward { surface } => {
            ("browser.forward", json!({"surface_ref": surface}))
        }
        BrowserCommand::Reload { surface } => {
            ("browser.reload", json!({"surface_ref": surface}))
        }
        BrowserCommand::GetUrl { surface } => {
            ("browser.url", json!({"surface_ref": surface}))
        }
        BrowserCommand::GetTitle { surface } => {
            ("browser.title", json!({"surface_ref": surface}))
        }
        BrowserCommand::GetText { surface, selector } => {
            ("browser.gettext", json!({"surface_ref": surface, "selector": selector}))
        }
        BrowserCommand::GetHtml { surface, selector } => {
            ("browser.gethtml", json!({"surface_ref": surface, "selector": selector}))
        }
        BrowserCommand::Screenshot { surface } => {
            ("browser.screenshot", json!({"surface_ref": surface}))
        }
        BrowserCommand::StreamEnable => ("browser.stream.enable", json!({})),
        BrowserCommand::StreamDisable => ("browser.stream.disable", json!({})),
    }
}

/// Convert a CLI command to a JSON-RPC method and params.
/// Raw is handled separately in run() — panics if called with Raw.
fn command_to_rpc(cmd: &Commands) -> (&'static str, serde_json::Value) {
    use serde_json::{json, Value};
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
            let mut p = serde_json::Map::new();
            p.insert("direction".into(), json!(direction));
            if let Some(ref id) = id {
                p.insert("id".into(), json!(id));
            }
            ("surface.split", Value::Object(p))
        }
        Commands::FocusSurface { id } => ("surface.focus", json!({"id": id})),
        Commands::CloseSurface { id } => ("surface.close", json!({"id": id})),
        Commands::SendText { text, id } => {
            let mut p = serde_json::Map::new();
            p.insert("text".into(), json!(text));
            if let Some(ref id) = id {
                p.insert("id".into(), json!(id));
            }
            ("surface.send_text", Value::Object(p))
        }
        Commands::SendKey { key, id } => {
            let mut p = serde_json::Map::new();
            p.insert("key".into(), json!(key));
            if let Some(ref id) = id {
                p.insert("id".into(), json!(id));
            }
            ("surface.send_key", Value::Object(p))
        }
        Commands::ReadText { id } => {
            let mut p = serde_json::Map::new();
            if let Some(ref id) = id {
                p.insert("id".into(), json!(id));
            }
            ("surface.read_text", Value::Object(p))
        }
        Commands::Health { id } => {
            let mut p = serde_json::Map::new();
            if let Some(ref id) = id {
                p.insert("id".into(), json!(id));
            }
            ("surface.health", Value::Object(p))
        }
        Commands::Refresh { id } => {
            let mut p = serde_json::Map::new();
            if let Some(ref id) = id {
                p.insert("id".into(), json!(id));
            }
            ("surface.refresh", Value::Object(p))
        }

        Commands::ListPanes => ("pane.list", json!({})),
        Commands::FocusPane { id } => {
            let mut p = serde_json::Map::new();
            if let Some(ref id) = id {
                p.insert("id".into(), json!(id));
            }
            ("pane.focus", Value::Object(p))
        }
        Commands::LastPane => ("pane.last", json!({})),

        Commands::ListWindows => ("window.list", json!({})),
        Commands::CurrentWindow => ("window.current", json!({})),

        Commands::Layout => ("debug.layout", json!({})),
        Commands::Type { text } => ("debug.type", json!({"text": text})),

        Commands::ListNotifications => ("notification.list", json!({})),
        Commands::ClearNotification { id } => {
            ("notification.clear", json!({"id": id}))
        }

        // New browser subcommand group
        Commands::Browser(cmd) => browser_command_to_rpc(cmd),

        // Legacy flat browser commands (backward compat)
        Commands::BrowserOpen { url } => ("browser.open", json!({"url": url})),
        Commands::BrowserClose => ("browser.close", json!({})),
        Commands::BrowserStreamEnable => ("browser.stream.enable", json!({})),
        Commands::BrowserStreamDisable => ("browser.stream.disable", json!({})),
        Commands::BrowserSnapshot => ("browser.snapshot", json!({})),
        Commands::BrowserScreenshot => ("browser.screenshot", json!({})),

        // Legacy browser navigation
        Commands::BrowserGoto { url } => ("browser.goto", json!({"url": url})),
        Commands::BrowserBack => ("browser.back", json!({})),
        Commands::BrowserForward => ("browser.forward", json!({})),
        Commands::BrowserReload => ("browser.reload", json!({})),

        // Legacy browser interaction
        Commands::BrowserClick { selector } => ("browser.click", json!({"selector": selector})),
        Commands::BrowserType { selector, text } => ("browser.type", json!({"selector": selector, "text": text})),
        Commands::BrowserFill { selector, value } => ("browser.fill", json!({"selector": selector, "value": value})),
        Commands::BrowserPress { key } => ("browser.press", json!({"key": key})),
        Commands::BrowserHover { selector } => ("browser.hover", json!({"selector": selector})),
        Commands::BrowserScroll { direction, amount } => ("browser.scroll", json!({"direction": direction, "amount": amount})),
        Commands::BrowserSelect { selector, value } => ("browser.select", json!({"selector": selector, "value": value})),
        Commands::BrowserEval { expression } => ("browser.eval", json!({"expression": expression})),
        Commands::BrowserWait { selector, timeout } => ("browser.wait", json!({"selector": selector, "timeout": timeout})),

        // Legacy browser getters
        Commands::BrowserGetUrl => ("browser.url", json!({})),
        Commands::BrowserGetTitle => ("browser.title", json!({})),
        Commands::BrowserGetText { selector } => ("browser.gettext", json!({"selector": selector})),
        Commands::BrowserGetHtml { selector } => ("browser.gethtml", json!({"selector": selector})),
    }
}
