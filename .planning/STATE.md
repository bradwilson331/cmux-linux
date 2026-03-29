---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Linux Packaging & Distribution
status: Ready to plan
stopped_at: Phase 12 context gathered
last_updated: "2026-03-29T17:31:52.190Z"
progress:
  total_phases: 4
  completed_phases: 1
  total_plans: 3
  completed_plans: 3
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-29)

**Core value:** A Linux user should get the same cmux experience as a Mac user: tabs, splits, workspaces, and socket CLI control -- powered by Ghostty's GPU-accelerated terminal.
**Current focus:** Phase 12 — native-packages-(.deb-+-.rpm)

## Current Position

Phase: 12
Plan: Not started

## Performance Metrics

**Velocity:**

- Total plans completed: 52 (v1.0)
- v1.1 plans completed: 3

*Updated after each plan completion*

## Accumulated Context

### Decisions

All v1.0 decisions archived in milestones/v1.0-phases/ and PROJECT.md Key Decisions table.

- [Phase 11]: Used underscore in reverse-DNS ID (com.cmux_lx.terminal) - appstreamcli rejects hyphens
- [Phase 11]: Created src/lib.rs exposing only pub mod cli for generator binary access without GTK4 deps
- [Phase 11]: Standalone generator binary pattern (not build.rs) for shell completions and man page
- [Phase 11]: Dual fallback tables for cross-distro package name resolution (native manager + static table)

### Roadmap Evolution

- v1.0 milestone archived to .planning/milestones/
- v1.1 roadmap: 4 phases (coarse granularity), 25 requirements mapped

### Pending Todos

None.

### Blockers/Concerns

- Research flags AppImage GTK4 bundling and Flatpak GPU sandbox as areas needing validation (Phase 13)

## Session Continuity

Last session: 2026-03-29T17:31:52.187Z
Stopped at: Phase 12 context gathered
Resume file: .planning/phases/12-native-packages-deb-rpm/12-CONTEXT.md
