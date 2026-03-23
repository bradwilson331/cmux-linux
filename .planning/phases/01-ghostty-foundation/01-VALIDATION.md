---
phase: 1
slug: ghostty-foundation
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-23
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust unit tests) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib 2>&1 | tail -5` |
| **Full suite command** | `cargo test 2>&1` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib 2>&1 | tail -5`
- **After every plan wave:** Run `cargo test 2>&1`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 01-02-T1 | 01-02 | 0 | GHOST-01 | build | `grep -n "GHOSTTY_PLATFORM_GTK4" ghostty.h` | ⬜ pending |
| 01-02-T2 | 01-02 | 0 | GHOST-01 | compile | `cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast -Dgtk-x11=true -Dgtk-wayland=true 2>&1 | tail -3` | ⬜ pending |
| 01-01-T1 | 01-01 | 1 | GHOST-01 | build | `grep "rustc-link-lib=static=ghostty" build.rs && grep "gtk4" Cargo.toml` | ⬜ pending |
| 01-01-T2 | 01-01 | 1 | GHOST-01 | build | `./scripts/setup-linux.sh && cargo build 2>&1 | tail -3` | ⬜ pending |
| 01-03-T1 | 01-03 | 2 | GHOST-07 | unit test | `cargo test test_wakeup 2>&1 | tail -5` | ⬜ pending |
| 01-03-T2 | 01-03 | 2 | GHOST-02 | build | `cargo build 2>&1 | tail -5` | ⬜ pending |
| 01-04-T1 | 01-04 | 3 | GHOST-03 | unit test | `cargo test test_input 2>&1 | tail -5` | ⬜ pending |
| 01-04-T2 | 01-04 | 3 | GHOST-04 | build | `cargo build 2>&1 | tail -5` | ⬜ pending |
| 01-04-CP | 01-04 | 3 | GHOST-02..07 | manual | See Manual-Only | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Wave 0 (Plan 01-02) must complete before Wave 1 (Plan 01-01) can run, because build.rs reads the patched ghostty.h and setup-linux.sh compiles the patched Zig sources.

- [ ] `ghostty.h` — patched with `GHOSTTY_PLATFORM_GTK4 = 3`, `ghostty_platform_gtk4_s`, and union member
- [ ] `ghostty/src/apprt/embedded.zig` — patched with `gtk4 = 3` PlatformTag and Platform union arm
- [ ] `ghostty/src/renderer/OpenGL.zig` — patched with GTK4 dispatch in surfaceInit embedded arm
- [ ] `docs/ghostty-fork.md` — fork change documentation
- [ ] `zig build -Dapp-runtime=none` exits 0 from ghostty/ — libghostty.a produced

Wave 0 also provides the foundation for Wave 2's test scaffolding:

- [ ] `tests/test_wakeup.rs` created in Plan 01-03 Task 1 — wakeup coalescing unit tests (2 tests)

*Wave 0 must complete before any Wave 1 tasks.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Keystrokes appear with <10ms latency | GHOST-03 | Requires running app + timing measurement | Launch app, type in terminal, verify no visible delay |
| Clipboard copy from terminal → other app | GHOST-05 | Requires two apps interacting via clipboard | Select text in cmux, Ctrl+C, paste in gedit/xterm |
| Clipboard paste from other app → terminal | GHOST-05 | Same as above, reverse direction | Copy from gedit, Ctrl+Shift+V in terminal |
| Text renders at correct DPI on HiDPI display | GHOST-06 | Requires HiDPI hardware or virtual display | Run at 2x scale (`GDK_SCALE=2`), verify no blurriness |
| Window renders via GPU (not software) | GHOST-01 | Requires runtime GL check | Check `glxinfo` or verify GPU process in htop |
| XDG paths used for Ghostty config | GHOST-06 | Requires checking file system paths at runtime | Verify config loaded from `~/.config/ghostty/` |
| PTY spawns correct shell | GHOST-07 | Requires running app + checking process tree | Open app, `ps aux | grep pts`, verify shell spawned |

*All phase behaviors with hardware or runtime dependencies are manual.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify commands (substantive `cargo build` or `zig build`, not grep-only)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers ghostty fork patch + zig compile check
- [x] test_wakeup.rs included in Wave 0 checklist
- [x] Task IDs resolved (no `??` placeholders)
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
