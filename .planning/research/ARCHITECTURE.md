# Architecture: Linux Packaging & Distribution

**Domain:** Linux packaging (.deb, .rpm, AppImage, Flatpak) + Gitea CI for a multi-language Rust/Zig/Go terminal multiplexer
**Researched:** 2026-03-28
**Confidence:** HIGH for .deb/.rpm/AppImage, MEDIUM for Flatpak (Zig build inside Flatpak sandbox is unusual), HIGH for Gitea Actions

---

## Existing Build Chain (What We Have)

The current build produces three artifacts from three languages:

```
ghostty submodule ──[zig build]──► ghostty/zig-out/lib/libghostty.a  (static lib)
Cargo.toml        ──[cargo build]──► target/release/cmux-app          (GUI binary)
                                     target/release/cmux              (CLI binary)
daemon/remote/    ──[go build]──► cmuxd-remote                        (Go binary)
```

**Build order is strict:** Zig must complete before Cargo (build.rs links libghostty.a). Go is independent.

Existing files that ship with the app:
- `resources/cmux.desktop` -- freedesktop .desktop entry
- `resources/cmux.svg` -- scalable icon (256x256 viewBox SVG)
- `ghostty.h` -- C header for bindgen (build-time only)

Missing files needed for packaging (must create):
- `resources/com.cmuxterm.cmux.metainfo.xml` -- AppStream metadata (required for Flatpak, recommended for .deb/.rpm)
- `resources/icons/` -- rasterized PNG icons at standard sizes (48x48 minimum, 128x128 and 256x256 recommended)
- Flatpak manifest
- Cargo.toml `[package.metadata.deb]` and `[package.metadata.generate-rpm]` sections

---

## Source Tree Organization for Packaging

All packaging configuration lives in a new `packaging/` directory at the repo root, except for tool-specific metadata in Cargo.toml.

```
cmux-linux/
├── resources/                          # EXISTING — desktop integration files
│   ├── cmux.desktop                    # EXISTING — freedesktop .desktop file
│   ├── cmux.svg                        # EXISTING — scalable icon
│   └── com.cmuxterm.cmux.metainfo.xml  # NEW — AppStream metadata
│
├── packaging/                          # NEW — all packaging scripts and configs
│   ├── build-all.sh                    # NEW — master script: build all formats
│   ├── build-deb.sh                    # NEW — .deb build script
│   ├── build-rpm.sh                    # NEW — .rpm build script
│   ├── build-appimage.sh              # NEW — AppImage build script
│   ├── build-flatpak.sh              # NEW — Flatpak build script
│   ├── detect-deps.sh                 # NEW — ldd/readelf runtime dep scanner
│   └── flatpak/                        # NEW — Flatpak-specific files
│       ├── com.cmuxterm.cmux.yml       # NEW — Flatpak manifest
│       └── flatpak-cargo-generator.py  # NEW — vendored script to generate cargo-sources.json
│
├── .gitea/                             # NEW — Gitea CI workflows
│   └── workflows/
│       ├── build-packages.yml          # NEW — triggered on tag push
│       └── ci.yml                      # NEW — PR/push validation (Linux build + clippy)
│
├── Cargo.toml                          # MODIFY — add [package.metadata.deb] and [package.metadata.generate-rpm]
└── ...
```

### Why This Layout

- `resources/` already exists with desktop files; adding metainfo.xml there keeps desktop integration files together.
- `packaging/` isolates build scripts from the source tree. Each format has its own script for independent testing.
- `.gitea/workflows/` is the Gitea Actions convention (mirrors `.github/workflows/`).
- Flatpak manifests reference source modules; keeping the manifest in `packaging/flatpak/` avoids cluttering the repo root.

---

## Component Architecture: How Each Format Works

### Component 1: Shared Build Foundation

All four package formats depend on the same compiled artifacts. A shared build step produces them:

```
┌─────────────────────────────────────────────────────────┐
│                   Shared Build Stage                     │
│                                                          │
│  1. zig build (ghostty submodule → libghostty.a)        │
│  2. cargo build --release (→ cmux-app, cmux binaries)   │
│  3. go build (daemon/remote → cmuxd-remote)             │
│                                                          │
│  Outputs:                                                │
│    target/release/cmux-app     (GUI application)         │
│    target/release/cmux         (CLI tool)                │
│    daemon/remote/cmuxd-remote  (SSH remote daemon)       │
└─────────────────┬───────────────────────────────────────┘
                  │
    ┌─────────────┼─────────────┬─────────────┐
    ▼             ▼             ▼             ▼
 .deb          .rpm        AppImage      Flatpak
```

