---
phase: 12
slug: native-packages-deb-rpm
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | bash scripts + dpkg/rpm CLI validation |
| **Config file** | packaging/scripts/validate-all.sh |
| **Quick run command** | `bash packaging/scripts/validate-all.sh` |
| **Full suite command** | `bash packaging/scripts/validate-all.sh && dpkg-deb --info dist/cmux.deb && rpm -qip dist/cmux.rpm 2>/dev/null` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bash packaging/scripts/validate-all.sh`
- **After every plan wave:** Run full suite command
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 12-01-01 | 01 | 1 | DEB-01,DEB-02,DEB-03,DEB-04 | integration | `dpkg-deb --info dist/cmux.deb` | ❌ W0 | ⬜ pending |
| 12-02-01 | 02 | 1 | RPM-01,RPM-02,RPM-03 | integration | `rpm -qip dist/cmux.rpm` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `dist/` output directory created during packaging
- [ ] `dpkg-deb` available (pre-installed on Debian/Ubuntu)
- [ ] `rpm` package installed (`sudo apt install rpm`) for rpmbuild

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| dpkg -i installs and cmux launches | DEB-01 | Requires clean system or container | Install .deb on Ubuntu 22.04, run `cmux --version` |
| dnf install and cmux launches | RPM-01 | Requires Fedora system or container | Install .rpm on Fedora 38+, run `cmux --version` |
| apt install -f resolves deps | DEB-02 | Requires system with missing deps | After dpkg -i, run apt install -f and verify all deps installed |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
