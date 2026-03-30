---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Linux Packaging & Distribution
status: Ready to plan
stopped_at: Phase 12.1 context gathered
last_updated: "2026-03-30T03:51:03.122Z"
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 5
  completed_plans: 5
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-29)

**Core value:** A Linux user should get the same cmux experience as a Mac user: tabs, splits, workspaces, and socket CLI control -- powered by Ghostty's GPU-accelerated terminal.
**Current focus:** Phase 12 — native-packages-deb-rpm

## Current Position

Phase: 13
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
- [Phase 12]: AutoReqProv disabled for pre-built binary RPM packaging with explicit Fedora Requires
- [Phase 12]: Non-t64 package names for Ubuntu 22.04 compat (libglib2.0-0 not libglib2.0-0t64)

### Roadmap Evolution

- v1.0 milestone archived to .planning/milestones/
- v1.1 roadmap: 4 phases (coarse granularity), 25 requirements mapped
- Phase 12.1 inserted after Phase 12: CLI Browser Parity & Skill Packaging (URGENT)

### Pending Todos

None.

### Blockers/Concerns

- Research flags AppImage GTK4 bundling and Flatpak GPU sandbox as areas needing validation (Phase 13)

## Session Continuity

Last session: 2026-03-30T03:51:03.120Z
Stopped at: Phase 12.1 context gathered
Resume file: .planning/phases/12.1-cli-browser-parity-skill-packaging/12.1-CONTEXT.md
