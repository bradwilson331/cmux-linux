# Phase 12: Native Packages (.deb + .rpm) - Research

**Researched:** 2026-03-29
**Domain:** Linux package creation (dpkg-deb, rpmbuild)
**Confidence:** HIGH

## Summary

This phase creates two packaging scripts that assemble pre-built cmux binaries into installable .deb and .rpm packages. Both formats are well-documented, mature systems. The .deb path uses `dpkg-deb --build` with a DEBIAN/control file. The .rpm path uses `rpmbuild -bb` with a .spec file. Both consume Phase 11's desktop metadata, icons, completions, and man page from the `packaging/` directory.

The key complexity is dependency declaration -- mapping shared library dependencies to correct distro package names. Phase 11 already created `detect-deps.sh` with verified fallback tables for both Debian and Fedora. The packaging scripts will use curated dependency lists derived from that output.

**Primary recommendation:** Two standalone scripts (`packaging/scripts/build-deb.sh` and `packaging/scripts/build-rpm.sh`) that each assemble a staging directory, copy binaries and metadata, and invoke the respective package tool. A shared helper extracts version from Cargo.toml.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use `dpkg-deb` for .deb package creation -- direct archive construction without debhelper/dh complexity
- **D-02:** Use `rpmbuild` for .rpm package creation -- standard Fedora/RHEL tooling with a `.spec` file
- **D-03:** Package pre-built binaries. Build steps (cargo, zig, go) are NOT embedded in package build scripts
- **D-04:** Build scripts take binary paths as arguments or use well-known locations (`target/release/cmux-app`, `target/release/cmux`, `daemon/remote/cmuxd-remote`)
- **D-05:** Binary install paths: `/usr/bin/cmux-app`, `/usr/bin/cmux`, `/usr/lib/cmux/cmuxd-remote`
- **D-06:** Desktop integration paths use `com.cmux_lx.terminal` reverse-DNS prefix
- **D-07:** Completion and man page paths follow standard FHS locations
- **D-08:** Use `detect-deps.sh` output as baseline for runtime dependency lists
- **D-09:** .deb `Depends:` field uses Debian package names from detect-deps.sh
- **D-10:** .rpm `Requires:` field uses Fedora package names; update detect-deps.sh fallback table if needed
- **D-11:** Core runtime deps that MUST appear: GTK4, fontconfig, freetype, oniguruma, GL/EGL, harfbuzz, pango, glib2, cairo

### Claude's Discretion
- Exact debian/control field values (Section, Priority, Architecture, Maintainer)
- RPM spec file structure and macro usage
- Whether to gzip man page during packaging or pre-gzip it
- Packaging script structure (one script per format vs combined)

### Deferred Ideas (OUT OF SCOPE)
None.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DEB-01 | `dpkg -i cmux.deb` installs on Ubuntu 22.04+/Debian 12+ | dpkg-deb --build with DEBIAN/control; verified package names work on 22.04 and 24.04 |
| DEB-02 | Runtime deps auto-installed via apt | Depends field with verified Debian package names from detect-deps.sh fallback table |
| DEB-03 | cmux-app and cmux CLI installed to `/usr/bin/` | FHS layout in staging directory, verified by dpkg -L after install |
| DEB-04 | cmuxd-remote bundled at `/usr/lib/cmux/cmuxd-remote` | Custom lib path in staging directory |
| RPM-01 | `dnf install cmux.rpm` on Fedora 38+/RHEL 9+ | rpmbuild -bb with .spec file; Requires field with Fedora package names |
| RPM-02 | Runtime deps mapped to Fedora/RHEL package names | Fedora fallback table in detect-deps.sh already maps all core deps |
| RPM-03 | Both binaries and cmuxd-remote installed to correct paths | %files section in .spec with %{_bindir} and %{_libdir}/cmux/ macros |
</phase_requirements>

## Standard Stack

### Core
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| dpkg-deb | 1.22.6 (verified on system) | Build .deb archives | Standard Debian tool, no additional deps needed |
| rpmbuild | 4.18.x (via `rpm` apt package) | Build .rpm archives | Standard Fedora/RHEL tool, available cross-distro via apt |
| fakeroot | 1.33 (verified on system) | Root-ownership simulation | Required for dpkg-deb to set correct file ownership |

