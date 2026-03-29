# Feature Landscape: Linux Packaging & Distribution

**Domain:** Linux package distribution for a Rust/GTK4 terminal multiplexer
**Researched:** 2026-03-28
**Milestone:** v1.1 Linux Packaging & Distribution
**Confidence:** MEDIUM-HIGH -- direct codebase inspection + web research of packaging tools and formats

---

## Context: What Already Exists

The v1.0 app ships two Rust binaries (`cmux-app` GUI, `cmux` CLI), a Go remote daemon (`cmuxd-remote`), and has:
- `resources/cmux.desktop` -- basic desktop entry (non-reverse-DNS naming)
- `resources/cmux.svg` -- SVG icon
- `.github/workflows/ci.yml` -- GitHub Actions with `linux-build` job (builds libghostty via Zig, runs clippy + cargo build + cargo test)
- No metainfo XML, no PNG icon set, no packaging configuration in Cargo.toml

Build toolchain: Rust stable + Zig 0.15.2 (for libghostty) + Go (for cmuxd-remote). System deps: `libgtk-4-dev libclang-dev libfontconfig1-dev libfreetype6-dev libonig-dev libgl-dev`.

---

## Table Stakes

Features users expect from any properly packaged Linux application. Missing = package feels broken or amateurish.

### Shared Across All Formats

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Freedesktop `.desktop` file with reverse-DNS ID | App appears in launchers (GNOME, KDE, etc.) and is Flatpak-compatible | Low | Existing `resources/cmux.desktop` needs rename to `com.cmux.terminal.desktop`. Must match metainfo ID |
| SVG icon at `hicolor/scalable/apps/` | Icon renders at any size in any DE | Low | Already exists at `resources/cmux.svg`. Install to `share/icons/hicolor/scalable/apps/com.cmux.terminal.svg` |
| PNG icons at 48px, 128px, 256px | Fallback for icon themes that prefer raster; required by some AppImage tools | Low | Generate from SVG via `rsvg-convert` at build time. 16/32/48/128/256 are the standard sizes |
| AppStream metainfo XML | Software centers (GNOME Software, KDE Discover) display app info; Flatpak requires it | Medium | Does not exist yet. Model after Ghostty's `com.mitchellh.ghostty.metainfo.xml` in the submodule |
| Runtime dependency declarations | Package manager installs GTK4 and other libs automatically | Medium | Must be mapped per-format (Debian vs Fedora package names differ). Core deps: GTK4, fontconfig, freetype, oniguruma, OpenGL |
| Both binaries installed and on PATH | `cmux-app` (GUI) and `cmux` (CLI) both available | Low | Two `[[bin]]` targets in Cargo.toml; both must appear in package assets at `/usr/bin/` |
| `cmuxd-remote` bundled | SSH workspace feature needs the Go remote daemon | Medium | Pre-built Go binary. Bundle at `/usr/lib/cmux/cmuxd-remote` or `/usr/libexec/cmux/cmuxd-remote` |
| Correct file permissions | Binaries 0755, data files 0644 | Low | All packaging tools handle this by default; just verify |
| Clean uninstall | Removing package removes all installed files | Low | Native to .deb/.rpm package managers. AppImage is a single file. Flatpak has `flatpak uninstall` |

### .deb Package Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `Depends:` in control file | apt installs runtime deps automatically | Medium | `libgtk-4-1`, `libfontconfig1`, `libfreetype6`, `libonig5`, `libgl1`. cargo-deb reads `[package.metadata.deb]` from Cargo.toml |
| `Section: x11` | Proper categorization | Low | Standard for terminal emulators |
| `Architecture: amd64` | Platform targeting | Low | amd64 only for now |
| `Maintainer` and `Description` fields | Required by Debian policy, shown in `apt show` | Low | |
| Post-install `update-desktop-database` | .desktop file registered with system | Low | Triggers via `triggers` file or postinst script in cargo-deb |

### .rpm Package Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `Requires:` runtime dependencies | dnf installs deps automatically | Medium | `gtk4`, `fontconfig`, `freetype`, `oniguruma`, `mesa-libGL` -- names differ from Debian! |
| `License:` field | Required by RPM policy | Low | |
| Correct `%files` paths | Files land where expected | Low | `/usr/bin/`, `/usr/share/applications/`, `/usr/share/icons/`, `/usr/share/metainfo/` |
| `Release` and `Epoch` | RPM versioning fields | Low | cargo-generate-rpm handles basic versioning |

