---
status: partial
phase: 04-notifications-hidpi-ssh
source: [04-VERIFICATION.md]
started: 2026-03-26T00:00:00Z
updated: 2026-03-26T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Bell attention dot appears in sidebar
expected: Triggering terminal bell in workspace 2 shows amber dot next to "Workspace 2" in sidebar; dot clears when workspace is focused
result: [pending]

### 2. Desktop notification fires when window unfocused
expected: Terminal bell while app window is unfocused triggers desktop notification with title "Terminal Bell" and body "{workspace} - Terminal bell"; rate limited to 1 per 5 seconds
result: [pending]

### 3. HiDPI rendering at multiple scale factors
expected: App renders correctly at 1x, 1.5x, and 2x; dragging between monitors with different DPI updates rendering without restart
result: [pending]

### 4. SSH workspace creation via socket API
expected: workspace.create with remote_target parameter creates SSH workspace with connection state subtitle in sidebar
result: [pending]

### 5. SSH terminal I/O (proxy.stream routing)
expected: Terminal sessions in SSH workspace execute on remote host — NOTE: proxy.stream routing is currently a TODO in tunnel.rs, so this is expected to be incomplete
result: [pending]

## Summary

total: 5
passed: 0
issues: 0
pending: 5
skipped: 0
blocked: 0

## Gaps
