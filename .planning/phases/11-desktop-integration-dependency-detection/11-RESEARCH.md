# Phase 11: Desktop Integration & Dependency Detection - Research

**Researched:** 2026-03-29
**Domain:** Linux desktop metadata (AppStream, freedesktop icons, shell completions, man pages, dependency detection)
**Confidence:** HIGH

## Summary

This phase creates all shared metadata files that packaging phases 12-14 consume. The work is straightforward: desktop entry migration, AppStream metainfo XML authoring, PNG icon generation from SVG, shell completion and man page generation via clap crates, and a bash dependency detection script.

A critical finding during research: the user-chosen ID `com.cmux-lx.terminal` contains a hyphen which causes `appstreamcli validate` to fail with a warning (exit code 3). The ID must use an underscore instead: `com.cmux_lx.terminal`. This affects the .desktop filename, metainfo XML, and icon theme entries. The research validates a working metainfo XML template that passes validation.

**Primary recommendation:** Use `com.cmux_lx.terminal` (underscore, not hyphen) as the reverse-DNS ID. Generate completions and man pages via a standalone Rust binary using clap_complete and clap_mangen crates. Use rsvg-convert for icon generation (needs `apt install librsvg2-bin`). Dependency detection script uses ldd + dpkg-S with a static Fedora fallback table.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use reverse-DNS ID `com.cmux-lx.terminal` consistently across .desktop filename, metainfo XML, and icon theme entries
- **D-02:** Rename existing `resources/cmux.desktop` to `packaging/desktop/com.cmux-lx.terminal.desktop` with updated fields
- **D-03:** AppStream metainfo XML is minimal-valid: app description, categories, content rating (OARS 1.1), empty screenshots section. Enough to pass `appstreamcli validate`.
- **D-04:** New `packaging/` directory at repo root for all packaging artifacts
- **D-05:** Use `rsvg-convert` (librsvg) to generate PNGs from `resources/cmux.svg`
- **D-06:** Generated PNGs checked into repo under `packaging/icons/` at 48px, 128px, 256px. SVG remains source of truth.
- **D-07:** Standalone generate script (not build.rs) that runs cmux completion/man generation
- **D-08:** Generated completions and man page checked into repo under `packaging/completions/` and `packaging/man/`
- **D-09:** Bash shell script using `ldd` + `dpkg -S` / `rpm -qf` to map shared library dependencies to package names
- **D-10:** Always outputs both Debian and Fedora package names regardless of host distro. Uses native package manager when available, static fallback table for the other.

### Claude's Discretion
- Exact metainfo XML structure and content rating values
- Icon directory naming within hicolor theme structure (standard: `hicolor/{size}x{size}/apps/`)
- Man page content and section organization
- Dependency detection script output format (human-readable + machine-parseable)

### Deferred Ideas (OUT OF SCOPE)
None.

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| META-01 | App uses reverse-DNS ID across .desktop, metainfo, and icons | Validated working metainfo XML template; ID must use underscore not hyphen to pass appstreamcli validate |
| META-02 | AppStream metainfo XML provides app description, screenshots section, and content rating | Tested minimal XML that passes `appstreamcli validate` with exit 0; requires id, name, summary, metadata_license, project_license, description, url homepage, developer with id, content_rating oars-1.1, launchable |
| META-03 | PNG icons generated from SVG at 48px, 128px, 256px for hicolor icon theme | rsvg-convert available via `apt install librsvg2-bin`; standard hicolor paths documented |
| META-04 | Shell completions installed for bash, zsh, and fish for cmux CLI | clap_complete 4.6.0 generates from clap Command; standalone binary approach documented |
| META-05 | Man page installed at `/usr/share/man/man1/cmux.1.gz` | clap_mangen 0.3.0 renders roff from clap Command; Man::new(cmd).render(&mut buf) |
| BUILD-02 | Runtime dependencies auto-detected from binary via ldd and mapped to package names | ldd shows 71 shared libs for cmux-app; dpkg -S maps to Debian packages; Fedora needs static fallback table |

