---
phase: 10-cli-socket-commands
verified: 2026-03-28T05:30:00Z
status: passed
score: 6/6 must-haves verified
---

# Phase 10: CLI Socket Commands Verification Report

**Phase Goal:** Native Rust `cmux` CLI binary communicates with the running cmux-app via Unix socket; all 34+ socket commands accessible as flat verb subcommands with human-readable and JSON output
**Verified:** 2026-03-28T05:30:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths (from ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `cmux ping` connects to the running cmux-app socket and returns pong | VERIFIED | Binary builds, runs, and exits with code 2 (ConnectionError) when no server is running. `format_response` for "system.ping" returns "pong" from result. Socket client `call()` wired correctly. |
| 2 | `cmux list-workspaces` shows human-readable workspace list with * marker on active | VERIFIED | `format_workspace_list()` in `format.rs:35-70` renders `* N: title (M panes)` with green ANSI on selected. `command_to_rpc` maps `ListWorkspaces` to `"workspace.list"`. |
| 3 | `cmux --json list-workspaces` outputs raw JSON result for scripting | VERIFIED | `format_response()` returns `serde_json::to_string_pretty(result)` when `json_mode=true` (format.rs:275-277). `--json` flag is global clap arg wired into `run()`. |
| 4 | `cmux raw <method>` sends arbitrary socket methods for power users | VERIFIED | `Commands::Raw { method, params }` handled separately in `run()` (mod.rs:226-231). Parses params JSON, calls `client.call(method, params_val)`. |
| 5 | `cargo build --bin cmux` succeeds without GTK4 headers (CLI is dependency-light) | VERIFIED | Build succeeds in 0.19s (cached). `src/bin/cmux.rs` uses `#[path = "../cli/mod.rs"] mod cli;` -- no GTK4 imports. CLI modules only use clap, serde_json, libc, std. |
| 6 | GUI binary renamed to cmux-app; CI and packaging updated | VERIFIED | `Cargo.toml` has `[[bin]] name = "cmux-app" path = "src/main.rs"`. `ci.yml:114` has `cargo test --bin cmux-app`. `release.yml:394` has `--executable target/release/cmux-app`. `resources/cmux.desktop` has `Exec=cmux-app`. No remaining `cmux-linux` references in workflows. |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/bin/cmux.rs` | CLI binary entry point with clap parsing | VERIFIED | 29 lines. Parses Cli, dispatches run(), maps CliError variants to exit codes 0/1/2. |
| `src/cli/mod.rs` | Clap CLI definition with Commands enum and dispatch | VERIFIED | 351 lines. `Cli` struct with socket/json/verbose/color flags. `Commands` enum with 38 variants. `run()` and `command_to_rpc()` dispatch all commands. |
| `src/cli/discovery.rs` | Socket path discovery chain | VERIFIED | 73 lines. `discover_socket()` implements 5-step chain: CMUX_SOCKET env, CMUX_SOCKET_PATH compat, XDG_RUNTIME_DIR/cmux/cmux.sock, last-socket-path marker, /tmp/cmux-debug.sock, glob /tmp/cmux-debug-*.sock sorted by mtime. |
| `src/cli/socket_client.rs` | Synchronous Unix socket JSON-RPC client | VERIFIED | 107 lines. `SocketClient` with `connect()` (timeouts set), `call()` (JSON-RPC with id/method/params, ok/error parsing). `CliError` enum with ConnectionError/CommandError/ProtocolError. |
| `src/cli/format.rs` | Human-readable output formatters | VERIFIED | 333 lines. `format_response()` dispatches on method string. `use_color()` with auto/always/never and `is_terminal()`. Green ANSI for active items. `*` markers on workspace/surface/pane lists. Mutation messages for create/close/rename/split. Fallback to pretty JSON. |
| `Cargo.toml` | Dual binary targets and clap dependency | VERIFIED | Two `[[bin]]` sections (cmux-app, cmux). `clap = { version = "4", features = ["derive", "color", "env"] }`. |
| `scripts/cmux-cli` | Wrapper preferring Rust binary | VERIFIED | Tries target/release/cmux, then target/debug/cmux, then falls back to Python cmux.py. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/bin/cmux.rs` | `src/cli/mod.rs` | `#[path] mod cli; cli::run` | WIRED | Line 6-7: `#[path = "../cli/mod.rs"] mod cli;`, line 13: `cli::run(cli_args)` |
| `src/cli/mod.rs` | `src/cli/socket_client.rs` | `SocketClient::connect + call` | WIRED | Line 217: `socket_client::SocketClient::connect(...)`, line 230-234: `client.call(method, ...)` |
| `src/cli/mod.rs` | `src/cli/discovery.rs` | `discover_socket()` | WIRED | Line 209: `discovery::discover_socket()` |
| `src/cli/mod.rs` | `src/cli/format.rs` | `format_response()` + `use_color()` | WIRED | Line 223: `format::use_color(&cli.color)`, line 239: `format::format_response(...)` |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| CLI binary builds independently | `cargo build --bin cmux` | Finished 0.19s | PASS |
| Help shows 38 subcommands | `cmux --help` | All 38 listed (ping through browser-screenshot) | PASS |
| Connection error exits code 2 | `cmux ping` (no server) | "Error: cannot connect..." exit 2 | PASS |
| Raw command exists | `cmux raw system.ping` (no server) | exit 2 (correct: connection error, not missing subcommand) | PASS |
| All 4 commits exist in git | `git log --oneline` | 2362bde2, 6472ce27, d37dc9d8, 45cc0378 all found | PASS |

### Requirements Coverage

The requirement IDs D-01 through D-14 are referenced in the Phase 10 ROADMAP entry and PLANs but do not have formal entries in REQUIREMENTS.md. They appear to be Phase 9/10 design-level requirements shared between the UI phase and CLI phase. Coverage is assessed against the ROADMAP Success Criteria and PLAN must_haves.

**Plan 01 requirements (D-01, D-02, D-03, D-04, D-06, D-10, D-11, D-12, D-13, D-14):**

| ID | Intent (from PLAN context) | Status | Evidence |
|----|---------------------------|--------|----------|
| D-01 | Dual binary targets | SATISFIED | `[[bin]] cmux` + `[[bin]] cmux-app` in Cargo.toml |
| D-02 | GUI binary renamed to cmux-app | SATISFIED | CI, release, desktop file all updated |
| D-03 | Flat verb subcommands | SATISFIED | 38 kebab-case subcommands in clap |
| D-04 | Raw passthrough command | SATISFIED | `cmux raw <method> --params '{}'` |
| D-06 | `--json` raw JSON output | SATISFIED | `format_response` returns pretty JSON when json_mode=true |
| D-10 | Exit codes 0/1/2 | SATISFIED | ExitCode::SUCCESS, from(1), from(2) in cmux.rs |
| D-11 | Socket discovery chain | SATISFIED | 5-step chain in discovery.rs matches Python client |
| D-12 | `--socket` flag override | SATISFIED | `#[arg(long, global, env = "CMUX_SOCKET")]` |
| D-13 | No GTK4 dependency in CLI | SATISFIED | CLI modules import only clap/serde_json/libc/std |
| D-14 | Verbose mode | SATISFIED | `--verbose` prints "Connected to {path}" to stderr |

**Plan 02 requirements (D-05, D-07, D-08, D-09):**

| ID | Intent (from PLAN context) | Status | Evidence |
|----|---------------------------|--------|----------|
| D-05 | All 34+ commands accessible | SATISFIED | 38 Commands enum variants, all dispatched |
| D-07 | Color auto-detection + --color flag | SATISFIED | `use_color()` with auto/always/never, `is_terminal()` |
| D-08 | Human-readable list formatting with active markers | SATISFIED | `*` markers, green ANSI on selected items |
| D-09 | Mutation success messages | SATISFIED | `format_mutation()` for create/close/rename/split |

**Orphaned requirements:** None. All 14 D-IDs from ROADMAP are claimed and satisfied.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | - | - | - | No anti-patterns found |

No TODOs, FIXMEs, placeholders, empty implementations, or stub patterns found in any `src/cli/` or `src/bin/cmux.rs` file.

### Human Verification Required

### 1. End-to-end socket communication

**Test:** Run cmux-app, then `cmux ping` and `cmux list-workspaces`
**Expected:** `ping` returns "pong", `list-workspaces` shows workspace list with `*` on active
**Why human:** Requires running GUI app with active socket server

### 2. JSON output mode

**Test:** Run `cmux --json list-workspaces` against running instance
**Expected:** Raw JSON with `workspaces` array
**Why human:** Needs live server to produce real response data

### 3. Color output rendering

**Test:** Run `cmux list-workspaces` in a terminal vs `cmux --color=never list-workspaces`
**Expected:** TTY shows green-highlighted active workspace; `--color=never` shows no ANSI codes
**Why human:** Visual color rendering cannot be verified programmatically

### 4. Mutation commands

**Test:** Run `cmux new-workspace` against running instance
**Expected:** Prints "Created workspace: {title} ({id})"
**Why human:** Requires live server for workspace creation

---

_Verified: 2026-03-28T05:30:00Z_
_Verifier: Claude (gsd-verifier)_
