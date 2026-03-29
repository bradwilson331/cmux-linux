# cmux Linux Port

## What This Is

A full Linux port of cmux — a GPU-accelerated terminal multiplexer with tabs, pane splits, workspaces, SSH remote sessions, browser automation, and programmatic socket control. Built in Rust on GTK4 with Ghostty as the terminal engine, achieving feature parity with the macOS Swift/AppKit app.

## Core Value

A Linux user should get the same cmux experience as a Mac user: tabs, splits, workspaces, and socket CLI control — powered by Ghostty's GPU-accelerated terminal.

## Requirements

### Validated

- ✓ GPU-accelerated terminal rendering via Ghostty (libghostty C API + GTK4 GtkGLArea) — v1.0
- ✓ Workspace (tab) management — create, close, switch, rename, persist — v1.0
- ✓ Pane splitting — horizontal/vertical splits, drag dividers, immutable tree layout (SplitEngine) — v1.0
- ✓ Unix socket control API — v2 JSON-RPC, 47 commands, SO_PEERCRED auth — v1.0
- ✓ Session persistence — atomic save/restore of full split tree topology with divider ratios — v1.0
- ✓ Configurable keyboard shortcuts via TOML config file — v1.0
- ✓ Notification/attention state — per-pane bell tracking, sidebar indicators, desktop notifications — v1.0
- ✓ SSH remote workspaces — cmuxd-remote deployment, bidirectional PTY proxy, reconnect — v1.0
- ✓ HiDPI/fractional scaling — correct at 1x, 1.5x, 2x with dynamic scale updates — v1.0
- ✓ Agent-browser integration — CDP screencast preview, navigation toolbar — v1.0
- ✓ GTK4 HeaderBar, hamburger menu, context menus, shortcuts window — v1.0
- ✓ Native Rust CLI (`cmux`) with 34+ subcommands and socket auto-discovery — v1.0
- ✓ GitHub Actions CI + AppImage distribution — v1.0

### Active

- ✓ .deb package for Debian/Ubuntu — Validated in Phase 12: Native Packages
- ✓ .rpm package for Fedora/RHEL — Validated in Phase 12: Native Packages
- [ ] AppImage portable binary
- [ ] Flatpak package
- [ ] Local build scripts for each format
- [ ] Gitea CI workflows for automated builds on tag
- [ ] Publish to Gitea package registry + release assets
- ✓ Auto-detect runtime dependencies from binary — Validated in Phase 11: Desktop Integration & Dependency Detection
- ✓ Desktop metadata (.desktop, AppStream metainfo, hicolor icons) — Validated in Phase 11
- ✓ Shell completions (bash, zsh, fish) and man page — Validated in Phase 11

## Current Milestone: v1.1 Linux Packaging & Distribution

**Goal:** Ship cmux as installable packages (.deb, .rpm, AppImage, Flatpak) with local build scripts and Gitea CI pipelines.

**Target features:**
- .deb, .rpm, AppImage, Flatpak packages
- Local shell scripts to build each or all formats
- Gitea-compatible CI workflows (http://192.168.7.6:8418/)
- Publish to Gitea package registry + release assets
- Runtime dependency auto-detection via ldd/readelf

### Out of Scope

- AppleScript support — macOS-only scripting interface
- Sparkle auto-update — replaced by Linux-native update mechanism (e.g. Flatpak/apt)
- Metal/IOSurface rendering — Linux uses OpenGL/Vulkan via Ghostty's GTK4 renderer
- macOS code signing/notarization — not applicable on Linux
- WASM plugin system — premature complexity; not in roadmap

## Context

Shipped v1.0 with 9,478 LOC Rust across 2,044 commits in 65 days.
Tech stack: Rust, GTK4 (gtk4-rs), Ghostty (libghostty C FFI), tokio, serde, clap.
Ghostty fork: manaflow-ai/ghostty with GHOSTTY_PLATFORM_GTK4 platform variant.
Go remote daemon (cmuxd-remote) reused from macOS codebase for SSH workspaces.

**Known tech debt (14 items):**
- SurfaceReadText returns empty (needs Ghostty screen buffer API)
- ~40 browser P0/P1 methods return NotImplemented (dispatch not wired)
- BrowserAction variant unreachable, SshBridge::output_tx dead code
- Human UAT pending: desktop notifications, SSH deploy/retry, HiDPI multi-monitor

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rewrite in Rust (not Swift) | Swift has no Linux UI story; Rust has strong GTK4 bindings | ✓ Good |
| Build on Ghostty's Linux GTK4 frontend | Avoids reinventing terminal surface embedding | ✓ Good |
| Preserve v2 JSON-RPC socket protocol | Cross-platform CLI/tooling compatibility | ✓ Good |
| gtk4-rs (not iced/egui/slint) | Ghostty surfaces require GtkGLArea; no viable alternative | ✓ Good |
| tokio + glib spawn_local bridge | Async socket I/O on tokio, UI on GTK main thread | ✓ Good |
| SplitEngine immutable tree | Port of Bonsplit logic; clean recursive layout model | ✓ Good |
| RefCell<AppState> on GTK main thread | Single-threaded GTK model; Rc<RefCell> avoids Arc overhead | ✓ Good |
| SO_PEERCRED for socket auth | Linux-native uid validation, no token management needed | ✓ Good |
| Agent-browser via CDP screencast | Headless Chrome reuse; WebSocket frame pipeline to GTK Picture | ✓ Good |
| Defer browser dispatch wiring | ~40 P0/P1 methods advertised but not routed; v2 work | ⚠️ Revisit |

## Constraints

- **Tech Stack**: Rust primary language — idiomatic, memory-safe, good GTK4 bindings
- **Terminal Engine**: Ghostty (manaflow-ai fork) — must remain the terminal emulator
- **Protocol Compatibility**: Socket v2 JSON-RPC wire-compatible with macOS cmux
- **UI Toolkit**: GTK4 via gtk4-rs (required for Ghostty GtkGLArea surface hosting)
- **Dependencies**: libghostty C API — Linux build requires Zig 0.15.2 toolchain

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
*Last updated: 2026-03-29 after Phase 12 completion*