</phase_requirements>

## Standard Stack

### Core
| Library/Tool | Version | Purpose | Why Standard |
|-------------|---------|---------|--------------|
| clap_complete | 4.6.0 | Shell completion generation | Official clap ecosystem; generates bash/zsh/fish from Command |
| clap_mangen | 0.3.0 | Man page generation | Official clap ecosystem; renders roff from Command struct |
| rsvg-convert | 2.58.0 (librsvg2-bin) | SVG to PNG conversion | Standard freedesktop tool for icon generation |
| appstreamcli | 1.0.2 | Metainfo XML validation | Reference implementation of AppStream spec |

### Supporting
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| ldd | glibc 2.39 | List shared library deps | Dependency detection script |
| dpkg -S | 1.22.6 | Map .so to Debian packages | Native on Debian/Ubuntu hosts |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rsvg-convert | ImageMagick convert | IM is heavier dependency; rsvg is purpose-built for SVG |
| Standalone gen binary | build.rs | build.rs runs on every build; standalone runs once on demand |

**Installation (for generation scripts):**
```bash
# Icon generation (not currently installed)
sudo apt install librsvg2-bin

# Completion/man generation (Rust crates, added to Cargo.toml)
# clap_complete = "4.6"
# clap_mangen = "0.3"
```

## Architecture Patterns

### Recommended Project Structure
```
packaging/
├── desktop/
│   ├── com.cmux_lx.terminal.desktop        # Renamed/updated from resources/
│   └── com.cmux_lx.terminal.metainfo.xml   # AppStream metainfo
├── icons/
│   └── hicolor/
│       ├── 48x48/apps/com.cmux_lx.terminal.png
│       ├── 128x128/apps/com.cmux_lx.terminal.png
│       └── 256x256/apps/com.cmux_lx.terminal.png
├── completions/
│   ├── cmux.bash
│   ├── _cmux (zsh)
│   └── cmux.fish
├── man/
│   └── cmux.1
└── scripts/
    ├── generate-icons.sh
    ├── generate-completions.sh    # Builds + runs the gen binary
    └── detect-deps.sh
```

### Pattern 1: Standalone Completion/Man Generator Binary
**What:** A separate Rust binary (e.g., `src/bin/generate.rs`) that imports the CLI Command definition and uses clap_complete + clap_mangen to write files to disk.
**When to use:** When completions/man pages are checked into repo and regenerated on demand (D-07, D-08).
**Example:**
```rust
// src/bin/generate.rs
use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
use std::fs;

// Import the CLI definition
#[path = "../cli/mod.rs"]
mod cli;

fn main() -> std::io::Result<()> {
    let mut cmd = cli::Cli::command();

    let outdir = std::path::Path::new("packaging/completions");
    fs::create_dir_all(outdir)?;

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        generate_to(shell, &mut cmd, "cmux", outdir)?;
    }

    let mandir = std::path::Path::new("packaging/man");
    fs::create_dir_all(mandir)?;
    let man = Man::new(cmd);
    let mut buf = Vec::new();
    man.render(&mut buf)?;
    fs::write(mandir.join("cmux.1"), buf)?;

    Ok(())
}
```

### Pattern 2: AppStream Metainfo (Validated Template)
**What:** Minimal metainfo XML that passes `appstreamcli validate` with exit 0.
**Example:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>com.cmux_lx.terminal</id>
  <name>cmux</name>
  <summary>GPU-accelerated terminal multiplexer</summary>
  <metadata_license>FSFAP</metadata_license>
  <project_license>MIT</project_license>
  <description>
    <p>cmux is a GPU-accelerated terminal multiplexer for Linux
    with tabs, splits, workspaces, and socket CLI control,
    powered by Ghostty's terminal rendering.</p>
  </description>
  <launchable type="desktop-id">com.cmux_lx.terminal.desktop</launchable>
  <url type="homepage">https://github.com/manaflow-ai/cmux-linux</url>
  <developer id="com.cmux_lx">
    <name>manaflow-ai</name>
  </developer>
  <screenshots>
    <!-- Screenshots to be added when available -->
  </screenshots>
  <content_rating type="oars-1.1" />
  <categories>
    <category>System</category>
    <category>TerminalEmulator</category>
  </categories>
