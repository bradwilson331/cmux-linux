---
phase: 2
slug: workspaces-pane-splits
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-23
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` with `#[cfg(test)]` modules |
| **Config file** | `Cargo.toml` (workspace root) |
| **Quick run command** | `cargo test 2>&1 \| grep -E 'FAILED\|error\[' \| head -20` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test 2>&1 | grep -E 'FAILED|error\[' | head -20`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 2-WS-01 | 02-00 | 0 | WS-01 | unit | `cargo test workspace` | ❌ W0 | ⬜ pending |
| 2-WS-02 | 02-00 | 0 | WS-02 | unit | `cargo test workspace` | ❌ W0 | ⬜ pending |
| 2-WS-03 | 02-00 | 0 | WS-03 | unit | `cargo test workspace` | ❌ W0 | ⬜ pending |
| 2-WS-04 | 02-00 | 0 | WS-04 | unit | `cargo test workspace` | ❌ W0 | ⬜ pending |
| 2-WS-05 | 02-00 | 0 | WS-05 | unit | `cargo test app_state` | ❌ W0 | ⬜ pending |
| 2-WS-06 | 02-04 | 3 | WS-06 | manual | Build + launch visual check | N/A | ⬜ pending |
| 2-SPLIT-01 | 02-00 | 0 | SPLIT-01 | unit | `cargo test split_engine` | ❌ W0 | ⬜ pending |
| 2-SPLIT-02 | 02-00 | 0 | SPLIT-02 | unit | `cargo test split_engine` | ❌ W0 | ⬜ pending |
| 2-SPLIT-03 | 02-05 | 4 | SPLIT-03 | manual | Focus routing visual check | N/A | ⬜ pending |
| 2-SPLIT-04 | 02-03 | 2 | SPLIT-04 | manual | Drag-to-resize visual check | N/A | ⬜ pending |
| 2-SPLIT-05 | 02-00 | 0 | SPLIT-05 | unit | `cargo test split_engine` | ❌ W0 | ⬜ pending |
| 2-SPLIT-06 | 02-00 | 0 | SPLIT-06 | unit | `cargo test split_engine` | ❌ W0 | ⬜ pending |
| 2-SPLIT-07 | 02-05 | 4 | SPLIT-07 | manual | Focus after sidebar click visual check | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/workspace.rs` — `#[cfg(test)]` module with stubs for WS-01 through WS-06 (create/close/rename/switch)
- [ ] `src/split_engine.rs` — `#[cfg(test)]` module with stubs for SPLIT-01, SPLIT-02, SPLIT-05, SPLIT-06 (tree split/close operations)
- [ ] `src/app_state.rs` — `#[cfg(test)]` module with compile-time stub for WS-05 `switch_to_index(usize)` (no GTK deps)

Wave 0 is delivered by `02-00-PLAN.md` which must execute before Plans 02-02 and 02-03.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Keyboard shortcut capture (Ctrl+N new workspace, Ctrl+W close, Ctrl+1..9 switch) | WS-05 | Requires live app; GTK event propagation cannot be unit-tested | Launch tagged debug app, press Ctrl+N repeatedly, verify new workspace tabs appear |
| Initial window layout (sidebar + terminal) | WS-06 | Requires visual confirmation | Launch app, verify 160px sidebar on left, terminal on right |
| Focus routing after split/close | SPLIT-03 | Requires visual confirmation that cursor blinks in correct pane | Split pane, press keys, verify only active pane receives input |
| Drag-to-resize divider | SPLIT-04 | Mouse drag interaction; GtkPaned handles natively | Drag GtkPaned divider, verify proportional resize |
| Memory leak after 50 cycles | SPLIT-07 | Requires runtime inspection | Create/close 50 workspaces, check for memory growth |
| Focus on sidebar click | SPLIT-07 | Requires visual confirmation that active pane receives keyboard input after sidebar click | Click sidebar row, type in terminal, verify focus went to new workspace pane |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (workspace.rs, split_engine.rs, app_state.rs)
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending execution
