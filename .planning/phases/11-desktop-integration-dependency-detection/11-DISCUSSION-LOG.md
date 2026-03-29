# Phase 11: Desktop Integration & Dependency Detection - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 11-desktop-integration-dependency-detection
**Areas discussed:** Desktop identity, Icon generation, Shell completions & man page, Dependency detection

---

## Desktop Identity

### Desktop entry filename

| Option | Description | Selected |
|--------|-------------|----------|
| Reverse-DNS filename | com.cmux-lx.terminal.desktop -- matches META-01, required for Flatpak, AppStream expects consistent IDs | ✓ |
| Keep cmux.desktop | Simpler filename but creates ID mismatch with metainfo/Flatpak | |

**User's choice:** Reverse-DNS filename
**Notes:** None

### Metainfo detail level

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal valid | Description, categories, content rating, empty screenshots. Enough to pass validation. | ✓ |
| Full with screenshots | Include screenshot URLs, requires hosting | |
| Full with releases | Screenshots + version history in releases section | |

**User's choice:** Minimal valid
**Notes:** None

### File layout

| Option | Description | Selected |
|--------|-------------|----------|
| New packaging/ directory | Separate packaging concerns from app resources. Phase 12-14 artifacts go here too. | ✓ |
| Keep in resources/ | Colocate with existing files, may get cluttered | |
| You decide | Claude picks | |

**User's choice:** New packaging/ directory
**Notes:** None

---

## Icon Generation

### Generation tool

| Option | Description | Selected |
|--------|-------------|----------|
| rsvg-convert | Part of librsvg, lightweight, commonly available | ✓ |
| ImageMagick convert | More widely installed but heavier, variable SVG quality | |
| You decide | Claude picks | |

**User's choice:** rsvg-convert
**Notes:** None

### Storage approach

| Option | Description | Selected |
|--------|-------------|----------|
| Check into repo | Pre-generated PNGs in packaging/icons/. Regenerate when SVG changes. | ✓ |
| Generate at build time | Always fresh but adds build dependency | |

**User's choice:** Check into repo
**Notes:** None

---

## Shell Completions & Man Page

### Generation approach

| Option | Description | Selected |
|--------|-------------|----------|
| Standalone generate script | Explicit script that runs cmux completion/man generation. Simple, runs when needed. | ✓ |
| build.rs generation | Couples packaging to build process, always fresh | |
| You decide | Claude picks | |

**User's choice:** Standalone generate script
**Notes:** None

### Storage approach

| Option | Description | Selected |
|--------|-------------|----------|
| Check into repo | packaging/completions/ and packaging/man/. Consistent with icon approach. | ✓ |
| Generate during packaging | Requires built binary, always fresh | |

**User's choice:** Check into repo
**Notes:** None

---

## Dependency Detection

### Script language

| Option | Description | Selected |
|--------|-------------|----------|
| Shell script | Bash using ldd + dpkg -S / rpm -qf. Simple, no extra deps. | ✓ |
| Python script | More structured, easier mapping tables. Adds Python dependency. | |
| You decide | Claude picks | |

**User's choice:** Shell script
**Notes:** None

### Output scope

| Option | Description | Selected |
|--------|-------------|----------|
| Always output both | Both Debian and Fedora names regardless of host. Uses native PM when available, static fallback for other. | ✓ |
| Auto-detect distro | Only outputs for current distro. Must run on each target separately. | |
| You decide | Claude picks for CI pipeline | |

**User's choice:** Always output both
**Notes:** None

---

## Claude's Discretion

- Exact metainfo XML structure and content rating values
- Hicolor icon theme directory naming
- Man page content and organization
- Dependency detection output format

## Deferred Ideas

None -- discussion stayed within phase scope.