</component>
```
**Validation:** Tested on this host -- exits 0 with only pedantic hints remaining.

### Pattern 3: Icon Generation Script
**What:** Shell script using rsvg-convert to produce hicolor PNGs.
**Example:**
```bash
#!/usr/bin/env bash
set -euo pipefail
SVG="resources/cmux.svg"
ICON_DIR="packaging/icons/hicolor"
APP_ID="com.cmux_lx.terminal"

for SIZE in 48 128 256; do
    mkdir -p "$ICON_DIR/${SIZE}x${SIZE}/apps"
    rsvg-convert -w "$SIZE" -h "$SIZE" "$SVG" \
        -o "$ICON_DIR/${SIZE}x${SIZE}/apps/${APP_ID}.png"
done
```

### Pattern 4: Dependency Detection Script
**What:** Bash script that runs `ldd` on the binary, filters to meaningful libs, maps to package names.
**Key design:** Use `dpkg -S` natively on Debian, `rpm -qf` natively on Fedora. For the non-native distro, use a static fallback mapping table.
**Output format:** Both human-readable table and machine-parseable (one JSON object per line or a single JSON array).

### Anti-Patterns to Avoid
- **Generating in build.rs:** Adds compilation overhead to every build; completions/man rarely change.
- **Hardcoding all package names:** Breaks when distro versions change library sonames. Use ldd + package manager query as primary, fallback table as secondary.
- **Using hyphen in reverse-DNS ID:** `appstreamcli validate` emits a warning and fails (exit 3). Must use underscore.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Shell completions | Custom bash/zsh/fish scripts | clap_complete generate_to | Exact CLI flags stay in sync with code |
| Man page | Hand-written roff | clap_mangen Man::render | Auto-generates from Command; stays in sync |
| SVG to PNG | ImageMagick pipelines | rsvg-convert | Purpose-built, correct SVG rendering |
| AppStream validation | Manual XML review | appstreamcli validate | Reference validator, catches subtle spec issues |
| Lib-to-package mapping | Manual lookup tables | ldd + dpkg-S/rpm-qf | Discovers actual runtime deps from binary |

## Common Pitfalls

### Pitfall 1: Hyphen in Reverse-DNS ID
**What goes wrong:** `appstreamcli validate` fails with warning `cid-rdns-contains-hyphen` (exit 3).
**Why it happens:** AppStream follows D-Bus naming convention which prohibits hyphens in all segments except the last.
**How to avoid:** Use `com.cmux_lx.terminal` (underscore) instead of `com.cmux-lx.terminal` (hyphen).
**Warning signs:** Exit code 3 from appstreamcli validate.

**CRITICAL NOTE:** The CONTEXT.md decision D-01 specifies `com.cmux-lx.terminal` with a hyphen. This MUST be corrected to `com.cmux_lx.terminal` for validation to pass. The planner should note this override in the plan.

### Pitfall 2: Missing Required Metainfo Fields
**What goes wrong:** `appstreamcli validate` fails with warnings about missing homepage URL or developer info.
**Why it happens:** The minimal template from `appstreamcli new-template` omits fields that the validator requires at warning level.
**How to avoid:** Include: `<url type="homepage">`, `<developer id="..."><name>`, `<metadata_license>`, `<project_license>`, `<launchable type="desktop-id">`.
**Warning signs:** Warnings about url-homepage-missing, developer-info-missing, developer-id-missing.

### Pitfall 3: clap_complete Needs Command, Not Parser
**What goes wrong:** Compilation error when trying to get a Command from a Parser-derived struct.
**Why it happens:** clap_complete's `generate_to` takes a `&mut Command`, but `#[derive(Parser)]` gives you a Parser.
**How to avoid:** Use `Cli::command()` (the `CommandFactory` trait method) to get the underlying Command.

