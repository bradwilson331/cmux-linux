---
phase: 12-native-packages-deb-rpm
plan: 01
subsystem: packaging
tags: [deb, dpkg, debian, ubuntu, packaging, FHS]

# Dependency graph
requires:
  - phase: 11-desktop-integration
    provides: desktop metadata, icons, completions, man page, detect-deps.sh
provides:
  - .deb packaging script (build-deb.sh) producing installable Debian packages
  - .deb validation script (validate-deb.sh) verifying package structure
affects: [12-02-rpm-packaging, 14-ci-workflows]

# Tech tracking
tech-stack:
  added: [dpkg-deb]
  patterns: [FHS binary layout, DEBIAN/control metadata, staging-dir-with-trap]

key-files:
  created:
    - packaging/scripts/build-deb.sh
    - packaging/scripts/validate-deb.sh
  modified: []

key-decisions:
  - "Non-t64 package names for Ubuntu 22.04 compatibility (libglib2.0-0 not libglib2.0-0t64)"
  - "eval-based check() in validate-deb.sh to support piped commands in validation"

patterns-established:
  - "Staging directory with trap cleanup for package assembly"
  - "Positional args with defaults for binary paths in build scripts"

requirements-completed: [DEB-01, DEB-02, DEB-03, DEB-04]

# Metrics
duration: 4min
completed: 2026-03-29
---

# Phase 12 Plan 01: .deb Packaging Summary

**dpkg-deb packaging script with FHS layout for 3 binaries, desktop metadata, icons, completions, man page, and runtime dependency declarations for GTK4 stack**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-29T17:47:07Z
- **Completed:** 2026-03-29T17:51:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Created build-deb.sh that assembles a complete .deb from pre-built binaries with all FHS-correct install paths
- Declared 15 runtime dependencies covering GTK4, GL, font rendering, and input handling
- Created validate-deb.sh that verifies 20 package properties (12 file paths + 8 metadata fields)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create .deb packaging script** - `6a235b46` (feat)
2. **Task 2: Create .deb validation script** - `6d5e94e1` (feat)

## Files Created/Modified
- `packaging/scripts/build-deb.sh` - Assembles .deb from pre-built binaries via dpkg-deb --build --root-owner-group
- `packaging/scripts/validate-deb.sh` - Validates .deb file listing and DEBIAN/control metadata

## Decisions Made
- Used non-t64 package names (e.g., libglib2.0-0) for Ubuntu 22.04 compatibility; these are virtual packages on 24.04
- Used eval-based check() helper in validate-deb.sh to support piped grep commands (matching validate-all.sh pattern would not work with pipes)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed validate-deb.sh check() function for piped commands**
- **Found during:** Task 2
- **Issue:** Initial implementation used "$@" execution pattern from validate-all.sh, but piped commands (echo | grep) don't work with that pattern since pipes are shell syntax
- **Fix:** Switched to eval-based check() as specified in the plan's action section
- **Files modified:** packaging/scripts/validate-deb.sh
- **Verification:** bash -n passes, eval approach correctly handles piped commands

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Caught before commit. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- .deb packaging scripts ready for integration with CI workflows
- validate-deb.sh can be used in CI to verify package builds
- RPM packaging (plan 02) can follow the same staging-dir pattern

---
*Phase: 12-native-packages-deb-rpm*
*Completed: 2026-03-29*