### AppImage Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Self-contained executable | Runs on any distro without installation | **High** | Must bundle GTK4 libs, GLib schemas, Pango modules, icon themes. GTK4 has complex runtime deps that linuxdeploy may miss |
| `.desktop` and icon embedded in AppDir | Desktop integration if user opts in | Low | linuxdeploy copies these from AppDir structure |
| `AppRun` entry point | Standard AppImage launch mechanism | Low | linuxdeploy generates this |
| GTK4 resources via linuxdeploy-plugin-gtk | Schemas, loaders, themes bundled | **High** | Set `DEPLOY_GTK_VERSION=4`. The plugin's GTK4 support is less proven than GTK3 -- this is the highest-risk item for AppImage |

### Flatpak Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| YAML/JSON manifest | Defines runtime, SDK, build instructions, permissions | Medium | Use `org.gnome.Platform//47` runtime (includes GTK4). SDK: `org.gnome.Sdk//47` + `org.freedesktop.Sdk.Extension.rust-stable` |
| Reverse-DNS app ID matching all files | ID in manifest = .desktop filename = metainfo filename = icon name | Low | Use `com.cmux.terminal` consistently |
| Sandbox permissions | Filesystem, network, display, GPU access | Medium | Minimum: `--socket=wayland`, `--socket=fallback-x11`, `--share=network` (SSH), `--share=ipc`, `--device=dri` (GPU), `--talk-name=org.freedesktop.Notifications`, `--filesystem=home` (config, sockets, shells) |
| Desktop file and metainfo at `/app/share/` | Flatpak requires `/app/` prefix paths, not `/usr/` | Low | Build system must install to correct prefix |
| Content rating | Required for Flathub submission | Low | `<content_rating type="oars-1.1" />` in metainfo |
| Zig toolchain in build sandbox | libghostty requires Zig 0.15.2 at build time | **High** | Must either: (a) add Zig as a build module in the Flatpak manifest, or (b) pre-build libghostty outside the sandbox and bundle the .a file. Option (b) is simpler |

---

## Differentiators

Features that set a good Linux package apart from a minimal one.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Shell completions for `cmux` CLI | Tab completion for 34+ subcommands in bash/zsh/fish | Low | clap generates these at build time. Install to standard completion dirs |
| Man page for `cmux` | `man cmux` works out of the box | Low | clap can generate man pages. Install to `/usr/share/man/man1/cmux.1.gz` |
| Automatic runtime dep detection via `ldd` | Build script analyzes binary, maps `.so` names to package names | Medium | Listed in PROJECT.md as a target feature. Reduces manual dependency tracking errors when deps change |
| AppImage update information | Delta updates via `zsync` or `appimageupdatetool` | Low | Embed update URL in AppImage for self-update. Minor effort, significant user convenience |
| Flatpak screenshots in metainfo | GNOME Software shows app screenshots; looks polished | Low | Add `<screenshots>` section to metainfo XML with hosted image URLs |
| GPG-signed packages | Verify package integrity and authenticity | Medium | cargo-generate-rpm supports `--signing_key`. dpkg-sig for .deb. Requires GPG key management |
| Changelog in package metadata | Users see what changed per version | Low | .deb supports `debian/changelog`. RPM uses `%changelog` section |
| D-Bus service file | Proper desktop activation for notifications | Medium | Ghostty ships one. Only needed if notification actions require app activation |
| Gitea package registry publishing | Users can `apt install` or `dnf install` from Gitea registry directly | Medium | Gitea has built-in .deb and .rpm package registries. Upload via API in CI |
| Multi-arch (amd64 + arm64) | ARM Linux users (Raspberry Pi, Asahi Linux) | **High** | Requires cross-compilation of both Rust and Zig. Defer to later milestone |

---

## Anti-Features

Features to explicitly NOT build in this milestone.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Snap package | Canonical-specific, contested in community, fourth format adds maintenance burden | Focus on Flatpak as the sandboxed universal format |
| Custom auto-update daemon | Fights the package manager; users update via `apt upgrade` / `dnf upgrade` / Flatpak | Publish new versions to registries; let package managers handle updates |
| Homebrew tap for Linux | Niche, redundant with native packages, complex to maintain | AppImage covers "just download and run" |
| Separate library package (libcmux-dev) | No external consumers of cmux internals | Ship everything in one package per format |
| Nix flake / Guix package | Maintainer-intensive, small overlap with target users | Let community contribute; don't maintain officially |
| Complex post-install configuration | Terminal apps should work with zero config | Ship sensible defaults; document customization |
| Flathub submission in this milestone | Flathub has its own review process and requirements beyond just having a manifest | Build and self-host the Flatpak first; submit to Flathub as a separate effort |
| Wayland-only or X11-only builds | Fragments user base unnecessarily | Build with both backends (already done: `--gtk-x11=true --gtk-wayland=true`) |

