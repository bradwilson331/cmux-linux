# Technology Stack: Linux Packaging & Distribution

**Project:** cmux-linux v1.1 Packaging & Distribution
**Researched:** 2026-03-28
**Scope:** New tools for .deb, .rpm, AppImage, Flatpak packaging and Gitea CI. Does NOT re-research existing Rust/GTK4/Ghostty stack (validated in v1.0).

---

## Recommended Stack

### .deb Packaging (Debian/Ubuntu)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| cargo-deb | 3.6.x | Generate .deb from Cargo.toml metadata | Native Rust integration -- reads `[package.metadata.deb]` from Cargo.toml, handles asset layout, dependency declaration, and DEBIAN/control generation in one step. Actively maintained (3.6.3 released ~Feb 2026). Standard tool for Rust binary .deb packaging. |
| dpkg-deb | system | Underlying .deb assembly (used by cargo-deb internally) | Already present on Debian/Ubuntu build hosts. cargo-deb shells out to it. No separate install needed. |

**Why cargo-deb over raw dpkg-deb:** Eliminates manual DEBIAN/control, postinst/postrm, and directory structure creation. Reads version, description, license from Cargo.toml. RustDesk, Alacritty, and other Rust terminal apps use this approach.

**Why NOT debcargo:** debcargo packages Rust crate libraries into Debian archives (one .deb per crate dependency). Wrong tool for packaging a binary application.

### .rpm Packaging (Fedora/RHEL/openSUSE)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| cargo-generate-rpm | 0.18.x+ | Generate .rpm from Cargo.toml metadata | Same philosophy as cargo-deb: reads `[package.metadata.generate-rpm]` from Cargo.toml, produces binary RPM without a .spec file or rpmbuild toolchain. Single command after `cargo build --release`. |

**Why NOT rpmbuild:** Requires a full .spec file, mock chroot setup, and rpmbuild installation. Heavyweight for self-distributed binary packages.

**Why NOT cargo-rpm:** End-of-life since 2022. Its own README redirects to cargo-generate-rpm.

**Why NOT rust2rpm:** Generates Fedora .spec files for inclusion in Fedora's official package repos. Useful for distro maintainers, wrong tool for self-distribution.

### AppImage (Portable/Universal)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| linuxdeploy | continuous | AppDir creation, shared library bundling | Recursively resolves and copies shared libraries into the AppDir. Plugin system handles GTK-specific resources. Official AppImage project recommendation. |
| linuxdeploy-plugin-gtk | latest | GTK resource bundling | Auto-detects GTK version (supports 2/3/4), bundles GLib schemas, GdkPixbuf loaders, icon themes, locale data. Without this plugin, GTK4 apps fail at runtime inside AppImage due to missing schemas/loaders. |
| appimagetool | continuous | Final AppImage assembly from AppDir | Takes the prepared AppDir and produces the self-extracting .AppImage binary. linuxdeploy calls it internally via `--output appimage`. |

**Pipeline:**
```
cargo build --release
  -> linuxdeploy --appdir AppDir --plugin gtk \
       --executable target/release/cmux-app \
       --desktop-file resources/cmux.desktop \
       --icon-file resources/cmux.svg \
       --output appimage
  -> cmux-x86_64.AppImage
```

**Why linuxdeploy over appimage-builder:** linuxdeploy is the official recommended tool from the AppImage project. appimage-builder uses Docker + apt-based dependency resolution -- unnecessary complexity for a project that already builds natively. linuxdeploy directly inspects the binary's ELF dependencies.

**GTK4 note (MEDIUM confidence):** linuxdeploy-plugin-gtk claims GTK4 auto-detection. This is well-tested for GTK3; GTK4 support is newer. May need testing and minor patching of the plugin script if it misses GTK4-specific paths.