### Supporting
| Tool | Purpose | When to Use |
|------|---------|-------------|
| gzip | Compress man page | During .deb and .rpm staging (man pages must be .gz) |
| install (coreutils) | Copy files with correct permissions | Setting 0755 on binaries, 0644 on data files |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| dpkg-deb | debhelper/dh_make | More automation but massive complexity for a simple binary package |
| rpmbuild | nfpm | Single tool for both formats but non-standard, less control |

**Installation (for rpmbuild on Debian/Ubuntu):**
```bash
sudo apt install rpm
```

## Architecture Patterns

### Recommended File Layout
```
packaging/
  scripts/
    build-deb.sh          # .deb packaging script
    build-rpm.sh           # .rpm packaging script
    detect-deps.sh         # (existing) dependency detection
    validate-all.sh        # (existing) artifact validation
  rpm/
    cmux.spec              # RPM spec file template
  completions/             # (existing from Phase 11)
  desktop/                 # (existing from Phase 11)
  icons/                   # (existing from Phase 11)
  man/                     # (existing from Phase 11)
```

### Pattern 1: .deb Staging Directory
**What:** Assemble FHS-compliant directory tree, add DEBIAN/control, call dpkg-deb
**When to use:** Every .deb build

```bash
# Staging directory structure
pkg_root/
  DEBIAN/
    control               # Package metadata + Depends
  usr/
    bin/
      cmux-app            # Terminal application binary (0755)
      cmux                # CLI tool binary (0755)
    lib/
      cmux/
        cmuxd-remote      # Remote daemon helper (0755)
    share/
      applications/
        com.cmux_lx.terminal.desktop
      metainfo/
        com.cmux_lx.terminal.metainfo.xml
      icons/
        hicolor/
          48x48/apps/com.cmux_lx.terminal.png
          128x128/apps/com.cmux_lx.terminal.png
          256x256/apps/com.cmux_lx.terminal.png
      bash-completion/
        completions/cmux
      zsh/
        vendor-completions/_cmux
      fish/
        vendor_completions.d/cmux.fish
      man/
        man1/
          cmux.1.gz       # Must be gzipped
```

**Build command:**
```bash
dpkg-deb --build --root-owner-group "$PKG_ROOT" "$OUTPUT_DIR/cmux_${VERSION}_amd64.deb"
```

### Pattern 2: RPM Spec File (Pre-built Binaries)
**What:** Spec file with empty %build, %install copies from source dir, %files declares paths
**When to use:** Every .rpm build

```spec
Name:           cmux
Version:        %{_cmux_version}
Release:        1%{?dist}
Summary:        GPU-accelerated terminal multiplexer
License:        Proprietary
URL:            https://cmux.dev

# No Source0 -- binaries provided externally
# No BuildRequires -- pre-built

Requires:       gtk4 fontconfig freetype oniguruma mesa-libGL mesa-libEGL
Requires:       harfbuzz glib2 cairo pango libepoxy libxkbcommon

%description
cmux is a GPU-accelerated terminal with tabs, splits, workspaces,
and socket CLI control -- powered by Ghostty.

%install
# Copy pre-built binaries and metadata into buildroot
install -Dm0755 %{_sourcedir}/cmux-app %{buildroot}%{_bindir}/cmux-app
install -Dm0755 %{_sourcedir}/cmux %{buildroot}%{_bindir}/cmux
install -Dm0755 %{_sourcedir}/cmuxd-remote %{buildroot}%{_libdir}/cmux/cmuxd-remote
# ... desktop, icons, completions, man page

%files
%{_bindir}/cmux-app
%{_bindir}/cmux
%{_libdir}/cmux/cmuxd-remote
%{_datadir}/applications/com.cmux_lx.terminal.desktop
%{_datadir}/metainfo/com.cmux_lx.terminal.metainfo.xml
%{_datadir}/icons/hicolor/*/apps/com.cmux_lx.terminal.png
%{_datadir}/bash-completion/completions/cmux
%{_datadir}/zsh/vendor-completions/_cmux
%{_datadir}/fish/vendor_completions.d/cmux.fish
%{_mandir}/man1/cmux.1.gz
```

