---
phase: 2
slug: workspaces-pane-splits
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | xcodebuild unit tests (Swift XCTest) |
| **Config file** | GhosttyTabs.xcodeproj |
| **Quick run command** | `xcodebuild -scheme cmux-unit -destination 'platform=macOS' test 2>&1 | tail -20` |
| **Full suite command** | `xcodebuild -scheme cmux-unit -destination 'platform=macOS' test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `xcodebuild -scheme cmux-unit -destination 'platform=macOS' test 2>&1 | tail -20`
- **After every plan wave:** Run `xcodebuild -scheme cmux-unit -destination 'platform=macOS' test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 2-WS-01 | workspace | 1 | WS-01 | unit | `xcodebuild -scheme cmux-unit test` | ❌ W0 | ⬜ pending |
| 2-WS-02 | workspace | 1 | WS-02 | unit | `xcodebuild -scheme cmux-unit test` | ❌ W0 | ⬜ pending |
| 2-WS-03 | workspace | 1 | WS-03 | unit | `xcodebuild -scheme cmux-unit test` | ❌ W0 | ⬜ pending |
| 2-WS-04 | workspace | 1 | WS-04 | unit | `xcodebuild -scheme cmux-unit test` | ❌ W0 | ⬜ pending |
| 2-WS-05 | workspace | 1 | WS-05 | manual | N/A — UI keyboard interaction | N/A | ⬜ pending |
| 2-WS-06 | workspace | 1 | WS-06 | unit | `xcodebuild -scheme cmux-unit test` | ❌ W0 | ⬜ pending |
| 2-SPLIT-01 | splits | 2 | SPLIT-01 | unit | `xcodebuild -scheme cmux-unit test` | ❌ W0 | ⬜ pending |
| 2-SPLIT-02 | splits | 2 | SPLIT-02 | unit | `xcodebuild -scheme cmux-unit test` | ❌ W0 | ⬜ pending |
| 2-SPLIT-03 | splits | 2 | SPLIT-03 | manual | Focus routing visual check | N/A | ⬜ pending |
| 2-SPLIT-04 | splits | 2 | SPLIT-04 | manual | Drag-to-resize visual check | N/A | ⬜ pending |
| 2-SPLIT-05 | splits | 2 | SPLIT-05 | unit | `xcodebuild -scheme cmux-unit test` | ❌ W0 | ⬜ pending |
| 2-SPLIT-06 | splits | 2 | SPLIT-06 | unit | `xcodebuild -scheme cmux-unit test` | ❌ W0 | ⬜ pending |
| 2-SPLIT-07 | splits | 3 | SPLIT-07 | manual | Memory profiler 50-cycle test | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `Tests/WorkspaceManagerTests.swift` — stubs for WS-01 through WS-06 (create/close/rename/switch/independent)
- [ ] `Tests/PaneSplitTests.swift` — stubs for SPLIT-01, SPLIT-02, SPLIT-05, SPLIT-06
- [ ] `Tests/SurfaceRegistryTests.swift` — stubs for global surface HashMap (SPLIT-07 precondition)

*Existing XCTest infrastructure (cmux-unit scheme) confirmed present from Phase 1.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Keyboard shortcut capture (Ctrl+N new workspace, Ctrl+W close, Ctrl+1..9 switch) | WS-05 | Requires live app; GTK event propagation can't be unit-tested | Launch tagged debug app, press Ctrl+N repeatedly, verify new workspace tabs appear |
| Focus routing after split/close | SPLIT-03 | Requires visual confirmation that cursor blinks in correct pane | Split pane, press keys, verify only active pane receives input |
| Drag-to-resize divider | SPLIT-04 | Mouse drag interaction | Drag GtkPaned divider, verify proportional resize |
| Memory leak after 50 cycles | SPLIT-07 | Requires Instruments or manual GObject ref inspection | Create/close 50 workspaces, check for memory growth |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
