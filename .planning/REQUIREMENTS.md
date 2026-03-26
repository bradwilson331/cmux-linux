# Requirements: cmux Linux Port

**Defined:** 2026-03-23
**Core Value:** A Linux user should get the same cmux experience as a Mac user: tabs, splits, workspaces, and socket CLI control — powered by Ghostty's GPU-accelerated terminal.

## v1 Requirements

### Ghostty Embedding

- [x] **GHOST-01**: manaflow-ai/ghostty fork extended with `GHOSTTY_PLATFORM_GTK4` Linux platform variant (prerequisite for all terminal rendering)
- [x] **GHOST-02**: Single Ghostty terminal surface renders in a GTK4 `GtkGLArea` widget from Rust
- [x] **GHOST-03**: Keyboard input routes to active terminal surface with < 10ms keystroke-to-render latency
- [x] **GHOST-04**: Mouse input (selection, scroll, click) routed to correct terminal surface
- [x] **GHOST-05**: Clipboard integration works on X11 and Wayland (copy/paste to/from terminal)
- [x] **GHOST-06**: Terminal renders at correct DPI (content scale driven from `gtk4::Widget::scale_factor()`)
- [x] **GHOST-07**: `wakeup_cb` dispatches to GLib main loop, never calls `ghostty_*` inline from Ghostty's thread

### Workspace Management

- [x] **WS-01**: User can create a new workspace (tab)
- [x] **WS-02**: User can close a workspace
- [x] **WS-03**: User can switch between workspaces via keyboard shortcut and click
- [x] **WS-04**: User can rename a workspace
- [x] **WS-05**: User can switch to workspace by number (1–9) via keyboard shortcut
- [x] **WS-06**: Workspace list is visible in a sidebar/tab bar

### Pane Splitting

- [x] **SPLIT-01**: User can split the active pane horizontally
- [x] **SPLIT-02**: User can split the active pane vertically
- [x] **SPLIT-03**: User can navigate between panes via keyboard shortcut
- [x] **SPLIT-04**: User can drag dividers to resize panes
- [x] **SPLIT-05**: User can close the active pane
- [x] **SPLIT-06**: Pane layout is represented as an immutable tree (SplitEngine — Bonsplit Rust port)
- [x] **SPLIT-07**: Focus routing: correct pane receives keyboard input; `ghostty_surface_set_focus` called on focus change

### Socket API

- [x] **SOCK-01**: Unix socket server starts at `$XDG_RUNTIME_DIR/cmux/cmux.sock` (mode 0600)
- [x] **SOCK-02**: v2 JSON-RPC protocol is wire-compatible with macOS cmux (same request/response schema)
- [x] **SOCK-03**: `cmux` CLI (macOS or Linux-native) can connect and control the Linux app
- [x] **SOCK-04**: `tests_v2/` Python protocol suite passes against the Linux socket server unmodified
- [x] **SOCK-05**: Socket command policy enforced: non-focus-intent commands never call `gtk_window_present()` or `ghostty_surface_set_focus()`
- [x] **SOCK-06**: Socket authentication: `SO_PEERCRED` uid validation on every connection accept

### Session Persistence

- [x] **SESS-01**: Workspace and pane layout is saved to `~/.local/share/cmux/session.json` on each change (debounced)
- [ ] **SESS-02**: Layout is fully restored on next app launch
- [x] **SESS-03**: Session file is written atomically (write `.tmp`, then `rename()`)
- [x] **SESS-04**: App launches cleanly if session file is missing or corrupted (graceful fallback)

### Configuration

- [x] **CFG-01**: cmux config file loaded from `~/.config/cmux/config.toml` at startup
- [x] **CFG-02**: Keyboard shortcuts are configurable via config file
- [x] **CFG-03**: Ghostty config (colors, font, shell, etc.) is loaded via Ghostty's own config mechanism
- [x] **CFG-04**: XDG Base Directory compliance: config in `$XDG_CONFIG_HOME/cmux/`, data in `$XDG_DATA_HOME/cmux/`, socket in `$XDG_RUNTIME_DIR/cmux/`

### Notification / Attention State

- [x] **NOTF-01**: Per-pane attention state tracks terminal activity (bell, OSC 99 markers)
- [x] **NOTF-02**: Workspace list shows visual indicator for workspaces with unread activity
- [x] **NOTF-03**: Desktop notification sent via GTK4 API when terminal rings bell while app is unfocused

### SSH Remote Workspaces

- [x] **SSH-01**: User can configure a workspace with a remote SSH target
- [x] **SSH-02**: cmuxd-remote Go daemon is deployed to remote host and establishes reverse tunnel
- [ ] **SSH-03**: Terminal sessions in an SSH workspace run on the remote host
- [x] **SSH-04**: SSH workspace reconnect works after network interruption

### HiDPI / Display