**Build command:**
```bash
rpmbuild -bb --define "_cmux_version ${VERSION}" \
  --define "_sourcedir ${STAGING_DIR}" \
  --define "_topdir ${BUILD_DIR}/rpmbuild" \
  packaging/rpm/cmux.spec
```

### Pattern 3: Version Extraction
**What:** Extract version from Cargo.toml to avoid duplication
```bash
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
```

### Anti-Patterns to Avoid
- **Embedding build steps in packaging scripts:** Build (cargo, zig, go) must happen before packaging. Packaging scripts only copy and assemble.
- **Hardcoding dependency lists:** Use detect-deps.sh fallback tables as source of truth, curated into the control/spec files.
- **Using `fakeroot` with rpmbuild:** rpmbuild handles ownership internally via %install macros; fakeroot is only needed for dpkg-deb.
- **Forgetting `--root-owner-group` with dpkg-deb:** Without this flag, files retain builder's UID/GID, which breaks on target systems.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| .deb creation | Custom tar+ar assembly | `dpkg-deb --build` | Handles format version, checksums, metadata correctly |
| .rpm creation | Custom cpio assembly | `rpmbuild -bb` | Handles RPM header, compression, signature slots |
| File ownership | Manual chown in scripts | `--root-owner-group` (deb) / `install -Dm` (rpm) | Correct ownership without actual root privileges |
| Man page compression | Custom gzip wrapper | `gzip -9n` on the man page during staging | Standard practice, -n removes timestamp for reproducibility |

## Common Pitfalls

### Pitfall 1: File Permissions in .deb
**What goes wrong:** Binaries installed without execute permission, or data files with wrong mode
**Why it happens:** `cp` preserves source permissions which may not be 0755
**How to avoid:** Use `install -Dm0755` for binaries, `install -Dm0644` for data files
**Warning signs:** `dpkg -i` succeeds but `cmux` command says "Permission denied"

### Pitfall 2: Missing --root-owner-group for dpkg-deb
**What goes wrong:** Files inside .deb owned by UID 1000 instead of root
**Why it happens:** dpkg-deb preserves filesystem ownership by default
**How to avoid:** Always pass `--root-owner-group` to dpkg-deb, or wrap in `fakeroot`
**Warning signs:** `dpkg-deb -c` shows non-root ownership in file listing

### Pitfall 3: Ubuntu t64 Transition Package Names
**What goes wrong:** Depends on `libglib2.0-0t64` which doesn't exist on Ubuntu 22.04
**Why it happens:** Ubuntu 24.04 renamed some packages with t64 suffix for 64-bit time_t
**How to avoid:** Use the non-t64 name (`libglib2.0-0`) -- it's a virtual package provided by `libglib2.0-0t64` on 24.04 and a real package on 22.04. Verified that `libgtk-4-1`, `libharfbuzz0b`, `libcairo2`, `libfreetype6`, `libfontconfig1`, `libonig5` all remain unchanged (no t64 suffix) on both releases.
**Warning signs:** `apt install -f` fails on older Ubuntu

### Pitfall 4: rpmbuild Directory Structure
**What goes wrong:** rpmbuild fails because expected directories don't exist
**Why it happens:** rpmbuild expects `~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}` or `_topdir` override
**How to avoid:** Always set `--define "_topdir ${BUILD_DIR}/rpmbuild"` and create subdirectories
**Warning signs:** "error: failed to open" messages from rpmbuild

### Pitfall 5: Man Page Not Gzipped
**What goes wrong:** `man cmux` works but lintian/rpmlint warns about uncompressed man page
**Why it happens:** Source man page in packaging/man/ is plaintext troff
**How to avoid:** Gzip during staging: `gzip -9n < packaging/man/cmux.1 > staging/usr/share/man/man1/cmux.1.gz`
**Warning signs:** Package linting warnings

