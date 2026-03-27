---
status: partial
phase: 04-notifications-hidpi-ssh
source: [04-VERIFICATION.md]
started: 2026-03-26T00:00:00Z
updated: 2026-03-27T20:15:00Z
---

## Current Test

[awaiting re-test after gap closure]

## Tests

### 1. Bell attention dot appears in sidebar
expected: Triggering terminal bell in workspace 2 shows amber dot next to "Workspace 2" in sidebar; dot clears when workspace is focused
result: pass

### 2. Desktop notification fires when window unfocused
expected: Terminal bell while app window is unfocused triggers desktop notification with title "Terminal Bell" and body "{workspace} - Terminal bell"; rate limited to 1 per 5 seconds
result: issue
reported: "I heard a bell but no desktop notification was seen"
severity: major
gap_closure: 04-06 (replaced gio::Notification with notify-rust)
re_test: pending

### 3. HiDPI rendering at multiple scale factors
expected: App renders correctly at 1x, 1.5x, and 2x; dragging between monitors with different DPI updates rendering without restart
result: pass

### 4. SSH workspace creation via socket API
expected: workspace.create with remote_target parameter creates SSH workspace with connection state subtitle in sidebar
result: issue
reported: "SSH deploy failed: cmuxd-remote binary not found at ~/.local/share/cmux/bin/cmuxd-remote-linux-amd64. After failure, enters infinite reconnect loop with no backoff cap."
severity: blocker
gap_closure: 04-07 (install script, MAX_RETRIES, permanent failure detection)
re_test: pending

### 5. SSH terminal I/O (proxy.stream routing)
expected: Terminal sessions in SSH workspace execute on remote host
result: blocked
blocked_by: test 4 re-test
reason: "Depends on SSH workspace creation working"

## Summary

total: 5
passed: 2
issues: 2
pending: 0
skipped: 0
blocked: 1
re_test_pending: 2

## Gaps

- truth: "Terminal bell while app window is unfocused triggers desktop notification"
  status: resolved
  reason: "Replaced gio::Notification with notify-rust (plan 04-06)"
  severity: major
  test: 2
  debug_session: ".planning/debug/bell-notification-missing.md"

- truth: "workspace.create with remote_target creates SSH workspace successfully"
  status: resolved
  reason: "Added install script, MAX_RETRIES=10, FailureKind permanent/transient (plan 04-07)"
  severity: blocker
  test: 4
  debug_session: ".planning/debug/ssh-remote-binary-and-reconnect.md"