### Flatpak

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| flatpak-builder | system | Build Flatpak from JSON/YAML manifest | Standard tool. Builds in sandboxed environment, handles source fetching, caching. |
| org.gnome.Platform | 47 | Runtime (provides GTK4, glib, pango, etc.) | GNOME 47 is current stable runtime with GTK4. GNOME 46 is EOL (April 2025). Version 48 exists but 47 has broader install base. |
| org.gnome.Sdk | 47 | Build SDK (compiler toolchain + platform libs) | Paired with Platform 47. |
| org.freedesktop.Sdk.Extension.rust-stable | 24.08 | Rust toolchain for Flatpak sandbox builds | Provides cargo/rustc inside the Flatpak build environment. Required because Flatpak builds happen in isolation from host. |

**Why org.gnome.Platform over org.freedesktop.Platform:** The Freedesktop runtime does NOT include GTK4. The GNOME runtime bundles GTK4, glib, pango, and all libraries cmux needs at runtime. Using Freedesktop would require building GTK4 from source inside the manifest -- unnecessary complexity.

**Manifest:** A single `com.cmuxterm.cmux.json` file that:
1. Sets `runtime: org.gnome.Platform//47` and `sdk: org.gnome.Sdk//47`
2. Adds `org.freedesktop.Sdk.Extension.rust-stable` SDK extension
3. Builds libghostty.a via Zig (vendored in sources)
4. Builds cmux-app and cmux CLI via cargo
5. Installs binaries, .desktop file, icon, and AppStream metainfo

**Zig in Flatpak sandbox:** Zig must be vendored as a source/archive in the manifest since there is no Flatpak SDK extension for Zig. Download the Zig 0.15.2 tarball as a manifest source and extract it during build.

### Dependency Detection

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| readelf | system (binutils) | Extract NEEDED shared libraries from ELF binary | Safe on any binary (unlike ldd which executes it). `readelf -d <binary> \| grep NEEDED` gets direct deps. |
| ldd | system (glibc) | Resolve full transitive dependency tree with paths | Shows all libraries including transitive deps. ONLY use on trusted binaries (own build output). |
| dpkg -S | system | Map .so files to deb package names | After ldd identifies libfoo.so.1, `dpkg -S libfoo.so.1` returns the owning deb package. |
| rpm -qf | system | Map .so files to rpm package names | `rpm -qf /usr/lib64/libfoo.so.1` returns the owning RPM package. |

**Recommended approach:** A `scripts/detect-deps.sh` script that:
1. Runs `readelf -d target/release/cmux-app | grep NEEDED` for direct .so deps
2. Uses `ldd target/release/cmux-app` to resolve full paths
3. Maps each .so to its owning package via `dpkg -S` (deb) or `rpm -qf` (rpm)
4. Outputs a dependency list for packaging metadata

**Why readelf over objdump:** readelf is focused on ELF inspection; objdump is heavier, designed for disassembly.

**Critical caveat:** Auto-detection misses dlopen'd libraries (GTK4 modules, GL drivers, GdkPixbuf loaders). The detected list is a starting point; the final dependency list in Cargo.toml metadata must be human-reviewed and include known dlopen'd deps like `libgtk-4-1`, `libgl1`, `libfontconfig1`.

### Gitea CI

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| act_runner | 0.2.13+ | Gitea Actions job execution | Official Gitea runner. Runs jobs in Docker containers. GitHub Actions YAML-compatible -- can adapt the existing `linux-build` job from `.github/workflows/ci.yml`. |
| Docker | system | Container runtime for CI jobs | act_runner executes each job in a Docker container. Required for reproducible builds. |

**Gitea instance:** `http://192.168.7.6:8418/` (from PROJECT.md)

**Compatibility with existing CI:**
- Workflow files go in `.gitea/workflows/` (not `.github/`)
- Accepts `GITHUB_` context prefix for compatibility (also supports `GITEA_`)
- `actions/checkout`, `actions/cache`, `dtolnay/rust-toolchain` work unchanged
- `mlugg/setup-zig` should work but needs testing (LOW confidence)
- **Limitation:** Concurrency groups are ignored -- no built-in queue management. Not a blocker.
- **Limitation:** No WarpBuild-equivalent runners -- all jobs on self-hosted act_runner

**Runner setup:**
1. Install act_runner binary on build host (or dedicated machine)
2. Register with `./act_runner register --instance http://192.168.7.6:8418/ --token <TOKEN>`
3. Mount Docker socket: `/var/run/docker.sock`
4. Disk: ~10GB for Zig cache + Rust target dir + Docker images
5. Run: `./act_runner daemon` (or systemd unit)

