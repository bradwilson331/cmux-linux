# Phase 3: Discussion Log

**Date:** 2026-03-25
**Phase:** 03-socket-api-session-persistence

---

## Area: tests_v2 scope

**Q:** Which tests_v2 tests must pass for Phase 3?
- Options: Core socket tests only / All 101 eventually / Protocol compliance tests only
- **Selected:** All 101 eventually

**Q:** For browser and command palette tests (require unimplemented features)?
- Options: Phase 3 targets core, browser/palette in Phase 4/5 / Stub all 101 / Include browser+palette in Phase 3
- **Selected:** Phase 3 targets core socket+session; browser/palette pass in Phase 4/5
- **Note:** User mentioned integrating `@agent-browser` in a future phase

---

## Area: CLI tool strategy

**Q:** What is the Linux CLI for Phase 3?
- Options: Python client (cmux.py) / cmuxd Zig binary / New Rust binary
- **Selected:** Python client (tests_v2/cmux.py)

**Q:** How should cmux.py handle Linux socket paths?
- Options: Update cmux.py to check XDG paths / CMUX_SOCKET env var only / Fork as cmux_linux.py
- **Selected:** Update cmux.py to check XDG paths on Linux

---

## Area: Session restore depth

**Q:** What gets saved to session.json?
- Options: Full layout (workspace + pane tree + CWD) / Workspace names + pane count / Workspace names only
- **Selected:** Full layout: workspace names + pane tree + shell CWD

**Q:** When does the session get saved?
- Options: Debounced 500ms / On app exit only / Timer every 30 seconds
- **Selected:** Debounced on every change, 500ms delay

---

## Area: GLib bridge upgrade

**Q:** Which bridge pattern for tokio → GTK socket command dispatch?
- Options: glib::MainContext::channel (event-driven) / Keep polling 5ms / glib::idle_add per command
- **Selected:** glib::MainContext::channel (event-driven)

---

## Area: v2 protocol command priority

**Q:** Which methods must be fully implemented vs. stubbed in Phase 3?
- Options: Tier-based (core first) / All 50 methods / Minimal (just test_ctrl_socket.py)
- **Selected:** Tier-based: core first, stubs for the rest

---

## Area: SO_PEERCRED + socket auth

**Q:** Linux socket authentication approach?
- Options: uid-only via SO_PEERCRED / Implement HMAC-SHA256 too / No auth in Phase 3
- **Selected:** uid-only via SO_PEERCRED

---

## Area: Socket path at $XDG_RUNTIME_DIR

**Q:** Fallback if XDG_RUNTIME_DIR is unset?
- Options: /run/user/{uid}/cmux/cmux.sock / /tmp/cmux-{uid}.sock / Error out
- **Selected:** /run/user/{uid}/cmux/cmux.sock fallback
