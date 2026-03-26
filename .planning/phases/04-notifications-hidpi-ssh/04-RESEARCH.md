# Phase 4: Notifications + HiDPI + SSH - Research

**Researched:** 2026-03-26
**Domain:** GTK4 notifications, Ghostty action_cb bell routing, HiDPI scaling, SSH remote relay integration
**Confidence:** HIGH

## Summary

Phase 4 adds three features to the cmux Linux port: (1) per-pane bell attention tracking with sidebar indicators and desktop notifications, (2) HiDPI verification/fix across scale factors, and (3) SSH remote workspaces using the existing Go `cmuxd-remote` daemon.

The bell/notification work is well-scoped. Ghostty already fires `GHOSTTY_ACTION_RING_BELL` (tag 49) and `GHOSTTY_ACTION_DESKTOP_NOTIFICATION` (tag 31) through `action_cb`. The current `action_cb` only handles `GHOSTTY_ACTION_RENDER` and returns false for everything else. Adding bell handling requires: catching the action, identifying which surface fired it via the `ghostty_target_s` union (tag=SURFACE, target.surface=ptr), mapping to pane_id via `SURFACE_REGISTRY`, setting an attention flag, and updating the sidebar. Desktop notifications use `gio::Application::send_notification()` (already available since the app is a `gtk4::Application` which implements `gio::Application`).

HiDPI is likely already working. The `notify::scale-factor` handler on GLArea (surface.rs:368) calls `ghostty_surface_set_content_scale` and `ghostty_surface_refresh`. GTK4 4.14.5 (installed) has full `wp_fractional_scale_v1` Wayland support. The work is verification and fixing any non-terminal chrome (sidebar, dividers) scaling issues.

SSH integration reuses the Go `cmuxd-remote` daemon which already implements proxy/session RPC over stdio. It needs to be cross-compiled for remote Linux targets, deployed via SSH/scp, and connected via a reverse tunnel. The Rust side needs a new workspace type (remote) with connection lifecycle management.

**Primary recommendation:** Implement bell tracking and notifications first (smallest scope, immediate user value), then HiDPI verification, then SSH as the largest feature.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: Track attention via terminal bell (`\a`) only. OSC 99 markers deferred.
- D-02: Route bell events through `action_cb` in `callbacks.rs` -- add bell/notification action handler that updates per-pane attention state.
- D-03: Per-pane attention state stored on the pane/surface model (e.g., `has_attention: bool` on leaf nodes in the split tree). Workspace has attention if any of its panes do.
- D-04: Small colored dot next to workspace name in the sidebar when any pane in that workspace has unread bell activity.
- D-05: Attention clears when the user switches to the workspace containing the bell pane (workspace-level focus clear, not pane-level).
- D-06: Desktop notification fires only when the GTK4 window is not focused. While focused, the sidebar dot is sufficient.
- D-07: Use GTK4's built-in `GNotification` via `GApplication`. No extra dependency (no libnotify/notify-rust).
- D-08: Notification content: workspace name + "Terminal bell" (or similar brief text).
- D-09: Verify-and-fix approach for HiDPI. Existing `notify::scale-factor` handler on GLArea already calls `ghostty_surface_set_content_scale`. Test at 1x, 1.5x, and 2x.
- D-10: Rely on GTK4's native Wayland fractional scaling (`wp_fractional_scale_v1`, available in GTK4 4.12+). No custom fractional logic.
- D-11: Scope is primarily verification -- Ghostty handles terminal DPI natively. Focus on non-terminal chrome (sidebar, dividers).
- D-12: Reuse the existing Go `daemon/remote/` relay daemon. Compile for Linux, deploy to remote host. No Rust rewrite.
- D-13: Configure SSH workspaces via socket API only (`workspace.create` with remote target parameter). No GUI config in Phase 4.
- D-14: Auto-reconnect with exponential backoff (1s, 2s, 4s... up to 30s cap) after network interruption. Show disconnected state in sidebar indicator.
- D-15: SSH workspace lifecycle: create via socket command -> deploy cmuxd-remote to remote host -> establish reverse tunnel -> terminal sessions run on remote host.
- D-16: Implement `notification.*` socket commands that Phase 3 stubbed as `not_implemented`.
- D-17: Per Phase 3 D-11 threading policy: notification state mutations scheduled on main thread via existing channel bridge.

