---
phase: 11-desktop-integration-dependency-detection
plan: 02
subsystem: packaging
tags: [clap_complete, clap_mangen, shell-completions, man-page, bash, zsh, fish]

# Dependency graph
requires:
  - phase: none
    provides: existing clap CLI definition in src/cli/mod.rs
provides:
  - shell completions for bash, zsh, fish generated from CLI definition
  - man page in roff format
  - generator binary (cmux-generate) for regeneration
  - wrapper script for on-demand regeneration
affects: [12-packaging-local-build-scripts, 13-flatpak-appimage-portable]

# Tech tracking
tech-stack:
  added: [clap_complete 4.6, clap_mangen 0.3]
  patterns: [standalone generator binary pattern, checked-in generated artifacts]

key-files:
  created:
    - src/lib.rs
    - src/bin/generate.rs
    - packaging/completions/cmux.bash
    - packaging/completions/_cmux
    - packaging/completions/cmux.fish
    - packaging/man/cmux.1
    - packaging/scripts/generate-completions.sh
  modified:
    - Cargo.toml

key-decisions:
  - "D-07: Standalone generator binary (not build.rs) for shell completions and man page"
  - "D-08: Generated files checked into repo for reproducible packaging"
  - "Created src/lib.rs exposing only pub mod cli to avoid pulling GTK4 dependencies into generator"

patterns-established:
  - "Generator binary pattern: separate bin target imports library crate for code generation"
  - "Packaging artifacts in packaging/ directory tree (completions/, man/, scripts/)"

requirements-completed: [META-04, META-05]

# Metrics
duration: 2min
completed: 2026-03-29
---

# Phase 11 Plan 02: Shell Completions & Man Page Summary

**Shell completions (bash/zsh/fish) and man page generated from clap CLI definition via cmux-generate binary**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-29T17:19:10Z
- **Completed:** 2026-03-29T17:21:00Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Generator binary produces bash, zsh, and fish completions containing all 34+ cmux subcommands
- Man page renders correctly via `man -l` with proper roff formatting
- Library crate (src/lib.rs) exposes CLI module without pulling in GTK4 dependencies
- Wrapper script enables one-command regeneration after CLI changes

## Task Commits

Each task was committed atomically:

1. **Task 1: Add dependencies and create generator binary** - `1fb34884` (feat)
2. **Task 2: Run generator, create wrapper script, verify outputs** - `0ed4ea6e` (feat)

## Files Created/Modified
- `Cargo.toml` - Added clap_complete, clap_mangen deps and cmux-generate binary target
- `src/lib.rs` - Library crate exposing `pub mod cli` for generator binary
- `src/bin/generate.rs` - Standalone binary generating completions and man page from Cli struct
- `packaging/completions/cmux.bash` - Bash completion script (3314 lines across all completions)
- `packaging/completions/_cmux` - Zsh completion script
- `packaging/completions/cmux.fish` - Fish completion script
- `packaging/man/cmux.1` - Man page in roff format
- `packaging/scripts/generate-completions.sh` - Wrapper script for regeneration

## Decisions Made
- Created `src/lib.rs` with only `pub mod cli` to expose the CLI module as a library crate without pulling in GTK4 and other GUI dependencies
- Used standalone generator binary pattern (not build.rs) per D-07 decision from research phase
- Checked generated files into repo per D-08 for reproducible packaging without requiring Rust toolchain

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Created src/lib.rs for library crate access**
- **Found during:** Task 1 (generator binary creation)
- **Issue:** No lib.rs existed; generator binary needed to import Cli type from the cmux-linux crate
- **Fix:** Created `src/lib.rs` with `pub mod cli;` to expose the CLI module
- **Files modified:** src/lib.rs
- **Verification:** `cargo build --bin cmux-generate` succeeded
- **Committed in:** 1fb34884 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Plan anticipated this need and documented the approach. Minimal addition.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Shell completions and man page ready for packaging phases to install to standard locations
- packaging/completions/ and packaging/man/ directories follow FHS conventions
- Wrapper script available for CI or developer use to regenerate after CLI changes

---
*Phase: 11-desktop-integration-dependency-detection*
*Completed: 2026-03-29*
