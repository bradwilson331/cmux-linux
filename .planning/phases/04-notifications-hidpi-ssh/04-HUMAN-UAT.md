---
status: complete
phase: 04-notifications-hidpi-ssh
source: [04-VERIFICATION.md]
started: 2026-03-26T00:00:00Z
updated: 2026-03-27T21:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Bell attention dot appears in sidebar
expected: Triggering terminal bell in workspace 2 shows amber dot next to "Workspace 2" in sidebar; dot clears when workspace is focused
result: pass

### 2. Desktop notification fires when window unfocused
expected: Terminal bell while app window is unfocused triggers desktop notification with title "Terminal Bell" and body "{workspace} - Terminal bell"
result: failed
severity: major
attempts:
  - "gio::Notification: silently fails without .desktop file registration and DBusActivatable=true"
  - "notify-rust: GNOME Shell destroys notification when D-Bus sender disconnects (PID-based app matching causes _onNameVanished to fire within 5-8ms)"
  - "notify-send subprocess: notification sent but appeared in another terminal, not as GNOME desktop notification"
status: deferred — bell attention dot (in-app indicator) works; desktop notification is a GNOME integration issue requiring further investigation outside Phase 4 scope

### 3. HiDPI rendering at multiple scale factors
expected: App renders correctly at 1x, 1.5x, and 2x; dragging between monitors with different DPI updates rendering without restart
result: pass

### 4. SSH workspace creation via socket API
expected: workspace.create with remote_target parameter creates SSH workspace with connection state subtitle in sidebar
result: pass
gap_closure: 04-07 (install script, MAX_RETRIES, permanent failure detection)
re_test: pass — binary deployed via SCP successfully, surface created (pane 6000), no infinite reconnect. Terminal I/O routing is Phase 7 scope.

### 5. SSH terminal I/O (proxy.stream routing)
expected: Terminal sessions in SSH workspace execute on remote host
result: deferred
deferred_to: phase-07-ssh-terminal-io
reason: "SSH workspace creation works (Phase 4 scope). Terminal I/O routing via proxy.stream is Phase 7 scope."

## Summary

total: 5
passed: 3
failed: 1
deferred: 1

## Gaps

- truth: "Desktop notification via GNOME notification daemon"
  status: deferred
  reason: "Three approaches tried (gio::Notification, notify-rust, notify-send subprocess) — all fail due to GNOME Shell's interaction with non-standard app registration. In-app bell attention dot works. Desktop notification deferred to future work."
  severity: major
  test: 2
  debug_sessions:
    - ".planning/debug/resolved/bell-notification-missing.md"
    - ".planning/debug/notify-rust-silent-fail.md"
