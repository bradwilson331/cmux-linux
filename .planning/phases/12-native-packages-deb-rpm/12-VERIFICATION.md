---
phase: 12-native-packages-deb-rpm
verified: 2026-03-29T18:15:00Z
status: passed
score: 12/12 must-haves verified
re_verification: false
---

# Phase 12: Native Packages (.deb + .rpm) Verification Report

**Phase Goal:** Users on Debian/Ubuntu and Fedora/RHEL can install cmux from a single package file with all dependencies resolved automatically
**Verified:** 2026-03-29T18:15:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | build-deb.sh produces a .deb file from pre-built binaries | VERIFIED | Script contains `dpkg-deb --build --root-owner-group` (line 80), accepts 3 positional binary args with defaults, staging dir assembly |
| 2 | .deb contains cmux-app and cmux at usr/bin/ | VERIFIED | `install -Dm0755` for cmux-app (line 34) and cmux (line 35) to `$PKG_ROOT/usr/bin/` |
| 3 | .deb contains cmuxd-remote at usr/lib/cmux/ | VERIFIED | `install -Dm0755` for cmuxd-remote (line 36) to `$PKG_ROOT/usr/lib/cmux/cmuxd-remote` |
| 4 | .deb declares runtime dependencies for GTK4 stack | VERIFIED | DEBIAN/control Depends line (line 71) lists 15 packages: libgtk-4-1, libfontconfig1, libfreetype6, libonig5, libgl1, libegl1, libharfbuzz0b, libglib2.0-0, libcairo2, libpango-1.0-0, libpangocairo-1.0-0, libpangoft2-1.0-0, libepoxy0, libxkbcommon0, libgraphene-1.0-0 |
| 5 | .deb includes all desktop metadata, icons, completions, and man page | VERIFIED | install lines for .desktop (39), .metainfo.xml (41), 3 icons (45-48), 3 completions (51-56), gzipped man page (60) |
| 6 | validate-deb.sh confirms .deb structure is correct | VERIFIED | 20 checks: 12 file paths via `dpkg-deb -c`, 8 metadata/dependency fields via `dpkg-deb -f` |
| 7 | build-rpm.sh produces a .rpm file from pre-built binaries | VERIFIED | Script contains `rpmbuild -bb` (line 63), stages all files to SOURCES, copies output RPM to dist/ |
| 8 | .rpm contains cmux-app and cmux at /usr/bin/ | VERIFIED | Spec %install uses `%{_bindir}/cmux-app` and `%{_bindir}/cmux` (lines 32-33); %files declares both |
| 9 | .rpm contains cmuxd-remote at /usr/lib64/cmux/ (Fedora uses lib64) | VERIFIED | Spec uses `%{_libdir}/cmux/cmuxd-remote` (line 34) which expands to /usr/lib64 on x86_64 |
| 10 | .rpm declares Requires for GTK4 stack | VERIFIED | 15 Requires lines in cmux.spec: gtk4, fontconfig, freetype, oniguruma, mesa-libGL, mesa-libEGL, harfbuzz, glib2, cairo, cairo-gobject, pango, gdk-pixbuf2, libepoxy, libxkbcommon, graphene |
| 11 | .rpm includes all desktop metadata, icons, completions, and man page | VERIFIED | Spec %install and %files declare all paths: .desktop, metainfo, 3 icons, 3 completions, man page |
| 12 | validate-rpm.sh confirms .rpm structure is correct | VERIFIED | Uses `rpm -qpl` (file listing), `rpm -qpi` (metadata), `rpm -qpR` (dependencies) with 23 checks |

**Score:** 12/12 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `packaging/scripts/build-deb.sh` | .deb packaging script | VERIFIED | 82 lines, executable, syntax valid, contains dpkg-deb --build |
| `packaging/scripts/validate-deb.sh` | .deb validation script | VERIFIED | 128 lines, executable, syntax valid, contains dpkg-deb checks |
| `packaging/rpm/cmux.spec` | RPM spec file | VERIFIED | 62 lines, 15 Requires, %install + %files for all paths, AutoReqProv disabled |
| `packaging/scripts/build-rpm.sh` | .rpm packaging script | VERIFIED | 77 lines, executable, syntax valid, rpmbuild -bb with --define injection |
| `packaging/scripts/validate-rpm.sh` | .rpm validation script | VERIFIED | 92 lines, executable, syntax valid, rpm -qp queries with temp file caching |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| build-deb.sh | packaging/desktop/ | install copies desktop entry and metainfo | WIRED | Lines 39-42 install .desktop and .metainfo.xml |
| build-deb.sh | packaging/completions/ | install copies shell completions | WIRED | Lines 51-56 install cmux.bash, _cmux, cmux.fish |
| build-deb.sh | packaging/man/cmux.1 | gzip and install man page | WIRED | Line 60: `gzip -9n < .../cmux.1` |
| build-rpm.sh | packaging/rpm/cmux.spec | rpmbuild -bb invokes spec file | WIRED | Line 67: `"$REPO_ROOT/packaging/rpm/cmux.spec"` |
| cmux.spec | packaging/desktop/ | %install copies desktop metadata | WIRED | Lines 36-37 install .desktop and .metainfo.xml |
| build-rpm.sh | packaging/desktop/ | cp stages desktop files to SOURCES | WIRED | Lines 46-47 cp .desktop and .metainfo.xml to staging |
| build-rpm.sh | packaging/completions/ | cp stages completions to SOURCES | WIRED | Lines 55-57 cp completions to staging |
| build-rpm.sh | packaging/man/cmux.1 | gzip stages man page | WIRED | Line 60: `gzip -9n < .../cmux.1` |

