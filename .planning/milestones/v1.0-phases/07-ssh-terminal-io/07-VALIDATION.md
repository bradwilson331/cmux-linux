---
phase: 07
slug: ssh-terminal-io
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (unit) + Python socket tests (integration) |
| **Config file** | Cargo.toml (bin tests) |
| **Quick run command** | `cargo test --bin cmux-linux -- ssh` |
| **Full suite command** | GitHub Actions CI |
| **Estimated runtime** | ~15 seconds (unit tests only) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --bin cmux-linux -- ssh`
- **After every plan wave:** Run CI build + clippy
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | SSH-03 | unit | `cargo test --bin cmux-linux -- ssh::proxy` | ❌ W0 | ⬜ pending |
| 07-01-02 | 01 | 1 | SSH-03 | unit | `cargo test --bin cmux-linux -- ssh::tunnel` | ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 1 | SSH-03 | unit | `cargo test --bin cmux-linux -- ssh::stream` | ❌ W0 | ⬜ pending |
| 07-03-01 | 03 | 2 | SSH-03 | manual | Manual: SSH workspace type + see output | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Unit tests for base64 encode/decode round-trip with proxy protocol framing
- [ ] Unit tests for JSON-RPC message construction (proxy.open, proxy.write, etc.)
- [ ] Unit tests for stream state management (pane_id <-> stream_id mapping)

*Existing cargo test infrastructure covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| End-to-end SSH terminal I/O | SSH-03 | Requires live SSH connection to remote host with cmuxd-remote | 1. Create SSH workspace 2. Type in pane 3. Verify keystrokes appear in remote shell 4. Verify remote output renders locally |
| Disconnect/reconnect UX | SSH-03 | Requires network interruption simulation | 1. Connect SSH workspace 2. Kill SSH tunnel 3. Verify freeze message 4. Verify reconnect message |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
