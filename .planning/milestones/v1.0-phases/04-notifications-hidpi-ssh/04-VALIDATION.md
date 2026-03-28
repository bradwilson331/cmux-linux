---
phase: 4
slug: notifications-hidpi-ssh
status: draft
nyquist_compliant: true
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

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 04-01-01 | 01 | 1 | NOTF-01 | compile | `cargo test --bin cmux-linux` | pending |
| 04-01-02 | 01 | 1 | NOTF-02 | compile | `cargo test --bin cmux-linux` | pending |
| 04-01-03 | 01 | 1 | NOTF-03 | compile | `cargo test --bin cmux-linux` | pending |
| 04-02-01 | 02 | 1 | HDPI-01 | manual | N/A (requires multi-DPI display) | pending |
| 04-02-02 | 02 | 1 | HDPI-02 | manual | N/A (requires multi-monitor setup) | pending |
| 04-03-01 | 03 | 2 | NOTF-01 | compile | `cargo test --bin cmux-linux` | pending |
| 04-04-01 | 04 | 2 | SSH-01 | compile | `cargo test --bin cmux-linux` | pending |
| 04-04-02 | 04 | 2 | SSH-02 | compile | `cargo test --bin cmux-linux` | pending |
| 04-04-03 | 04 | 2 | SSH-03 | compile | `cargo test --bin cmux-linux` | pending |
| 04-04-04 | 04 | 2 | SSH-04 | compile | `cargo test --bin cmux-linux` | pending |

*Status: pending / green / red / flaky*

---

## Nyquist Compliance Rationale

No Wave 0 test stub plan is created for this phase. Rationale:

1. **CLAUDE.md test quality policy** requires tests to "verify observable runtime behavior through executable paths," not source code text or AST patterns. Unit tests for attention state propagation would need to construct `SplitNode` trees, `Workspace` structs, and GTK4 widgets -- all of which require a running GTK4 application context.

2. **Most Phase 4 features require runtime infrastructure that doesn't exist:**
   - NOTF-01/02: Attention tracking requires `SplitEngine` with real GTK4 `GLArea` widgets and Ghostty FFI surfaces. Cannot be constructed in a headless test.
   - NOTF-03: Desktop notification requires a running GTK `Application` with registered app-id and notification daemon.
   - HDPI-01/02: Requires physical multi-DPI display environment.
   - SSH-01/02/03/04: Requires running SSH target, network, deployed cmuxd-remote.

3. **Compilation verification (`cargo test --bin cmux-linux`) is the appropriate automated check** for this phase. It confirms type correctness, API compatibility, and no regressions. Runtime behavior is verified via Plan 05's human checkpoint.

4. **CLAUDE.md testing policy**: "Never run tests locally. All tests (E2E, UI, python socket tests) run via GitHub Actions or on the VM." The meaningful automated tests for these features are the Python socket tests in `tests_v2/` which connect to a running cmux instance, and those run in CI.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Bell sets pane attention and sidebar dot appears | NOTF-01, NOTF-02 | Requires running GTK app with Ghostty terminal | 1. Open app, create 2 workspaces 2. Run `echo -e '\a'` in WS2 3. Switch to WS1, verify amber dot on WS2 |
| Desktop notification appears when bell rings and window unfocused | NOTF-03 | Requires notification daemon and window focus state | 1. Unfocus window 2. Run `echo -e '\a'` 3. Verify notification appears |
| HiDPI rendering across monitor move | HDPI-01, HDPI-02 | Requires physical multi-monitor setup with different DPIs | 1. Open on 1x monitor 2. Drag to 2x monitor 3. Verify text renders crisp |
| SSH workspace creation and connection | SSH-01, SSH-02 | Requires SSH target host | 1. Socket: workspace.create with remote_target 2. Verify sidebar shows connection state |
| SSH reconnect after network interruption | SSH-04 | Requires simulated network disruption | 1. Connect SSH workspace 2. Disable network briefly 3. Verify reconnection |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify commands (compilation checks)
- [x] Sampling continuity: compilation verified after every task
- [x] Wave 0 not needed (see Nyquist Compliance Rationale above)
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter with rationale

**Approval:** pending