### Validation Utilities

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| file | system | Verify ELF binary type (x86_64, aarch64) | Smoke test in CI after build |
| strip | system (binutils) | Remove debug symbols from release binary | Before packaging -- reduces binary size 50-80% |
| desktop-file-validate | system (desktop-file-utils) | Validate .desktop file syntax | CI lint step |
| appstreamcli validate | system (appstream) | Validate AppStream metainfo.xml | Required for Flatpak/Flathub submission |

---

## Existing Assets (Reuse, Do Not Recreate)

| Asset | Path | Status |
|-------|------|--------|
| Desktop entry | `resources/cmux.desktop` | Exists, well-formed |
| SVG icon | `resources/cmux.svg` | Exists |
| CI linux-build job | `.github/workflows/ci.yml` (lines 77-114) | Working build pipeline to adapt for Gitea |
| System deps list | `scripts/setup-linux.sh` | apt/dnf package names for build deps |
| Two binaries | `cmux-app` (GUI) + `cmux` (CLI) | Both defined in Cargo.toml `[[bin]]` |

---

## New Files Needed

| File | Purpose |
|------|---------|
| `packaging/build-deb.sh` | Shell script: cargo build + cargo deb |
| `packaging/build-rpm.sh` | Shell script: cargo build + cargo generate-rpm |
| `packaging/build-appimage.sh` | Shell script: cargo build + linuxdeploy pipeline |
| `packaging/build-flatpak.sh` | Shell script: flatpak-builder wrapper |
| `packaging/build-all.sh` | Orchestrator: calls all four |
| `packaging/com.cmuxterm.cmux.json` | Flatpak manifest |
| `packaging/com.cmuxterm.cmux.metainfo.xml` | AppStream metadata (for Flatpak + deb) |
| `.gitea/workflows/release.yml` | Gitea CI: build all packages on tag push |
| `scripts/detect-deps.sh` | Dependency detection utility |

---

## Cargo.toml Metadata Additions

```toml
[package.metadata.deb]
maintainer = "cmux maintainers"
copyright = "2025-2026 Manaflow"
license-file = ["LICENSE", "0"]
depends = "libgtk-4-1 (>= 4.12), libglib2.0-0 (>= 2.76), libc6 (>= 2.35), libfontconfig1, libfreetype6, libgl1"
section = "utils"
priority = "optional"
assets = [
    ["target/release/cmux-app", "usr/bin/", "755"],
    ["target/release/cmux", "usr/bin/", "755"],
    ["resources/cmux.desktop", "usr/share/applications/", "644"],
    ["resources/cmux.svg", "usr/share/icons/hicolor/scalable/apps/", "644"],
]

[package.metadata.generate-rpm]
assets = [
    { source = "target/release/cmux-app", dest = "/usr/bin/cmux-app", mode = "755" },
    { source = "target/release/cmux", dest = "/usr/bin/cmux", mode = "755" },
    { source = "resources/cmux.desktop", dest = "/usr/share/applications/cmux.desktop", mode = "644" },
    { source = "resources/cmux.svg", dest = "/usr/share/icons/hicolor/scalable/apps/cmux.svg", mode = "644" },
]

[package.metadata.generate-rpm.requires]
gtk4 = ">= 4.12"
fontconfig = "*"
freetype = "*"
```

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| .deb | cargo-deb | Raw dpkg-deb | Unnecessary boilerplate for a Cargo project |
| .deb | cargo-deb | debcargo | debcargo is for library crates, not app binaries |
| .rpm | cargo-generate-rpm | rpmbuild + .spec | Spec file maintenance overhead for self-distributed binary |
| .rpm | cargo-generate-rpm | cargo-rpm | EOL since 2022 |
| .rpm | cargo-generate-rpm | rust2rpm | For Fedora repo maintainers, not self-distribution |
| AppImage | linuxdeploy + plugin-gtk | appimage-builder | Extra Docker/apt layer; linuxdeploy is official recommendation |
| Flatpak runtime | org.gnome.Platform 47 | org.freedesktop.Platform | No GTK4 in freedesktop runtime; would need to build GTK4 from source |
| Flatpak runtime | org.gnome.Platform 47 | org.gnome.Platform 48 | 48 exists but 47 has broader install base today |
| Dep detection | readelf + ldd + manual review | Fully automated | dlopen'd deps (GL, GTK modules) need human review |
| CI | Gitea Actions (act_runner) | Jenkins | Gitea Actions is built-in, YAML-compatible with existing GitHub Actions |
| CI | Gitea Actions (act_runner) | Drone CI | Another option but Gitea Actions is native, no extra service to run |
| Universal pkg | NOT adding Snap | snapd/snapcraft | Requires snapd daemon, Canonical-centric, poor GTK4 theming, adds maintenance with no benefit over AppImage + Flatpak |

