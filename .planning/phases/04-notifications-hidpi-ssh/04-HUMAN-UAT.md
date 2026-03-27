---
status: complete
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
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""

- truth: "workspace.create with remote_target creates SSH workspace successfully"
  status: failed
  reason: "User reported: SSH deploy failed: cmuxd-remote binary not found at ~/.local/share/cmux/bin/cmuxd-remote-linux-amd64. Enters infinite reconnect loop with no backoff cap."
  severity: blocker
  test: 4
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
