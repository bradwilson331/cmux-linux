---
phase: 5
slug: config-distribution
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | `Cargo.toml` `[dev-dependencies]` section |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test && cargo clippy -- -D warnings` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test && cargo clippy -- -D warnings`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | CFG-01 | unit | `cargo test config::` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | CFG-02 | unit | `cargo test config::` | ❌ W0 | ⬜ pending |
| 05-01-03 | 01 | 1 | CFG-04 | unit | `cargo test config::` | ❌ W0 | ⬜ pending |
| 05-02-01 | 02 | 1 | CFG-02 | unit | `cargo test shortcuts::` | ❌ W0 | ⬜ pending |
| 05-03-01 | 03 | 2 | DIST-01 | ci | `gh workflow run ci.yml` | ❌ W0 | ⬜ pending |
| 05-03-02 | 03 | 2 | DIST-02 | ci | `gh workflow run ci.yml` | ❌ W0 | ⬜ pending |
| 05-04-01 | 04 | 2 | DIST-03 | ci | `gh workflow run release.yml` | ❌ W0 | ⬜ pending |
| 05-04-02 | 04 | 2 | DIST-04 | manual | N/A | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/config.rs` — test module with stubs for TOML parsing, default shortcuts, error handling
- [ ] `tests/config_integration.rs` — integration tests for config file loading from XDG paths
- [ ] `tempfile` dev-dependency — for config file integration tests

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| App launches on Wayland session | DIST-04 | Requires Wayland compositor | Launch in Wayland session, verify rendering and input |
| App launches on X11/XWayland session | DIST-04 | Requires X11 display server | Launch in X11 session, verify rendering and input |
| AppImage runs on clean system | DIST-03 | Requires clean Linux install | Download AppImage, chmod +x, run on fresh Ubuntu |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
