# Phase 10: CLI Socket Commands - Research

**Researched:** 2026-03-27
**Domain:** Rust CLI binary (clap), Unix socket client, JSON-RPC protocol
**Confidence:** HIGH

## Summary

Phase 10 implements a native Rust `cmux` CLI binary that communicates with the running cmux-app over the existing Unix socket. The CLI is a pure client -- it connects to the socket, sends JSON-RPC requests, formats responses for human or machine consumption, and exits. No GTK4, no Ghostty, no server logic. This makes it a straightforward Rust binary with minimal dependencies: `clap` for argument parsing, `serde_json` for JSON-RPC framing, and `std::os::unix::net::UnixStream` for socket I/O (no tokio needed -- the CLI is synchronous request/response).

The complete set of socket methods is already defined in `src/socket/mod.rs` dispatch table (34 methods across 7 namespaces). The CLI maps each method to a flat verb subcommand per D-03. The `cmux raw` passthrough command (D-04) provides direct access to any method string for scripting. Socket discovery logic from `tests_v2/cmux.py` `_default_socket_path()` is ported to Rust.

**Primary recommendation:** Add a `[[bin]]` target in Cargo.toml for `src/bin/cmux.rs` alongside the existing `src/main.rs` GUI binary. Use clap derive API for subcommand definitions. Keep the CLI binary dependency-light -- no GTK4, no tokio. Rename the existing binary from `cmux-linux` to `cmux-app` via Cargo.toml `[[bin]]` configuration.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Native Rust binary via `[[bin]]` target in the existing Cargo crate (`src/bin/cmux.rs`). Not a separate crate. Uses `clap` for argument parsing.
- **D-02:** Binary named `cmux`. GUI app binary renamed from `cmux-linux` to `cmux-app` to avoid collision.
- **D-03:** Flat verb subcommands matching macOS cmux CLI style: `cmux list-workspaces`, `cmux split`, `cmux send-text`, etc. Not noun-verb (`cmux workspace list`).
- **D-04:** Include `cmux raw <method> [--params '{}']` passthrough subcommand for direct socket method access (power users and scripting).
- **D-05:** All socket command groups get CLI subcommands (workspace, surface, pane, browser, notification, debug/system, window).
- **D-06:** Human-readable output by default. Global `--json` flag switches all output to machine-parseable JSON.
- **D-07:** Colored output with TTY auto-detection. Active workspace/pane highlighted. Standard `--color=always/never/auto` flag.
- **D-08:** List commands use simple lines with markers: `* 1: Main (3 panes)` where `*` marks active. Like tmux list-sessions style.
- **D-09:** Mutation commands (new-workspace, split, close) print result ID/name on success.
- **D-10:** Errors to stderr with non-zero exit code. Standard CLI convention.
- **D-11:** Port cmux.py discovery logic to Rust. Same order: `CMUX_SOCKET` env -> `$XDG_RUNTIME_DIR/cmux/cmux.sock` -> last-socket-path marker -> `/tmp` fallbacks.
- **D-12:** Global `--socket <path>` flag before subcommand to override discovery. Also respects `CMUX_SOCKET` env var (flag takes precedence).
- **D-13:** Multiple instances: connect to first found socket in discovery order. User targets specific instance via `--socket` or `CMUX_SOCKET`.
- **D-14:** Socket path shown only with `--verbose`/`-v` flag.

### Claude's Discretion
- Exact clap subcommand naming for each socket method (e.g., `list-workspaces` vs `ls-workspaces`)
- Internal Rust module structure for the CLI binary
- Connection timeout and retry behavior
- Help text wording and examples
- Whether to use `serde_json` directly or a helper for JSON-RPC framing

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.6.0 | CLI argument parsing with derive macros | De facto Rust CLI standard; derive API eliminates boilerplate |
| serde_json | 1.x | JSON-RPC request/response serialization | Already in Cargo.toml; shared with server side |
| serde | 1.x | Derive serialization for response structs | Already in Cargo.toml |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| clap (color feature) | 4.6.0 | `--color` flag auto-detection | Included in default clap features |
| libc | 0.2.x | `getuid()` for socket path fallback | Already in Cargo.toml |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| clap derive | clap builder | Builder is more flexible but more verbose; derive is ideal for this fixed subcommand structure |
| std::os::unix::net | tokio UnixStream | tokio adds async overhead for what is a simple blocking request/response; std is simpler |
| No color library | termcolor/owo-colors | clap handles `--color` flag; use ANSI escape codes directly for the few colored output lines |

