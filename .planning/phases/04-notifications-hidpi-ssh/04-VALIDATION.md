---
phase: 4
slug: notifications-hidpi-ssh
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust unit/integration) + pytest (Python socket tests) |
| **Config file** | Cargo.toml (Rust) / tests_v2/ (Python) |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | NOTF-01 | unit | `cargo test bell` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | NOTF-02 | unit | `cargo test notification` | ❌ W0 | ⬜ pending |
| 04-01-03 | 01 | 1 | NOTF-03 | unit | `cargo test bell_debounce` | ❌ W0 | ⬜ pending |
| 04-02-01 | 02 | 1 | HDPI-01 | integration | `cargo test hidpi` | ❌ W0 | ⬜ pending |
| 04-02-02 | 02 | 1 | HDPI-02 | integration | `cargo test scale_change` | ❌ W0 | ⬜ pending |
| 04-03-01 | 03 | 2 | SSH-01 | unit | `cargo test ssh_config` | ❌ W0 | ⬜ pending |
| 04-03-02 | 03 | 2 | SSH-02 | integration | `cargo test ssh_connect` | ❌ W0 | ⬜ pending |
| 04-03-03 | 03 | 2 | SSH-03 | unit | `cargo test ssh_reconnect` | ❌ W0 | ⬜ pending |
| 04-03-04 | 03 | 2 | SSH-04 | integration | `cargo test ssh_workspace` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Test stubs for bell routing (NOTF-01, NOTF-02, NOTF-03)
- [ ] Test stubs for HiDPI verification (HDPI-01, HDPI-02)
- [ ] Test stubs for SSH workspace (SSH-01 through SSH-04)

*Existing cargo test infrastructure covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Desktop notification appears when bell rings and window unfocused | NOTF-02 | Requires desktop notification daemon and window focus state | 1. Open terminal, run `echo -e '\a'` 2. Switch to another window 3. Verify notification appears |
| HiDPI rendering across monitor move | HDPI-02 | Requires physical multi-monitor setup with different DPIs | 1. Open app on 1x monitor 2. Drag to 2x monitor 3. Verify text renders crispy without restart |
| SSH reconnect after network interruption | SSH-03 | Requires simulated network disruption | 1. Connect SSH workspace 2. Disable network briefly 3. Re-enable, verify session resumes |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