### Component 2: .deb Package (cargo-deb)

**Tool:** `cargo-deb` (v3.6.x) -- reads metadata from `Cargo.toml [package.metadata.deb]`.

**How it integrates:** cargo-deb calls `cargo build --release` internally, then packages the resulting binaries with debian metadata. Since our build requires the Zig step first, the build script must:
1. Run `zig build` to produce libghostty.a
2. Run `cargo deb` which handles cargo build + .deb packaging in one step

**Cargo.toml additions:**

```toml
[package.metadata.deb]
maintainer = "cmux team"
copyright = "2025-2026 Manaflow AI"
license-file = ["LICENSE", "0"]
extended-description = "GPU-accelerated terminal multiplexer with tabs, splits, workspaces, and socket control"
section = "utils"
priority = "optional"
depends = "$auto"  # auto-detect from ldd
assets = [
    ["target/release/cmux-app", "usr/bin/", "755"],
    ["target/release/cmux", "usr/bin/", "755"],
    ["daemon/remote/cmuxd-remote", "usr/libexec/cmux/", "755"],
    ["resources/cmux.desktop", "usr/share/applications/", "644"],
    ["resources/cmux.svg", "usr/share/icons/hicolor/scalable/apps/", "644"],
    ["resources/com.cmuxterm.cmux.metainfo.xml", "usr/share/metainfo/", "644"],
]
```

**Key detail:** `$auto` dependency detection uses `dpkg-shlibdeps` to scan the binary's linked shared libraries and generate the correct `Depends:` line. This handles GTK4, fontconfig, freetype, libonig, libGL automatically.

**Output:** `target/debian/cmux-linux_0.1.0-1_amd64.deb`

### Component 3: .rpm Package (cargo-generate-rpm)

**Tool:** `cargo-generate-rpm` (v0.15.x) -- reads metadata from `Cargo.toml [package.metadata.generate-rpm]`.

**How it integrates:** Unlike cargo-deb, this tool does NOT run cargo build. It only packages pre-built binaries. The build script must:
1. Run `zig build`
2. Run `cargo build --release`
3. Build cmuxd-remote with `go build`
4. Run `cargo generate-rpm`

**Cargo.toml additions:**

```toml
[package.metadata.generate-rpm]
assets = [
    { source = "target/release/cmux-app", dest = "/usr/bin/cmux-app", mode = "755" },
    { source = "target/release/cmux", dest = "/usr/bin/cmux", mode = "755" },
    { source = "daemon/remote/cmuxd-remote", dest = "/usr/libexec/cmux/cmuxd-remote", mode = "755" },
    { source = "resources/cmux.desktop", dest = "/usr/share/applications/cmux.desktop", mode = "644" },
    { source = "resources/cmux.svg", dest = "/usr/share/icons/hicolor/scalable/apps/cmux.svg", mode = "644" },
    { source = "resources/com.cmuxterm.cmux.metainfo.xml", dest = "/usr/share/metainfo/com.cmuxterm.cmux.metainfo.xml", mode = "644" },
]

[package.metadata.generate-rpm.requires]
gtk4 = ">= 4.12"
fontconfig = "*"
freetype = "*"
```

**Key difference from .deb:** No automatic dependency detection. Runtime deps must be listed explicitly. The `detect-deps.sh` script scans `ldd` output and maps .so names to RPM package names for the target distro.

**Output:** `target/generate-rpm/cmux-linux-0.1.0-1.x86_64.rpm`

### Component 4: AppImage (linuxdeploy)

**Tool:** `linuxdeploy` + `linuxdeploy-plugin-gtk` -- bundles binary with all shared libraries into a portable executable.

**How it integrates:** linuxdeploy takes a pre-built binary plus an AppDir structure and produces a self-contained AppImage. The GTK plugin handles bundling GTK4 schemas, icons, and theme engine libraries.

**Build flow:**

```
1. Build all binaries (zig → cargo → go)
2. Create AppDir structure:
   AppDir/
   ├── AppRun → usr/bin/cmux-app (symlink)
   ├── cmux.desktop
   ├── cmux.svg
   └── usr/
       ├── bin/
       │   ├── cmux-app
       │   └── cmux
       ├── libexec/cmux/
       │   └── cmuxd-remote
       └── share/
           ├── applications/cmux.desktop
           ├── icons/hicolor/scalable/apps/cmux.svg
           └── metainfo/com.cmuxterm.cmux.metainfo.xml
3. Run linuxdeploy:
   DEPLOY_GTK_VERSION=4 linuxdeploy \
     --appdir AppDir \
     --desktop-file resources/cmux.desktop \
     --icon-file resources/cmux.svg \
     --plugin gtk \
     --output appimage
```