**Installation:**
```bash
# Add to Cargo.toml [dependencies]
clap = { version = "4.6", features = ["derive", "color"] }
# serde, serde_json, libc already present
```

**Version verification:** clap 4.6.0 confirmed via `cargo search clap` on 2026-03-27. serde_json 1.x and libc 0.2.x already in Cargo.toml.

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs              # GUI app (renamed binary: cmux-app)
├── bin/
│   └── cmux.rs          # CLI entry point (binary: cmux)
├── cli/
│   ├── mod.rs           # Clap CLI definition, subcommand dispatch
│   ├── socket_client.rs # Unix socket connect + JSON-RPC send/receive
│   ├── discovery.rs     # Socket path discovery (port of cmux.py logic)
│   └── format.rs        # Human-readable and JSON output formatting
├── socket/              # (existing server-side code, unchanged)
│   ├── mod.rs
│   ├── commands.rs
│   └── handlers.rs
└── ...
```

### Pattern 1: Clap Derive Subcommands
**What:** Define CLI structure with Rust enums + derive macros
**When to use:** Fixed set of subcommands known at compile time
**Example:**
```rust
use clap::{Parser, Subcommand, ColorChoice};

#[derive(Parser)]
#[command(name = "cmux", about = "Control cmux terminal multiplexer")]
struct Cli {
    /// Socket path override
    #[arg(long, global = true, env = "CMUX_SOCKET")]
    socket: Option<String>,

    /// Output raw JSON instead of human-readable text
    #[arg(long, global = true)]
    json: bool,

    /// Verbose output (show socket path)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Color output control
    #[arg(long, global = true, default_value = "auto")]
    color: ColorChoice,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ping the cmux server
    Ping,
    /// List all workspaces
    ListWorkspaces,
    /// Create a new workspace
    NewWorkspace,
    /// Select a workspace by ID or index
    SelectWorkspace {
        /// Workspace ID (UUID) or index number
        id: String,
    },
    // ... etc for all commands
    /// Send raw JSON-RPC method call
    Raw {
        /// Method name (e.g., "workspace.list")
        method: String,
        /// JSON params
        #[arg(long, default_value = "{}")]
        params: String,
    },
}
```

### Pattern 2: Synchronous Socket Client
**What:** Blocking Unix socket connect, write request, read response
**When to use:** CLI binary -- one request per invocation, no concurrency needed
**Example:**
```rust
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

pub struct SocketClient {
    stream: UnixStream,
    reader: BufReader<UnixStream>,
    next_id: u64,
}

impl SocketClient {
    pub fn connect(path: &str, timeout: Duration) -> Result<Self, String> {
        let stream = UnixStream::connect(path)
            .map_err(|e| format!("Failed to connect to {}: {}", path, e))?;
        stream.set_read_timeout(Some(timeout)).ok();
        stream.set_write_timeout(Some(timeout)).ok();
        let reader = BufReader::new(stream.try_clone().unwrap());
        Ok(Self { stream, reader, next_id: 1 })
    }

