---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Linux Packaging & Distribution
status: Ready to plan
stopped_at: Completed 12.1-02-PLAN.md
last_updated: "2026-03-30T04:19:52.028Z"
progress:
  total_phases: 5
  completed_phases: 3
  total_plans: 8
  completed_plans: 8
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-29)

**Core value:** A Linux user should get the same cmux experience as a Mac user: tabs, splits, workspaces, and socket CLI control -- powered by Ghostty's GPU-accelerated terminal.
**Current focus:** Phase 12.1 — cli-browser-parity-skill-packaging

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
- [Phase 12.1]: Only cmux and cmux-browser skills packaged; excluded cmux-debug-windows, cmux-markdown, release per D-13
- [Phase 12.1]: Skills installed to FHS-compliant /usr/share/cmux/skills/ for agent discoverability
- [Phase 12.1]: Kept BrowserStreamEnable/Disable explicit; consolidated BrowserClose/Snapshot/Screenshot into generic proxy
- [Phase 12.1]: Surface ref pattern: monotonic counter, surface:N short refs, resolve_surface_ref helper
- [Phase 12.1]: Browser commands default to JSON output (--no-json to disable) per agent workflow

### Roadmap Evolution

- v1.0 milestone archived to .planning/milestones/
- v1.1 roadmap: 4 phases (coarse granularity), 25 requirements mapped
- Phase 12.1 inserted after Phase 12: CLI Browser Parity & Skill Packaging (URGENT)

### Pending Todos

None.

### Blockers/Concerns

- Research flags AppImage GTK4 bundling and Flatpak GPU sandbox as areas needing validation (Phase 13)

## Session Continuity

Last session: 2026-03-30T04:13:46.889Z
Stopped at: Completed 12.1-02-PLAN.md
Resume file: None