- [x] **HDPI-01**: App renders correctly at 1x, 1.5x, and 2x display scale factors
- [x] **HDPI-02**: Scale factor updates when window moves between monitors with different DPI

### Distribution

- [x] **DIST-01**: GitHub Actions CI pipeline: build, clippy lint, unit tests on ubuntu-latest
- [x] **DIST-02**: AppImage artifact produced on each release tag
- [x] **DIST-03**: `.desktop` file included for application launcher integration
- [x] **DIST-04**: App runs on Wayland and X11/XWayland

## v2 Requirements

### Browser Panel

- **BROW-01**: User can open a WebKit browser panel in a pane split alongside terminals
- **BROW-02**: Browser panel supports JavaScript console access
- **BROW-03**: Browser panel has find-in-page UI

### Distribution

- **DIST-05**: .deb package for Debian/Ubuntu
- **DIST-06**: .rpm package for Fedora/RHEL
- **DIST-07**: Flatpak distribution (pending PTY/sandbox investigation)
- **DIST-08**: Automatic update mechanism (Flatpak or self-hosted appcast)

### Input Methods

- **IME-01**: IBus/Fcitx IME preedit support (GTK4 InputMethod API + `ghostty_surface_preedit()`)

### Systemd Integration

- **SYS-01**: Systemd user service unit for auto-starting cmux as a background socket server

## Out of Scope

| Feature | Reason |
|---------|--------|
| AppleScript | macOS-only scripting interface — no Linux equivalent |
| Sparkle auto-update | macOS-only; Linux uses Flatpak/apt/AppImage update mechanisms |
| Metal/IOSurface rendering | macOS GPU APIs; Linux uses OpenGL/Vulkan via Ghostty's GTK4 renderer |
| macOS code signing/notarization | Not applicable on Linux |
| iced/egui/slint UI | Eliminated — cannot host GTK4 GdkSurface; Ghostty embedding requires GTK4 |
| tmux compatibility mode | Two protocol layers forever; scope creep; cmux socket API is the identity |
| WASM plugin system | Premature complexity; not in roadmap for v1 or v2 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| GHOST-01 | Phase 1 | Complete |
| GHOST-02 | Phase 1 | Complete |
| GHOST-03 | Phase 1 | Complete |
| GHOST-04 | Phase 1 | Complete |
| GHOST-05 | Phase 1 | Complete |
| GHOST-06 | Phase 1 | Complete |
| GHOST-07 | Phase 1 | Complete |
| WS-01 | Phase 2 | Complete |
| WS-02 | Phase 2 | Complete |
| WS-03 | Phase 2 | Complete |
| WS-04 | Phase 2 | Complete |
| WS-05 | Phase 2 | Complete |
| WS-06 | Phase 2 | Complete |
| SPLIT-01 | Phase 2 | Complete |
| SPLIT-02 | Phase 2 | Complete |
| SPLIT-03 | Phase 2 | Complete |
| SPLIT-04 | Phase 2 | Complete |
| SPLIT-05 | Phase 2 | Complete |
| SPLIT-06 | Phase 2 | Complete |
| SPLIT-07 | Phase 2 | Complete |
| SOCK-01 | Phase 3 | Complete |
| SOCK-02 | Phase 3 | Complete |
| SOCK-03 | Phase 3 | Complete |
| SOCK-04 | Phase 3 | Complete |
| SOCK-05 | Phase 3 | Complete |
| SOCK-06 | Phase 3 | Complete |
| SESS-01 | Phase 3 | Complete |
| SESS-02 | Phase 6 | Pending |
| SESS-03 | Phase 3 | Complete |
| SESS-04 | Phase 3 | Complete |
| NOTF-01 | Phase 4 | Complete |
| NOTF-02 | Phase 4 | Complete |
| NOTF-03 | Phase 4 | Complete |
| HDPI-01 | Phase 4 | Complete |
| HDPI-02 | Phase 4 | Complete |
| SSH-01 | Phase 4 | Complete |
| SSH-02 | Phase 4 | Complete |
| SSH-03 | Phase 7 | Pending |
| SSH-04 | Phase 4 | Complete |
| CFG-01 | Phase 5 | Complete |
| CFG-02 | Phase 5 | Complete |
| CFG-03 | Phase 5 | Complete |
| CFG-04 | Phase 5 | Complete |
| DIST-01 | Phase 5 | Complete |
| DIST-02 | Phase 5 | Complete |
| DIST-03 | Phase 5 | Complete |
| DIST-04 | Phase 5 | Complete |

**Coverage:**
- v1 requirements: 47 total
- Mapped to phases: 47
- Satisfied: 45
- Pending (gap closure): 2 (SESS-02, SSH-03)
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-23*
*Last updated: 2026-03-23 — traceability updated to match 5-phase roadmap*