**Critical: GTK4 plugin.** Without `DEPLOY_GTK_VERSION=4`, the plugin defaults to GTK3 and will bundle wrong libraries. The plugin copies GLib schemas, pixbuf loaders, and GTK modules into the AppImage.

**Output:** `cmux-x86_64.AppImage`

### Component 5: Flatpak (flatpak-builder)

**Tool:** `flatpak-builder` -- builds in an isolated sandbox from a manifest file.

**This is the most complex format** because Flatpak's sandbox means:
- No network access during build (all deps must be pre-declared)
- Zig must be available inside the sandbox (not in standard SDK)
- Go must be available inside the sandbox (not in standard SDK)
- Cargo deps must be vendored via `flatpak-cargo-generator.py`

**Manifest structure** (`packaging/flatpak/com.cmuxterm.cmux.yml`):

```yaml
app-id: com.cmuxterm.cmux
runtime: org.gnome.Platform
runtime-version: '46'
sdk: org.gnome.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
  - org.freedesktop.Sdk.Extension.golang
command: cmux-app

finish-args:
  - --share=ipc
  - --socket=fallback-x11
  - --socket=wayland
  - --device=dri          # GPU access for OpenGL rendering
  - --socket=pulseaudio   # terminal bell audio
  - --share=network       # SSH workspaces, browser automation
  - --talk-name=org.freedesktop.Notifications
  - --filesystem=home     # terminal needs home directory access

build-options:
  append-path: /usr/lib/sdk/rust-stable/bin:/usr/lib/sdk/golang/bin
  env:
    CARGO_HOME: /run/build/cmux/cargo
    GOPATH: /run/build/cmux/go

modules:
  # Module 1: Zig toolchain (downloaded as tarball)
  - name: zig
    buildsystem: simple
    build-commands:
      - install -Dm755 zig /app/bin/zig
      - cp -r lib /app/lib/zig
    sources:
      - type: archive
        url: https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz
        sha256: <pinned-hash>

  # Module 2: cmux (builds ghostty, then Rust, then Go)
  - name: cmux
    buildsystem: simple
    build-commands:
      - cd ghostty && /app/bin/zig build -Dapp-runtime=none -Doptimize=ReleaseFast -Dgtk-x11=true -Dgtk-wayland=true
      - cargo --offline fetch --manifest-path Cargo.toml
      - cargo --offline build --release
      - cd daemon/remote && go build -o cmuxd-remote .
      - install -Dm755 target/release/cmux-app /app/bin/cmux-app
      - install -Dm755 target/release/cmux /app/bin/cmux
      - install -Dm755 daemon/remote/cmuxd-remote /app/libexec/cmux/cmuxd-remote
      - install -Dm644 resources/cmux.desktop /app/share/applications/com.cmuxterm.cmux.desktop
      - install -Dm644 resources/cmux.svg /app/share/icons/hicolor/scalable/apps/com.cmuxterm.cmux.svg
      - install -Dm644 resources/com.cmuxterm.cmux.metainfo.xml /app/share/metainfo/com.cmuxterm.cmux.metainfo.xml
    sources:
      - type: dir
        path: ../..
      # Generated from: python3 flatpak-cargo-generator.py ../../Cargo.lock -o cargo-sources.json
      - cargo-sources.json
```

**Flatpak-specific complications:**
1. **Zig toolchain:** Not available in any Flatpak SDK extension. Must be downloaded as a tarball module and installed into `/app/bin/`.
2. **Cargo offline build:** `flatpak-cargo-generator.py` parses `Cargo.lock` and produces a JSON file listing every crate as a downloadable source. This must be regenerated whenever `Cargo.lock` changes.
3. **Go modules:** The `golang` SDK extension provides Go, but `go.sum` deps must also be vendored or pre-fetched. Simplest approach: `go mod vendor` in the source tree before building.
4. **Ghostty submodule:** Must be included in the Flatpak source. The `type: dir` source pulls the full repo including submodules.

---

## Desktop Integration File Specifications

### .desktop File (EXISTING -- needs minor update)

Current `resources/cmux.desktop` is correct. One change needed: for Flatpak, the desktop file must use the app ID as filename (`com.cmuxterm.cmux.desktop`). The build scripts handle this rename at install time.

