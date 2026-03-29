# Requirements: cmux Linux Packaging & Distribution

**Defined:** 2026-03-29
**Core Value:** A Linux user should get the same cmux experience as a Mac user: tabs, splits, workspaces, and socket CLI control -- powered by Ghostty's GPU-accelerated terminal.

## v1.1 Requirements

Requirements for packaging and distribution milestone. Each maps to roadmap phases.

### Desktop Integration

- [x] **META-01**: App uses reverse-DNS ID `com.cmux-lx.terminal` across .desktop, metainfo, and icons
- [x] **META-02**: AppStream metainfo XML provides app description, screenshots section, and content rating
- [x] **META-03**: PNG icons generated from SVG at 48px, 128px, 256px for hicolor icon theme
- [ ] **META-04**: Shell completions installed for bash, zsh, and fish for cmux CLI
- [ ] **META-05**: Man page installed at `/usr/share/man/man1/cmux.1.gz`

### Debian Package

- [ ] **DEB-01**: User can install cmux via `dpkg -i cmux.deb` on Ubuntu 22.04+/Debian 12+
- [ ] **DEB-02**: Runtime dependencies (GTK4, fontconfig, freetype, oniguruma, GL) auto-installed via apt
- [ ] **DEB-03**: Both cmux-app and cmux CLI binaries installed to `/usr/bin/`
- [ ] **DEB-04**: cmuxd-remote bundled at `/usr/lib/cmux/cmuxd-remote`

### RPM Package

- [ ] **RPM-01**: User can install cmux via `dnf install cmux.rpm` on Fedora 38+/RHEL 9+
- [ ] **RPM-02**: Runtime dependencies mapped to Fedora/RHEL package names and declared
- [ ] **RPM-03**: Both binaries and cmuxd-remote installed to correct paths

### AppImage

- [ ] **APPIMG-01**: User can download and run cmux AppImage on any x86_64 Linux without installation
- [ ] **APPIMG-02**: GTK4 runtime resources (schemas, loaders, themes) bundled via linuxdeploy-plugin-gtk
- [ ] **APPIMG-03**: Desktop integration (icon + .desktop) embedded in AppImage

### Flatpak

- [ ] **FLAT-01**: User can install cmux via Flatpak manifest with `org.gnome.Platform//47` runtime
- [ ] **FLAT-02**: Sandbox permissions configured for Wayland/X11, GPU, network (SSH), filesystem, notifications
- [ ] **FLAT-03**: Zig/Rust/Go toolchain handled in build (pre-build or sandbox strategy)

### Build Scripts

- [ ] **BUILD-01**: Local script builds all four package formats (or individually)
- [ ] **BUILD-02**: Runtime dependencies auto-detected from binary via ldd/readelf and mapped to package names
- [ ] **BUILD-03**: GPG signing supported for .deb and .rpm packages

### Gitea CI

- [ ] **CI-01**: Gitea Actions workflow triggers on tag push to build all packages
- [ ] **CI-02**: Built packages published to Gitea package registry (deb + rpm repos)
- [ ] **CI-03**: Built packages attached as Gitea release assets
- [ ] **CI-04**: act_runner configured for host-mode builds with full toolchain

## Future Requirements

### Multi-Architecture

- **ARCH-01**: ARM64 (aarch64) builds for Raspberry Pi / Asahi Linux
- **ARCH-02**: Cross-compilation pipeline for Rust + Zig + Go

### Distribution

- **DIST-01**: Flathub submission
- **DIST-02**: AUR (Arch User Repository) PKGBUILD
- **DIST-03**: AppImage delta updates via zsync

## Out of Scope

| Feature | Reason |
|---------|--------|
| Snap package | Canonical-specific, lower priority than Flatpak |
| Nix/NixOS package | Complex derivation, small user base relative to effort |
| Docker image | Not applicable for GUI terminal app |
| ARM64 cross-compilation | High complexity, defer to v1.2+ |
| Flathub submission | Requires upstream review process, separate from building |
| Auto-update mechanism | Each format has its own update path (apt, dnf, AppImage zsync) |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| META-01 | Phase 11 | Complete |
| META-02 | Phase 11 | Complete |
| META-03 | Phase 11 | Complete |
| META-04 | Phase 11 | Pending |
| META-05 | Phase 11 | Pending |
| BUILD-02 | Phase 11 | Pending |
| DEB-01 | Phase 12 | Pending |
| DEB-02 | Phase 12 | Pending |
| DEB-03 | Phase 12 | Pending |
| DEB-04 | Phase 12 | Pending |
| RPM-01 | Phase 12 | Pending |
| RPM-02 | Phase 12 | Pending |
| RPM-03 | Phase 12 | Pending |
| APPIMG-01 | Phase 13 | Pending |
| APPIMG-02 | Phase 13 | Pending |
| APPIMG-03 | Phase 13 | Pending |
| FLAT-01 | Phase 13 | Pending |
| FLAT-02 | Phase 13 | Pending |
| FLAT-03 | Phase 13 | Pending |
| BUILD-01 | Phase 14 | Pending |
| BUILD-03 | Phase 14 | Pending |
| CI-01 | Phase 14 | Pending |
| CI-02 | Phase 14 | Pending |
| CI-03 | Phase 14 | Pending |
| CI-04 | Phase 14 | Pending |

**Coverage:**
- v1.1 requirements: 25 total
- Mapped to phases: 25
- Unmapped: 0

---
*Requirements defined: 2026-03-29*
*Last updated: 2026-03-29 after roadmap creation*
