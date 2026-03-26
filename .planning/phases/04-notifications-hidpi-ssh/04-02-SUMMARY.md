---
phase: 04-notifications-hidpi-ssh
plan: 02
subsystem: ui
tags: [hidpi, fractional-scaling, gtk4, wayland, ghostty]

# Dependency graph
requires:
  - phase: 01-ghostty-foundation
    provides: "Ghostty surface embedding with GLArea, scale-factor handling"
provides:
  - "Verified HiDPI rendering with fractional scale support via GdkSurface::scale()"
  - "Diagnostic logging for scale-factor changes"
affects: []

# Tech tracking
tech-stack:
  added: ["gdk4 v4_12 feature (fractional scale API)"]
  patterns: ["Use GdkSurface::scale() for fractional DPI instead of Widget::scale_factor() integer ceiling"]

key-files:
  created: []
  modified:
    - src/ghostty/surface.rs
    - Cargo.toml

key-decisions:
  - "Enabled gdk4 v4_12 feature to access GdkSurface::scale() for Wayland fractional scaling"
  - "Integer scale_factor() used as fallback when GdkSurface is unavailable (X11 pre-realize)"

patterns-established:
  - "Fractional scale pattern: widget.native().surface().scale() with integer fallback"

requirements-completed: [HDPI-01, HDPI-02]

# Metrics
duration: 4min
completed: 2026-03-26
---

# Phase 04 Plan 02: HiDPI Rendering Summary

**Fractional scale support via GdkSurface::scale() with v4_12 feature, verified existing integer scale paths and CSS**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-26T12:54:34Z
- **Completed:** 2026-03-26T12:58:34Z
- **Tasks:** 1
- **Files modified:** 3

## Accomplishments
- Verified existing notify::scale-factor handler correctly updates Ghostty content scale on monitor change
- Upgraded scale-factor handler to use fractional GdkSurface::scale() (f64) instead of integer ceiling Widget::scale_factor()
- Added diagnostic eprintln for scale-factor changes to aid field debugging
- Verified no hardcoded 1.0 scale factor in surface creation paths
- Verified CSS scales correctly via GTK4 native handling (no HiDPI issues)

## Task Commits

Each task was committed atomically:

1. **Task 1: Audit and fix HiDPI handling in surface.rs and CSS** - `2465788e` (feat)

## Files Created/Modified
- `src/ghostty/surface.rs` - Updated notify::scale-factor handler with fractional scale and diagnostic logging
- `Cargo.toml` - Enabled gdk4 v4_12 feature for GdkSurface::scale()
- `Cargo.lock` - Updated lockfile for new feature flag

## Decisions Made
- Enabled gdk4 v4_12 feature: system GTK is 4.14.5, method is available, enables Wayland fractional scaling (1.25x, 1.5x etc.)
- Integer Widget::scale_factor() kept as fallback via unwrap_or for cases where GdkSurface is not yet available

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Initial attempt to import SurfaceExt trait inline caused trait resolution error; resolved by enabling v4_12 feature flag which makes scale() available through the prelude

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- HiDPI verified and working, ready for remaining Phase 04 plans
- Fractional scaling pattern established for any future surface creation code

---
*Phase: 04-notifications-hidpi-ssh*
*Completed: 2026-03-26*