### Claude's Discretion
- Exact Ghostty action tag for bell events (inspect `ghostty_action_tag_e` enum in FFI bindings)
- Sidebar dot color and GTK4 widget choice (CSS class vs. custom draw)
- GNotification category/priority settings
- cmuxd-remote deployment mechanism (scp binary, or bundled in app)
- SSH tunnel port selection and multiplexing details
- Whether `surface.health` response should include attention state

### Deferred Ideas (OUT OF SCOPE)
- OSC 99 progress markers -- richer attention tracking beyond bell
- GUI SSH configuration dialog -- config file + GUI comes in Phase 5
- Rust rewrite of cmuxd-remote -- Go version works fine cross-platform
- Browser panel integration -- separate future phase
- `notification.*` socket commands for external notification providers (Slack, etc.)
- Multi-window support (`window.create`) -- still single-window in Phase 4
- `pane.break`, `pane.join`, `pane.swap` -- advanced pane operations remain deferred
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| NOTF-01 | Per-pane attention state tracks terminal activity (bell, OSC 99 markers) | Bell-only per D-01; `GHOSTTY_ACTION_RING_BELL` (tag 49) in action_cb; `has_attention: bool` on SplitNode::Leaf |
| NOTF-02 | Workspace list shows visual indicator for workspaces with unread activity | Sidebar dot CSS class on ListBoxRow; cleared on workspace switch per D-05 |
| NOTF-03 | Desktop notification sent via GTK4 API when terminal rings bell while app is unfocused | `gio::Application::send_notification()` with `gio::Notification`; check `gtk4::Window::is_active()` for focus state |
| HDPI-01 | App renders correctly at 1x, 1.5x, and 2x display scale factors | Existing handler at surface.rs:368; GTK4 4.14.5 has wp_fractional_scale_v1; verification-focused |
| HDPI-02 | Scale factor updates when window moves between monitors with different DPI | `notify::scale-factor` already connected on GLArea; verify sidebar/divider chrome also updates |
| SSH-01 | User can configure a workspace with a remote SSH target | Socket API `workspace.create` with `remote_target` parameter per D-13 |
| SSH-02 | cmuxd-remote Go daemon is deployed to remote host and establishes reverse tunnel | Go 1.24.5 available; `GOOS=linux GOARCH=amd64 go build` for cross-compile; scp deployment |
| SSH-03 | Terminal sessions in SSH workspace run on remote host | cmuxd-remote stdio RPC protocol with proxy.open/write/stream for TCP tunneling |
| SSH-04 | SSH workspace reconnect works after network interruption | Exponential backoff per D-14; connection state enum in workspace model |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Never run tests locally; all tests via GitHub Actions or VM
- Socket command threading policy: parse off-main, schedule UI on main via mpsc channel
- Socket focus policy: non-focus commands must not steal focus
- Test quality policy: tests must verify runtime behavior, not source code text
- Do not add app-level display link or manual `ghostty_surface_draw` loop
- Typing-latency-sensitive paths must not gain new allocations or I/O

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| gtk4 (gtk4-rs) | 0.10.3 | UI framework, GNotification, scale-factor signals | Already in use; provides gio::Notification |
| glib | 0.21.5 | Main loop integration, idle callbacks | Already in use |
| tokio | 1.x | Async I/O for socket server, SSH tunnels | Already in use |
| Go stdlib | 1.24.5 | cmuxd-remote daemon | Already used in daemon/remote/ |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| gio (via gtk4) | 0.10.3 | GNotification, Application send_notification | Desktop notifications (NOTF-03) |
| serde_json | 1.x | JSON serialization for socket commands | Already in use for socket protocol |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| gio::Notification | libnotify/notify-rust | Extra dependency; GNotification is already available via GApplication -- no new deps needed (D-07) |
| Go cmuxd-remote | Rust rewrite | Go version works, cross-compiles trivially; rewrite deferred |

## Architecture Patterns

### Bell Event Flow
```
Ghostty terminal → \a bell → action_cb(RING_BELL, target=surface)
  → SURFACE_REGISTRY lookup → pane_id
  → SplitNode::Leaf.has_attention = true
  → workspace.has_attention = any(pane.has_attention)
  → sidebar row CSS class update ("has-attention")
  → if !window.is_active(): send_notification()
```

