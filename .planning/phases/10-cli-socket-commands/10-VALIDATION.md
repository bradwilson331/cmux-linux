---
phase: 10
slug: cli-socket-commands
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-27
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml `[[test]]` sections |
| **Quick run command** | `cargo test --bin cmux --lib -- cli` |
| **Full suite command** | `cargo test --bin cmux` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --bin cmux --lib -- cli`
- **After every plan wave:** Run `cargo test --bin cmux`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | TBD | unit | `cargo test --bin cmux` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/cli/mod.rs` — CLI module with socket client and subcommand routing
- [ ] `src/bin/cmux.rs` — CLI binary entry point
- [ ] Unit test stubs for socket discovery and command formatting

*Existing infrastructure covers socket protocol (serde_json already in Cargo.toml).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CLI connects to running app | D-11 | Requires running cmux-app | Start app, run `cmux list-workspaces` |
| Human-readable output formatting | D-08 | Visual quality check | Run list commands, verify aligned columns |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
