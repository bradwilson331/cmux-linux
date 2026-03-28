# Phase 8: Add Agent-Browser - Context

**Gathered:** 2026-03-26 (updated 2026-03-27)
**Status:** Plans complete, post-phase feature work in progress

<domain>
## Phase Boundary

Integrate the `agent-browser` headless Chrome automation CLI into cmux-linux. Provide a bundled binary, comprehensive socket API proxying all P0+P1 browser commands, an interactive preview pane with input forwarding (mouse, keyboard, scroll), a navigation toolbar, and a DevTools snapshot overlay — making the preview pane a fully usable remote browser surface.

</domain>

<decisions>
## Implementation Decisions

### Integration Model
- **D-01:** Hybrid integration — agent-browser binary bundled with cmux, plus a thin socket API layer for key operations. Agents can use either the CLI directly from terminal panes or the cmux socket for lifecycle/streaming commands.

### Visual Rendering
- **D-02:** CDP streaming to preview pane — agent-browser runs headless Chrome, streams screenshots/DOM snapshots via its `stream` command, cmux renders them in a pane. Not a real browser widget, but shows live feedback of what the agent sees.
- **D-03:** Embedded browser pane (WebKitGTK or CEF) is a stretch goal / future enhancement, not in initial scope.

### Socket API Surface
- **D-04:** ~~Lifecycle + streaming commands only.~~ **UPDATED:** Full P0+P1 command parity via generic `BrowserAction` proxy. All P0 commands (navigate, click, type, fill, eval, wait, get.*, is.*, scroll, screenshot, snapshot) and P1 commands (find.*, frame.*, dialog.*, console, errors, highlight, state.*) are proxied to the agent-browser daemon. P2 commands (network intercept, emulation, trace, raw input injection) return explicit `not_supported` errors.

### Lifecycle Management
- **D-05:** Auto-start on first use — cmux spawns agent-browser daemon automatically when the first `browser.*` socket command is issued. User stops manually via `browser.close` or cmux shutdown. No repeated cold starts.

### Navigation Bar
- **D-06:** Single toolbar row layout: `[ ◀ ][ ▶ ][ ↻/✕ ] [ URL entry... ] [ → ] [ {} ]`. Compact ~36px height. Familiar Chrome/Firefox UX.
- **D-07:** Reload/Stop swap behavior — shows ↻ (reload) when idle, ✕ (stop) during page load. Requires tracking navigation loading state from the daemon.

### Input Forwarding
- **D-08:** Hybrid async/sync model — mouse motion events fire-and-forget via tokio channel (non-blocking for high-frequency hover). Mouse click, keyboard, and scroll events use synchronous `send_command` for reliable delivery.
- **D-09:** Auto-refocus viewport on click — clicking the Picture area sets GTK focus back to the preview container so keyboard events resume flowing to Chrome. URL bar has independent focus; user clicks viewport to return keyboard to browser.

### DevTools View
- **D-10:** DOM snapshot overlay — the `{ }` toggle button calls `browser.snapshot` and renders the accessibility tree as a scrollable text overlay on the viewport. No extra panes or panels. Toggling off restores the normal viewport view.

### Claude's Discretion
- How to bundle the agent-browser binary (embed in AppImage, ship alongside, or expect system install)
- Chrome download/install management (delegate to `agent-browser install` or bundle Chrome)
- Error handling for Chrome not found / agent-browser not installed
- Exact throttle rate for async mouse motion forwarding
- Loading state detection mechanism (poll daemon or listen for navigation events)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Agent-Browser Port Spec
- User-provided port spec document (in conversation context) — comprehensive command inventory, P0/P1/P2 tier definitions, CLI spec, move/reorder spec, test port plan

### Agent-Browser Project (the tool being integrated)
- `agent-browser/README.md` — Full CLI command reference, installation, architecture
- `agent-browser/cli/src/main.rs` — Rust CLI entry point
- `agent-browser/cli/src/commands.rs` — Command implementations
- `agent-browser/cli/src/connection.rs` — CDP connection logic

### Requirements
- `.planning/REQUIREMENTS.md` §Browser Panel — BROW-01, BROW-02, BROW-03 (v2 requirements)