---

## Installation

```bash
# === Packaging tools (install on build host) ===

# .deb and .rpm generators
cargo install cargo-deb
cargo install cargo-generate-rpm

# === AppImage tools (download pre-built binaries) ===
mkdir -p tools/
wget -O tools/linuxdeploy-x86_64.AppImage \
  https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
wget -O tools/linuxdeploy-plugin-gtk.sh \
  https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh
wget -O tools/appimagetool-x86_64.AppImage \
  https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x tools/linuxdeploy-x86_64.AppImage tools/linuxdeploy-plugin-gtk.sh tools/appimagetool-x86_64.AppImage

# === Flatpak tools ===
sudo apt-get install -y flatpak flatpak-builder
flatpak install -y flathub org.gnome.Platform//47 org.gnome.Sdk//47
flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//24.08

# === Validation tools ===
sudo apt-get install -y desktop-file-utils appstream

# === Dependency detection (already on most Linux systems) ===
# readelf (part of binutils), ldd (part of glibc) -- no install needed
```

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| cargo-deb | HIGH | Widely used, verified v3.6.3 on crates.io, standard for Rust .deb |
| cargo-generate-rpm | HIGH | Standard Rust RPM tool, actively maintained |
| linuxdeploy + GTK plugin | MEDIUM | GTK4 auto-detection claimed but not verified first-hand; GTK3 battle-tested, GTK4 newer |
| Flatpak manifest | MEDIUM | Pattern well-documented, but Zig build inside sandbox needs testing |
| Gitea Actions compat | MEDIUM | YAML compat confirmed by docs, but specific actions (mlugg/setup-zig) need testing |
| readelf/ldd dep detection | HIGH | Standard Linux tooling, straightforward |
| act_runner setup | MEDIUM | Docs clear, but local instance specifics (http://192.168.7.6:8418/) need validation |

---

## Sources

- [cargo-deb on GitHub](https://github.com/kornelski/cargo-deb) -- v3.6.3, actively maintained
- [cargo-generate-rpm on crates.io](https://crates.io/crates/cargo-generate-rpm) -- v0.18.x+
- [linuxdeploy on GitHub](https://github.com/linuxdeploy/linuxdeploy) -- continuous releases
- [linuxdeploy-plugin-gtk](https://github.com/linuxdeploy/linuxdeploy-plugin-gtk) -- supports GTK 2/3/4
- [How to Flatpak a Rust application](https://belmoussaoui.com/blog/8-how-to-flatpak-a-rust-application/) -- canonical guide
- [Flatpak Available Runtimes](https://docs.flatpak.org/en/latest/available-runtimes.html) -- GNOME 47 current
- [Gitea Actions Quick Start](https://docs.gitea.com/usage/actions/quickstart) -- workflow YAML compat
- [Gitea act_runner docs](https://docs.gitea.com/usage/actions/act-runner) -- runner setup, v0.2.13+
- [ldd(1) man page](https://man7.org/linux/man-pages/man1/ldd.1.html) -- security note about untrusted binaries
- [RustDesk Linux Packaging (DeepWiki)](https://deepwiki.com/rustdesk/rustdesk/7.4-platform-packaging) -- reference Rust app packaging
- [Linux packaging format comparison](https://michaelneuper.com/posts/what-linux-packaging-format-to-use/)