### Key Data Flow: action_cb Target Identification
```rust
// action_cb receives ghostty_target_s with tag=SURFACE and target.surface=ptr
// The surface ptr maps to pane_id via SURFACE_REGISTRY (HashMap<usize, u64>)
if action.tag == ffi::ghostty_action_tag_e_GHOSTTY_ACTION_RING_BELL {
    if target.tag == ffi::ghostty_target_tag_e_GHOSTTY_TARGET_SURFACE {
        let surface_ptr = unsafe { target.target.surface } as usize;
        // lookup pane_id from SURFACE_REGISTRY
    }
}
```

### Attention State Architecture
```
SplitNode::Leaf { has_attention: bool, ... }
  └─ set to true on bell
  └─ set to false when workspace is focused (D-05)

Workspace { has_attention: bool, ... }
  └─ derived: any leaf in split tree has attention
  └─ drives sidebar dot visibility

AppState
  └─ clear_workspace_attention(index) on switch_to_index()
  └─ set_pane_attention(pane_id) called from action_cb bell handler
```

### SSH Workspace Architecture
```
socket API: workspace.create { remote_target: "user@host" }
  → spawn tokio task for SSH lifecycle
  → scp cmuxd-remote binary to remote
  → ssh -L localport:localhost:remoteport user@host cmuxd-remote serve --stdio
  → JSON-RPC over stdio pipe to cmuxd-remote
  → proxy.open to connect terminal sessions to remote shell
  → proxy.stream.subscribe for bidirectional data

Connection state: Connected | Disconnected | Reconnecting(attempt, next_retry)
  → shown in sidebar indicator
  → exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s cap
```

### Recommended Changes by File
```
src/
├── ghostty/
│   └── callbacks.rs      # Add RING_BELL handler in action_cb
├── split_engine.rs       # Add has_attention: bool to SplitNode::Leaf
├── workspace.rs          # Add has_attention: bool, connection_state fields
├── app_state.rs          # Add set_pane_attention(), clear_workspace_attention()
├── sidebar.rs            # Add attention dot widget, connection state indicator
├── socket/
│   ├── commands.rs       # Add Notification* variants, workspace.create remote param
│   ├── handlers.rs       # Implement notification.* handlers
│   └── mod.rs            # Add notification.* dispatch in dispatch_line()
├── notification.rs       # NEW: GNotification helper, attention state management
└── ssh/                  # NEW: SSH workspace lifecycle
    ├── mod.rs            # SSH connection manager
    ├── tunnel.rs         # SSH tunnel establishment and monitoring
    └── deploy.rs         # cmuxd-remote binary deployment
daemon/remote/
└── cmd/cmuxd-remote/     # Existing Go daemon (no changes needed)
```

### Anti-Patterns to Avoid
- **Polling for bell events:** Do not poll or check terminal state periodically. Use `action_cb` which Ghostty calls synchronously on the main thread.
- **Allocations in action_cb:** `action_cb` is called frequently (every render frame). The bell handler must be lightweight -- set a bool, schedule sidebar update via idle_add if needed.
- **Synchronous SSH operations on main thread:** All SSH/tunnel operations must run in tokio tasks. Only UI updates (sidebar indicator) go to main thread via mpsc channel.
- **Direct libnotify usage:** Per D-07, use `gio::Notification` through the existing `GApplication`, not libnotify or notify-rust.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Desktop notifications | Custom DBus notification calls | `gio::Application::send_notification()` | Handles notification daemon differences, action callbacks, icon resolution |
| SSH connection | Raw TCP socket management | `tokio::process::Command` with ssh | SSH handles auth, key exchange, multiplexing; reimplementing is a security risk |
| Fractional DPI scaling | Manual pixel ratio calculations | GTK4 native `wp_fractional_scale_v1` + `notify::scale-factor` | GTK4 4.12+ handles this natively; custom logic would break on edge cases |
| SSH binary deployment | Custom file transfer protocol | scp via `tokio::process::Command` | Reliable, handles permissions, works with all SSH auth methods |
| Reconnection backoff | Manual timer management | Simple loop with `tokio::time::sleep` + exponential calculation | Easy to get wrong with jitter, max cap, state management |

## Common Pitfalls

### Pitfall 1: action_cb Runs on Main Thread -- But SURFACE_REGISTRY Might Deadlock
**What goes wrong:** `action_cb` runs during `ghostty_app_tick()` on the GLib main thread. If you try to lock `SURFACE_REGISTRY` while another main-thread closure already holds it, you deadlock.
**Why it happens:** Mutex is not re-entrant. Multiple idle callbacks or signal handlers on the same thread.
**How to avoid:** `SURFACE_REGISTRY` lock must be very short-lived in `action_cb`. Lock, clone the pane_id, drop lock, then do state mutations.
**Warning signs:** App freezes when bell fires.