---

## Feature Dependencies

```
resources/cmux.svg
    |--[rsvg-convert]--> PNG icon set (48, 128, 256px)

Reverse-DNS app ID decision ("com.cmux.terminal")
    |--> .desktop file rename
    |--> metainfo XML filename
    |--> Flatpak manifest app-id
    |--> icon filename in hicolor

AppStream metainfo XML
    |--> required by Flatpak
    |--> recommended for .deb/.rpm (shown in software centers)
    |--> optional: <screenshots> for GNOME Software

Runtime dep list (from ldd analysis)
    |--> cargo-deb Depends: field (Debian package names)
    |--> cargo-generate-rpm Requires: field (Fedora package names)
    |--> AppImage bundling decisions

Local build scripts (one per format + "build all")
    |--> Gitea CI workflows call these scripts
    |--> Scripts must work on a fresh Ubuntu/Fedora with deps installed

cargo build --release
    |--> .deb (cargo-deb reads binary from target/release/)
    |--> .rpm (cargo-generate-rpm reads binary from target/release/)
    |--> AppImage (linuxdeploy packages binary + libs)
    |--> Flatpak (builds inside sandbox, or uses pre-built binary)

libghostty.a (Zig build)
    |--> Required before cargo build (linked via build.rs/cc)
    |--> Flatpak: must either build Zig in sandbox or pre-build outside

cmuxd-remote (Go build)
    |--> Bundle as additional asset in all package formats
    |--> Separate build step from Rust binary

Shell completions + man page (clap generate)
    |--> cargo build --release must generate these
    |--> Install as additional assets in all formats

Gitea CI workflows
    |--> Require: self-hosted runner with Rust, Zig 0.15.2, Go, GTK4 dev libs
    |--> Require: PAT token for package registry publishing
    |--> Call same scripts as local builds
```

---

## MVP Recommendation

Build in this order, because each step builds on the previous:

