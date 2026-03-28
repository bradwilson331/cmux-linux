# Phase 10: CLI Socket Commands - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement a native Rust `cmux` CLI binary that communicates with the running cmux app via Unix socket. Port all socket commands as flat verb subcommands matching the macOS CLI style. Include a raw passthrough mode for power users. Rename the GUI app binary to `cmux-app` to avoid collision.

</domain>

<decisions>
## Implementation Decisions

### CLI Architecture
- **D-01:** Native Rust binary via `[[bin]]` target in the existing Cargo crate (`src/bin/cmux.rs`). Not a separate crate. Uses `clap` for argument parsing.
- **D-02:** Binary named `cmux`. GUI app binary renamed from `cmux-linux` to `cmux-app` to avoid collision.
- **D-03:** Flat verb subcommands matching macOS cmux CLI style: `cmux list-workspaces`, `cmux split`, `cmux send-text`, etc. Not noun-verb (`cmux workspace list`).
- **D-04:** Include `cmux raw <method> [--params '{}']` passthrough subcommand for direct socket method access (power users and scripting).

### Command Coverage
- **D-05:** All socket command groups get CLI subcommands:
  - Core: workspace management (list/create/select/close/rename/next/prev/last/reorder), surface management (list/split/focus/close/send-text/send-key/read-text/health/refresh), pane management (list/focus/last)
  - Browser: open/close/navigate/back/forward/reload/stream-enable/stream-disable/snapshot/screenshot
  - Notification: list/clear
  - Debug/system: ping, identify, capabilities, layout, type
  - Window: list, current

### Output Format
- **D-06:** Human-readable output by default. Global `--json` flag switches all output to machine-parseable JSON.
- **D-07:** Colored output with TTY auto-detection. Active workspace/pane highlighted. Standard `--color=always/never/auto` flag.
- **D-08:** List commands use simple lines with markers: `* 1: Main (3 panes)` where `*` marks active. Like tmux list-sessions style.
- **D-09:** Mutation commands (new-workspace, split, close) print result ID/name on success: `Created workspace: Dev (uuid)`.
- **D-10:** Errors to stderr with non-zero exit code: `Error: workspace not found: foo`. Standard CLI convention.

### Socket Discovery
- **D-11:** Port cmux.py discovery logic to Rust. Same order: `CMUX_SOCKET` env → `$XDG_RUNTIME_DIR/cmux/cmux.sock` → last-socket-path marker → `/tmp` fallbacks. Ensures CLI finds the same socket as Python tests.
- **D-12:** Global `--socket <path>` flag before subcommand to override discovery. Also respects `CMUX_SOCKET` env var (flag takes precedence).
- **D-13:** Multiple instances: connect to first found socket in discovery order. User targets specific instance via `--socket` or `CMUX_SOCKET`.
- **D-14:** Socket path shown only with `--verbose`/`-v` flag: `Connected to /run/user/1000/cmux/cmux.sock`. Silent by default for clean script output.

### Claude's Discretion
- Exact clap subcommand naming for each socket method (e.g., `list-workspaces` vs `ls-workspaces`)
- Internal Rust module structure for the CLI binary
- Connection timeout and retry behavior
- Help text wording and examples
- Whether to use `serde_json` directly or a helper for JSON-RPC framing

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Socket Protocol
- `tests_v2/cmux.py` — v2 JSON-RPC client library; protocol docstring, method calls, response parsing define the wire format and all available commands
- `src/socket/mod.rs` — Server-side dispatch_line() maps method strings to SocketCommand variants; defines the complete command set
- `src/socket/commands.rs` — SocketCommand enum with all variants and their parameters
- `src/socket/handlers.rs` — handle_command() implementations; response shapes for each command

### Existing CLI
- `scripts/cmux-cli` — Current bash wrapper invoking cmux.py
- `tests_v2/cmux.py` `main()` function — Existing argparse CLI entrypoint (--method/--params)
- `tests_v2/cmux.py` `_default_socket_path()` — Socket discovery logic to port to Rust

### Build Configuration
- `Cargo.toml` — Current binary target configuration; needs [[bin]] addition and clap dependency
- `.planning/phases/05-config-distribution/05-CONTEXT.md` — CI/AppImage configuration (CLI binary must be included)

### Policies
- `CLAUDE.md` §Socket command threading policy — CLI is a client, not a server, but responses must be handled correctly
- `CLAUDE.md` §Socket focus policy — CLI commands that map to focus-intent socket commands

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/socket/mod.rs`: JSON-RPC framing pattern (newline-delimited JSON) — CLI client needs same framing
- `tests_v2/cmux.py`: Complete command API reference with all methods, params, and response shapes
- `src/socket/commands.rs`: SocketCommand enum — CLI subcommand list should mirror these variants

### Established Patterns
- Socket communication: newline-delimited JSON-RPC (one JSON object per line)
- Request format: `{"id": N, "method": "namespace.action", "params": {...}}`
- Response format: `{"id": N, "ok": true, "result": {...}}` or `{"id": N, "ok": false, "error": {...}}`
- Socket path: `$XDG_RUNTIME_DIR/cmux/cmux.sock` with marker file discovery

### Integration Points
- Cargo.toml: add `[[bin]]` section for cmux CLI, add clap dependency
- AppImage packaging (ci.yml/release.yml): must include cmux binary alongside cmux-app
- scripts/cmux-cli: can be updated to invoke Rust binary instead of Python

</code_context>

<specifics>
## Specific Ideas

- The Rust CLI should produce identical JSON output to what cmux.py returns when `--json` is used — this ensures scripts can switch from Python to Rust CLI without changes
- `cmux raw` should accept the exact same method names as the socket server (e.g., `cmux raw workspace.list`)
- The `--socket` flag must come before the subcommand: `cmux --socket /tmp/foo.sock list-workspaces`
- Exit codes: 0 for success, 1 for command error, 2 for connection error (socket not found/refused)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 10-cli-socket-commands*
*Context gathered: 2026-03-28 via discuss-phase*
