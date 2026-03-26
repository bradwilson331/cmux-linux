---
phase: 04-notifications-hidpi-ssh
verified: 2026-03-26T14:30:00Z
status: human_needed
score: 8/9 must-haves verified
human_verification:
  - test: "Trigger terminal bell in background workspace, verify amber dot in sidebar"
    expected: "Amber 8px dot appears next to workspace name, disappears on switch"
    why_human: "Requires running app with GTK4 rendering and multiple workspaces"
  - test: "Trigger bell with window unfocused, verify desktop notification"
    expected: "GNotification with 'Terminal Bell' title appears, rate-limited to 1/5s"
    why_human: "Requires running app and desktop notification daemon"
  - test: "Move window between monitors with different DPI"
    expected: "Terminal text stays crisp, scale-factor log message appears"
    why_human: "Requires multi-DPI hardware setup"
  - test: "Create SSH workspace via socket API with remote_target"
    expected: "Sidebar shows SSH workspace with connection state subtitle"
    why_human: "Requires running app and SSH target host"
  - test: "Verify proxy.stream terminal I/O routes to remote host"
    expected: "Terminal sessions execute on remote, not local"
    why_human: "Known gap: proxy.stream routing is TODO, needs end-to-end SSH test"
---

# Phase 04: Notifications, HiDPI, SSH Verification Report

**Phase Goal:** Users see per-pane activity indicators and desktop notifications; the app renders correctly at any display scale; SSH workspaces connect to remote hosts
**Verified:** 2026-03-26T14:30:00Z
**Status:** human_needed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A terminal bell sets has_attention=true on the pane's SplitNode::Leaf | VERIFIED | `src/split_engine.rs` line 29: `has_attention: bool` on Leaf; `set_attention()` at line 160; `action_cb` handles `GHOSTTY_ACTION_RING_BELL` at line 111 of `callbacks.rs`; `BELL_PENDING` polled in `main.rs` line 269 dispatches to `set_pane_attention` |
| 2 | Workspace has_attention is derived from any pane having attention | VERIFIED | `src/app_state.rs` line 382: `set_pane_attention()` calls `engine.root.set_attention()` then `engine.root.any_attention()` to derive workspace state |
| 3 | Sidebar shows amber dot next to workspace name when workspace has attention | VERIFIED | `src/app_state.rs` lines 93,158: `attention-dot` CSS class on dot widget; `src/main.rs` line 35: `.attention-dot` CSS with `background-color: #e8a444`; `update_sidebar_attention()` at line 418 toggles visibility |
| 4 | Attention clears when user switches to the workspace | VERIFIED | `src/app_state.rs` line 293: `switch_to_index` calls `clear_workspace_attention(index)` |
| 5 | Desktop notification fires via GNotification when bell rings and window is unfocused | VERIFIED | `src/app_state.rs` line 465: `send_bell_notification()` uses `gio::Notification`; called from `set_pane_attention` when `!window_focused` (line 398) |
| 6 | Bell notifications are rate-limited to 1 per workspace per 5 seconds | VERIFIED | `src/workspace.rs` line 59: `last_notification: Option<Instant>`; `src/app_state.rs` checks elapsed >= 5s before sending |
| 7 | Terminal surface renders correctly at multiple scale factors | VERIFIED | `src/ghostty/surface.rs`: `notify::scale-factor` handler at line 364, calls `ghostty_surface_set_content_scale` at line 380; fractional scale via `gdk4 v4_12` feature; diagnostic logging at line 378 |
| 8 | SSH workspace creation, deployment, tunnel, and reconnection infrastructure exists | VERIFIED | `src/ssh/mod.rs`, `tunnel.rs`, `deploy.rs` exist; `run_ssh_lifecycle` with exponential backoff; `deploy_remote` via scp; socket `workspace.create` accepts `remote_target` |
| 9 | Terminal sessions in SSH workspace run on the remote host via cmuxd-remote | PARTIAL | SSH tunnel connects and sends JSON-RPC handshake, but proxy.stream data routing to terminal surfaces is a TODO (tunnel.rs line 73). Sessions do not actually execute on the remote host yet. |

