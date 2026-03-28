---
phase: 04-notifications-hidpi-ssh
verified: 2026-03-27T20:15:00Z
status: human_needed
score: 9/9 must-haves verified
re_verification:
  previous_status: human_needed
  previous_score: 8/9
  gaps_closed:
    - "Desktop notification fires via notify-rust D-Bus instead of silently-failing gio::Notification"
    - "SSH deploy has install script; lifecycle has MAX_RETRIES=10 and FailureKind permanent/transient classification"
    - "proxy.stream routing from cmuxd-remote to terminal surfaces is now implemented (was TODO)"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Trigger terminal bell with window unfocused, verify desktop notification appears"
    expected: "notify-rust sends notification via org.freedesktop.Notifications D-Bus; notification daemon shows 'Terminal Bell' with workspace name"
    why_human: "Requires running app with notification daemon; gap closure plan 06 replaced gio::Notification with notify-rust but needs re-test"
  - test: "Run scripts/install-cmuxd-remote.sh then create SSH workspace via socket"
    expected: "cmuxd-remote installs to XDG data dir; workspace.create with remote_target shows connection state in sidebar; no infinite reconnect on failure"
    why_human: "Requires SSH target host and running app; gap closure plan 07 added install script and retry bounds"
  - test: "HiDPI rendering across monitor move"
    expected: "Terminal text stays crisp when window moves between monitors with different DPI"
    why_human: "Requires multi-DPI hardware setup"
---

# Phase 04: Notifications, HiDPI, SSH Verification Report