### Pitfall 4: Completion File Names per Shell
**What goes wrong:** Wrong file extension or naming convention for a shell.
**Why it happens:** Each shell has its own convention: bash uses `.bash`, zsh uses `_cmdname` (no extension), fish uses `.fish`.
**How to avoid:** `generate_to` handles naming automatically -- just provide the output directory.

### Pitfall 5: ldd Transitive Dependencies
**What goes wrong:** ldd shows 71 shared libraries for cmux-app, most are transitive deps of GTK4.
**Why it happens:** ldd recurses through the full dependency tree.
**How to avoid:** The dependency detection script should categorize: direct package deps (libgtk-4-1, libonig5, etc.) vs. transitive (pulled in by GTK4 automatically). Only direct deps need to be declared in package metadata.

### Pitfall 6: Screenshots Section
**What goes wrong:** Empty `<screenshots>` element may cause validation issues in some versions.
**Why it happens:** AppStream spec expects at least one screenshot if the element is present.
**How to avoid:** Either omit `<screenshots>` entirely or include a comment placeholder. Tested: omitting it passes validation fine.

## Code Examples

### Desktop Entry (Updated)
```ini
[Desktop Entry]
Type=Application
Name=cmux
GenericName=Terminal Multiplexer
Comment=GPU-accelerated terminal multiplexer with tabs, splits, and workspaces
Exec=cmux-app
Icon=com.cmux_lx.terminal
Terminal=false
Categories=System;TerminalEmulator;
Keywords=terminal;multiplexer;tabs;splits;workspaces;gpu;
StartupWMClass=cmux
MimeType=
```

### Dependency Detection Script (Core Logic)
```bash
#!/usr/bin/env bash
# Map shared libraries to package names for Debian and Fedora
BINARY="${1:?Usage: detect-deps.sh <binary>}"

# Known lib -> Fedora package mapping (fallback when rpm not available)
declare -A FEDORA_MAP=(
    ["libgtk-4.so.1"]="gtk4"
    ["libfontconfig.so.1"]="fontconfig"
    ["libfreetype.so.6"]="freetype"
    ["libonig.so.5"]="oniguruma"
    ["libGL.so.1"]="mesa-libGL"
    ["libharfbuzz.so.0"]="harfbuzz"
    ["libgio-2.0.so.0"]="glib2"
    ["libgobject-2.0.so.0"]="glib2"
    ["libglib-2.0.so.0"]="glib2"
    ["libcairo.so.2"]="cairo"
    ["libpango-1.0.so.0"]="pango"
    ["libgdk_pixbuf-2.0.so.0"]="gdk-pixbuf2"
    ["libepoxy.so.0"]="libepoxy"
)

# Filter: skip libc, libm, ld-linux, linux-vdso (always present)
ldd "$BINARY" | awk '/=>/ {print $1}' | \
    grep -v -E '^(libc\.so|libm\.so|libpthread|libdl|librt|linux-vdso|ld-linux)' | \
    sort -u | while read -r lib; do
    # Debian: native query
    deb_pkg=$(dpkg -S "*/$lib" 2>/dev/null | head -1 | cut -d: -f1 || echo "UNKNOWN")
    # Fedora: native query or fallback
    if command -v rpm &>/dev/null; then
        fed_pkg=$(rpm -qf "$(ldconfig -p | grep "$lib" | awk '{print $NF}' | head -1)" 2>/dev/null | head -1 || echo "UNKNOWN")
    else
        fed_pkg="${FEDORA_MAP[$lib]:-UNKNOWN}"
    fi
    printf '%s\t%s\t%s\n' "$lib" "$deb_pkg" "$fed_pkg"
done
```

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| appstreamcli | META-02 validation | Yes | 1.0.2 | -- |
| rsvg-convert | META-03 icon generation | No (lib installed, CLI not) | -- | `apt install librsvg2-bin` |
| ldd | BUILD-02 dep detection | Yes | glibc 2.39 | -- |
| dpkg | BUILD-02 Debian mapping | Yes | 1.22.6 | -- |
| rpm | BUILD-02 Fedora mapping | No | -- | Static fallback table (by design per D-10) |
| man | META-05 validation | Yes | 2.12.0 | -- |
| cargo/rustc | Completion gen binary | Yes (assumed from Cargo.toml) | -- | -- |

