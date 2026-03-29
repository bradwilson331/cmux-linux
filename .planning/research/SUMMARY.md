# Project Research Summary

**Project:** cmux-linux v1.1 Packaging & Distribution
**Domain:** Linux packaging and CI for a multi-language (Rust/Zig/Go) GTK4 terminal multiplexer
**Researched:** 2026-03-28
**Confidence:** MEDIUM-HIGH

## Executive Summary

cmux-linux needs packaging across four Linux distribution formats (.deb, .rpm, AppImage, Flatpak) plus a self-hosted Gitea CI pipeline to automate builds and publishing. The project's multi-language build chain (Zig for libghostty, Rust for the app, Go for the remote daemon) makes this harder than typical single-language packaging. The recommended approach uses established Rust-native tools (cargo-deb, cargo-generate-rpm) for native packages, linuxdeploy with its GTK plugin for AppImage, and flatpak-builder for Flatpak -- progressing from simplest to most complex format.

The dominant risk is glibc baseline mismatch: building on a modern distro produces packages that crash on older ones. This must be addressed first by pinning the CI build environment to Ubuntu 22.04. The second major risk is GTK4 runtime resource bundling -- GTK4 apps need schemas, icon themes, and pixbuf loaders that go beyond shared library dependencies. Each format handles this differently (.deb/.rpm via package deps, AppImage via bundling, Flatpak via the GNOME runtime). Flatpak is the most complex format because its build sandbox blocks network access, requiring all three toolchains (Zig, Rust, Go) and their dependencies to be pre-declared or pre-built.

The project already has working desktop integration files, a GitHub Actions CI job to adapt, and a Gitea instance at `192.168.7.6:8418`. The path forward is: create shared metadata files first, then build packaging scripts from simplest (.deb) to hardest (Flatpak), and wire everything into Gitea CI last.

## Key Findings

### Recommended Stack

The stack leverages Rust-ecosystem packaging tools that read metadata directly from Cargo.toml, avoiding manual spec/control file maintenance. See [STACK.md](STACK.md) for full details.

**Core technologies:**
- **cargo-deb** (v3.6.x): .deb generation -- reads `[package.metadata.deb]` from Cargo.toml, handles asset layout and dependency declaration
- **cargo-generate-rpm** (v0.18.x+): .rpm generation -- same Cargo.toml metadata approach, actively maintained (replaces dead cargo-rpm)
- **linuxdeploy + linuxdeploy-plugin-gtk**: AppImage creation -- resolves ELF dependencies, bundles GTK4 schemas/loaders/icons
- **flatpak-builder** + org.gnome.Platform//47: Flatpak -- GNOME runtime provides GTK4 out of the box; Zig must be vendored as a tarball
- **act_runner** (v0.2.13+): Gitea Actions runner -- GitHub Actions YAML-compatible, self-hosted on build machine
- **readelf + ldd**: dependency detection -- maps shared libraries to package names for accurate dependency declarations

### Expected Features

See [FEATURES.md](FEATURES.md) for the complete landscape.

**Must have (table stakes):**
- Reverse-DNS app ID (`com.cmuxterm.cmux`) applied consistently across .desktop, metainfo, Flatpak manifest, and icon filenames
- AppStream metainfo XML (required for Flatpak, enables GNOME Software/KDE Discover display)
- Runtime dependency declarations per format (Debian and Fedora package names differ)
- All three binaries installed: `cmux-app` (GUI), `cmux` (CLI), `cmuxd-remote` (SSH daemon)
- PNG icon set at 48/128/256px generated from existing SVG

**Should have (differentiators):**
- Shell completions for `cmux` CLI (clap generates at build time)
- Man page for `cmux` CLI
- Automatic runtime dep detection via ldd/readelf
- Gitea package registry publishing (users can `apt install` / `dnf install` directly)

**Defer (v2+):**
- Multi-arch (aarch64) -- requires Zig/Rust/Go cross-compilation validation
- GPG package signing -- requires key infrastructure
- Flathub submission -- separate review process
- Snap package -- adds maintenance with no benefit over AppImage + Flatpak

### Architecture Approach

The build pipeline is strictly ordered: Zig first (libghostty.a), then Rust (cmux-app + cmux), Go in parallel (cmuxd-remote). All four packaging formats consume the same build artifacts except Flatpak, which rebuilds inside its sandbox. A new `packaging/` directory holds format-specific build scripts; Cargo.toml gains metadata sections for .deb and .rpm; `.gitea/workflows/` holds CI definitions. See [ARCHITECTURE.md](ARCHITECTURE.md) for full component diagrams.

