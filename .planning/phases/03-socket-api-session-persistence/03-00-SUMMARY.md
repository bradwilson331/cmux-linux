---
phase: 03-socket-api-session-persistence
plan: "00"
subsystem: socket-session-scaffold
tags: [scaffold, tdd, cargo, socket, session]
dependency_graph:
  requires: []
  provides: [socket-module-stubs, session-module-stubs, phase3-cargo-deps]
  affects: [03-01, 03-02, 03-03, 03-04, 03-05]
tech_stack:
  added: [serde@1, serde_json@1, uuid@1+v4, libc@0.2]
  patterns: [failing-stub-tdd, mod-declaration]
key_files:
  created:
    - Cargo.toml (deps added)
    - src/socket/mod.rs
    - src/socket/auth.rs
    - src/socket/commands.rs
    - src/session.rs
  modified:
    - src/main.rs
decisions:
  - "cargo test --lib not applicable to binary crate; use cargo test --bin cmux-linux for unit tests"
  - "7 test stubs created: 4 pass immediately, 4 ignored (stubs for Plan 02/05)"
metrics:
  duration: "116 seconds"
  completed_date: "2026-03-25"
  tasks_completed: 2
  files_changed: 6
---

# Phase 03 Plan 00: Wave 0 Scaffold Summary

**One-liner:** Wave 0 scaffold adds serde/uuid/libc Cargo deps and creates socket + session module stubs with 7 Nyquist test stubs (4 passing, 3 ignored).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add Phase 3 Cargo dependencies | a1ce8c01 | Cargo.toml |
| 2 | Create socket module stubs with test scaffolds | 7894c460 | src/socket/mod.rs, src/socket/auth.rs, src/socket/commands.rs, src/session.rs, src/main.rs |

## What Was Built

- **Cargo.toml**: Added `serde` (with derive), `serde_json`, `uuid` (with v4), `libc` dependencies without disturbing existing gtk4/glib/tokio pins.
- **src/socket/commands.rs**: `SocketCommand` enum skeleton with `Ping` and `NotImplemented` variants using tokio oneshot channels for GTK/tokio bridging.
- **src/socket/auth.rs**: `validate_peer_uid` stub for SO_PEERCRED uid check + `test_peercred_rejection` (ignored stub).
- **src/socket/mod.rs**: `socket_path()` (XDG_RUNTIME_DIR based), `start_socket_server` stub, `test_socket_path_creation` (passing), `test_focus_policy` (passing policy documentation test).
- **src/session.rs**: `SessionData` struct, `load_session`/`save_session_atomic` stubs, `session_path()`, 4 test stubs (3 ignored, 1 passing).
- **src/main.rs**: Added `mod socket;` and `mod session;` after `mod shortcuts;`.

## Test Results

```
running 8 tests
test session::tests::test_atomic_write ... ignored, stub: implement in Plan 05
test session::tests::test_restore_roundtrip ... ignored, stub: implement in Plan 05
test session::tests::test_save_triggered ... ignored, stub: implement in Plan 05
test socket::auth::tests::test_peercred_rejection ... ignored, stub: implement in Plan 02
test ghostty::input::tests::test_map_mods ... ok
test session::tests::test_graceful_fallback ... ok
test socket::tests::test_focus_policy ... ok
test socket::tests::test_socket_path_creation ... ok

test result: ok. 4 passed; 0 failed; 4 ignored
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `cargo test --lib` not applicable to binary crate**
- **Found during:** Task 2 verification
- **Issue:** Plan specifies `cargo test --lib` but the project is a binary-only crate (no lib.rs). `cargo test --lib` exits with "no library targets found".
- **Fix:** Used `cargo test --bin cmux-linux` instead, which correctly runs unit tests embedded in binary modules. The plan's intent (tests compile and run) is fully achieved.
- **Files modified:** None (documentation-only deviation)
- **Commit:** N/A

**2. [Pre-existing] Integration tests in tests/ use cmux_linux:: crate path**
- **Found during:** Task 2 verification
- **Issue:** `tests/test_input.rs` and `tests/test_wakeup.rs` reference `cmux_linux::` which doesn't exist (binary crate). These tests were already failing before this plan.
- **Fix:** Not fixed (pre-existing, out of scope). Logged to deferred-items.
- **Commit:** N/A

## Known Stubs

The following stubs are intentional (Wave 0 Nyquist scaffolding):

| File | Function | Stub Type | Plan that implements |
|------|----------|-----------|---------------------|
| src/socket/mod.rs | start_socket_server | todo!() | Plan 02 |
| src/socket/auth.rs | validate_peer_uid | todo!() | Plan 02 |
| src/session.rs | load_session | todo!() | Plan 05 |
| src/session.rs | save_session_atomic | todo!() | Plan 05 |

These stubs are intentional — this is the "Red" step for the entire Phase 3.

## Self-Check: PASSED