    pub fn call(&mut self, method: &str, params: serde_json::Value)
        -> Result<serde_json::Value, String>
    {
        let id = self.next_id;
        self.next_id += 1;
        let req = serde_json::json!({"id": id, "method": method, "params": params});
        writeln!(self.stream, "{}", req).map_err(|e| format!("Write failed: {}", e))?;
        let mut line = String::new();
        self.reader.read_line(&mut line).map_err(|e| format!("Read failed: {}", e))?;
        let resp: serde_json::Value = serde_json::from_str(&line)
            .map_err(|e| format!("Invalid response: {}", e))?;
        if resp.get("ok") == Some(&serde_json::Value::Bool(true)) {
            Ok(resp.get("result").cloned().unwrap_or(serde_json::Value::Null))
        } else {
            let err = resp.get("error").and_then(|e| e.get("message"))
                .and_then(|m| m.as_str()).unwrap_or("Unknown error");
            Err(err.to_string())
        }
    }
}
```

### Pattern 3: Socket Discovery Chain
**What:** Port the cmux.py `_default_socket_path()` logic to Rust
**When to use:** Always -- the CLI must find the socket automatically
**Example:**
```rust
pub fn discover_socket() -> Option<String> {
    // 1. CMUX_SOCKET env var
    if let Ok(path) = std::env::var("CMUX_SOCKET") {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    // 2. XDG_RUNTIME_DIR/cmux/cmux.sock
    let xdg = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));
    let xdg_sock = format!("{}/cmux/cmux.sock", xdg);
    if std::path::Path::new(&xdg_sock).exists() {
        return Some(xdg_sock);
    }

    // 3. last-socket-path marker
    let marker = format!("{}/cmux/last-socket-path", xdg);
    if let Ok(contents) = std::fs::read_to_string(&marker) {
        let path = contents.trim().to_string();
        if !path.is_empty() && std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    // 4. /tmp fallbacks
    for candidate in &["/tmp/cmux-debug.sock"] {
        if std::path::Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }

    // 5. Glob /tmp/cmux-debug-*.sock
    if let Ok(entries) = std::fs::read_dir("/tmp") {
        let mut found: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("cmux-debug-") && name.ends_with(".sock")
            })
            .collect();
        found.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok()
            .and_then(|m| m.modified().ok())));
        if let Some(entry) = found.first() {
            return Some(entry.path().to_string_lossy().to_string());
        }
    }

    None
}
```

### Pattern 4: Flat Subcommand to Method Mapping
**What:** Map each clap subcommand to its socket method string + params
**When to use:** Central dispatch in CLI main
**Example:**
```rust
fn dispatch(cmd: &Commands) -> (&str, serde_json::Value) {
    match cmd {
        Commands::Ping => ("system.ping", json!({})),
        Commands::ListWorkspaces => ("workspace.list", json!({})),
        Commands::SelectWorkspace { id } => ("workspace.select", json!({"id": id})),
        Commands::Split { direction, id } => ("surface.split", json!({
            "direction": direction,
            "id": id,
        })),
        Commands::Raw { method, params } => {
            let p: serde_json::Value = serde_json::from_str(params)
                .unwrap_or(json!({}));
            (method.as_str(), p)
        }
        // ...
    }
}
```

### Anti-Patterns to Avoid
- **Pulling in tokio for the CLI binary:** The CLI is synchronous -- one request, one response. `std::os::unix::net::UnixStream` is sufficient. Adding tokio increases binary size and compile time for no benefit.
- **Sharing modules between GUI and CLI that pull in GTK4:** The CLI binary must NOT link against GTK4. Use `#[cfg]` guards or keep CLI-only code in `src/cli/` and `src/bin/cmux.rs` which do not `mod` any GTK-dependent modules.
- **Duplicating socket path logic:** The server's `socket::socket_path()` and the CLI's discovery logic overlap. However, the CLI has a broader discovery chain (markers, fallbacks, globs) that the server doesn't need. Keep them separate -- the CLI discovery is a superset.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI argument parsing | Custom arg parser | clap 4.6 derive | Handles help text, color, error messages, shell completions |
| JSON serialization | Manual string building | serde_json | Already in use; no bugs from manual JSON construction |
| TTY detection | Manual isatty check | clap ColorChoice + `std::io::IsTerminal` | Standard Rust 1.70+ trait for TTY detection |
| Exit code handling | Custom process::exit calls | `std::process::ExitCode` or early returns | Cleaner than scattered exit() calls |

## Common Pitfalls

### Pitfall 1: CLI Binary Linking GTK4
**What goes wrong:** `src/bin/cmux.rs` uses `mod` to pull in modules that import `gtk4`, causing the CLI binary to require GTK4 installed just to run `cmux list-workspaces`.
**Why it happens:** The `src/cli/` modules are in the same crate as the GTK4 app. Cargo compiles all modules for all binaries.
**How to avoid:** The CLI binary (`src/bin/cmux.rs`) must only use `src/cli/*` modules. Do NOT `mod app_state` or `mod socket` from the CLI binary. The CLI has its own socket client implementation -- it does not share server-side code. Use `[[bin]]` with separate module trees.
**Warning signs:** Compile errors about missing GTK4 headers when building just the CLI. Binary size > 50MB.

### Pitfall 2: Cargo Workspace Binary Confusion
**What goes wrong:** With `[[bin]]` targets, Cargo may try to compile the GUI binary when you only want the CLI, pulling in GTK4 deps.
**Why it happens:** `cargo build` builds all binaries by default.
**How to avoid:** Use `cargo build --bin cmux` to build only the CLI. CI should build both: `cargo build --bin cmux --bin cmux-app`. The CLI binary's `src/bin/cmux.rs` should have a completely independent module tree from `src/main.rs`.
**Warning signs:** `cargo build --bin cmux` fails with GTK4 link errors on a machine without GTK4 dev headers.

