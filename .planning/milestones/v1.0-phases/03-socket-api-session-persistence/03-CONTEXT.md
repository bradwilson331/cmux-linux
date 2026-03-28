# Phase 3: Socket API + Session Persistence - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a Unix socket JSON-RPC server (v2 wire-compatible with macOS cmux) so the Python test
suite and CLI can control the Linux app; save and restore workspace/pane layout atomically
across restarts.

No new UI. Backend/API phase only.

</domain>

<decisions>
## Implementation Decisions

### tests_v2 Coverage Scope

- **D-01:** Phase 3 targets the core socket + session test subset: `test_cli_*`, `test_close_*`,
  `test_ctrl_socket.py`, `test_nested_split_*`, `test_initial_terminal_*`, and any other tests
  that exercise workspace/surface/pane operations without browser or command palette.
- **D-02:** Browser (`test_browser_*`), command palette (`test_command_palette_*`), and macOS-specific
  tests (`test_lint_swiftui_patterns.py`, SwiftUI split tests) are explicitly out of scope for
  Phase 3. They pass in Phase 4/5 when those features land (note: `@agent-browser` integration
  targeted for Phase 4/5).
- **D-03:** SOCK-04 ("tests_v2 passes unmodified") is satisfied by the Linux-applicable subset
  running green. macOS-only tests are excluded by a platform check or skip marker.

### CLI Tool

- **D-04:** The "cmux CLI" for Phase 3 is the Python client `tests_v2/cmux.py`. A native Rust
  CLI binary (`src/bin/cmux.rs`) comes in Phase 5 (Config + Distribution).
- **D-05:** Update `tests_v2/cmux.py` to check XDG paths on Linux — add
  `$XDG_RUNTIME_DIR/cmux/cmux.sock` (and the `/run/user/{uid}/cmux/cmux.sock` fallback) to
  the socket discovery list when `platform.system() == "Linux"`. The `CMUX_SOCKET` env var
  override already works and is used by the test harness.

### Socket Server

- **D-06:** Socket path: `$XDG_RUNTIME_DIR/cmux/cmux.sock`. If `XDG_RUNTIME_DIR` is unset,
  fall back to `/run/user/{getuid()}/cmux/cmux.sock`. Create the `cmux/` subdirectory at
  startup (mode 0700). Socket file mode: 0600.
- **D-07:** Authentication: `SO_PEERCRED` uid validation on every `accept()`. Reject connections
  from UIDs that don't match the app owner UID. No HMAC-SHA256 password auth needed in Phase 3
  — Linux `SO_PEERCRED` is more reliable and sufficient.
- **D-08:** Threading: Replace the current polling `mpsc::channel` + 100ms timer in `main.rs`
  with `glib::MainContext::channel::<SocketCommand>()`. Socket accept loop runs in tokio;
  commands are sent via the channel sender and processed in the GTK main loop receiver.
  This is the idiomatic gtk4-rs pattern — no polling, instant delivery.

### v2 Protocol Command Tier

- **D-09:** Tier 1 — fully implemented in Phase 3:
  - `system.ping`, `system.identify`, `system.capabilities`
  - `workspace.list`, `workspace.current`, `workspace.create`, `workspace.select`,
    `workspace.close`, `workspace.rename`, `workspace.next`, `workspace.previous`,
    `workspace.last`, `workspace.reorder`
  - `surface.list`, `surface.split`, `surface.focus`, `surface.close`, `surface.send_text`,
    `surface.send_key`, `surface.read_text`, `surface.health`, `surface.refresh`
  - `pane.list`, `pane.focus`, `pane.last`
  - `debug.layout`, `debug.type`
  - `window.list`, `window.current` (maps to single GTK window)
- **D-10:** Tier 2 — stub with `{"ok": false, "error": "not_implemented"}` in Phase 3:
  - All `browser.*` methods
  - All `notification.*` methods
  - `window.create`, `window.close`, `window.focus`
  - `pane.create`, `pane.break`, `pane.join`, `pane.swap`, `pane.surfaces`
  - `surface.drag_to_split`, `surface.move`, `surface.reorder`, `surface.trigger_flash`,
    `surface.clear_history`, `surface.create`

### Socket Command Threading Policy

- **D-11:** Per CLAUDE.md socket command threading policy: parse and validate arguments
  off-main; schedule state mutations with `glib::MainContext::channel` receiver (main thread).
  Non-focus-intent commands (workspace.list, surface.list, etc.) never call GTK focus APIs.