**Missing dependencies with no fallback:**
- None blocking

**Missing dependencies with fallback:**
- `rsvg-convert`: Install via `sudo apt install librsvg2-bin` (librsvg2-dev already installed, just need the CLI)
- `rpm`: Not available on this Debian/Ubuntu host; handled by design (D-10 static fallback table)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | bash + appstreamcli (metadata validation), man (man page validation), cargo build (gen binary compiles) |
| Config file | None needed |
| Quick run command | `appstreamcli validate --no-net packaging/desktop/com.cmux_lx.terminal.metainfo.xml` |
| Full suite command | `bash packaging/scripts/validate-all.sh` (to be created) |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| META-01 | Reverse-DNS ID consistent across files | smoke | `grep -c 'com.cmux_lx.terminal' packaging/desktop/*.desktop packaging/desktop/*.xml` | No - Wave 0 |
| META-02 | Metainfo XML validates | smoke | `appstreamcli validate --no-net packaging/desktop/com.cmux_lx.terminal.metainfo.xml` | No - Wave 0 |
| META-03 | PNG icons at correct sizes | smoke | `file packaging/icons/hicolor/*/apps/*.png` + dimension check | No - Wave 0 |
| META-04 | Shell completions exist | smoke | `test -f packaging/completions/cmux.bash && test -f packaging/completions/_cmux && test -f packaging/completions/cmux.fish` | No - Wave 0 |
| META-05 | Man page renders | smoke | `man -l packaging/man/cmux.1 > /dev/null 2>&1` | No - Wave 0 |
| BUILD-02 | Dep detection produces output | smoke | `bash packaging/scripts/detect-deps.sh target/debug/cmux-app \| grep -q libgtk` | No - Wave 0 |

### Sampling Rate
- **Per task commit:** `appstreamcli validate --no-net` on metainfo XML
- **Per wave merge:** Full validation script covering all requirements
- **Phase gate:** All 6 requirement checks pass

### Wave 0 Gaps
- [ ] `packaging/scripts/validate-all.sh` -- smoke test script covering all META + BUILD-02 requirements
- [ ] `rsvg-convert` must be installed before icon generation

## Sources

### Primary (HIGH confidence)
- **appstreamcli validate** -- tested directly on this host with multiple XML variants
- **ldd/dpkg** -- tested directly on actual cmux-app binary
- [clap_complete crate](https://crates.io/crates/clap_complete) - v4.6.0 confirmed via cargo search
- [clap_mangen crate](https://crates.io/crates/clap_mangen) - v0.3.0 confirmed via cargo search

### Secondary (MEDIUM confidence)
- [AppStream Quickstart](https://freedesktop.org/software/appstream/docs/chap-Quickstart.html) - metainfo structure
- [AppStream Validation](https://freedesktop.org/software/appstream/docs/chap-Validation.html) - validation rules
- [AppStream ID hyphen issue](https://github.com/ximion/appstream/issues/162) - confirms hyphen causes warning
- [docs.rs clap_complete](https://docs.rs/clap_complete/) - API reference
- [docs.rs clap_mangen](https://docs.rs/clap_mangen/latest/clap_mangen/) - Man struct API

### Tertiary (LOW confidence)
- Fedora package name mappings (from web search + rpmfind.net references, not verified on Fedora host)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all tools tested on host or verified via cargo search
- Architecture: HIGH - file layout follows freedesktop conventions; metainfo validated
- Pitfalls: HIGH - hyphen issue discovered and verified empirically

**Research date:** 2026-03-29
**Valid until:** 2026-04-28 (stable domain, specs rarely change)