### Pitfall 3: Binary Name Collision During Transition
**What goes wrong:** Renaming the GUI binary from `cmux-linux` to `cmux-app` breaks CI, scripts, or the AppImage that reference the old name.
**Why it happens:** The binary name is referenced in `Cargo.toml`, CI workflows (`cargo test --bin cmux-linux`), and possibly AppImage scripts.
**How to avoid:** Update all references atomically: Cargo.toml `[[bin]]` name, CI workflow `cargo test --bin`, and any packaging scripts. Search the repo for `cmux-linux` references.
**Warning signs:** CI failures after the rename.

### Pitfall 4: Socket Timeout Hang
**What goes wrong:** CLI hangs forever waiting for a response from a crashed or unresponsive server.
**Why it happens:** `UnixStream` default has no read timeout.
**How to avoid:** Set `set_read_timeout(Some(Duration::from_secs(5)))` on the socket. Print a clear error on timeout.
**Warning signs:** `cmux ping` hangs instead of erroring when the app is not running.

### Pitfall 5: JSON Output Inconsistency
**What goes wrong:** `--json` output for the same command produces different JSON shapes than the raw socket response.
**Why it happens:** CLI reformats the response for human output, and the `--json` path also transforms it differently.
**How to avoid:** Per D-06, `--json` should output the raw socket response `result` field directly. The `raw` command should output the full response envelope. Human formatting is a separate code path that reads the same result.
**Warning signs:** Scripts using `cmux --json list-workspaces | jq` break when switching from `cmux raw`.

## Code Examples

### Complete Subcommand List (derived from socket dispatch table)

Based on `src/socket/mod.rs` dispatch_line(), here are all 34 methods mapped to flat CLI subcommands:

```
# System
cmux ping                           -> system.ping
cmux identify                       -> system.identify
cmux capabilities                   -> system.capabilities

# Workspace
cmux list-workspaces                -> workspace.list
cmux current-workspace              -> workspace.current
cmux new-workspace                  -> workspace.create
cmux select-workspace <id>          -> workspace.select
cmux close-workspace <id>           -> workspace.close
cmux rename-workspace <id> <name>   -> workspace.rename
cmux next-workspace                 -> workspace.next
cmux prev-workspace                 -> workspace.previous
cmux last-workspace                 -> workspace.last
cmux reorder-workspace <id> <pos>   -> workspace.reorder

# Surface
cmux list-surfaces                  -> surface.list
cmux split [--direction h|v] [--id <uuid>]  -> surface.split
cmux focus-surface <id>             -> surface.focus
cmux close-surface <id>             -> surface.close
cmux send-text <text> [--id <uuid>] -> surface.send_text
cmux send-key <key> [--id <uuid>]   -> surface.send_key
cmux read-text [--id <uuid>]        -> surface.read_text
cmux health [--id <uuid>]           -> surface.health
cmux refresh [--id <uuid>]          -> surface.refresh

# Pane
cmux list-panes                     -> pane.list
cmux focus-pane [<id>]              -> pane.focus
cmux last-pane                      -> pane.last

# Window
cmux list-windows                   -> window.list
cmux current-window                 -> window.current

# Debug
cmux layout                         -> debug.layout
cmux type <text>                    -> debug.type

# Notification
cmux list-notifications             -> notification.list
cmux clear-notification <id>        -> notification.clear

# Browser
cmux browser-open <url>             -> browser.open
cmux browser-close                  -> browser.close
cmux browser-stream-enable          -> browser.stream.enable
cmux browser-stream-disable         -> browser.stream.disable
cmux browser-snapshot               -> browser.snapshot
cmux browser-screenshot             -> browser.screenshot

# Raw passthrough
cmux raw <method> [--params '{}']   -> any method
```

### Cargo.toml Binary Configuration
```toml
[[bin]]
name = "cmux-app"
path = "src/main.rs"

[[bin]]
name = "cmux"
path = "src/bin/cmux.rs"
# CLI-only deps handled via optional features or kept minimal
```