### Pitfall 2: GNotification Requires GApplication Registration
**What goes wrong:** `send_notification` silently fails if the GApplication is not properly registered or has no app-id.
**Why it happens:** DBus notification protocol requires an application identity.
**How to avoid:** The app already uses `APP_ID = "io.cmux.App"` and `Application::builder().application_id(APP_ID)`. This should work. Verify notification appears on both GNOME and KDE.
**Warning signs:** No notification appears, no error in logs.

### Pitfall 3: Scale Factor Signals Only Fire on GLArea, Not on Sidebar Widgets
**What goes wrong:** Terminal surfaces scale correctly when moving monitors, but sidebar text/dot stays at wrong scale.
**Why it happens:** `notify::scale-factor` is connected on GLArea specifically. Sidebar is a separate widget tree.
**How to avoid:** GTK4 handles DPI for standard widgets (Label, ListBox) automatically. The sidebar should scale without custom code. Only custom-drawn elements need attention.
**Warning signs:** Sidebar text appears too small or too large after monitor change.

### Pitfall 4: SSH Process Cleanup on Workspace Close
**What goes wrong:** SSH subprocess and cmuxd-remote on remote host are left running after workspace is closed.
**Why it happens:** Dropping a tokio task doesn't automatically kill child processes.
**How to avoid:** Store `tokio::process::Child` handle. On workspace close, send SIGTERM to the SSH process, which will clean up cmuxd-remote on the remote side.
**Warning signs:** Zombie ssh processes, remote cmuxd-remote still listening.

### Pitfall 5: Bell Storm Flooding Notifications
**What goes wrong:** A terminal program generating many bells (e.g., `yes $'\a'`) floods desktop with hundreds of notifications.
**Why it happens:** Each bell fires `action_cb` independently.
**How to avoid:** Debounce/coalesce: once a pane sets `has_attention`, ignore subsequent bells until attention is cleared. For desktop notifications, rate-limit to max 1 per workspace per N seconds (e.g., 5 seconds).
**Warning signs:** Notification daemon crashes or desktop becomes unresponsive.

### Pitfall 6: SSH Tunnel Port Conflicts
**What goes wrong:** Two SSH workspaces targeting different hosts try to use the same local port for their tunnels.
**Why it happens:** Hardcoded or predictable port selection.
**How to avoid:** Bind to port 0 and let the OS assign an ephemeral port, or use SSH stdio mode (`-W`) which avoids TCP tunnels entirely.
**Warning signs:** "Address already in use" errors on second SSH workspace.

## Code Examples

### Bell Handler in action_cb
```rust
// In callbacks.rs action_cb, after the RENDER handler:
if action.tag == ffi::ghostty_action_tag_e_GHOSTTY_ACTION_RING_BELL {
    // Identify which surface rang the bell
    if target.tag == ffi::ghostty_target_tag_e_GHOSTTY_TARGET_SURFACE {
        let surface_ptr = unsafe { target.target.surface } as usize;
        if let Ok(reg) = SURFACE_REGISTRY.lock() {
            if let Some(&pane_id) = reg.get(&surface_ptr) {
                // Schedule attention state update on main thread
                // (we're already on main thread in action_cb, so direct call is safe)
                // The AppState update is done via a function pointer or channel
                // depending on architecture choice
            }
        }
    }
    return true; // handled
}
```

### Desktop Notification via gio
```rust
// In notification.rs or similar:
use gtk4::gio;

pub fn send_bell_notification(app: &gtk4::Application, workspace_name: &str) {
    let notification = gio::Notification::new("Terminal Bell");
    notification.set_body(Some(&format!("{} - Terminal bell", workspace_name)));
    // Priority: NORMAL is appropriate for bells
    notification.set_priority(gio::NotificationPriority::Normal);
    // Use workspace-specific ID so we can withdraw/replace
    app.send_notification(Some("terminal-bell"), &notification);
}
```