### Pitfall 6: Fedora Dependency Name Mismatches
**What goes wrong:** `dnf install cmux.rpm` fails due to unknown package name in Requires
**Why it happens:** Fedora package names differ from Debian (e.g., `gtk4` vs `libgtk-4-1`, `glib2` vs `libglib2.0-0`)
**How to avoid:** Verify Fedora names against detect-deps.sh FEDORA_FALLBACK table; test on actual Fedora system
**Warning signs:** "No match for argument" from dnf

## Code Examples

### debian/control File (Verified Fields)
```
Package: cmux
Version: 0.1.0
Architecture: amd64
Maintainer: cmux <packaging@cmux.dev>
Section: x11
Priority: optional
Depends: libgtk-4-1, libfontconfig1, libfreetype6, libonig5, libgl1, libegl1, libharfbuzz0b, libglib2.0-0, libcairo2, libpango-1.0-0, libpangocairo-1.0-0, libpangoft2-1.0-0, libepoxy0, libxkbcommon0, libgraphene-1.0-0
Homepage: https://cmux.dev
Description: GPU-accelerated terminal multiplexer
 cmux provides tabs, splits, workspaces, and socket CLI control
 powered by Ghostty's GPU-accelerated terminal rendering.
```

Note: Package names verified against `detect-deps.sh` DEBIAN_FALLBACK table and confirmed on Ubuntu 24.04 system via `dpkg -S`. Using non-t64 names for 22.04 compatibility.

### RPM Requires (Verified Fedora Names)
```
Requires: gtk4, fontconfig, freetype, oniguruma, mesa-libGL, mesa-libEGL
Requires: harfbuzz, glib2, cairo, pango, libepoxy, libxkbcommon, graphene
```

Names from `detect-deps.sh` FEDORA_FALLBACK table. Confidence MEDIUM -- not verified on actual Fedora system.

### Packaging Script Pattern
```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Extract version from Cargo.toml
VERSION=$(grep '^version' "$REPO_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')

# Default binary locations (overridable)
CMUX_APP="${1:-$REPO_ROOT/target/release/cmux-app}"
CMUX_CLI="${2:-$REPO_ROOT/target/release/cmux}"
CMUXD_REMOTE="${3:-$REPO_ROOT/daemon/remote/cmuxd-remote}"

# Verify binaries exist
for bin in "$CMUX_APP" "$CMUX_CLI" "$CMUXD_REMOTE"; do
    [[ -f "$bin" ]] || { echo "ERROR: Binary not found: $bin" >&2; exit 1; }
done
```