**Phase Goal:** Users see per-pane activity indicators and desktop notifications; the app renders correctly at any display scale; SSH workspaces connect to remote hosts
**Verified:** 2026-03-27T20:15:00Z
**Status:** human_needed
**Re-verification:** Yes -- after gap closure (plans 06 and 07)

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A terminal bell sets has_attention=true on the pane's SplitNode::Leaf | VERIFIED | `src/split_engine.rs` line 29: `has_attention: bool` on Leaf; `set_attention()` at line 217; `action_cb` handles `GHOSTTY_ACTION_RING_BELL` at line 111 of `callbacks.rs`; `BELL_PENDING` polled in `main.rs` line 330 dispatches to `set_pane_attention` |
| 2 | Workspace has_attention is derived from any pane having attention | VERIFIED | `src/app_state.rs`: `set_pane_attention()` calls `engine.root.set_attention()` then `engine.root.any_attention()` to derive workspace state |
| 3 | Sidebar shows amber dot next to workspace name when workspace has attention | VERIFIED | `src/sidebar.rs` line 163: `attention-dot` CSS class; `src/main.rs` line 39: `.attention-dot` CSS with amber background; `update_sidebar_attention()` toggles visibility |
| 4 | Attention clears when user switches to the workspace | VERIFIED | `src/app_state.rs`: `switch_to_index` calls `clear_workspace_attention(index)` |
| 5 | Desktop notification fires via notify-rust when bell rings and window is unfocused | VERIFIED | `src/app_state.rs` line 566: `send_bell_notification()` uses `notify_rust::Notification::new()` with background thread; called when `!window_focused` |
| 6 | Bell notifications are rate-limited to 1 per workspace per 5 seconds | VERIFIED | `src/workspace.rs`: `last_notification: Option<Instant>`; `src/app_state.rs` checks elapsed >= 5s before sending |
| 7 | Terminal surface renders correctly at multiple scale factors | VERIFIED | `src/ghostty/surface.rs`: `notify::scale-factor` handler at line 393, calls `ghostty_surface_set_content_scale`; fractional scale via `gdk4 v4_12` feature |
| 8 | SSH workspace creation, deployment, tunnel, and reconnection with bounded retries | VERIFIED | `src/ssh/tunnel.rs`: `run_ssh_lifecycle` with `MAX_RETRIES=10`, `FailureKind` enum for permanent vs transient; `deploy.rs` references install script; `scripts/install-cmuxd-remote.sh` exists |
| 9 | Terminal sessions in SSH workspace route I/O to remote host via proxy.stream | VERIFIED | `src/ssh/tunnel.rs`: `open_remote_stream()` sends `session.spawn` + `proxy.stream.subscribe` JSON-RPC; handles `proxy.stream.data/eof/error` events; `src/ssh/bridge.rs`: `SshBridge` maps pane_id to stream_id, `ssh_io_write_cb` sends user keystrokes as base64 via `WriteRequest` |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/split_engine.rs` | has_attention on Leaf, attention methods | VERIFIED | set_attention, any_attention, clear_all_attention, pane_has_attention all present |
| `src/workspace.rs` | ConnectionState enum, remote_target, has_attention, last_notification | VERIFIED | All fields and enum variants present |
| `src/app_state.rs` | set_pane_attention, clear_workspace_attention, send_bell_notification with notify-rust | VERIFIED | notify_rust::Notification replaces gio::Notification; background thread dispatch |
| `src/ghostty/callbacks.rs` | RING_BELL handler, BELL_PENDING, BELL_PANE_ID atomics | VERIFIED | Handler at line 111, atomics at lines 38-40 |
| `src/ghostty/surface.rs` | Scale-factor change handler, set_content_scale FFI | VERIFIED | notify::scale-factor at line 393, fractional scale support |
| `src/sidebar.rs` | attention-dot in row layout | VERIFIED | Line 163: dot widget with attention-dot CSS class |
| `src/ssh/mod.rs` | SshEvent enum, channel types | VERIFIED | 29 lines, SshEvent::StateChanged with workspace_id and ConnectionState |
| `src/ssh/tunnel.rs` | run_ssh_lifecycle, MAX_RETRIES, FailureKind, open_remote_stream, proxy.stream handling | VERIFIED | 487 lines; bounded retries, permanent failure exit, full proxy routing |
| `src/ssh/deploy.rs` | deploy_remote via scp | VERIFIED | 58 lines; SSH mkdir + scp + chmod workflow |
| `src/ssh/bridge.rs` | SshBridge pane-stream mapping, IoWriteContext, ssh_io_write_cb | VERIFIED | 166 lines; bidirectional pane-stream routing |
| `scripts/install-cmuxd-remote.sh` | Dev install script for cmuxd-remote binary | VERIFIED | 875 bytes, executable |
| `src/socket/commands.rs` | NotificationList, NotificationClear, remote_target on WorkspaceCreate | VERIFIED | Present |
| `src/socket/mod.rs` | notification.list, notification.clear dispatch | VERIFIED | Present |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| callbacks.rs RING_BELL | app_state.rs set_pane_attention | BELL_PENDING atomic polled in main.rs line 330 | WIRED | Main thread timer polls atomic, dispatches to AppState |
| app_state.rs | sidebar GTK dot | update_sidebar_attention toggles dot visibility | WIRED | Navigates row hierarchy, sets dot visible/invisible |
| app_state.rs send_bell_notification | notify-rust D-Bus | notify_rust::Notification in background thread | WIRED | Replaces gio::Notification; no .desktop file required |
| surface.rs scale-factor | ghostty FFI | ghostty_surface_set_content_scale on notify | WIRED | Called at realize, initial setup, and scale-factor change |
| socket workspace.create | SSH lifecycle | remote_target triggers create_remote_workspace + run_ssh_lifecycle | WIRED | handlers.rs spawns tokio task for SSH lifecycle |
| tunnel.rs proxy.stream events | bridge.rs | stream_to_pane mapping dispatches to output_tx | WIRED | Data decoded from base64, routed via OutputEvent to GTK |
| bridge.rs ssh_io_write_cb | tunnel.rs | WriteRequest via mpsc channel | WIRED | User keystrokes base64-encoded and sent through tunnel |
| tunnel.rs FailureKind | lifecycle loop | Permanent exits, Transient retries up to MAX_RETRIES | WIRED | Binary-not-found classified as permanent |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| sidebar attention dot | has_attention | action_cb RING_BELL -> BELL_PENDING -> set_pane_attention | Yes (Ghostty bell events) | FLOWING |
| desktop notification | workspace_name | AppState.workspaces[idx].name | Yes (live workspace state) | FLOWING |
| connection-state subtitle | ConnectionState | SshEvent channel from tunnel.rs | Yes (SSH process state) | FLOWING |
| SSH terminal I/O | proxy.stream.data | cmuxd-remote stdio via tunnel.rs | Yes (base64 decoded, routed to pane via bridge) | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Binary tests pass | `cargo test --bin cmux-linux` | 34 passed, 0 failed | PASS |
| MAX_RETRIES bounded | test_max_retries_is_reasonable in test suite | Asserts 5 <= MAX_RETRIES <= 20 | PASS |
| No TODOs in SSH module | grep TODO src/ssh/ | 0 matches | PASS |
| notify-rust in Cargo.toml | grep notify-rust Cargo.toml | `notify-rust = "4"` present | PASS |
| install script exists | ls scripts/install-cmuxd-remote.sh | 875 bytes, executable | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| NOTF-01 | 04-01, 04-03 | Per-pane attention state tracks terminal bell activity | SATISFIED | has_attention on SplitNode::Leaf, RING_BELL handler, notification.list socket command |
| NOTF-02 | 04-01, 04-03 | Workspace list shows visual indicator for unread activity | SATISFIED | Amber attention-dot in sidebar, CSS styling, visibility toggle |
| NOTF-03 | 04-01, 04-06 | Desktop notification via D-Bus on unfocused bell | SATISFIED | notify-rust replaces gio::Notification; background thread dispatch; rate-limited |
| HDPI-01 | 04-02 | App renders correctly at 1x, 1.5x, 2x scale factors | SATISFIED | Fractional scale via GdkSurface::scale() with v4_12 feature, integer fallback |
| HDPI-02 | 04-02 | Scale factor updates on monitor move | SATISFIED | notify::scale-factor handler calls ghostty_surface_set_content_scale |
| SSH-01 | 04-04 | User can configure workspace with remote SSH target | SATISFIED | workspace.create accepts remote_target, creates SSH workspace |
| SSH-02 | 04-04, 04-07 | cmuxd-remote deployed to remote host | SATISFIED | deploy.rs scp workflow; install-cmuxd-remote.sh dev install script |
| SSH-03 | 04-04 | Terminal sessions in SSH workspace run on remote host | SATISFIED | proxy.stream routing implemented in tunnel.rs + bridge.rs; bidirectional I/O via ssh_io_write_cb. Note: REQUIREMENTS.md maps SSH-03 to Phase 7 for full completion, but infrastructure is present. |
| SSH-04 | 04-04, 04-07 | SSH workspace reconnects after network interruption | SATISFIED | Exponential backoff (1s-30s cap), MAX_RETRIES=10, FailureKind permanent vs transient |

No orphaned requirements found.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | - | No TODOs, FIXMEs, or stubs found in phase 4 artifacts | - | - |

### Human Verification Required

### 1. Desktop Notification (Post-Gap-Closure Re-test)

**Test:** Unfocus the app window. In a terminal, run `echo -e '\a'`. Verify desktop notification appears.
**Expected:** Notification with "Terminal Bell" title and "{workspace} - Terminal bell" body via notify-rust D-Bus. Rapid bells rate-limited to 1 per 5 seconds.
**Why human:** Gap closure plan 06 replaced gio::Notification with notify-rust. The UAT reported "no desktop notification seen" with the old implementation. Needs re-test to confirm the fix works.

### 2. SSH Workspace (Post-Gap-Closure Re-test)

**Test:** Run `scripts/install-cmuxd-remote.sh` to install the binary. Send `workspace.create` with `remote_target` via socket. Then intentionally disrupt SSH to test reconnect.
**Expected:** Binary installs to XDG data dir. Workspace appears in sidebar with connection state. On failure, retries up to 10 times with backoff. Binary-not-found exits immediately (permanent failure).
**Why human:** Gap closure plan 07 added install script and retry bounds. The UAT reported "binary not found" and "infinite reconnect". Needs re-test.

### 3. HiDPI Multi-Monitor Rendering

**Test:** If multi-DPI monitors available, drag window between them.
**Expected:** Terminal text stays crisp, scale-factor change log message appears in stderr.
**Why human:** Requires multi-DPI hardware setup.

### Gaps Summary

All 9 observable truths are now verified at the code level. The two gaps found during UAT (desktop notification failure and SSH deploy/retry issues) have been closed by plans 06 and 07 respectively. The proxy.stream routing that was previously a TODO is now fully implemented with bidirectional I/O through SshBridge.

Three items require human re-verification: (1) desktop notifications with the new notify-rust implementation, (2) SSH workspace creation with the install script and bounded retries, and (3) HiDPI rendering on multi-DPI hardware.

All 34 unit tests pass. No TODOs, stubs, or anti-patterns remain in phase 4 artifacts.

---

_Verified: 2026-03-27T20:15:00Z_
_Verifier: Claude (gsd-verifier)_