### Phase 1: Shared Metadata Files
Create once, used by all formats:
1. Choose and apply reverse-DNS app ID (`com.cmux.terminal`)
2. Rename `.desktop` file and update contents
3. Create AppStream metainfo XML (model on Ghostty's)
4. Generate PNG icon set from SVG (build script)
5. Add shell completions and man page generation via clap

### Phase 2: .deb Package (cargo-deb)
Lowest complexity native format, largest user base (Ubuntu/Debian/Pop!_OS/Mint):
1. Add `[package.metadata.deb]` section to Cargo.toml with assets, depends, section
2. Write build script: `scripts/build-deb.sh` (builds libghostty, cargo build --release, cargo deb)
3. Test on Ubuntu 24.04

### Phase 3: .rpm Package (cargo-generate-rpm)
Second-largest user base (Fedora/RHEL):
1. Add `[package.metadata.generate-rpm]` section to Cargo.toml with assets, requires
2. Write build script: `scripts/build-rpm.sh`
3. Test on Fedora 41

### Phase 4: AppImage (linuxdeploy)
Portable "just download and run" -- already partially existed in v1.0 CI:
1. Write build script: `scripts/build-appimage.sh` (linuxdeploy + linuxdeploy-plugin-gtk with `DEPLOY_GTK_VERSION=4`)
2. Bundle GTK4 runtime resources
3. Test on multiple distros (Ubuntu 22.04, Fedora, Arch)

### Phase 5: Flatpak Manifest
Most complex due to sandbox and Zig toolchain:
1. Write Flatpak manifest (`flatpak/com.cmux.terminal.yml`)
2. Strategy: pre-build libghostty.a outside sandbox, include as source in manifest
3. Configure sandbox permissions
4. Test with `flatpak-builder --user --install`

### Phase 6: Gitea CI + Publishing
Automates everything above:
1. Port build scripts into `.gitea/workflows/` YAML (near-identical to GitHub Actions syntax)
2. Add package registry publishing steps (Gitea API)
3. Add release asset upload on tag
4. Configure self-hosted runner with full toolchain

### Defer

| Feature | Reason |
|---------|--------|
| Multi-arch (arm64) | Requires Zig cross-compilation validation; separate effort |
| GPG package signing | Requires key infrastructure; add after packages work |
| Flathub submission | Separate review process; self-host first |
| D-Bus service file | Only needed if notification actions require activation |

---

## Gitea Actions vs GitHub Actions: Feature Gaps

| Capability | GitHub Actions | Gitea Actions | Impact on Packaging |
|------------|---------------|---------------|---------------------|
| Workflow YAML syntax | Full spec | Nearly identical, minor gaps | **LOW** -- packaging workflows are simple |
| Runner hosting | GitHub-hosted or self-hosted | Self-hosted only (`act_runner`) | **MEDIUM** -- must provision Linux runner with Rust, Zig 0.15.2, Go, GTK4 dev libs, linuxdeploy, flatpak-builder |
| Package registry | ghcr.io, npm, etc. | Built-in generic + .deb + .rpm registries | **LOW** -- Gitea registries work for this use case |
| Default token permissions | `GITHUB_TOKEN` has broad defaults | `GITEA_TOKEN` is read-only | **MEDIUM** -- need PAT for publishing packages |
| Concurrency groups | Supported | Ignored (not implemented) | **LOW** -- packaging builds are infrequent |
| Caching (`actions/cache`) | Full support | Compatible | **LOW** |
| Artifact upload | `actions/upload-artifact` | Compatible | **LOW** |
| Matrix builds | Full support | Supported | **LOW** |
| `uses:` third-party actions | Any GitHub action | Most work; some may need adjustment | **LOW** -- mostly standard actions (checkout, setup-go, etc.) |

**Key operational requirement:** The Gitea runner must be a persistent Linux machine (or VM) with the full build toolchain pre-installed. Unlike GitHub Actions where you get a fresh ubuntu-latest with common tools, Gitea runners start bare.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| .deb packaging (cargo-deb) | HIGH | Well-documented Rust tool, directly inspected on crates.io and GitHub |
| .rpm packaging (cargo-generate-rpm) | HIGH | Active project, configurable via Cargo.toml metadata |
| AppImage / linuxdeploy | MEDIUM | GTK4 support in linuxdeploy-plugin-gtk is less documented than GTK3. May need manual intervention for GTK4 resource bundling |
| Flatpak | MEDIUM | Well-documented format, but Zig build toolchain inside sandbox is uncharted territory. Pre-building libghostty is the safer path |
| Gitea Actions compatibility | HIGH | Official docs confirm near-identical syntax. Main gap is runner provisioning |
| Freedesktop metadata standards | HIGH | Stable specs, well-documented on ArchWiki and freedesktop.org |
| Runtime dependency names (Debian vs Fedora) | MEDIUM | Verified common ones via package search; edge cases may need ldd-based discovery |

---

## Sources

- [Flatpak Requirements & Conventions](https://docs.flatpak.org/en/latest/conventions.html)
- [Flathub MetaInfo Guidelines](https://docs.flathub.org/docs/for-app-authors/metainfo-guidelines)
- [Flatpak Manifests Documentation](https://docs.flatpak.org/en/latest/manifests.html)
- [Flatpak Available Runtimes](https://docs.flatpak.org/en/latest/available-runtimes.html)
- [cargo-deb on GitHub](https://github.com/kornelski/cargo-deb)
- [cargo-generate-rpm on crates.io](https://crates.io/crates/cargo-generate-rpm)
- [linuxdeploy User Guide](https://docs.appimage.org/packaging-guide/from-source/linuxdeploy-user-guide.html)
- [linuxdeploy-plugin-gtk](https://github.com/linuxdeploy/linuxdeploy-plugin-gtk)
- [AppImage Best Practices](https://docs.appimage.org/reference/best-practices.html)
- [Gitea Actions vs GitHub Actions](https://docs.gitea.com/usage/actions/comparison)
- [Gitea Actions Overview](https://docs.gitea.com/usage/actions/overview)
- [Freedesktop Desktop Entries - ArchWiki](https://wiki.archlinux.org/title/Desktop_entries)
- [Fedora RPM Packaging Guide](https://developer.fedoraproject.org/deployment/rpm/about.html)
- [Comprehensive Guide to .deb and .rpm for Rust](https://dev.to/mbayoun95/comprehensive-guide-to-generating-deb-and-rpm-packages-for-rust-applications-41h7)
- [How to Flatpak a Rust Application](https://belmoussaoui.com/blog/8-how-to-flatpak-a-rust-application/)
- Ghostty's packaging files: `ghostty/dist/linux/`, `ghostty/zig-out/share/metainfo/com.mitchellh.ghostty.metainfo.xml` (direct inspection)
- Existing project files: `resources/cmux.desktop`, `resources/cmux.svg`, `Cargo.toml`, `.github/workflows/ci.yml` (direct inspection)