### AppStream Metainfo (NEW)

Required for Flatpak submission (Flathub), strongly recommended for .deb/.rpm (shows in GNOME Software/KDE Discover).

`resources/com.cmuxterm.cmux.metainfo.xml`:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>com.cmuxterm.cmux</id>
  <name>cmux</name>
  <summary>GPU-accelerated terminal multiplexer</summary>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <description>
    <p>cmux is a terminal multiplexer with tabs, pane splits, workspaces,
    SSH remote sessions, and programmatic socket control. Powered by
    Ghostty's GPU-accelerated terminal engine.</p>
  </description>
  <launchable type="desktop-id">cmux.desktop</launchable>
  <url type="homepage">https://cmuxterm.com</url>
  <provides>
    <binary>cmux-app</binary>
    <binary>cmux</binary>
  </provides>
  <content_rating type="oars-1.1" />
  <releases>
    <release version="0.1.0" date="2026-03-28" />
  </releases>
</component>
```

### Icons

The existing `cmux.svg` works as the scalable icon. For compatibility with older icon themes and notification areas, generate rasterized PNGs:

```
resources/
├── cmux.svg                              # scalable (already exists)
└── icons/
    ├── 48x48/apps/cmux.png              # minimum required
    ├── 128x128/apps/cmux.png            # recommended
    └── 256x256/apps/cmux.png            # recommended
```

Install paths in packages:
- `/usr/share/icons/hicolor/scalable/apps/cmux.svg`
- `/usr/share/icons/hicolor/48x48/apps/cmux.png`
- `/usr/share/icons/hicolor/128x128/apps/cmux.png`
- `/usr/share/icons/hicolor/256x256/apps/cmux.png`

Generate PNGs from SVG at build time with `rsvg-convert` (from librsvg):
```bash
for size in 48 128 256; do
  rsvg-convert -w $size -h $size resources/cmux.svg -o resources/icons/${size}x${size}/apps/cmux.png
done
```

---

## Install Path Conventions

All formats install to the same logical paths:

| File | Install Path | Notes |
|------|-------------|-------|
| cmux-app | `/usr/bin/cmux-app` | GUI binary |
| cmux | `/usr/bin/cmux` | CLI binary |
| cmuxd-remote | `/usr/libexec/cmux/cmuxd-remote` | SSH remote daemon (not in PATH) |
| cmux.desktop | `/usr/share/applications/cmux.desktop` | Desktop entry |
| cmux.svg | `/usr/share/icons/hicolor/scalable/apps/cmux.svg` | Scalable icon |
| metainfo.xml | `/usr/share/metainfo/com.cmuxterm.cmux.metainfo.xml` | AppStream metadata |

Flatpak uses `/app/` prefix instead of `/usr/`. AppImage bundles everything inside the AppDir.

---

## Build Order (Full Pipeline)

```
Phase 1: Environment Setup
├── Install system deps (GTK4-dev, Zig 0.15.2, Go, Rust)
└── Install packaging tools (cargo-deb, cargo-generate-rpm, linuxdeploy)

Phase 2: Compile Artifacts (sequential where noted)
├── [MUST BE FIRST] cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast -Dgtk-x11=true -Dgtk-wayland=true
├── [AFTER ZIG]     cargo build --release  (produces cmux-app + cmux)
└── [PARALLEL OK]   cd daemon/remote && go build -o cmuxd-remote .

Phase 3: Package (all parallel, all read-only on build artifacts)
├── cargo deb --no-build              (.deb — uses pre-built binary)
├── cargo generate-rpm                (.rpm — uses pre-built binary)
├── linuxdeploy + appimagetool       (AppImage)
└── flatpak-builder                   (Flatpak — rebuilds inside sandbox)
```

**Key constraint:** Flatpak cannot reuse Phase 2 artifacts because it builds in its own sandbox. The Flatpak build repeats the full compile chain inside `flatpak-builder`. The other three formats all reuse the same `target/release/` binaries.

---

## Gitea Actions Runner Architecture

### How Gitea Actions Works

Gitea Actions uses `act_runner` -- a standalone binary that polls the Gitea server for jobs and executes them. It is architecturally identical to GitHub Actions self-hosted runners.

```
┌──────────────────────┐         ┌──────────────────────────┐
│   Gitea Server       │  HTTP   │    act_runner             │
│   192.168.7.6:8418   │◄───────►│    (on build machine)     │
│                      │  poll   │                            │
│   - Stores workflows │         │   Execution modes:         │
│   - Dispatches jobs  │         │   ├── Docker (default)     │
│   - Receives results │         │   ├── Host (bare metal)    │
│   - Hosts packages   │         │   └── LXC                  │
└──────────────────────┘         └──────────────────────────┘
```

### Runner Registration

```bash
# 1. Download act_runner binary
wget https://gitea.com/gitea/act_runner/releases/latest/download/act_runner-linux-amd64

