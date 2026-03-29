# Phase 12: Native Packages (.deb + .rpm) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 12-native-packages-deb-rpm
**Areas discussed:** Package build tooling, Binary build strategy, Install path layout, Dependency declaration
**Mode:** auto (all decisions auto-selected)

---

## Package Build Tooling

| Option | Description | Selected |
|--------|-------------|----------|
| dpkg-deb + rpmbuild | Direct archive tools, standard for each distro | ✓ |
| debhelper/dh + rpmbuild | Full Debian packaging framework, more complex | |
| fpm (both formats) | Single tool for both, less distro-standard | |

**User's choice:** [auto] dpkg-deb for .deb, rpmbuild for .rpm (recommended default)
**Notes:** Simplest approach producing standard packages. fpm would unify tooling but adds a Ruby dependency and is less standard.

---

## Binary Build Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Pre-built binaries | Package scripts assume binaries already built | ✓ |
| Build from source | Embed cargo/zig/go build in package scripts | |

**User's choice:** [auto] Pre-built binaries (recommended default)
**Notes:** Rust + Zig + Go toolchain is too complex to embed in package build. Separation of build and packaging is cleaner.

---

## Install Path Layout

| Option | Description | Selected |
|--------|-------------|----------|
| FHS-standard paths | /usr/bin, /usr/lib, /usr/share | ✓ |
| /opt/cmux prefix | Self-contained under /opt | |

**User's choice:** [auto] FHS-standard paths (recommended default)
**Notes:** Standard FHS layout integrates naturally with distro package managers and desktop environments.

---

## Dependency Declaration

| Option | Description | Selected |
|--------|-------------|----------|
| detect-deps.sh + curation | Automated baseline, manually curated | ✓ |
| Hardcoded list only | Manual dependency list, no automation | |
| Automatic shlibs:Depends | Let dpkg-shlibdeps resolve (deb only) | |

**User's choice:** [auto] detect-deps.sh output as baseline with curation (recommended default)
**Notes:** Automated detection catches all shared lib deps. Manual curation ensures correct package names and fills UNKNOWN entries.

---

## Claude's Discretion

- Exact debian/control field values
- RPM spec file structure
- Man page compression strategy
- Packaging script structure

## Deferred Ideas

None
