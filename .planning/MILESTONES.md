# Milestones

## v1.0 Linux Port MVP (Shipped: 2026-03-28)

**Phases:** 11 phases (1-10 + 7.1), 53 plans, 83 tasks
**Timeline:** 65 days (2026-01-22 → 2026-03-28)
**Commits:** 2,044 | **Rust LOC:** 9,478

**Key accomplishments:**

1. Ghostty terminal engine embedded in GTK4 via Rust FFI with GPU-accelerated OpenGL rendering
2. Full workspace management + recursive pane splitting with drag-to-resize dividers (SplitEngine)
3. Wire-compatible v2 JSON-RPC socket API (47 commands) with SO_PEERCRED auth and session persistence
4. SSH remote workspaces with cmuxd-remote deployment, bidirectional PTY proxy, and reconnect lifecycle
5. Agent-browser integration with CDP screencast preview panes and navigation toolbar
6. Native Rust CLI (`cmux`) with 34+ subcommands, human-readable + JSON output, and socket auto-discovery

**Requirements:** 47/47 satisfied
**Tech debt:** 14 items across 7 phases (non-critical)
**Audit:** See milestones/v1.0-MILESTONE-AUDIT.md

---
