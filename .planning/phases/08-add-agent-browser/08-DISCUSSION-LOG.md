# Phase 8: Add Agent-Browser - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-27
**Phase:** 08-add-agent-browser
**Areas discussed:** Navigation bar design, Input forwarding model, P0 command parity scope, DevTools view
**Mode:** --analyze (trade-off tables provided for each area)

---

## Navigation Bar Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Single toolbar row | [ ◀ ][ ▶ ][ ↻ ] [ URL... ] [ → ] [ {} ] — compact ~36px, familiar browser UX | ✓ |
| Split rows | Nav buttons on one row, URL bar on second row — 64px overhead | |
| URL bar only + shortcuts | Minimal UI, Alt+Left/Right for nav, F5 for reload | |

**User's choice:** Single toolbar row
**Notes:** Matches Chrome/Firefox convention. URL entry hexpand handles narrow panes.

---

## Reload/Stop Swap

| Option | Description | Selected |
|--------|-------------|----------|
| Swap Reload↔Stop | Chrome behavior: ✕ during load, ↻ when idle. Requires loading state tracking. | ✓ |
| Always show Reload | Simpler — no state tracking. Stop via Escape key. | |
| Show both | Reload and Stop side by side. More space, no state tracking. | |

**User's choice:** Swap Reload↔Stop
**Notes:** Standard browser UX. Loading state tracked from daemon navigation events.

---

## Input Forwarding Model

| Option | Description | Selected |
|--------|-------------|----------|
| Hybrid async/sync | Mouse motion fire-and-forget via tokio channel. Clicks/keyboard sync. | ✓ |
| All synchronous | Current implementation. May stutter on rapid mouse movement. | |
| All async fire-and-forget | Best perf but click/key events could be silently lost. | |

**User's choice:** Hybrid async/sync
**Notes:** Balances performance (no stutter on hover) with reliability (clicks/keys confirmed).

---

## Focus Handoff (URL bar ↔ viewport)

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-refocus viewport on click | Clicking Picture sets focus back to container for keyboard input. | ✓ |
| Require explicit Escape or Tab | URL bar keeps focus until user explicitly leaves. | |
| Split focus model | Both can receive keyboard simultaneously. More complex. | |

**User's choice:** Auto-refocus viewport on click
**Notes:** Natural browser UX. URL bar is independent; click viewport to return keyboard to Chrome.

---

## P0 Command Parity Scope

| Option | Description | Selected |
|--------|-------------|----------|
| P0+P1 wired, P2 not_supported | Generic proxy for all P0+P1. Explicit not_supported for P2. | ✓ |
| P0 only, P1 deferred to 8.1 | Strip P1 routing to separate phase. | |
| Wire everything, best-effort | Proxy all including P2. Daemon returns errors for unsupported. | |

**User's choice:** P0+P1 wired, P2 not_supported
**Notes:** Matches port spec recommendation. P2 commands (network intercept, emulation, trace) explicitly rejected.

---

## DevTools View

| Option | Description | Selected |
|--------|-------------|----------|
| DOM snapshot overlay | Toggle shows browser.snapshot accessibility tree as scrollable overlay on viewport. | ✓ |
| Console log panel | Shows console.list + errors.list as scrollable log below viewport. | |
| Split bottom panel (DOM + Console tabs) | Resizable bottom panel with tabs. Most complete but significant UI work. | |
| Defer DevTools entirely | Ship without DevTools toggle. Add later. | |

**User's choice:** DOM snapshot overlay
**Notes:** Compact, no extra panes. Uses existing browser.snapshot command. Shows element refs useful for agents.

---

## Claude's Discretion

- Async mouse motion channel implementation details (throttle rate, channel type)
- Loading state detection mechanism
- DevTools overlay styling and scroll behavior
- P2 command filtering — at routing layer or handler layer

## Deferred Ideas

- Console log panel (complement to DOM snapshot)
- P2 command support (network, emulation, trace)
- Full DevTools panel with tabs
