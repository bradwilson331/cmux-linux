---
phase: 11-desktop-integration-dependency-detection
plan: 03
subsystem: infra
tags: [bash, ldd, dpkg, rpm, dependency-detection, packaging]

requires:
  - phase: none
    provides: standalone script (no prior phase dependency)
provides:
  - dependency detection script mapping ldd output to Debian and Fedora package names
  - TSV and JSON output modes for human and machine consumption
affects: [12-deb-packaging, 13-rpm-packaging, 14-appimage-flatpak]

tech-stack:
  added: [ldd, dpkg -S, rpm -qf, ldconfig]
  patterns: [static fallback table for cross-distro package name resolution]

key-files:
  created:
    - packaging/scripts/detect-deps.sh
  modified: []

key-decisions:
  - "Dual fallback tables: native package manager when available, static table for the other distro"
  - "30+ library-to-package mappings covering GTK4 stack, graphics, text rendering"
  - "System library skip pattern filters libc, libm, libpthread, ld-linux, libgcc_s, libstdc++"

patterns-established:
  - "Cross-distro packaging scripts: always produce both Debian and Fedora output regardless of host"

requirements-completed: [BUILD-02]

duration: 3min
completed: 2026-03-29
---

# Phase 11 Plan 03: Dependency Detection Script Summary

**Bash script mapping ldd shared library output to Debian and Fedora package names with TSV and JSON modes**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-29T17:18:48Z
- **Completed:** 2026-03-29T17:22:33Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Created detect-deps.sh that runs ldd on any binary and maps shared libraries to distro package names
- Native dpkg -S resolution on Debian, native rpm -qf on Fedora, static fallback tables for the other
- 30+ library mappings covering GTK4, glib, pango, cairo, fontconfig, freetype, harfbuzz, oniguruma, mesa, wayland, X11, vulkan
- Both TSV (human-readable) and JSON (machine-parseable) output modes
- Summary section listing unique Debian and Fedora package names

## Task Commits

Each task was committed atomically:

1. **Task 1: Create dependency detection script** - `d317bbbf` (feat)

## Files Created/Modified
- `packaging/scripts/detect-deps.sh` - Executable bash script that maps ldd output to Debian/Fedora package names

## Decisions Made
- Used dual associative arrays (FEDORA_FALLBACK, DEBIAN_FALLBACK) for cross-distro resolution without requiring both package managers
- Filtered system libraries via regex pattern (libc, libm, libpthread, etc.) to keep output focused on actual dependencies
- JSON mode outputs array of objects with library/debian/fedora keys for easy parsing by packaging scripts

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- cmux-app binary could not be built in this worktree (Ghostty linker error), so testing was performed against /usr/bin/gnome-text-editor (a GTK4 application) which exercises the same library detection paths for GTK4, fontconfig, freetype, and other key dependencies

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- detect-deps.sh ready for use by .deb packaging (phase 12), .rpm packaging (phase 13), and AppImage/Flatpak (phase 14)
- Script will discover actual cmux-app dependencies once binary is built in packaging environment

## Self-Check: PASSED

- FOUND: packaging/scripts/detect-deps.sh (executable)
- FOUND: commit d317bbbf
- FOUND: 11-03-SUMMARY.md

---
*Phase: 11-desktop-integration-dependency-detection*
*Completed: 2026-03-29*
