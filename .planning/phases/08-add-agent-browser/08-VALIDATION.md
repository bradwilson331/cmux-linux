---
phase: 8
slug: add-agent-browser
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust unit/integration tests) |
| **Config file** | `Cargo.toml` — workspace test config |
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
| 8-01-01 | 01 | 1 | BROW-01 | unit | `cargo test browser_daemon` | ❌ W0 | ⬜ pending |
| 8-01-02 | 01 | 1 | BROW-01 | unit | `cargo test browser_socket` | ❌ W0 | ⬜ pending |
| 8-02-01 | 02 | 1 | BROW-02 | unit | `cargo test browser_stream` | ❌ W0 | ⬜ pending |
| 8-02-02 | 02 | 2 | BROW-03 | integration | `cargo test preview_pane` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/socket/browser_tests.rs` — stubs for BROW-01 socket commands
- [ ] `src/browser/mod_tests.rs` — stubs for daemon lifecycle
- [ ] Test fixtures for mock CDP WebSocket frames

*Existing cargo test infrastructure covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Preview pane renders CDP screenshots | BROW-02 | Requires GTK display context | Launch cmux, run `browser.open`, verify image appears in pane |
| Stream updates refresh preview in real time | BROW-03 | Requires live Chrome + CDP | Navigate in agent-browser, verify preview updates within 1s |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
