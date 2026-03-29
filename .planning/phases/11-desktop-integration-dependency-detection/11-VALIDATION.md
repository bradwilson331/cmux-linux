---
phase: 11
slug: desktop-integration-dependency-detection
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | bash + appstreamcli (metadata validation), man (man page validation), cargo build (gen binary compiles) |
| **Config file** | None needed |
| **Quick run command** | `appstreamcli validate --no-net packaging/desktop/com.cmux_lx.terminal.metainfo.xml` |
| **Full suite command** | `bash packaging/scripts/validate-all.sh` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `appstreamcli validate --no-net packaging/desktop/com.cmux_lx.terminal.metainfo.xml`
- **After every plan wave:** Run `bash packaging/scripts/validate-all.sh`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 11-01-01 | 01 | 1 | META-01 | smoke | `grep -c 'com.cmux_lx.terminal' packaging/desktop/*.desktop packaging/desktop/*.xml` | ❌ W0 | ⬜ pending |
| 11-01-02 | 01 | 1 | META-02 | smoke | `appstreamcli validate --no-net packaging/desktop/com.cmux_lx.terminal.metainfo.xml` | ❌ W0 | ⬜ pending |
| 11-02-01 | 02 | 1 | META-03 | smoke | `file packaging/icons/hicolor/*/apps/*.png` + dimension check | ❌ W0 | ⬜ pending |
| 11-03-01 | 03 | 2 | META-04 | smoke | `test -f packaging/completions/cmux.bash && test -f packaging/completions/_cmux && test -f packaging/completions/cmux.fish` | ❌ W0 | ⬜ pending |
| 11-03-02 | 03 | 2 | META-05 | smoke | `man -l packaging/man/cmux.1 > /dev/null 2>&1` | ❌ W0 | ⬜ pending |
| 11-04-01 | 04 | 1 | BUILD-02 | smoke | `bash packaging/scripts/detect-deps.sh target/debug/cmux-app \| grep -q libgtk` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `packaging/scripts/validate-all.sh` — smoke test script covering all META + BUILD-02 requirements
- [ ] `rsvg-convert` must be installed (`apt install librsvg2-bin`) before icon generation

*Existing infrastructure covers no phase requirements — all validation is new.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Desktop entry launches app | META-01 | Requires running desktop environment | Install .desktop, run `gtk-launch com.cmux_lx.terminal` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