# 2. Generate config
./act_runner generate-config > config.yaml

# 3. Register with Gitea instance (get token from Gitea admin UI)
./act_runner register \
  --instance http://192.168.7.6:8418 \
  --token <registration-token> \
  --name linux-builder \
  --labels ubuntu-latest:host

# 4. Run as daemon (systemd service recommended)
./act_runner daemon --config config.yaml
```

### Runner Mode: Host vs Docker

**Use `host` mode** for this project because:
1. Zig build of Ghostty requires ~4GB RAM and takes minutes -- Docker container spin-up adds overhead
2. The runner needs GPU access for testing (OpenGL)
3. System dependencies (GTK4-dev, libfontconfig-dev, etc.) are easier to manage on the host than in custom Docker images
4. Flatpak builds require access to the host's flatpak installation

**Labels configuration:** The runner registers with labels that map to `runs-on:` in workflows. Use `ubuntu-latest:host` so existing GitHub Actions workflow syntax works.

### Workflow Syntax (Gitea vs GitHub)

Gitea Actions uses the same YAML syntax as GitHub Actions with these differences relevant to this project:

| Feature | GitHub Actions | Gitea Actions |
|---------|---------------|---------------|
| Workflow dir | `.github/workflows/` | `.gitea/workflows/` |
| `uses:` actions | `actions/checkout@v4` | Same, or full URL: `https://github.com/actions/checkout@v4` |
| `concurrency:` | Supported | **Ignored** -- must handle manually |
| `runs-on:` | Complex matrix | Simple string or array only |
| Secrets | `${{ secrets.X }}` | Same syntax |
| Package registry | N/A | Built-in Debian/RPM/Generic registries |
| Cache | `actions/cache@v4` | Supported (same syntax) |

### Gitea Package Registry Upload

Gitea has built-in package registries for Debian and RPM. Upload from CI:

```bash
# Debian package
curl --user "${GITEA_USER}:${GITEA_TOKEN}" \
  --upload-file target/debian/cmux-linux_*.deb \
  "http://192.168.7.6:8418/api/packages/${GITEA_USER}/debian/pool/jammy/main/upload"

# RPM package
curl --user "${GITEA_USER}:${GITEA_TOKEN}" \
  --upload-file target/generate-rpm/cmux-linux-*.rpm \
  "http://192.168.7.6:8418/api/packages/${GITEA_USER}/rpm/upload"

# AppImage + Flatpak bundle (generic registry)
curl --user "${GITEA_USER}:${GITEA_TOKEN}" \
  --upload-file cmux-x86_64.AppImage \
  "http://192.168.7.6:8418/api/packages/${GITEA_USER}/generic/cmux/${VERSION}/cmux-x86_64.AppImage"
```

Users can then add the Gitea instance as an apt/dnf repository:
```bash
# Debian/Ubuntu
echo "deb http://192.168.7.6:8418/api/packages/USER/debian jammy main" | sudo tee /etc/apt/sources.list.d/cmux.list
sudo apt update && sudo apt install cmux-linux

# Fedora/RHEL
dnf config-manager --add-repo http://192.168.7.6:8418/api/packages/USER/rpm.repo
dnf install cmux-linux
```

---

## Gitea CI Workflow Design

```yaml
# .gitea/workflows/build-packages.yml
name: Build Packages

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: ubuntu-latest  # maps to host runner label
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive

      - name: Install system deps
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-4-dev libclang-dev libfontconfig1-dev \
            libfreetype6-dev libonig-dev libgl-dev librsvg2-bin

      - name: Install Zig 0.15.2
        run: |
          curl -fSL "https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz" -o /tmp/zig.tar.xz
          tar xf /tmp/zig.tar.xz -C /tmp
          sudo cp /tmp/zig-x86_64-linux-0.15.2/zig /usr/local/bin/
          sudo cp -r /tmp/zig-x86_64-linux-0.15.2/lib /usr/local/lib/zig

      - name: Build libghostty
        run: cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast -Dgtk-x11=true -Dgtk-wayland=true

      - name: Build Rust binaries
        run: cargo build --release

      - name: Build Go remote daemon
        run: cd daemon/remote && go build -o cmuxd-remote .

      - name: Build .deb
        run: cargo deb --no-build

      - name: Build .rpm
        run: cargo generate-rpm

      - name: Build AppImage
        run: ./packaging/build-appimage.sh

      - name: Upload to Gitea packages
        run: ./packaging/upload-packages.sh
        env:
          GITEA_TOKEN: ${{ secrets.GITEA_TOKEN }}

      - name: Create release with assets
        run: |
          TAG="${GITHUB_REF#refs/tags/}"
          # Gitea release creation via API
```