### Attention Dot in Sidebar
```rust
// In sidebar.rs, when building/updating a workspace row:
// Use a GtkBox to hold label + dot indicator
let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
let label = gtk4::Label::new(Some(&workspace.name));
label.set_halign(gtk4::Align::Start);
label.set_hexpand(true);
hbox.append(&label);

// Attention dot: small colored circle via CSS
let dot = gtk4::Label::new(None);
dot.add_css_class("attention-dot");
dot.set_visible(false); // shown when has_attention
hbox.append(&dot);
row.set_child(Some(&hbox));
```

### CSS for Attention Dot
```css
.attention-dot {
    background-color: #e8a444;
    border-radius: 50%;
    min-width: 8px;
    min-height: 8px;
    max-width: 8px;
    max-height: 8px;
    margin: 0 4px;
}
```

### SSH Workspace Lifecycle (tokio task)
```rust
// Pseudocode for SSH workspace connection:
async fn ssh_connect(target: &str, cmuxd_path: &Path) -> Result<SshConnection> {
    // 1. Deploy cmuxd-remote if not present
    let deploy_cmd = tokio::process::Command::new("scp")
        .args([cmuxd_path.to_str().unwrap(), &format!("{}:~/.local/bin/cmuxd-remote", target)])
        .status().await?;

    // 2. Start SSH with cmuxd-remote on remote
    let child = tokio::process::Command::new("ssh")
        .args([target, ".local/bin/cmuxd-remote", "serve", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // 3. Send hello handshake
    // 4. Return connection handle with stdin/stdout for RPC
    Ok(SshConnection { child, ... })
}
```

### Window Focus Check for Notification Gating
```rust
// Check if the GTK window is focused before sending notification
// GTK4: use is_active() on the ApplicationWindow
fn window_is_focused(app: &gtk4::Application) -> bool {
    app.active_window()
        .map(|w| w.is_active())
        .unwrap_or(false)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| libnotify C library | gio::Notification (GLib 2.40+) | GLib 2.40 (2014) | No extra dependency needed; works through GApplication |
| glib::MainContext::channel | tokio mpsc + spawn_local | glib 0.18 (2023) | Already adapted in Phase 3; same pattern for Phase 4 |
| Integer scale factors | wp_fractional_scale_v1 | GTK4 4.12 (2023) | GTK4 4.14.5 has full support; no custom work needed |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| GTK4 | Notifications, UI | Yes | 4.14.5 | -- |
| Go compiler | cmuxd-remote cross-compile | Yes | 1.24.5 | -- |
| OpenSSH client | SSH workspace tunnels | Yes | 9.6p1 | -- |
| scp | Binary deployment | Yes | (via OpenSSH) | rsync or sftp |
| Notification daemon | Desktop notifications | Assumed | -- | Notifications silently skipped |

**Missing dependencies with no fallback:** None

**Missing dependencies with fallback:** None

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust) + Go test |
| Config file | Cargo.toml (Rust), go.mod (Go) |
| Quick run command | `cargo test --bin cmux-linux` |
| Full suite command | `cargo test --bin cmux-linux && cd daemon/remote && go test ./...` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| NOTF-01 | Bell sets pane attention flag | unit | `cargo test --bin cmux-linux -- test_bell_sets_attention -x` | Wave 0 |
| NOTF-02 | Workspace has_attention derived from panes | unit | `cargo test --bin cmux-linux -- test_workspace_attention -x` | Wave 0 |
| NOTF-03 | Notification sent when unfocused | manual-only | Requires running GTK app + notification daemon | N/A |
| HDPI-01 | Correct rendering at multiple scales | manual-only | Requires multi-DPI display environment | N/A |
| HDPI-02 | Scale factor updates on monitor change | manual-only | Requires multi-monitor setup | N/A |
| SSH-01 | Remote workspace creation via socket | unit | `cargo test --bin cmux-linux -- test_ssh_workspace_create -x` | Wave 0 |
| SSH-02 | cmuxd-remote builds for linux | unit | `cd daemon/remote && go test ./...` | Yes |
| SSH-03 | Remote terminal sessions via proxy | integration | Requires SSH target + running app | N/A |
| SSH-04 | Reconnect after interruption | integration | Requires network simulation | N/A |

### Sampling Rate
- **Per task commit:** `cargo test --bin cmux-linux`
- **Per wave merge:** `cargo test --bin cmux-linux && cd daemon/remote && go test ./...`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Attention state unit tests (NOTF-01, NOTF-02) -- test has_attention propagation through split tree
- [ ] SSH workspace model tests (SSH-01) -- test workspace creation with remote_target parameter
- [ ] Note: NOTF-03, HDPI-01/02, SSH-03/04 are manual-only or integration tests requiring a running app environment

## Discretion Recommendations

### Bell Action Tag
**Recommendation:** Use `GHOSTTY_ACTION_RING_BELL` (tag 49). Confirmed in FFI bindings at `/home/twilson/code/cmux-linux/target/debug/build/cmux-linux-1ad153c21a790008/out/ghostty_sys.rs`. The `GHOSTTY_ACTION_DESKTOP_NOTIFICATION` (tag 31) is for OSC sequences requesting desktop notifications directly -- handle that too for completeness but the primary bell attention trigger is RING_BELL.

### Sidebar Dot Implementation
**Recommendation:** Use a GtkLabel with CSS class `attention-dot` (8px orange circle). Simpler than custom drawing, consistent with GTK4 theming. Color: `#e8a444` (warm amber, distinct from the active-workspace blue `#5b8dd9`). Place inside a GtkBox alongside the workspace name label.

