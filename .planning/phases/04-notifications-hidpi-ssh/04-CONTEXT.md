# Phase 4: Notifications + HiDPI + SSH - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete the feature set: per-pane terminal bell tracking with workspace indicators and
desktop notifications; verify and fix HiDPI rendering at all scale factors; add SSH remote
workspaces using the existing Go daemon/remote relay.

No config file (Phase 5). No browser panel. No new socket protocol — implement the
`notification.*` commands that Phase 3 stubbed as `not_implemented`.

</domain>

<decisions>
## Implementation Decisions

### Bell & Attention Tracking

- **D-01:** Track attention via terminal bell (`\a`) only. OSC 99 markers deferred.
- **D-02:** Route bell events through `action_cb` in `callbacks.rs` — currently only handles
  `.render`. Add a bell/notification action handler that updates per-pane attention state.
- **D-03:** Per-pane attention state stored on the pane/surface model (e.g., `has_attention: bool`
  on leaf nodes in the split tree). Workspace has attention if any of its panes do.

### Workspace Attention Indicator

- **D-04:** Small colored dot next to workspace name in the sidebar when any pane in that
  workspace has unread bell activity. Matches macOS cmux behavior.
- **D-05:** Attention clears when the user switches to the workspace containing the bell pane
  (workspace-level focus clear, not pane-level).

### Desktop Notifications

- **D-06:** Desktop notification fires only when the GTK4 window is not focused. While
  focused, the sidebar dot is sufficient.
- **D-07:** Use GTK4's built-in `GNotification` via `GApplication`. No extra dependency
  (no libnotify/notify-rust).
- **D-08:** Notification content: workspace name + "Terminal bell" (or similar brief text).

### HiDPI / Fractional Scaling

- **D-09:** Verify-and-fix approach. Existing `notify::scale-factor` handler on GLArea
  (`surface.rs:368`) already calls `ghostty_surface_set_content_scale`. Test at 1x, 1.5x,
  and 2x. Fix any issues found in sidebar text, font sizing, or divider rendering.
- **D-10:** Rely on GTK4's native Wayland fractional scaling (`wp_fractional_scale_v1`,
  available in GTK4 4.12+). No custom fractional logic.
- **D-11:** Scope is primarily verification — Ghostty handles terminal DPI natively. Focus
  on ensuring non-terminal chrome (sidebar, dividers) scales correctly.

### SSH Remote Workspaces

- **D-12:** Reuse the existing Go `daemon/remote/` relay daemon. Compile for Linux, deploy
  to remote host. No Rust rewrite.
- **D-13:** Configure SSH workspaces via socket API only (`workspace.create` with remote
  target parameter). No GUI config in Phase 4 — CLI/socket-driven. Config file integration
  comes in Phase 5.
- **D-14:** Auto-reconnect with exponential backoff (1s, 2s, 4s... up to 30s cap) after
  network interruption. Show disconnected state in sidebar indicator.
- **D-15:** SSH workspace lifecycle: create via socket command → deploy cmuxd-remote to
  remote host → establish reverse tunnel → terminal sessions run on remote host.

### Socket Commands (Phase 3 → Phase 4)

- **D-16:** Implement `notification.*` socket commands that Phase 3 stubbed as `not_implemented`.
  Replace `SocketCommand::NotImplemented` routing for notification methods with real handlers.
- **D-17:** Per Phase 3 D-11 threading policy: notification state mutations scheduled on
  main thread via existing channel bridge.

### Claude's Discretion

- Exact Ghostty action tag for bell events (inspect `ghostty_action_tag_e` enum in FFI bindings)
- Sidebar dot color and GTK4 widget choice (CSS class vs. custom draw)
- GNotification category/priority settings
- cmuxd-remote deployment mechanism (scp binary, or bundled in app)
- SSH tunnel port selection and multiplexing details
- Whether `surface.health` response should include attention state

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Notification / Attention State — NOTF-01, NOTF-02, NOTF-03
- `.planning/REQUIREMENTS.md` §HiDPI / Display — HDPI-01, HDPI-02
- `.planning/REQUIREMENTS.md` §SSH Remote Workspaces — SSH-01, SSH-02, SSH-03, SSH-04

