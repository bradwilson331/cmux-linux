---
status: diagnosed
phase: 04-notifications-hidpi-ssh
source: [04-VERIFICATION.md]
started: 2026-03-26T00:00:00Z
updated: 2026-03-27T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Bell attention dot appears in sidebar
expected: Triggering terminal bell in workspace 2 shows amber dot next to "Workspace 2" in sidebar; dot clears when workspace is focused
result: pass

### 2. Desktop notification fires when window unfocused
expected: Terminal bell while app window is unfocused triggers desktop notification with title "Terminal Bell" and body "{workspace} - Terminal bell"; rate limited to 1 per 5 seconds
result: issue
reported: "I heard a bell but no desktop notification was seen"
severity: major

### 3. HiDPI rendering at multiple scale factors
expected: App renders correctly at 1x, 1.5x, and 2x; dragging between monitors with different DPI updates rendering without restart
result: pass

### 4. SSH workspace creation via socket API
expected: workspace.create with remote_target parameter creates SSH workspace with connection state subtitle in sidebar
result: issue
reported: "SSH deploy failed: cmuxd-remote binary not found at ~/.local/share/cmux/bin/cmuxd-remote-linux-amd64. After failure, enters infinite reconnect loop with no backoff cap."
severity: blocker

### 5. SSH terminal I/O (proxy.stream routing)
expected: Terminal sessions in SSH workspace execute on remote host — NOTE: proxy.stream routing is currently a TODO in tunnel.rs, so this is expected to be incomplete
result: blocked
blocked_by: prior-phase
reason: "Depends on test 4 (SSH workspace creation); proxy.stream routing is an acknowledged TODO in tunnel.rs"

## Summary

total: 5
passed: 2
issues: 2
pending: 0
skipped: 0
blocked: 1

## Gaps

- truth: "Terminal bell while app window is unfocused triggers desktop notification"
  status: failed
  reason: "User reported: I heard a bell but no desktop notification was seen"
  severity: major
  test: 2
  root_cause: "gio::Notification silently fails on Linux — requires matching .desktop file installed to XDG applications dir, D-Bus service file, and DBusActivatable=true. App ID is io.cmux.App but desktop file is resources/cmux.desktop (wrong name), not installed, no D-Bus service file exists."
  artifacts:
    - path: "src/app_state.rs:565-572"
      issue: "send_bell_notification uses gio::Notification which silently drops on Linux without proper D-Bus setup"
    - path: "src/main.rs:20,94-97"
      issue: "APP_ID mismatch with desktop file; NON_UNIQUE flag may skip D-Bus registration"
    - path: "resources/cmux.desktop"
      issue: "Filename doesn't match app ID; missing DBusActivatable=true"
  missing:
    - "Replace gio::Notification with notify-rust crate (uses org.freedesktop.Notifications D-Bus directly, no .desktop file needed)"
  debug_session: ".planning/debug/bell-notification-missing.md"

- truth: "workspace.create with remote_target creates SSH workspace successfully"
  status: failed
  reason: "User reported: SSH deploy failed: cmuxd-remote binary not found at ~/.local/share/cmux/bin/cmuxd-remote-linux-amd64. Enters infinite reconnect loop with no backoff cap."
  severity: blocker
  test: 4
  root_cause: "Two issues: (1) No build/install step copies cmuxd-remote to ~/.local/share/cmux/bin/. Pre-built binary sits in daemon/remote/ source tree only. (2) run_ssh_lifecycle() in tunnel.rs has unconditional loop{} with no max retry and no permanent-failure detection — missing binary triggers infinite reconnect at 30s cap."
  artifacts:
    - path: "src/ssh/deploy.rs:6-12"
      issue: "local_daemon_path() expects binary at XDG data path but nothing installs it there"
    - path: "src/ssh/tunnel.rs:28-136"
      issue: "run_ssh_lifecycle() has no max retry count, no permanent vs transient failure distinction"
    - path: "scripts/build_remote_daemon_release_assets.sh"
      issue: "Only builds for release packaging, no dev install target"
  missing:
    - "Add build/install step that copies cmuxd-remote to ~/.local/share/cmux/bin/"
    - "Add max retry count (e.g., 10) to run_ssh_lifecycle()"
    - "Distinguish permanent failures (binary not found) from transient (network blip) — exit immediately on permanent"
  debug_session: ".planning/debug/ssh-remote-binary-and-reconnect.md"
