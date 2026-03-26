# Phase 8: Add Agent-Browser - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-26
**Phase:** 08-add-agent-browser
**Areas discussed:** Integration model, Visual rendering, Socket API surface, Lifecycle management

---

## Integration Model

| Option | Description | Selected |
|--------|-------------|----------|
| Socket proxy | cmux socket exposes `browser.*` commands that proxy to agent-browser's CDP connection. Tight coupling. | |
| Bundled CLI | agent-browser binary ships with cmux, launchable from terminal panes. Loose coupling. | |
| Hybrid | Bundled binary + thin socket API layer for key operations. Agents can use either path. | ✓ |

**User's choice:** Hybrid
**Notes:** None

---

## Visual Rendering

| Option | Description | Selected |
|--------|-------------|----------|
| Headless only | No visual pane. Agents interact via CLI/socket. Screenshots via CLI. | |
| Embedded browser pane | WebKitGTK or CEF renders live browser content in a split pane. | (stretch goal) |
| Headless + preview pane | Browser runs headless, cmux renders screenshots/snapshots in a pane on demand. | ✓ |

**User's choice:** Both embedded browser pane and headless + preview — then clarified to CDP streaming model
**Notes:** Follow-up question on rendering engine resolved to option 3: CDP streaming to preview. agent-browser streams screenshots/DOM snapshots via `stream` command, cmux renders them. No real browser widget. Embedded pane noted as stretch goal.

---

## Socket API Surface

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal control | Just `browser.open`, `browser.close`, `browser.snapshot`, `browser.screenshot`. | |
| Full proxy | Mirror most agent-browser commands through socket. | |
| Lifecycle + streaming | `browser.open`, `browser.close`, `browser.stream.enable/disable`, `browser.snapshot`, `browser.screenshot`. Interaction via CLI. | ✓ |

**User's choice:** Lifecycle + streaming
**Notes:** Interaction commands (click, fill, eval, etc.) go through agent-browser CLI directly.

---

## Lifecycle Management

| Option | Description | Selected |
|--------|-------------|----------|
| cmux-managed | Auto-start and auto-stop with workspace lifecycle. | |
| User-managed | User starts agent-browser themselves, cmux connects. | |
| Auto-start, manual stop | cmux spawns on first use, leaves running. User stops explicitly. | ✓ |

**User's choice:** Auto-start, manual stop
**Notes:** Avoids repeated cold starts. Stops via `browser.close` or cmux shutdown.

---

## Claude's Discretion

- Binary bundling strategy
- CDP streaming transport details
- Preview pane GTK4 widget choice
- Chrome install management
- Error handling for missing dependencies

## Deferred Ideas

- Full embedded browser pane (stretch goal)
- Browser tab management
- JavaScript console UI
- Find-in-page overlay
- Browser history/bookmarks
- macOS wire-compatibility for browser commands