### GNotification Settings
**Recommendation:** `gio::NotificationPriority::Normal` priority. Use a per-workspace notification ID (e.g., `"bell-{workspace_uuid}"`) so repeated bells replace rather than stack notifications. No icon needed beyond the default app icon.

### cmuxd-remote Deployment
**Recommendation:** scp the pre-compiled binary. Cross-compile with `GOOS=linux GOARCH=amd64 go build -o cmuxd-remote-linux-amd64 ./cmd/cmuxd-remote/` at build time. Store alongside the cmux binary or in `~/.local/share/cmux/bin/`. On `workspace.create` with remote target, scp to `~/.local/bin/cmuxd-remote` on the remote host.

### SSH Tunnel Approach
**Recommendation:** Use SSH stdio mode (`ssh user@host cmuxd-remote serve --stdio`) rather than TCP port tunneling. This avoids port conflicts entirely -- the cmuxd-remote daemon communicates over the SSH stdin/stdout pipe. The existing Go daemon already supports `serve --stdio` mode. No need for `-L` port forwarding.

### surface.health and Attention
**Recommendation:** Yes, include `has_attention: bool` in the `surface.health` response. This is free information and useful for external tooling.

## Open Questions

1. **cmuxd-remote binary architecture detection**
   - What we know: We can cross-compile for linux/amd64 and linux/arm64
   - What's unclear: How to detect remote host architecture before deploying
   - Recommendation: Default to linux/amd64; add `--arch` parameter to socket command; cross-compile both variants at build time

2. **SSH key/auth handling**
   - What we know: `ssh` command handles auth natively (agent, keys, password)
   - What's unclear: Whether to support password prompts or require key-based auth
   - Recommendation: Require key-based auth (no interactive prompts in Phase 4); document requirement

3. **Bell notification rate limiting threshold**
   - What we know: Need debouncing to prevent notification flood
   - What's unclear: Optimal threshold
   - Recommendation: 5-second cooldown per workspace; configurable in Phase 5

## Sources

### Primary (HIGH confidence)
- FFI bindings: `target/debug/build/cmux-linux-*/out/ghostty_sys.rs` -- confirmed GHOSTTY_ACTION_RING_BELL=49, GHOSTTY_ACTION_DESKTOP_NOTIFICATION=31, ghostty_target_s with SURFACE tag
- Source code: `src/ghostty/callbacks.rs` -- current action_cb implementation, SURFACE_REGISTRY
- Source code: `src/ghostty/surface.rs:364-378` -- existing notify::scale-factor handler
- Source code: `daemon/remote/cmd/cmuxd-remote/main.go` -- Go daemon RPC protocol with proxy/session support
- Source code: `src/socket/` -- current socket command dispatch pattern
- System: GTK4 4.14.5 installed (pkg-config), Go 1.24.5, OpenSSH 9.6p1

### Secondary (MEDIUM confidence)
- GTK4 gio::Notification API -- based on gtk4-rs 0.10 crate documentation and GLib upstream docs

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in use, no new dependencies
- Architecture: HIGH -- action_cb pattern, SURFACE_REGISTRY, and socket dispatch well understood from source code inspection
- Pitfalls: HIGH -- identified from direct code analysis (deadlock risk, notification gating, process cleanup)
- SSH integration: MEDIUM -- Go daemon inspected and understood, but SSH lifecycle management is new code with network edge cases

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (stable -- no fast-moving dependencies)
