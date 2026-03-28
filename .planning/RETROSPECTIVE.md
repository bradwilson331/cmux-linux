# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — Linux Port MVP

**Shipped:** 2026-03-28
**Phases:** 11 | **Plans:** 53 | **Commits:** 2,044

### What Was Built
- Complete Ghostty terminal embedding in GTK4 via Rust FFI (GPU-accelerated OpenGL rendering)
- Full workspace + pane split management with SplitEngine immutable tree
- Wire-compatible v2 JSON-RPC socket API (47 commands) with session persistence
- SSH remote workspaces with bidirectional PTY proxy and reconnect lifecycle
- Agent-browser integration with CDP screencast preview panes
- Native Rust CLI with 34+ subcommands and auto-discovery
- GTK4 HeaderBar, hamburger menu, context menus, and keyboard shortcut customization

### What Worked
- Gap closure pattern: phases produced working code, then gap closure plans fixed integration issues — kept momentum while catching real problems
- Ghostty fork strategy: extending the C API with GHOSTTY_PLATFORM_GTK4 avoided reimplementing terminal rendering
- Reusing cmuxd-remote (Go daemon) from macOS codebase saved significant SSH infrastructure work
- SplitEngine immutable tree design ported cleanly from Swift Bonsplit to Rust
- tokio + glib spawn_local bridge pattern worked reliably for async socket I/O alongside GTK main thread

### What Was Inefficient
- Phase 1 required 9 plans (5 gap closure) — initial plan underestimated Ghostty FFI complexity (linking, GLAD loader, thread safety)
- Multiple phases had stale VERIFICATION.md files showing gaps_found after gaps were already resolved
- Browser dispatch wiring left incomplete (~40 P0/P1 methods advertised but not routed) — scope creep from adding browser without full command wiring
- REQUIREMENTS.md footer became stale (showed 45/47 satisfied when all 47 were done)

### Patterns Established
- RefCell<AppState> on GTK main thread for single-threaded UI model
- SO_PEERCRED uid validation for socket auth on Linux
- SurfaceIoMode enum to share create_surface between local and SSH remote surfaces
- 100ms GTK timer for polling async events (bell notifications, SSH state)
- GIO action system for menu/toolbar/shortcut unification

### Key Lessons
1. Plan for 2-3x gap closure plans on FFI-heavy phases — initial plans discover the real API surface
2. Keep VERIFICATION.md and REQUIREMENTS.md footers in sync after gap closure — stale status creates audit noise
3. Browser integration should wire full dispatch before advertising capabilities — partial wiring creates tech debt
4. The tokio↔GTK bridge (mpsc + spawn_local) is the canonical async pattern for this codebase

### Cost Observations
- Model mix: primarily opus for planning/execution, sonnet for research/exploration
- 65 days wall time, 11 phases, 53 plans
- Notable: gap closure plans were typically fast (1-2 files) compared to foundation plans

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Commits | Phases | Key Change |
|-----------|---------|--------|------------|
| v1.0 | 2,044 | 11 | Initial milestone — established gap closure pattern, FFI bridge patterns |

### Cumulative Quality

| Milestone | Rust LOC | Requirements | Tech Debt Items |
|-----------|----------|-------------|-----------------|
| v1.0 | 9,478 | 47/47 | 14 |

### Top Lessons (Verified Across Milestones)

1. FFI phases need generous gap closure budget — the real API surface emerges during implementation
2. Keep traceability artifacts in sync — stale status wastes audit time