---

## Data Flow: Source to Package

```
Source Tree
    │
    ├── ghostty/ (submodule)
    │     └──[zig build]──► ghostty/zig-out/lib/libghostty.a
    │
    ├── src/ (Rust)
    │     └──[cargo build --release]──► target/release/cmux-app
    │                                    target/release/cmux
    │
    ├── daemon/remote/ (Go)
    │     └──[go build]──► daemon/remote/cmuxd-remote
    │
    ├── resources/
    │     ├── cmux.desktop ──────────────────────────┐
    │     ├── cmux.svg ─────────────────────────────┐│
    │     └── com.cmuxterm.cmux.metainfo.xml ──────┐││
    │                                              │││
    └── Cargo.toml                                 │││
          ├── [package.metadata.deb] ──► cargo-deb │││
          │     └──► target/debian/cmux-linux_*.deb ◄┘│
          │                                          ││
          ├── [package.metadata.generate-rpm]         ││
          │     └──► cargo generate-rpm              ││
          │           └──► target/generate-rpm/*.rpm ◄┘│
          │                                           │
          └── packaging/build-appimage.sh             │
                └──► linuxdeploy + plugin-gtk        │
                      └──► cmux-x86_64.AppImage ◄────┘
```

---

## Scalability Considerations

| Concern | Now (single arch) | Later (multi-arch) |
|---------|-------------------|-------------------|
| Architectures | x86_64 only | Add aarch64 via cross-compilation or QEMU runner |
| Zig cross-compile | Not needed | Zig natively cross-compiles; add `-Dtarget=aarch64-linux-gnu` |
| Cargo cross-compile | Not needed | Use `cross` tool or `cargo-zigbuild` |
| Go cross-compile | Not needed | `GOARCH=arm64 go build` |
| CI runners | 1 host runner | Add aarch64 runner or use QEMU in Docker |

---

## Sources

- [cargo-deb](https://github.com/kornelski/cargo-deb) -- Debian package builder for Rust (HIGH confidence, official repo)
- [cargo-generate-rpm](https://crates.io/crates/cargo-generate-rpm) -- RPM package builder for Rust (HIGH confidence, crates.io)
- [linuxdeploy](https://docs.appimage.org/packaging-guide/from-source/linuxdeploy-user-guide.html) -- AppImage packaging tool (HIGH confidence, official docs)
- [linuxdeploy-plugin-gtk](https://github.com/linuxdeploy/linuxdeploy-plugin-gtk) -- GTK bundling for AppImage (HIGH confidence, official repo)
- [Flatpak manifest docs](https://docs.flatpak.org/en/latest/manifests.html) -- Flatpak build specification (HIGH confidence, official docs)
- [How to Flatpak a Rust application](https://belmoussaoui.com/blog/8-how-to-flatpak-a-rust-application/) -- Rust-specific Flatpak guide (MEDIUM confidence, community)
- [Gitea Actions docs](https://docs.gitea.com/usage/actions/overview) -- CI system documentation (HIGH confidence, official docs)
- [Gitea vs GitHub Actions comparison](https://docs.gitea.com/usage/actions/comparison) -- Syntax differences (HIGH confidence, official docs)
- [Gitea Debian package registry](https://docs.gitea.com/usage/packages/debian) -- Package hosting API (HIGH confidence, official docs)
- [Gitea RPM package registry](https://docs.gitea.com/next/usage/packages/rpm) -- RPM hosting API (HIGH confidence, official docs)
- [Freedesktop icon theme spec](https://specifications.freedesktop.org/icon-theme/latest/) -- Icon installation paths (HIGH confidence, official spec)
- [Freedesktop desktop entry spec](https://specifications.freedesktop.org/desktop-entry/desktop-entry-spec-latest.html) -- .desktop file format (HIGH confidence, official spec)

---

*Architecture research for: cmux Linux Packaging & Distribution (v1.1 milestone)*
*Researched: 2026-03-28*
