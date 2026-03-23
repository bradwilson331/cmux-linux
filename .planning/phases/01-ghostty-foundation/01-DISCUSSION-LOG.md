# Phase 1: Ghostty Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 01-ghostty-foundation
**Areas discussed:** Repo layout, Phase 1 window scope, Ghostty fork strategy, libghostty linkage, Bindgen strategy, Tokio/GLib threading model, Clipboard architecture, Error handling

---

## Repo Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Root-level Cargo.toml | Cargo.toml at repo root, src/ next to Swift files. `cargo build` from root. | ✓ |
| linux/ subdirectory | All Rust under linux/. Cleaner separation, requires `cd linux`. | |
| Separate repo / git subtree | Own repo, referenced as submodule or subtree. Most overhead. | |

**User's choice:** Root-level Cargo.toml
**Notes:** Simple setup, macOS Swift files already live at top-level so mixed structure is acceptable.

---

## Phase 1 Window Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Bare terminal window | GtkApplicationWindow + one GtkGLArea. No chrome. Phase 2 adds UI structure. | ✓ |
| Minimal UI stub | Window with placeholder sidebar + header bar, one terminal. Reduces Phase 2 scope. | |

**User's choice:** Bare terminal window
**Notes:** Keeps Phase 1 focused on embedding foundation. No premature UI structure.

---

## Ghostty Fork Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Thin embedded variant | Add GHOSTTY_PLATFORM_GTK4 + ghostty_platform_gtk4_s struct to embedded.zig. Minimal fork diff. | ✓ |
| Research-first spike | Researcher reads ghostty/src/apprt/embedded.zig first to determine correct extension point. | |

**User's choice:** Thin embedded variant
**Notes:** User added: "The key component of this is to provide socket API for agent automation and access to a in-terminal browser." — captures primary motivation for the Linux port.

---

## libghostty Linkage

| Option | Description | Selected |
|--------|-------------|----------|
| build.rs invokes zig build | Cargo build.rs runs zig build, links static archive. Zig is a build-time dep. | |
| Pre-built .a via setup script | setup.sh pre-builds libghostty.a; build.rs links it. Zig not a Cargo dep. | ✓ |
| You decide | Researcher determines best pattern. | |

**User's choice:** Pre-built .a downloaded in setup script
**Notes:** User added: "This must have a socket API for agent automation." — confirms socket API is a hard requirement for the project (Phase 3). Phase 1 architecture must not block it.

---

## Bindgen Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| bindgen at build time | build.rs generates src/ghostty_sys.rs from ghostty.h. Auto-regenerates on header changes. | ✓ |
| Hand-written extern blocks | Only the needed ghostty_* declarations written manually. Zero extra tooling. | |

**User's choice:** bindgen at build time
**Notes:** Auto-generation preferred to keep up with fork evolution as GTK4 support is added.

---

## Tokio / GLib Threading Model

| Option | Description | Selected |
|--------|-------------|----------|
| Defer tokio to Phase 3 | No tokio in Phase 1. GLib async (glib::spawn_future_local) only. | ✓ |
| tokio thread from the start | Placeholder tokio runtime spun up in Phase 1 to validate bridge pattern early. | |

**User's choice:** Defer tokio to Phase 3
**Notes:** Phase 1 has no inter-thread async work. Simpler without tokio/GLib bridge.

---

## Clipboard Architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Async gdk::Clipboard | gdk4::Clipboard::read_text_future() / set_text() via glib::spawn_future_local. GDK handles X11/Wayland. | ✓ |
| wl-clipboard / xclip binaries | Subprocess clipboard utilities. Fragile, requires external tools. | |
| You decide | Researcher determines correct GTK4 pattern. | |

**User's choice:** Async gdk::Clipboard
**Notes:** GDK handles X11/Wayland divergence transparently — no manual protocol branching needed in Phase 1.

---

## Error Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Stderr log + non-zero exit | Print diagnostic to stderr, exit code 1. Developer tool, no GUI dialog needed. | ✓ |
| GTK4 error dialog + clean exit | GtkMessageDialog with error message. More user-friendly, more Phase 1 scope. | |

**User's choice:** Stderr log + non-zero exit
**Notes:** Phase 1 targets developers. GUI error handling deferred.

---

## Claude's Discretion

- Exact Zig build target flags for producing libghostty.a on Linux
- DPI scale factor update signal name and Ghostty API call for monitor switches
- Whether GtkGLArea is sufficient or a custom GDK surface backend is needed

## Deferred Ideas

- In-terminal browser (BROW-01..03) — v2 scope; webkit2gtk-rs embedding is non-trivial
- Socket API (SOCK-01..06) — Phase 3 scope; architecture must not block it
- Tokio/GLib bridge validation — deferred to Phase 3 when tokio is actually needed
- GTK4 error dialogs — future phase concern