### Output File Naming Convention
```
cmux_0.1.0_amd64.deb      # Debian: {name}_{version}_{arch}.deb
cmux-0.1.0-1.x86_64.rpm   # RPM: {name}-{version}-{release}.{arch}.rpm
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| debhelper/dh_make for all .debs | dpkg-deb --build for simple binary packages | Always valid | Simpler, no debhelper dependency chain |
| checkinstall | Direct dpkg-deb | checkinstall unmaintained | More reliable, explicit control |
| fakeroot mandatory | --root-owner-group flag on dpkg-deb | dpkg 1.21.10+ (2022) | No fakeroot needed if flag available |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| dpkg-deb | .deb creation | Yes | 1.22.6 | -- |
| rpmbuild | .rpm creation | No | -- | `sudo apt install rpm` |
| fakeroot | .deb ownership (backup) | Yes | 1.33 | --root-owner-group flag preferred |
| gzip | Man page compression | Yes | system | -- |
| install (coreutils) | File copying with permissions | Yes | system | -- |

**Missing dependencies with no fallback:**
- None (rpmbuild is installable via apt)

**Missing dependencies with fallback:**
- rpmbuild: Not currently installed but available as `apt install rpm` package. Scripts should check and provide clear error message.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Bash validation scripts |
| Config file | packaging/scripts/validate-all.sh (existing) |
| Quick run command | `bash packaging/scripts/validate-deb.sh dist/cmux_*.deb` |
| Full suite command | `bash packaging/scripts/validate-all.sh` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DEB-01 | .deb installs via dpkg | smoke (requires root/docker) | `dpkg-deb -c dist/cmux_*.deb` (structure check) | No -- Wave 0 |
| DEB-02 | Dependencies declared correctly | unit | `dpkg-deb -f dist/cmux_*.deb Depends` (verify field) | No -- Wave 0 |
| DEB-03 | Binaries at /usr/bin/ | unit | `dpkg-deb -c dist/cmux_*.deb \| grep usr/bin/` | No -- Wave 0 |
| DEB-04 | cmuxd-remote at /usr/lib/cmux/ | unit | `dpkg-deb -c dist/cmux_*.deb \| grep usr/lib/cmux/` | No -- Wave 0 |
| RPM-01 | .rpm installs via dnf | smoke (requires Fedora) | `rpm -qpi dist/cmux-*.rpm` (metadata check) | No -- Wave 0 |
| RPM-02 | Dependencies declared correctly | unit | `rpm -qpR dist/cmux-*.rpm` (verify Requires) | No -- Wave 0 |
| RPM-03 | Binaries at correct paths | unit | `rpm -qpl dist/cmux-*.rpm \| grep usr/bin/` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** Build .deb and .rpm, run `dpkg-deb -c` and `rpm -qpl` to verify contents
- **Per wave merge:** Full validate script checking both packages
- **Phase gate:** Both packages build without errors, contain all expected files

### Wave 0 Gaps
- [ ] `packaging/scripts/validate-deb.sh` -- validate .deb structure, file list, dependencies
- [ ] `packaging/scripts/validate-rpm.sh` -- validate .rpm structure, file list, Requires

## Open Questions

1. **Fedora package name verification**
   - What we know: FEDORA_FALLBACK table in detect-deps.sh covers all core deps
   - What's unclear: Names not verified on actual Fedora system (only from static table)
   - Recommendation: Accept MEDIUM confidence; verify in CI when Fedora runner available

2. **Maintainer email for control/spec files**
   - What we know: Need a maintainer field
   - What's unclear: What email to use
   - Recommendation: Use placeholder that Phase 14 CI can override; suggest `cmux <noreply@cmux.dev>`

## Project Constraints (from CLAUDE.md)

- **Testing policy:** Never run tests locally; all tests via GitHub Actions or VM. Package validation scripts can do structural checks (dpkg-deb -c) but actual install tests require CI/containers.
- **Test quality policy:** Tests must verify observable runtime behavior. For packages, this means verifying the built artifact (file listing, metadata fields) rather than source files.
- **Script conventions:** `#!/usr/bin/env bash`, `set -euo pipefail`, `REPO_ROOT` pattern.
- **Build conventions:** Release builds use `-Doptimize=ReleaseFast` for zig, `cargo build --release` for Rust, pre-built Go binary.

## Sources

### Primary (HIGH confidence)
- [dpkg-deb(1) man page](https://man7.org/linux/man-pages/man1/dpkg-deb.1.html) -- build command, --root-owner-group flag
- [deb-control(5)](https://manpages.debian.org/testing/dpkg-dev/deb-control.5.en.html) -- control file fields
- [RPM Packaging Guide](https://rpm-packaging-guide.github.io/) -- spec file format, rpmbuild -bb, %install/%files
- Local system verification via `dpkg -S`, `dpkg -l` -- confirmed package names on Ubuntu 24.04
- Existing `packaging/scripts/detect-deps.sh` -- verified Debian and Fedora fallback tables

### Secondary (MEDIUM confidence)
- [Building binary deb packages guide](https://www.internalpointers.com/post/build-binary-deb-package-practical-guide) -- practical walkthrough, verified against official docs
- [libgtk-4-1 Ubuntu package](https://launchpad.net/ubuntu/noble/+package/libgtk-4-1) -- confirmed no t64 rename for GTK4

### Tertiary (LOW confidence)
- Fedora package names from static fallback table -- not verified on Fedora system

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- dpkg-deb and rpmbuild are mature, well-documented tools verified on system
- Architecture: HIGH -- FHS paths locked in CONTEXT.md, staging directory pattern is standard
- Pitfalls: HIGH -- t64 transition verified locally, other pitfalls from official documentation
- Dependencies: MEDIUM for Fedora names (static table, not runtime-verified)

**Research date:** 2026-03-29
**Valid until:** 2026-06-29 (stable tools, no fast-moving components)