### Existing Implementation (current state)
- `src/browser.rs` — BrowserManager (daemon lifecycle, stream pipeline, preview pane factory)
- `src/shortcuts.rs` — Browser keyboard shortcuts, input forwarding controllers (click, hover, scroll, keyboard)
- `src/socket/handlers.rs` — Socket command handlers including `BrowserAction` generic proxy
- `src/socket/commands.rs` — SocketCommand enum with `BrowserAction` variant
- `src/socket/mod.rs` — Command routing with `merge_subcommand` for compound commands
- `src/split_engine.rs` — `SplitNode::Preview` variant for browser pane
- `src/config.rs` — `BrowserOpen`/`BrowserClose` shortcut actions
- `.planning/phases/08-add-agent-browser/08-UI-SPEC.md` — Updated UI spec with nav bar, input focus, and CSS

### Prior Deferred References
- `.planning/phases/03-socket-api-session-persistence/03-CONTEXT.md` §Deferred — original `@agent-browser` integration note
- `.planning/phases/05-config-distribution/05-CONTEXT.md` §Deferred — re-deferred agent-browser integration

</canonical_refs>

<code_context>
## Existing Code Insights

### Already Implemented
- `BrowserManager` — daemon spawn, socket communication, WebSocket stream, frame decoding
- `SplitNode::Preview` — preview pane variant in split tree with Picture + URL entry
- `create_preview_pane()` — GTK4 widget factory (Box > Entry + Overlay > Picture + Label)
- 6 lifecycle socket commands: browser.open/close/stream.enable/stream.disable/snapshot/screenshot
- ~50 P0+P1 socket commands via generic `BrowserAction` proxy in handlers.rs
- `GestureClick` + `EventControllerMotion` for mouse click/hover forwarding
- `EventControllerScroll` for scroll wheel forwarding
- `EventControllerKey` for keyboard input forwarding with GDK-to-CDP keyval mapping
- `gdk_keyval_to_cdp()` and `cdp_modifiers()` helper functions

### Still Needed
- Navigation toolbar (Back/Forward/Reload-Stop/Go/DevTools buttons)
- Async mouse motion channel (currently all events are synchronous)
- Loading state tracking for Reload↔Stop swap
- DevTools snapshot overlay (toggle on/off)
- Auto-refocus viewport on Picture click (grab_focus on click controller)
- P2 `not_supported` error responses for unsupported commands

### Established Patterns
- Socket command handlers: parse JSON-RPC params, dispatch to handler fn, return JSON response
- Tokio tasks for async I/O (agent-browser daemon communication)
- `mpsc::UnboundedSender` for cross-task events to GTK main thread
- `glib::MainContext::spawn_local` for async receivers on GTK thread

### Integration Points
- `src/shortcuts.rs` `handle_browser_open()` — attach nav bar + input controllers
- `src/browser.rs` `create_preview_pane()` — add nav bar widget row
- `src/socket/mod.rs` dispatch table — add P2 `not_supported` filtering

</code_context>

<specifics>
## Specific Ideas

- Navigation toolbar lives inside the preview container Box (above the overlay), not in a separate widget area
- DevTools overlay uses a `gtk4::ScrolledWindow` containing a `gtk4::Label` with monospace font for the snapshot tree
- Loading state can be inferred from daemon responses to `navigate` (start loading) and a periodic poll or stream event (load complete)
- Mouse motion throttling: ~60ms debounce (16fps) via `glib::timeout_add_local` to avoid flooding daemon

</specifics>

<deferred>
## Deferred Ideas

- Full embedded browser pane via WebKitGTK or CEF (D-03 stretch goal)
- Browser tab management within cmux (multiple browser tabs per pane)
- JavaScript console panel (separate from DOM snapshot overlay)
- Find-in-page UI overlay on preview pane
- Browser history / bookmarks
- macOS `browser.*` socket command wire-compatibility (adapt once Linux implementation stabilizes)
- P2 commands: network intercept/mock, emulation settings, trace/video/screencast, script injection, raw input device injection
- Console log panel as alternative/complement to DOM snapshot overlay

</deferred>

---

*Phase: 08-add-agent-browser*
*Context gathered: 2026-03-26, updated: 2026-03-27 via discuss-phase --analyze*
