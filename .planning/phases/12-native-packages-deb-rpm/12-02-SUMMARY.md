---
phase: 12-native-packages-deb-rpm
plan: 02
subsystem: packaging
tags: [rpm, fedora, rpmbuild, spec]

# Dependency graph
requires:
  - phase: 11-desktop-integration
    provides: desktop metadata, icons, completions, man page, detect-deps.sh FEDORA_FALLBACK table
provides:
  - RPM spec file with 15 Fedora dependency declarations
  - build-rpm.sh script for building .rpm from pre-built binaries
  - validate-rpm.sh script for verifying .rpm structure and dependencies
affects: [14-gitea-ci, packaging-validation]

# Tech tracking
tech-stack:
  added: [rpmbuild]
  patterns: [pre-built binary RPM packaging via --define injection]

key-files:
  created:
    - packaging/rpm/cmux.spec
    - packaging/scripts/build-rpm.sh
    - packaging/scripts/validate-rpm.sh
  modified: []

key-decisions:
  - "AutoReqProv disabled -- pre-built binaries need explicit dependency declarations"
  - "Version injected via rpmbuild --define, not hardcoded in spec"
  - "15 Requires from FEDORA_FALLBACK table including mesa-libEGL, libepoxy, libxkbcommon, graphene"

patterns-established:
  - "RPM spec uses _sourcedir flat staging with rpmbuild --define for version injection"
  - "Validation scripts use temp files for rpm query caching with grep-based checks"

requirements-completed: [RPM-01, RPM-02, RPM-03]

# Metrics
duration: 2min
completed: 2026-03-29
---

# Phase 12 Plan 02: RPM Packaging Summary

**RPM spec with 15 Fedora Requires, rpmbuild -bb build script, and rpm -qp validation script**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-29T17:47:06Z
- **Completed:** 2026-03-29T17:49:22Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- RPM spec file with complete Fedora dependency declarations from detect-deps.sh FEDORA_FALLBACK table
- Build script that stages all binaries, desktop metadata, icons, completions, and gzipped man page into rpmbuild SOURCES
- Validation script that checks file listing (12 paths), metadata, and dependencies (9 packages) using rpm -qp queries

## Task Commits

Each task was committed atomically:

1. **Task 1: Create RPM spec file and build script** - `85f3a348` (feat)
2. **Task 2: Create .rpm validation script** - `9c8a7c5b` (feat)

## Files Created/Modified
- `packaging/rpm/cmux.spec` - RPM spec with 15 Requires, %install for all files, %files manifest
- `packaging/scripts/build-rpm.sh` - Stages binaries and metadata, invokes rpmbuild -bb with version injection
- `packaging/scripts/validate-rpm.sh` - Validates .rpm file listing, metadata, and dependencies

## Decisions Made
- AutoReqProv disabled since pre-built binaries need explicit dependency declarations rather than auto-detection
- Version injected at build time via `--define "_cmux_version ${VERSION}"` rather than hardcoded in spec
- Included 15 Requires (beyond the 9 core) to cover mesa-libEGL, cairo-gobject, gdk-pixbuf2, libepoxy, libxkbcommon, graphene
- Validation script caches rpm query output to temp files for reliable grep-based checking (avoids pipe issues with check() function)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed pipe incompatibility in validate-rpm.sh check() function**
- **Found during:** Task 2
- **Issue:** Plan suggested `echo "$VAR" | grep -q` pattern inside check() but the `"$@"` execution pattern doesn't support shell pipes
- **Fix:** Used temp files for rpm query caching with direct `grep -q pattern file` calls
- **Files modified:** packaging/scripts/validate-rpm.sh
- **Verification:** bash -n syntax check passes
- **Committed in:** 9c8a7c5b

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Essential fix for script correctness. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- RPM packaging artifacts ready for CI integration (Phase 14)
- build-rpm.sh can be invoked after cargo build to produce installable .rpm
- validate-rpm.sh can verify CI-built packages

---
*Phase: 12-native-packages-deb-rpm*
*Completed: 2026-03-29*