### Human-Readable Output Formatting (D-08)
```rust
fn format_workspace_list(result: &serde_json::Value, color: bool) -> String {
    let workspaces = result.get("workspaces")
        .and_then(|w| w.as_array())
        .cloned()
        .unwrap_or_default();
    let mut out = String::new();
    for ws in &workspaces {
        let selected = ws.get("selected").and_then(|s| s.as_bool()).unwrap_or(false);
        let marker = if selected { "*" } else { " " };
        let index = ws.get("index").and_then(|i| i.as_u64()).unwrap_or(0);
        let title = ws.get("title").and_then(|t| t.as_str()).unwrap_or("untitled");
        let id = ws.get("id").and_then(|i| i.as_str()).unwrap_or("");
        let pane_count = ws.get("pane_count").and_then(|p| p.as_u64()).unwrap_or(1);

        if color && selected {
            out.push_str(&format!("\x1b[1;32m{} {}: {} ({} panes)\x1b[0m\n",
                marker, index, title, pane_count));
        } else {
            out.push_str(&format!("{} {}: {} ({} panes)\n",
                marker, index, title, pane_count));
        }
    }
    out
}
```

### Exit Code Convention (from D-10 specifics)
```rust
fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(CliError::CommandError(msg)) => {
            eprintln!("Error: {}", msg);
            std::process::ExitCode::from(1)
        }
        Err(CliError::ConnectionError(msg)) => {
            eprintln!("Error: {}", msg);
            std::process::ExitCode::from(2)
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| clap 3.x builder API | clap 4.x derive API | 2023 | Less boilerplate, better ergonomics |
| structopt crate | Merged into clap 4.x derive | 2023 | structopt is deprecated |
| Manual color handling | `std::io::IsTerminal` (Rust 1.70+) | 2023 | No external crate needed for TTY detection |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) |
| Config file | Cargo.toml |
| Quick run command | `cargo test --bin cmux` |
| Full suite command | `cargo test --bin cmux && cargo test --bin cmux-app` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLI-01 | Socket discovery finds correct path | unit | `cargo test --bin cmux -- discovery` | Wave 0 |
| CLI-02 | Subcommands map to correct methods | unit | `cargo test --bin cmux -- dispatch` | Wave 0 |
| CLI-03 | JSON output matches raw socket response | unit | `cargo test --bin cmux -- format` | Wave 0 |
| CLI-04 | Human-readable formatting with markers | unit | `cargo test --bin cmux -- format` | Wave 0 |
| CLI-05 | Exit codes: 0/1/2 for success/error/connection | unit | `cargo test --bin cmux -- exit_code` | Wave 0 |
| CLI-06 | Raw passthrough sends exact method/params | unit | `cargo test --bin cmux -- raw` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --bin cmux`
- **Per wave merge:** `cargo test --bin cmux && cargo clippy --bin cmux -- -D warnings`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/cli/mod.rs` -- test module for subcommand dispatch
- [ ] `src/cli/discovery.rs` -- tests for socket path resolution
- [ ] `src/cli/format.rs` -- tests for human-readable output formatting

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build | Yes | stable | -- |
| cargo | Build | Yes | bundled with rustc | -- |
| clap 4.6 | CLI parsing | Yes (crates.io) | 4.6.0 | -- |
| Unix socket (AF_UNIX) | Socket comms | Yes (Linux) | kernel | -- |

No missing dependencies.

## Sources

### Primary (HIGH confidence)
- `src/socket/mod.rs` -- Complete dispatch table with all 34 socket methods
- `src/socket/commands.rs` -- SocketCommand enum defining all variants and parameters
- `src/socket/handlers.rs` -- Response shapes for each command
- `tests_v2/cmux.py` -- Python client with socket discovery logic, method signatures, response parsing
- `Cargo.toml` -- Current binary target and dependency configuration
- `.github/workflows/ci.yml` -- CI job referencing `cargo test --bin cmux-linux`

### Secondary (MEDIUM confidence)
- `cargo search clap` output (2026-03-27) -- clap 4.6.0 is current
- clap derive API patterns -- well-established, verified against training data

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- clap 4.6, serde_json, std::os::unix::net are well-established and verified
- Architecture: HIGH -- `[[bin]]` multi-binary pattern is standard Cargo; socket client is straightforward
- Pitfalls: HIGH -- GTK4 linkage separation is the main risk, well-understood from crate structure analysis

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable domain, no fast-moving dependencies)