### Data-Flow Trace (Level 4)

Not applicable -- these are shell scripts that produce package files, not components rendering dynamic data.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| build-deb.sh syntax valid | `bash -n packaging/scripts/build-deb.sh` | Exit 0 | PASS |
| validate-deb.sh syntax valid | `bash -n packaging/scripts/validate-deb.sh` | Exit 0 | PASS |
| build-rpm.sh syntax valid | `bash -n packaging/scripts/build-rpm.sh` | Exit 0 | PASS |
| validate-rpm.sh syntax valid | `bash -n packaging/scripts/validate-rpm.sh` | Exit 0 | PASS |
| build-deb.sh has >= 9 install lines | `grep -c 'install -Dm' build-deb.sh` = 9 | 9 | PASS |
| cmux.spec has 15 Requires | `grep -c 'Requires:' cmux.spec` = 15 | 15 | PASS |
| Phase 11 dependencies exist | All 9 files present | All found | PASS |
| Commits verified | 4 commits in git log | 6a235b46, 6d5e94e1, 85f3a348, 9c8a7c5b | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| DEB-01 | 12-01 | User can install cmux via `dpkg -i cmux.deb` on Ubuntu 22.04+ | SATISFIED | build-deb.sh produces installable .deb via dpkg-deb --build; non-t64 package names for 22.04 compat |
| DEB-02 | 12-01 | Runtime dependencies auto-installed via apt | SATISFIED | DEBIAN/control Depends lists 15 packages covering GTK4, GL, font, input stacks |
| DEB-03 | 12-01 | Both cmux-app and cmux CLI installed to /usr/bin/ | SATISFIED | install -Dm0755 for both binaries to usr/bin/; validate-deb.sh checks both paths |
| DEB-04 | 12-01 | cmuxd-remote bundled at /usr/lib/cmux/ | SATISFIED | install -Dm0755 to usr/lib/cmux/cmuxd-remote; validate-deb.sh checks path |
| RPM-01 | 12-02 | User can install cmux via `dnf install cmux.rpm` on Fedora 38+ | SATISFIED | cmux.spec + build-rpm.sh produce installable RPM via rpmbuild -bb |
| RPM-02 | 12-02 | Runtime dependencies mapped to Fedora package names | SATISFIED | 15 Requires in cmux.spec using Fedora names (gtk4, mesa-libGL, etc.) from FEDORA_FALLBACK |
| RPM-03 | 12-02 | Both binaries and cmuxd-remote installed to correct paths | SATISFIED | Spec uses %{_bindir} and %{_libdir}/cmux/; validate-rpm.sh checks /usr/lib64/cmux/ |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | -- | -- | -- | No anti-patterns found in any of the 5 artifacts |

### Human Verification Required

### 1. DEB Install Test on Ubuntu

**Test:** Build the .deb on an Ubuntu machine with pre-built binaries, run `dpkg -i cmux_*.deb`, then `apt install -f` to resolve deps, then launch `cmux-app`
**Expected:** Package installs cleanly, apt resolves all 15 dependencies, cmux-app launches
**Why human:** Requires running dpkg/apt on a real or containerized Ubuntu system

### 2. RPM Install Test on Fedora

**Test:** Build the .rpm on a Fedora machine with pre-built binaries, run `dnf install cmux-*.rpm`, then launch `cmux-app`
**Expected:** Package installs cleanly, dnf resolves all 15 dependencies, cmux-app launches
**Why human:** Requires running rpmbuild/dnf on a real or containerized Fedora system

### 3. Validate-deb.sh End-to-End

**Test:** After building a .deb, run `bash packaging/scripts/validate-deb.sh` against it
**Expected:** All 20 checks PASS, script exits 0
**Why human:** Requires a built .deb file which requires compiled binaries

### 4. Validate-rpm.sh End-to-End

**Test:** After building an .rpm, run `bash packaging/scripts/validate-rpm.sh` against it
**Expected:** All 23 checks PASS, script exits 0
**Why human:** Requires a built .rpm file which requires compiled binaries and rpmbuild

### Gaps Summary

No gaps found. All 12 observable truths are verified at the code level. All 7 requirement IDs (DEB-01 through DEB-04, RPM-01 through RPM-03) are satisfied by substantive, wired artifacts. All 5 scripts pass bash syntax validation. No anti-patterns detected.

The phase goal of enabling package-based installation is achieved at the script/spec level. Full end-to-end validation (actual package build + install on target distros) requires human verification in CI or containerized environments.

---

_Verified: 2026-03-29T18:15:00Z_
_Verifier: Claude (gsd-verifier)_
