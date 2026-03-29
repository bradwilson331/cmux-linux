# Phase 12: Native Packages (.deb + .rpm) - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Create .deb and .rpm packages that install cmux (terminal app + CLI + remote daemon) with all runtime dependencies declared. Both packages consume the shared desktop metadata, icons, completions, and man page from Phase 11's `packaging/` directory. No AppImage, Flatpak, CI, or build script unification here -- only .deb and .rpm.

</domain>

<decisions>
## Implementation Decisions

### Package Build Tooling
- **D-01:** Use `dpkg-deb` for .deb package creation -- direct archive construction without debhelper/dh complexity. A packaging script assembles the directory tree and calls `dpkg-deb --build`.
- **D-02:** Use `rpmbuild` for .rpm package creation -- standard Fedora/RHEL tooling. A `.spec` file defines the package; packaging script invokes `rpmbuild`.

### Binary Build Strategy
- **D-03:** Package pre-built binaries. Packaging scripts assume `cmux-app`, `cmux`, and `cmuxd-remote` are already built before packaging runs. Build steps (cargo, zig, go) are NOT embedded in package build scripts.
- **D-04:** Build scripts take binary paths as arguments or use well-known locations (`target/release/cmux-app`, `target/release/cmux`, `daemon/remote/cmuxd-remote`).

### Install Path Layout (FHS-standard)
- **D-05:** Binary install paths:
  - `/usr/bin/cmux-app` -- terminal application
  - `/usr/bin/cmux` -- CLI tool
  - `/usr/lib/cmux/cmuxd-remote` -- remote daemon helper (not user-facing)
- **D-06:** Desktop integration paths:
  - `/usr/share/applications/com.cmux_lx.terminal.desktop`
  - `/usr/share/metainfo/com.cmux_lx.terminal.metainfo.xml`
  - `/usr/share/icons/hicolor/{48x48,128x128,256x256}/apps/com.cmux_lx.terminal.png`
- **D-07:** Completion and man page paths:
  - `/usr/share/bash-completion/completions/cmux`
  - `/usr/share/zsh/vendor-completions/_cmux`
  - `/usr/share/fish/vendor_completions.d/cmux.fish`
  - `/usr/share/man/man1/cmux.1.gz`

### Dependency Declaration
- **D-08:** Use `detect-deps.sh` output as baseline for runtime dependency lists, curated into explicit package names for each format.
- **D-09:** .deb `Depends:` field lists Debian package names from detect-deps.sh (native dpkg -S resolution on build host).
- **D-10:** .rpm `Requires:` field lists Fedora package names. Update detect-deps.sh Fedora fallback table to fill UNKNOWN entries before building .rpm.
- **D-11:** Core runtime deps that MUST appear: GTK4, fontconfig, freetype, oniguruma, GL/EGL, harfbuzz, pango, glib2, cairo.

### Claude's Discretion
- Exact debian/control field values (Section, Priority, Architecture, Maintainer)
- RPM spec file structure and macro usage
- Whether to gzip man page during packaging or pre-gzip it
- Packaging script structure (one script per format vs combined)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 11 Outputs (consumed by this phase)
- `packaging/desktop/com.cmux_lx.terminal.desktop` -- Desktop entry to install
- `packaging/desktop/com.cmux_lx.terminal.metainfo.xml` -- AppStream metainfo to install
- `packaging/icons/hicolor/*/apps/com.cmux_lx.terminal.png` -- Icons at 48/128/256px
- `packaging/completions/cmux.bash` -- Bash completion
- `packaging/completions/_cmux` -- Zsh completion
- `packaging/completions/cmux.fish` -- Fish completion
- `packaging/man/cmux.1` -- Man page source
- `packaging/scripts/detect-deps.sh` -- Dependency detection script

### Project Files
- `.planning/REQUIREMENTS.md` -- DEB-01 through DEB-04, RPM-01 through RPM-03
- `Cargo.toml` -- Package name, binary targets (cmux-app, cmux, cmux-generate)
- `daemon/remote/cmuxd-remote` -- Pre-built Go remote daemon binary

### Standards (external)
- Debian Policy Manual -- for debian/control fields and package structure
- RPM Packaging Guide (Fedora) -- for .spec file format and rpmbuild usage

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `packaging/scripts/detect-deps.sh` -- Runtime dependency detection, produces Debian and Fedora package names
- `packaging/scripts/validate-all.sh` -- Validation script for all phase 11 artifacts
- All desktop metadata, completions, man page already exist in `packaging/`

### Established Patterns
- Packaging scripts in `packaging/scripts/` with `#!/usr/bin/env bash` and `set -euo pipefail`
- REPO_ROOT pattern: `REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"`
- Binary targets: cmux-app (main app), cmux (CLI), cmux-generate (completion generator)

### Integration Points
- `target/release/cmux-app` and `target/release/cmux` -- Rust release binaries (must be built with `cargo build --release` before packaging)
- `daemon/remote/cmuxd-remote` -- Go binary, already pre-built at repo root
- Phase 14 build scripts will call these packaging scripts

</code_context>

<specifics>
## Specific Ideas

No specific requirements -- open to standard approaches within the decisions above.

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope.

</deferred>

---

*Phase: 12-native-packages-deb-rpm*
*Context gathered: 2026-03-29*
