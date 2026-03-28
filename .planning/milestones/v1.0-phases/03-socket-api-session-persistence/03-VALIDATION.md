---
phase: 3
slug: socket-api-session-persistence
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-25
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[cfg(test)]` unit tests (`cargo test`) + Python integration tests (`tests_v2/`) |
| **Config file** | `Cargo.toml` (workspace root — no separate test config) |
| **Quick run command** | `cargo test --lib 2>&1 \| tail -20` |
| **Full suite command** | `cargo test 2>&1` |
| **Estimated runtime** | ~20 seconds (unit tests) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test 2>&1`
- **Before `/gsd:verify-work`:** Full suite green + Python integration subset passes
- **Max feedback latency:** 20 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 3-SOCK-01 | 03-00 | 0 | SOCK-01 | unit | `cargo test socket::tests::test_socket_path_creation` | ❌ W0 | ⬜ pending |
| 3-SOCK-02 | 03-02 | 1 | SOCK-02 | integration | `python3 tests_v2/test_ctrl_socket.py` | ✅ | ⬜ pending |
| 3-SOCK-03 | 03-00 | 0 | SOCK-03 | unit | `cargo test socket::tests::test_socket_path_creation` (XDG_RUNTIME_DIR env override verifies fallback path; human checkpoint 03-06 validates live socket path) | ❌ W0 | ⬜ pending |
| 3-SOCK-04 | 03-04 | 2 | SOCK-04 | integration | `python3 tests_v2/test_ctrl_socket.py && python3 tests_v2/test_close_workspace_selection.py` | ✅ | ⬜ pending |
| 3-SOCK-05 | 03-00 | 0 | SOCK-05 | unit (design stub) | `cargo test socket::tests::test_focus_policy` — documents the focus-intent whitelist; behavioral enforcement verified in Plan 04 integration test | ❌ W0 | ⬜ pending |
| 3-SOCK-06 | 03-01 | 1 | SOCK-06 | unit | `cargo test socket::auth::tests::test_peercred_rejection` | ❌ W0 | ⬜ pending |
| 3-SESS-01 | 03-05 | 2 | SESS-01 | unit | `cargo test session::tests::test_save_triggered` (asserts session.json exists on disk) | ❌ W0 | ⬜ pending |
| 3-SESS-02 | 03-05 | 2 | SESS-02 | unit | `cargo test session::tests::test_restore_roundtrip` | ❌ W0 | ⬜ pending |
| 3-SESS-03 | 03-05 | 2 | SESS-03 | unit | `cargo test session::tests::test_atomic_write` | ❌ W0 | ⬜ pending |
| 3-SESS-04 | 03-05 | 2 | SESS-04 | unit | `cargo test session::tests::test_graceful_fallback` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

### Notes on specific rows

**3-SOCK-03 (XDG socket path fallback):** The previous stub referenced `test_xdg_path_fallback`
which does not exist in any plan. The real coverage is `test_socket_path_creation` in 03-00,
which sets `XDG_RUNTIME_DIR` to a temp path and asserts the full socket path is deterministic.
The `/run/user/{uid}` fallback (when `XDG_RUNTIME_DIR` is unset) is verified by the 03-06
human checkpoint during live app testing. No additional unit test is needed.

**3-SOCK-05 (focus policy — test_focus_policy):** This test is a design stub in 03-00.
It documents the focus-intent method whitelist and always passes (it does not exercise
`handlers.rs` enforcement). Behavioral enforcement — that non-focus commands do not mutate
`active_pane_id` — is verified by the Plan 04 integration test against a running app.
The stub is intentionally kept rather than deleted because it gives the whitelist a
canonical, reviewable location in the test suite.

---

## Wave 0 Requirements

- [ ] `Cargo.toml` — add `serde` (derive), `serde_json`, `uuid` (v4), `libc` dependencies before any Phase 3 source compiles
- [ ] `src/socket/mod.rs` — placeholder module with `#[cfg(test)] mod tests` block; stubs for SOCK-01, SOCK-05
- [ ] `src/socket/auth.rs` — `test_peercred_rejection` stub (SOCK-06)
- [ ] `src/session.rs` — `test_save_triggered`, `test_restore_roundtrip`, `test_atomic_write`, `test_graceful_fallback` stubs (SESS-01 through SESS-04)

Wave 0 is delivered by `03-00-PLAN.md` which must execute before any socket or session plan.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Python integration test suite runs against live Linux app | SOCK-04 | Requires live running cmux-linux process connected to socket | Set `CMUX_SOCKET=$XDG_RUNTIME_DIR/cmux/cmux.sock`, run `python3 tests_v2/test_ctrl_socket.py` against running app |
| Session restored visually after relaunch | SESS-02 | Requires visual confirmation of workspace/pane layout | Create 2 workspaces + splits, quit app, relaunch, verify layout restored |
| kill -9 mid-save does not corrupt session | SESS-03 | Requires timed kill during save window | During active mutations, `kill -9 $(pgrep cmux-linux)`, relaunch, verify session loads or gracefully fallbacks |
| XDG_RUNTIME_DIR fallback socket path | SOCK-03 | Requires unset XDG_RUNTIME_DIR at launch | Unset `XDG_RUNTIME_DIR`, launch app, verify socket appears at `/run/user/{uid}/cmux/cmux.sock` |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (socket/mod.rs, socket/auth.rs, session.rs, Cargo.toml deps)
- [x] No watch-mode flags
- [x] Feedback latency < 20s
- [x] `nyquist_compliant: true` set in frontmatter
- [x] SOCK-03 row references existing test (test_socket_path_creation) not nonexistent test_xdg_path_fallback
- [x] SOCK-05 test_focus_policy documented as design stub with behavioral verification deferred to Plan 04

**Approval:** pending execution