**Major components:**
1. **Shared build stage** -- produces cmux-app, cmux, cmuxd-remote from three toolchains (Zig/Rust/Go)
2. **Per-format packaging scripts** -- `packaging/build-{deb,rpm,appimage,flatpak}.sh`, each self-contained and independently testable
3. **Desktop integration files** -- .desktop entry, metainfo XML, SVG + PNG icons, all in `resources/`
4. **Gitea CI pipeline** -- tag-triggered workflow that builds all formats and publishes to Gitea package registry

### Critical Pitfalls

See [PITFALLS.md](PITFALLS.md) for all 17 pitfalls with full prevention strategies.

1. **glibc baseline mismatch** -- Build on Ubuntu 22.04 to get lowest glibc floor; verify with `strings binary | grep GLIBC_`. Affects all formats.
2. **GTK4 runtime resources missing** -- Depend on `adwaita-icon-theme`, `gsettings-desktop-schemas` for .deb/.rpm; use linuxdeploy-plugin-gtk for AppImage; use GNOME runtime for Flatpak.
3. **Flatpak GPU/OpenGL broken in sandbox** -- Must add `--device=dri` to finish-args; test on both Mesa and NVIDIA. Showstopper for a GPU-accelerated terminal.
4. **Zig cache hash in build.rs breaks CI** -- Hardcoded `.zig-cache` hash path for simdutf.o changes per machine/version. Replace with dynamic lookup or stable copy path.
5. **Flatpak offline build blocks Zig/Go downloads** -- Pre-build all binaries outside the Flatpak sandbox and package them directly. Do not attempt to run `zig build` inside flatpak-builder.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: CI Infrastructure & Build Foundation
**Rationale:** Everything else depends on a working CI build. The Zig cache hash pitfall (#4) and stubs.o/glad.o issue (#17) must be fixed before any packaging script works in CI.
**Delivers:** Gitea act_runner registered and running; `.gitea/workflows/ci.yml` that builds all three binaries from a clean checkout on Ubuntu 22.04.
**Addresses:** Build reproducibility, glibc baseline pinning
**Avoids:** Zig cache hash pitfall (#4), glibc mismatch (#1), runner OOM (#13)

### Phase 2: Desktop Integration Metadata
**Rationale:** All four packaging formats need the same metadata files. Create them once before any packaging work begins.
**Delivers:** Reverse-DNS app ID applied; AppStream metainfo XML; PNG icon generation; shell completions and man page via clap; updated .desktop file.
**Addresses:** Table stakes features shared across all formats
**Avoids:** Missing desktop files pitfall (#11)

### Phase 3: .deb Package
**Rationale:** Largest user base (Ubuntu/Debian/Mint/Pop!_OS), lowest packaging complexity, validates the asset list and dependency declarations that .rpm reuses.
**Delivers:** Working `cmux-linux_*.deb` installable via `dpkg -i` with correct deps.
**Uses:** cargo-deb with `$auto` dep detection (dpkg-shlibdeps)
**Avoids:** GTK4 runtime resources pitfall (#2), missing Go binary (#6)

### Phase 4: .rpm Package
**Rationale:** Second-largest user base (Fedora/RHEL), nearly identical to .deb but with different package names for deps.
**Delivers:** Working `cmux-linux-*.rpm` installable via `dnf install`.
**Uses:** cargo-generate-rpm (NOT dead cargo-rpm)
**Avoids:** Dead cargo-rpm pitfall (#10)

### Phase 5: AppImage
**Rationale:** Portable "download and run" format. More complex than native packages due to library bundling, but simpler than Flatpak.
**Delivers:** Self-contained `cmux-x86_64.AppImage` that runs on Ubuntu 22.04+, Fedora, Arch.
**Uses:** linuxdeploy + linuxdeploy-plugin-gtk with `DEPLOY_GTK_VERSION=4`
**Avoids:** glibc bundling pitfall (#5), missing GL drivers (#15), missing GTK4 resources (#2)

### Phase 6: Flatpak
**Rationale:** Most complex format due to sandbox constraints. Pre-build strategy (build all binaries outside sandbox) sidesteps the Zig/Go offline build problem. Saved for last because lessons from earlier formats inform the manifest.
**Delivers:** Working Flatpak installable via `flatpak install`.
**Uses:** flatpak-builder, org.gnome.Platform//47, pre-built binaries
**Avoids:** GPU sandbox pitfall (#3), offline build pitfall (#9), theme mismatch (#12)

### Phase 7: Gitea CI Publishing & Release Automation
**Rationale:** Automates all packaging into a tag-triggered release pipeline. Requires all formats working first.
**Delivers:** Push a tag, get .deb/.rpm/AppImage published to Gitea package registry and release assets.
**Addresses:** Gitea package registry publishing, release automation
**Avoids:** Token auth pitfall (#8), concurrency group pitfall (#7), silent syntax differences (#16)

### Phase Ordering Rationale

- CI infrastructure must come first because the Zig cache hash bug blocks all clean-checkout builds.
- Metadata files must precede packaging because every format references them.
- .deb before .rpm because cargo-deb's `$auto` dep detection validates the dependency list that .rpm must declare manually.
- AppImage before Flatpak because AppImage is complex (library bundling) but debuggable (just run it); Flatpak is complex AND opaque (sandbox).
- CI publishing last because it orchestrates everything -- all formats must work standalone first.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 5 (AppImage):** linuxdeploy-plugin-gtk's GTK4 support is less battle-tested than GTK3. May need manual intervention for resource bundling. Research the Gaphor GTK4 AppImage migration as a reference.
- **Phase 6 (Flatpak):** Pre-build strategy needs validation. The manifest must correctly handle three pre-built binaries from different toolchains. GPU access on NVIDIA requires testing.

Phases with standard patterns (skip research-phase):
- **Phase 2 (Metadata):** Freedesktop specs are stable and well-documented. Follow Ghostty's metainfo.xml as template.
- **Phase 3 (.deb):** cargo-deb is mature and well-documented. Standard pattern.
- **Phase 4 (.rpm):** cargo-generate-rpm follows same pattern as cargo-deb.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All tools are actively maintained, widely used in Rust ecosystem. Versions verified on crates.io/GitHub. |
| Features | MEDIUM-HIGH | Feature landscape well-mapped from codebase inspection + packaging standards. GTK4-specific AppImage behavior less certain. |
| Architecture | HIGH | Build chain is deterministic. File layout follows freedesktop conventions. Gitea Actions syntax is documented. |
| Pitfalls | HIGH | Pitfalls sourced from official docs, known issues, and codebase analysis. glibc and GTK4 resource issues are well-documented failure modes. |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **linuxdeploy-plugin-gtk GTK4 support:** Claimed but not verified first-hand. Test early in Phase 5 with a minimal GTK4 binary before packaging the full app.
- **Zig inside Flatpak sandbox:** If pre-build strategy fails (e.g., ABI mismatch between host-built libghostty.a and Flatpak runtime), will need to vendor Zig tarball in the manifest. Test in Phase 6.
- **mlugg/setup-zig on Gitea:** This GitHub Action may not work on Gitea. If it fails, use manual Zig tarball download (pattern already in ARCHITECTURE.md workflow example).
- **Gitea package registry API:** Upload endpoints documented but not tested against the specific instance at 192.168.7.6:8418. Validate in Phase 7.
- **NVIDIA GPU in Flatpak:** Driver version matching between host and runtime is fragile. May need `org.freedesktop.Platform.GL.nvidia-*` extension. Requires testing on an NVIDIA system.

## Sources

### Primary (HIGH confidence)
- [cargo-deb](https://github.com/kornelski/cargo-deb) -- v3.6.3, Debian packaging for Rust
- [cargo-generate-rpm](https://crates.io/crates/cargo-generate-rpm) -- v0.18.x+, RPM packaging for Rust
- [Flatpak manifest docs](https://docs.flatpak.org/en/latest/manifests.html) -- build specification
- [Gitea Actions docs](https://docs.gitea.com/usage/actions/overview) -- CI system
- [Gitea Actions comparison](https://docs.gitea.com/usage/actions/comparison) -- syntax differences from GitHub
- [Freedesktop desktop entry spec](https://specifications.freedesktop.org/desktop-entry/desktop-entry-spec-latest.html)
- [Freedesktop icon theme spec](https://specifications.freedesktop.org/icon-theme/latest/)

### Secondary (MEDIUM confidence)
- [linuxdeploy-plugin-gtk](https://github.com/linuxdeploy/linuxdeploy-plugin-gtk) -- GTK4 auto-detection claimed
- [How to Flatpak a Rust application](https://belmoussaoui.com/blog/8-how-to-flatpak-a-rust-application/) -- community guide
- [RustDesk Linux Packaging](https://deepwiki.com/rustdesk/rustdesk/7.4-platform-packaging) -- reference implementation

### Tertiary (LOW confidence)
- [Gaphor AppImage GTK4 migration](https://github.com/gaphor/gaphor/pull/1857) -- only known GTK4 AppImage reference
- mlugg/setup-zig on Gitea Actions -- untested, may need replacement

---
*Research completed: 2026-03-28*
*Ready for roadmap: yes*