### Session Persistence

- **D-12:** Save path: `~/.local/share/cmux/session.json`
  (respects `$XDG_DATA_HOME/cmux/session.json` if set — CFG-04 compliance).
- **D-13:** Save content (full layout):
  - Workspace list: name, UUID, active_pane_id
  - Per-workspace SplitNode tree: recursive branch/leaf structure, each leaf contains
    surface UUID, shell command (e.g. `/bin/zsh`), CWD path
- **D-14:** Save trigger: debounced 500ms after any workspace/pane mutation (workspace create,
  close, rename; pane split, close, focus change). Implemented with a `tokio::time::sleep`
  debounce or `glib::timeout_add_once` after each mutation.
- **D-15:** Atomic write: write to `session.json.tmp`, then `rename()` to `session.json`.
  This is SESS-03 — kill -9 mid-save never corrupts the file.
- **D-16:** Restore: on app launch, read `session.json` before building UI. If missing or
  JSON-invalid, log and start with one default workspace (SESS-04 graceful fallback).

### Claude's Discretion

- v2 JSON-RPC framing (one JSON object per line, newline-delimited) — implement as specified
  by `tests_v2/cmux.py` protocol docstring.
- Error response schema: `{"id": N, "ok": false, "error": "error_code", "message": "human text"}`.
- Request ID type: accept both integer and string IDs, echo back what was sent.
- `system.identify` response shape: include `version`, `platform: "linux"`, `socket_path`.
- `glib::MainContext::channel` buffer capacity: 256 commands (sufficient for burst scenarios).
- `SO_PEERCRED` implementation: `getsockopt(SOL_SOCKET, SO_PEERCRED)` on accepted fd.
- Session JSON schema: versioned (`"version": 1`) for forward-compatibility.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Socket API — SOCK-01 through SOCK-06 (socket server requirements)
- `.planning/REQUIREMENTS.md` §Session Persistence — SESS-01 through SESS-04 (session requirements)

### Roadmap
- `.planning/ROADMAP.md` §Phase 3 — success criteria, phase goal, requirement IDs

### Protocol Source of Truth
- `tests_v2/cmux.py` — v2 JSON-RPC client library; protocol docstring, method calls, and
  response parsing define the expected wire format

### Project State & Policies
- `.planning/STATE.md` — Socket command threading policy, socket focus policy, key decisions

### Existing Implementation (read before modifying)
- `src/main.rs` — existing tokio runtime and GLib bridge (polling channel to replace)
- `src/app_state.rs` — AppState struct (workspace/pane state owned here)
- `src/workspace.rs` — Workspace model
- `src/split_engine.rs` — SplitNode tree (pane layout — must serialize for session)

</canonical_refs>

<specifics>
## Specific Ideas

- The `CMUX_SOCKET` env var in `cmux.py` must take precedence over XDG path discovery —
  test harness will use `CMUX_SOCKET=/run/user/$(id -u)/cmux/cmux.sock` to target Linux.
- `system.identify` should return `"platform": "linux"` so tests can branch on platform.
- `debug.type` is Tier 1 because `test_ctrl_socket.py` and interactive tests use it to send
  keystrokes programmatically.
- Session file directory (`~/.local/share/cmux/`) must be created on first save if missing.
- The `last-socket-path` marker file written by the macOS app should also be written by the
  Linux app at `$XDG_RUNTIME_DIR/cmux/last-socket-path` so cmux.py can discover it.

</specifics>

<deferred>
## Deferred Ideas

- `@agent-browser` integration — browser panel embedding for Linux. Targeting Phase 4/5
  so browser tests (`test_browser_*`) can pass.
- HMAC-SHA256 password auth — macOS-style socket auth. Not needed for Phase 3 (SO_PEERCRED
  is sufficient on Linux).
- Rust CLI binary (`src/bin/cmux.rs`) — native cmux CLI for Linux. Phase 5 (Config + Distribution).
- `notification.*` socket commands — deferred to Phase 4 (Notifications + HiDPI).
- `pane.break`, `pane.join`, `pane.swap` — advanced pane operations. Deferred beyond Phase 3.
- `window.create` / multi-window — macOS cmux supports multiple windows. Linux Phase 3 is
  single-window; multi-window can come later.
- Systemd socket activation (SYS-01) — out of scope for Phase 3.

</deferred>

---

*Phase: 03-socket-api-session-persistence*
*Context gathered: 2026-03-25 via discuss-phase*
