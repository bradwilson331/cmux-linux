# cmux Linux Port

## What This Is

A full Linux port of cmux — a GPU-accelerated terminal multiplexer with tabs, pane splits, workspaces, and programmatic socket control. The macOS version is built on Swift/SwiftUI/AppKit + GhosttyKit; the Linux port is a ground-up rewrite in Rust using Ghostty's existing Linux GTK4 frontend as the terminal engine, targeting full feature parity with the macOS app.

## Core Value

A Linux user should get the same cmux experience as a Mac user: tabs, splits, workspaces, and socket CLI control — powered by Ghostty's GPU-accelerated terminal.

## Requirements

### Validated

<!-- These represent what the macOS app already delivers — the target for the Linux port. -->

- ✓ GPU-accelerated terminal rendering via Ghostty (libghostty C API) — existing (macOS)
- ✓ Workspace (tab) management — create, close, switch, persist — existing (macOS)
- ✓ Pane splitting — horizontal/vertical splits, drag dividers, tree-based layout — existing (macOS)
- ✓ Unix socket control API — v1 text protocol + v2 JSON-RPC for scripting/automation — existing (macOS)
- ✓ Session persistence — restore workspaces/panes on relaunch — existing (macOS)
- ✓ Configurable keyboard shortcuts — existing (macOS)
- ✓ Notification/attention state — track terminal activity per pane — existing (macOS)
- ✓ SSH remote workspaces via cmuxd-remote daemon — existing (macOS)

### Active

- [x] Rust project scaffolding with GTK4 bindings (gtk4-rs) wrapping Ghostty terminal surfaces — Validated in Phase 01: ghostty-foundation
- [ ] Tab/workspace management in Rust — create, close, switch, named workspaces
- [ ] Pane splitting in Rust — horizontal/vertical splits, divider dragging, tree layout (port Bonsplit logic)
- [ ] Unix socket server in Rust — v2 JSON-RPC protocol compatibility with macOS cmux
- [ ] Keyboard shortcut configuration — load from config file, bind to actions
- [ ] Session persistence — save/restore workspace+pane layout to JSON on disk
- [ ] Terminal notification/attention state — activity detection per pane
- [ ] CI/CD for Linux (GitHub Actions, Ubuntu runners)
- [ ] Distribution packaging — .deb, .AppImage, or Flatpak

### Out of Scope

- AppleScript support — macOS-only scripting interface
- Sparkle auto-update — replaced by Linux-native update mechanism (e.g. Flatpak/apt)
- Metal/IOSurface rendering — Linux uses OpenGL/Vulkan via Ghostty's GTK4 renderer
- macOS code signing/notarization — not applicable on Linux
- Browser panel (WebKit) — deferred; WebKit embedding in GTK4/Rust is complex, not core

## Context

**Existing macOS codebase:**
- Swift + SwiftUI + AppKit hybrid, macOS 13+
- Ghostty embedded via `GhosttyKit.xcframework` (built from `ghostty/` submodule — manaflow-ai fork)
- Pane layout via `vendor/bonsplit` (Swift library — needs Rust reimplementation)
- Socket control: `TerminalController.swift` handles v1/v2 protocol
- Zig daemon (`cmuxd`) handles CLI — may be partially reusable on Linux
- Go remote daemon (`daemon/remote/`) for SSH relay — platform-agnostic, potentially reusable

**Ghostty on Linux:**
- Ghostty already ships a GTK4 Linux frontend — the terminal engine is cross-platform
- cmux-linux will build on top of Ghostty's Linux app architecture rather than re-embedding from scratch
- libghostty C API is the integration point (same as macOS GhosttyKit)

**Key technical unknowns:**
- How to embed multiple Ghostty terminal surfaces in a GTK4 + Rust application
- Whether iced (Elm-style) or gtk4-rs is the right UI layer (iced is preferred but GTK4 may be necessary for Ghostty surface embedding)
- cmuxd (Zig) reuse on Linux vs. Rust reimplementation

## Constraints

- **Tech Stack**: Rust primary language — idiomatic, memory-safe, good GTK4 bindings available
- **Terminal Engine**: Ghostty (manaflow-ai fork) — must remain the terminal emulator, not replaced
- **Protocol Compatibility**: Socket v2 JSON-RPC must be wire-compatible with macOS cmux so shared tooling works across platforms
- **UI Toolkit**: GTK4 via gtk4-rs likely required for Ghostty surface hosting, even if iced is preferred for application logic
- **Dependencies**: libghostty C API — Linux build requires Zig toolchain (same as macOS)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rewrite in Rust (not Swift) | Swift has no Linux UI story; Rust has strong GTK4 bindings and system programming capability | — Pending |
| Build on Ghostty's Linux GTK4 frontend | Ghostty already solves terminal rendering on Linux; avoids reinventing surface embedding | — Pending |
| Preserve v2 JSON-RPC socket protocol | Cross-platform tooling compatibility; existing CLI scripts work on both Mac and Linux | — Pending |
| Defer browser panel | WebKit/GTK4 embedding is non-trivial; not core to terminal multiplexer value | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-23 after initialization*
