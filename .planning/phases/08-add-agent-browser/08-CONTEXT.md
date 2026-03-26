# Phase 8: Add Agent-Browser - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Integrate the `agent-browser` headless Chrome automation CLI (already in-repo at `agent-browser/`) into cmux-linux. Provide a bundled binary, thin socket API for lifecycle and streaming, and a CDP-streaming preview pane so users can see what the agent sees — without embedding a full browser engine.

</domain>

<decisions>
## Implementation Decisions

### Integration Model
- **D-01:** Hybrid integration — agent-browser binary bundled with cmux, plus a thin socket API layer for key operations. Agents can use either the CLI directly from terminal panes or the cmux socket for lifecycle/streaming commands.

### Visual Rendering
- **D-02:** CDP streaming to preview pane — agent-browser runs headless Chrome, streams screenshots/DOM snapshots via its `stream` command, cmux renders them in a pane. Not a real browser widget, but shows live feedback of what the agent sees.
- **D-03:** Embedded browser pane (WebKitGTK or CEF) is a stretch goal / future enhancement, not in initial scope.

### Socket API Surface
- **D-04:** Lifecycle + streaming commands only: `browser.open`, `browser.close`, `browser.stream.enable`, `browser.stream.disable`, `browser.snapshot`, `browser.screenshot`. Interaction commands (`click`, `fill`, `eval`, etc.) go through agent-browser CLI directly.

### Lifecycle Management
- **D-05:** Auto-start on first use — cmux spawns agent-browser daemon automatically when the first `browser.*` socket command is issued. User stops manually via `browser.close` or cmux shutdown. No repeated cold starts.

### Claude's Discretion
- How to bundle the agent-browser binary (embed in AppImage, ship alongside, or expect system install)
- CDP streaming transport (WebSocket from agent-browser `stream` command vs polling screenshots)
- Preview pane widget implementation (GTK4 image widget refreshed on stream events vs custom drawing area)
- Chrome download/install management (delegate to `agent-browser install` or bundle Chrome)
- Error handling for Chrome not found / agent-browser not installed

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Agent-Browser Project (the tool being integrated)
- `agent-browser/README.md` — Full CLI command reference, installation, architecture
- `agent-browser/cli/src/main.rs` — Rust CLI entry point
- `agent-browser/cli/src/commands.rs` — Command implementations
- `agent-browser/cli/src/connection.rs` — CDP connection logic

### Requirements
- `.planning/REQUIREMENTS.md` §Browser Panel — BROW-01, BROW-02, BROW-03 (v2 requirements)

### Existing Socket Infrastructure
- `src/socket/handlers.rs` — Socket command handler pattern (add `browser.*` handlers here)
- `src/socket/mod.rs` — Socket server setup, command routing

### Prior Deferred References
- `.planning/phases/03-socket-api-session-persistence/03-CONTEXT.md` §Deferred — original `@agent-browser` integration note
- `.planning/phases/05-config-distribution/05-CONTEXT.md` §Deferred — re-deferred agent-browser integration

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `agent-browser/` — Complete Rust CLI with CDP protocol, Chrome management, streaming support already implemented
- `src/socket/handlers.rs` — Established pattern for adding new socket command namespaces (see `workspace.*`, `pane.*`, `surface.*`)
- `src/split_engine.rs` — Pane tree management; preview pane would be a new leaf type (non-terminal)

### Established Patterns
- Socket command handlers: parse JSON-RPC params, dispatch to handler fn, return JSON response
- Tokio tasks for async I/O (agent-browser daemon communication)
- `mpsc::UnboundedSender` for cross-task events to GTK main thread

### Integration Points
- `src/socket/handlers.rs` — Add `browser.*` command namespace
- `src/split_engine.rs` — SplitNode leaf may need a variant for non-terminal content (preview image pane)
- `src/app_state.rs` — Agent-browser process lifecycle management (spawn, track PID, cleanup on shutdown)
- `agent-browser/cli/src/connection.rs` — CDP connection details for cmux to connect/manage

</code_context>

<specifics>
## Specific Ideas

- agent-browser's `stream enable` command starts a WebSocket that pushes runtime events — cmux can subscribe and render screenshots/snapshots as they arrive
- Preview pane shows the latest screenshot from the CDP stream, refreshed on each navigation or interaction event
- `browser.open <url>` socket command: spawns agent-browser if not running, then calls `agent-browser open <url>`
- The bundled binary path could follow the same pattern as cmuxd — shipped alongside the main binary

</specifics>

<deferred>
## Deferred Ideas

- Full embedded browser pane via WebKitGTK or CEF (D-03 stretch goal)
- Browser tab management within cmux (multiple browser tabs per pane)
- JavaScript console UI in cmux
- Find-in-page UI overlay on preview pane
- Browser history / bookmarks
- macOS `browser.*` socket command wire-compatibility (adapt once Linux implementation stabilizes)

</deferred>

---

*Phase: 08-add-agent-browser*
*Context gathered: 2026-03-26 via discuss-phase*