### Roadmap
- `.planning/ROADMAP.md` §Phase 4 — success criteria, phase goal, requirement IDs

### Prior Phase Context
- `.planning/phases/03-socket-api-session-persistence/03-CONTEXT.md` — Socket command threading
  policy (D-11), `notification.*` stub routing (D-10), session persistence patterns
- `.planning/phases/02-workspaces-pane-splits/02-CONTEXT.md` — Sidebar implementation (D-01),
  split tree structure (D-05, D-06), keyboard shortcuts (D-10)

### Existing Implementation (read before modifying)
- `src/ghostty/callbacks.rs` — `action_cb` handles `.render` only; bell action must be added
- `src/ghostty/surface.rs:364-378` — Existing `notify::scale-factor` handler for HiDPI
- `src/socket/commands.rs:52-53` — `NotImplemented` variant for Tier-2 stubs
- `src/socket/handlers.rs:513-514` — `not_implemented` error response for stubbed commands
- `src/sidebar.rs` — Sidebar widget; attention dot must be added here
- `src/workspace.rs` — Workspace model; needs attention state field
- `src/split_engine.rs` — Split tree with leaf nodes; needs per-pane attention tracking
- `src/app_state.rs` — Central state; session save triggers on mutations

### SSH Remote Daemon
- `daemon/remote/` — Go relay daemon (platform-agnostic); read `cmd/` and `go.mod` for
  current architecture before integrating

### Ghostty FFI
- `src/ghostty/ffi.rs` — FFI bindings; check `ghostty_action_tag_e` for bell/notification
  action variants

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `action_cb` in `callbacks.rs` — action dispatch point; add bell handling alongside render
- `notify::scale-factor` handler in `surface.rs` — may already satisfy HDPI-01/02
- `SocketCommand` enum + handler dispatch in `socket/` — pattern for adding notification commands
- `GL_AREA_REGISTRY` in `callbacks.rs` — maps surfaces for wakeup; can be extended for bell routing
- `trigger_session_save()` debounce pattern in `app_state.rs` — reusable for notification coalescing

### Established Patterns
- Socket command threading: parse off-main, schedule UI on main via mpsc channel
- GTK4 widget tree: `GtkPaned` nesting with `GtkGLArea` leaves
- State mutations trigger session save via `save_notify.notify_one()`
- Per-surface lifecycle managed in `surface.rs` (realize → render → scale-factor → destroy)

### Integration Points
- `action_cb` — add bell action branch (currently returns false for non-render)
- `sidebar.rs` — add attention dot widget per workspace row
- `workspace.rs` — add `has_attention: bool` field
- `split_engine.rs` — per-leaf attention tracking, bubble up to workspace
- `socket/commands.rs` — new `Notification*` command variants replacing `NotImplemented`

</code_context>

<specifics>
## Specific Ideas

- Bell events should propagate: Ghostty action_cb → per-pane attention flag → workspace
  attention flag → sidebar dot update + desktop notification (if unfocused)
- Desktop notification should include workspace name so user knows which workspace needs attention
- SSH workspace sidebar entry should show connection state (connected/disconnected/reconnecting)
- The Go daemon/remote/ is already platform-agnostic — just cross-compile for Linux targets

</specifics>

<deferred>
## Deferred Ideas

- OSC 99 progress markers — richer attention tracking beyond bell
- GUI SSH configuration dialog — config file + GUI comes in Phase 5
- Rust rewrite of cmuxd-remote — Go version works fine cross-platform
- Browser panel integration — separate future phase
- `notification.*` socket commands for external notification providers (Slack, etc.)
- Multi-window support (`window.create`) — still single-window in Phase 4
- `pane.break`, `pane.join`, `pane.swap` — advanced pane operations remain deferred

</deferred>

---

*Phase: 04-notifications-hidpi-ssh*
*Context gathered: 2026-03-26 via discuss-phase*