**Score:** 8/9 truths verified (1 partial -- SSH terminal I/O routing incomplete)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/split_engine.rs` | has_attention on Leaf, set_attention/any_attention/clear_all_attention/pane_has_attention | VERIFIED | All methods present, has_attention initialized to false in all constructors |
| `src/workspace.rs` | ConnectionState enum, remote_target, has_attention, last_notification, new_remote | VERIFIED | All fields and enum present with Local/Connected/Disconnected/Reconnecting variants |
| `src/app_state.rs` | set_pane_attention, clear_workspace_attention, build_sidebar_row, create_remote_workspace, update_connection_state, ssh_event_tx, runtime_handle | VERIFIED | All methods and fields present and wired |
| `src/ghostty/callbacks.rs` | RING_BELL handler, BELL_PENDING, BELL_PANE_ID | VERIFIED | Handler at line 111, atomics at lines 38-40 |
| `src/main.rs` | attention-dot CSS, connection-state CSS, bell processing timer, SSH event channel, mod ssh | VERIFIED | All present: CSS lines 35-52, bell poll line 269, SSH event recv line 276, mod ssh line 14 |
| `src/sidebar.rs` | attention-dot in row layout | VERIFIED | Line 163: dot widget with attention-dot CSS class |
| `src/ssh/mod.rs` | SshEvent enum, channel types | VERIFIED | SshEvent::StateChanged, SshEventTx/Rx types |
| `src/ssh/tunnel.rs` | run_ssh_lifecycle, backoff, reconnection | VERIFIED | Lifecycle loop with deploy, connect, backoff (1s-30s cap), reconnect |
| `src/ssh/deploy.rs` | deploy_remote via scp | VERIFIED | SSH mkdir + scp + chmod workflow |
| `src/socket/commands.rs` | NotificationList, NotificationClear, remote_target on WorkspaceCreate | VERIFIED | Lines 18, 53-54 |
| `src/socket/mod.rs` | notification.list, notification.clear dispatch, remote_target extraction | VERIFIED | Lines 248-255, 156 |
| `src/socket/handlers.rs` | Notification handlers, SSH lifecycle spawn | VERIFIED | Lines 543-566, 88-89 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| callbacks.rs | app_state.rs | BELL_PENDING atomic -> glib timer -> set_pane_attention | WIRED | main.rs line 269 polls BELL_PENDING, calls set_pane_attention |
| app_state.rs | sidebar (GTK) | update_sidebar_attention toggles dot visibility | WIRED | Line 418: navigates row hierarchy, sets dot.set_visible() |
| app_state.rs | switch_to_index | clear_workspace_attention called before switch | WIRED | Line 293 in switch_to_index |
| surface.rs | ghostty FFI | ghostty_surface_set_content_scale on scale change | WIRED | Lines 122, 209, 380: called in realize, initial setup, and scale-factor notify |
| socket/mod.rs | commands.rs | notification.list/clear dispatched to enum variants | WIRED | Lines 248-255 map to NotificationList/NotificationClear |
| handlers.rs | app_state.rs | NotificationClear calls clear_workspace_attention | WIRED | Line 564 |
| socket/mod.rs | ssh lifecycle | workspace.create with remote_target triggers SSH | WIRED | handlers.rs line 89: remote_target branch calls create_remote_workspace + spawns run_ssh_lifecycle |
| ssh/tunnel.rs | workspace model | ConnectionState updates via SshEvent channel | WIRED | tunnel.rs sends SshEvent::StateChanged; main.rs line 276-278 receives and calls update_connection_state |
| ssh/tunnel.rs | cmuxd-remote stdio | JSON-RPC handshake sent, responses read | PARTIAL | Handshake sent (system.hello), responses logged but proxy.stream not routed to terminal surfaces |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| sidebar attention dot | has_attention | action_cb RING_BELL -> BELL_PENDING -> set_pane_attention | Yes (bell events from Ghostty) | FLOWING |
| notification.list response | workspaces[].has_attention | AppState.workspaces | Yes (live workspace state) | FLOWING |
| connection-state subtitle | ConnectionState | SshEvent channel from tunnel.rs | Yes (SSH process state changes) | FLOWING |
| SSH terminal I/O | proxy.stream | cmuxd-remote stdio | No (responses logged, not routed) | STATIC |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All tests pass | `cargo test --bin cmux-linux` | 14 passed, 0 failed | PASS |
| Backoff duration correctness | `test_backoff_duration` in test suite | 1s, 2s, 4s, 8s, 16s, 30s, 30s verified | PASS |
| Compilation with all phase 4 code | `cargo test` output | Compiled with 13 warnings (unused vars), 0 errors | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| NOTF-01 | 04-01, 04-03 | Per-pane attention state tracks terminal bell activity | SATISFIED | has_attention on SplitNode::Leaf, RING_BELL handler, notification.list socket command |
| NOTF-02 | 04-01, 04-03 | Workspace list shows visual indicator for unread activity | SATISFIED | Amber attention-dot in sidebar, notification.list returns has_attention |
| NOTF-03 | 04-01 | Desktop notification via GTK4 API on unfocused bell | SATISFIED | GNotification in send_bell_notification, window focus check, rate limiting |
| HDPI-01 | 04-02 | App renders correctly at 1x, 1.5x, 2x scale factors | SATISFIED | Fractional scale via GdkSurface::scale() with v4_12 feature, integer fallback |
| HDPI-02 | 04-02 | Scale factor updates on monitor move | SATISFIED | notify::scale-factor handler calls ghostty_surface_set_content_scale |
| SSH-01 | 04-04 | User can configure workspace with remote SSH target | SATISFIED | workspace.create accepts remote_target, creates SSH workspace |
| SSH-02 | 04-04 | cmuxd-remote deployed to remote host | SATISFIED | deploy.rs: scp binary, mkdir, chmod workflow |
| SSH-03 | 04-04 | Terminal sessions in SSH workspace run on remote host | NEEDS HUMAN | SSH tunnel connects and sends JSON-RPC, but proxy.stream routing to terminal surfaces is a TODO -- sessions do not actually execute remotely yet |
| SSH-04 | 04-04 | SSH workspace reconnects after network interruption | SATISFIED | Exponential backoff in run_ssh_lifecycle (1s-30s cap), ConnectionState::Reconnecting updates |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| src/ssh/tunnel.rs | 73 | TODO: Route proxy.stream data to terminal surfaces | Warning | SSH terminal I/O not routed to surfaces; sessions cannot execute remotely. Known and documented gap. |
| src/ssh/mod.rs | 16 | Unused type alias SshEventRx | Info | Compiler warning, no functional impact |

### Human Verification Required

### 1. Bell Attention Dot End-to-End

**Test:** Open app with two workspaces. In workspace 2 run `echo -e '\a'`. Switch to workspace 1.
**Expected:** Amber dot appears next to "Workspace 2" in sidebar. Switching back to workspace 2 clears it.
**Why human:** Requires running GTK4 app with Ghostty terminal surfaces

### 2. Desktop Notification on Unfocused Bell

**Test:** With app window unfocused, trigger bell in a workspace.
**Expected:** Desktop notification appears with "Terminal Bell" title. Rapid bells are rate-limited (1 per 5s per workspace).
**Why human:** Requires desktop notification daemon and window focus state

### 3. HiDPI Multi-Monitor Rendering

**Test:** If multi-DPI monitors available, drag window between them.
**Expected:** Terminal text stays crisp, stderr shows scale-factor change message.
**Why human:** Requires multi-DPI hardware

### 4. SSH Workspace Creation via Socket

**Test:** Send `{"id":1,"method":"workspace.create","params":{"remote_target":"user@host"}}` via socket.
**Expected:** Sidebar shows new workspace with "SSH: user@host" name and connection state subtitle.
**Why human:** Requires running app and SSH target host

### 5. SSH Terminal I/O (Known Gap)

**Test:** In SSH workspace, verify terminal commands execute on remote host.
**Expected:** Commands run remotely. Currently known to NOT work -- proxy.stream routing is TODO.
**Why human:** Requires SSH target, and implementation is incomplete

### Gaps Summary

The phase is substantially complete with all 9 requirements addressed at the infrastructure level. The one significant gap is SSH-03 (terminal sessions run on remote host): while the SSH tunnel connects, deploys cmuxd-remote, establishes JSON-RPC handshake, and handles reconnection with exponential backoff, the actual proxy.stream terminal I/O routing from cmuxd-remote back to Ghostty terminal surfaces is not implemented (TODO at tunnel.rs:73). This means SSH workspaces can be created and show connection state in the sidebar, but terminal commands do not actually execute on the remote host.

All notification features (NOTF-01/02/03) are fully wired from Ghostty action_cb through to sidebar dot and desktop notification. HiDPI support (HDPI-01/02) is verified with fractional scale support. The notification socket API (notification.list/clear) is fully operational. SSH infrastructure (SSH-01/02/04) is complete -- only the terminal I/O proxy (SSH-03) remains incomplete.

All 14 unit tests pass. Compilation succeeds with only minor warnings.

---

_Verified: 2026-03-26T14:30:00Z_
_Verifier: Claude (gsd-verifier)_
