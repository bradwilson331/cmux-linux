# Phase 11: Desktop Integration & Dependency Detection - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Create all shared metadata files that packaging phases (12-14) consume: desktop entry, AppStream metainfo, hicolor icons, shell completions, man page, and a dependency detection script. No packaging format artifacts are created here -- only the inputs they reference.

</domain>

<decisions>
## Implementation Decisions

### Desktop Identity
- **D-01:** Use reverse-DNS ID `com.cmux-lx.terminal` consistently across .desktop filename, metainfo XML, and icon theme entries
- **D-02:** Rename existing `resources/cmux.desktop` to `packaging/desktop/com.cmux-lx.terminal.desktop` with updated fields
- **D-03:** AppStream metainfo XML is minimal-valid: app description, categories, content rating (OARS 1.1), empty screenshots section. Enough to pass `appstreamcli validate`.

### File Layout
- **D-04:** New `packaging/` directory at repo root for all packaging artifacts:
  - `packaging/desktop/` -- .desktop and metainfo XML
  - `packaging/icons/` -- generated PNG icons (48, 128, 256px)
  - `packaging/completions/` -- bash, zsh, fish completions
  - `packaging/man/` -- man page
  - `packaging/scripts/` -- dependency detection script, icon generation script, completion generation script

### Icon Generation
- **D-05:** Use `rsvg-convert` (librsvg) to generate PNGs from `resources/cmux.svg`
- **D-06:** Generated PNGs checked into repo under `packaging/icons/` at 48px, 128px, 256px. SVG remains source of truth -- regenerate when SVG changes.

### Shell Completions & Man Page
- **D-07:** Standalone generate script (not build.rs) that runs `cmux` completion/man generation
- **D-08:** Generated completions and man page checked into repo under `packaging/completions/` and `packaging/man/`

### Dependency Detection
- **D-09:** Bash shell script using `ldd` + `dpkg -S` / `rpm -qf` to map shared library dependencies to package names
- **D-10:** Always outputs both Debian and Fedora package names regardless of host distro. Uses native package manager when available, static fallback table for the other.

### Claude's Discretion
- Exact metainfo XML structure and content rating values
- Icon directory naming within hicolor theme structure (standard: `hicolor/{size}x{size}/apps/`)
- Man page content and section organization
- Dependency detection script output format (human-readable + machine-parseable)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Assets
- `resources/cmux.desktop` -- Current .desktop file to migrate/update
- `resources/cmux.svg` -- SVG icon source for PNG generation

### Requirements
- `.planning/REQUIREMENTS.md` -- META-01 through META-05, BUILD-02 requirements for this phase

### Standards (external)
- AppStream metainfo specification (freedesktop.org) -- for metainfo XML structure
- Freedesktop icon theme specification -- for hicolor directory layout
- Clap derive documentation -- for shell completion and man page generation APIs

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `resources/cmux.desktop` -- Existing desktop entry, needs reverse-DNS rename and field updates
- `resources/cmux.svg` -- Existing SVG icon, source for all PNG sizes
- `Cargo.toml` uses `clap = { version = "4", features = ["derive", "color", "env"] }` -- supports `clap_complete` and `clap_mangen` for generation

### Established Patterns
- Scripts live in `scripts/` directory (build helpers, release scripts)
- New packaging scripts go in `packaging/scripts/` to separate concerns

### Integration Points
- CLI binary (`cmux`) must be built before completion/man generation can run
- `cmux-app` binary is the target for `ldd` dependency analysis
- Phase 12-14 will reference files in `packaging/` directly

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

*Phase: 11-desktop-integration-dependency-detection*
*Context gathered: 2026-03-29*
