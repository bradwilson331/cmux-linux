# Feature Research

**Domain:** Linux GPU-accelerated terminal multiplexer (cmux Linux port)
**Researched:** 2026-03-23
**Confidence:** MEDIUM — web search tools unavailable; analysis based on training knowledge of tmux/zellij/wezterm/kitty ecosystems (knowledge cutoff Aug 2025) plus codebase review. Flags where live verification needed.

---

## Linux Terminal Multiplexer Ecosystem Baseline

Before categorizing cmux features, the context for what "terminal multiplexer" means to Linux users:

**The two major paradigms:**

1. **TUI multiplexers** (tmux, zellij, screen): Run inside a single terminal emulator window. All UI is text/escape sequences. Sessions are server-backed — survive terminal close, SSH disconnect, and machine suspend. Keyboard-driven, no mouse required.

2. **GUI terminal emulators with multiplexing** (wezterm, kitty, ghostty, alacritty): Native GPU-rendered window with tab/pane support baked in. No session survival (close window = sessions die unless wrapped in tmux). Mouse-friendly. Richer rendering.

cmux occupies a third category: **GUI terminal emulator with workspace management and programmatic socket control** — closer to wezterm/kitty in rendering but adds the scripting story of tmux. This is the key differentiator.

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features Linux terminal users assume exist. Missing = product feels broken or unshippable.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Multiple tabs / workspaces | Every modern terminal (wezterm, kitty, ghostty) has tabs; tmux has windows. Minimum unit. | LOW | Already in macOS cmux; needs Rust/GTK4 reimplementation |
| Horizontal and vertical pane splits | tmux, zellij, wezterm, kitty all support this. Users cannot imagine a multiplexer without it. | MEDIUM | Bonsplit logic needs Rust port |
| Keyboard shortcut navigation | All pane/tab navigation must be keyboard-accessible. Mouse-only is unacceptable for power users. | LOW | Config file loading + keybinding engine needed |
| Per-pane working directory tracking | Expected from all modern terminals. cmux macOS likely does this via Ghostty shell integration. | LOW | Depends on Ghostty shell integration on Linux |
| Copy/paste (clipboard integration) | Broken clipboard = product is unusable. xclip/xsel/wl-clipboard on Linux vs pbpaste on macOS. | MEDIUM | Linux clipboard is split: X11 (xclip) vs Wayland (wl-copy). Must handle both. |
| Scrollback buffer | Every terminal since 1985. Users expect unlimited (or very large) scrollback. | LOW | Handled by Ghostty's terminal engine |
| Mouse support (click to focus, drag dividers) | wezterm, kitty, ghostty all have this. Linux GUI terminal users expect mouse. | MEDIUM | Divider dragging is the complex part; GTK4 pointer events |
| Configurable color themes / appearance | Ghostty config file handles this; cmux should expose/pass through it. | LOW | Ghostty config passthrough |
| Unicode and emoji rendering | Linux developers use emoji in commit messages, scripts, tmux status bars. Breaking Unicode = hard dealbreaker. | LOW | Ghostty handles rendering layer |
| Session persistence across restarts | Not universal in TUI multiplexers (requires tmux-resurrect plugin), but GUI apps universally do this. | MEDIUM | Already in macOS cmux (session.json); port to XDG paths |
| Configurable keyboard shortcuts | tmux has `bind-key`, zellij has keybindings config, kitty has `kitty.conf`. Expected. | LOW | Config file parsing in Rust |
| Named workspaces | tmux windows are named; zellij tabs are named; wezterm workspaces are named. | LOW | macOS cmux has this |
| Multiple windows (OS-level) | Expected from a GUI app. Opening a second window should work. | LOW | GTK4 multi-window |
| XDG Base Directory compliance | Linux users expect config in `~/.config/cmux/`, data in `~/.local/share/cmux/`. violating XDG is a reddit complaint magnet. | LOW | Must move away from `~/.cmux/` macOS convention to `~/.config/cmux/` and `~/.local/share/cmux/` |
| Unix socket control API | This is cmux's identity feature. Present in macOS. Linux users scripting with tmux expect programmatic control. | HIGH | v2 JSON-RPC protocol compatibility required |
| Terminal activity / bell notification | tmux has activity monitoring; users expect to know when background panes produce output. | MEDIUM | macOS cmux has notification/attention state |
| Resizable dividers (drag or keyboard) | wezterm and kitty allow resize by keyboard and mouse. Drag dividers = table stakes for a GUI mux. | MEDIUM | Bonsplit drag logic |
| Shell integration (OSC sequences) | Ghostty has shell integration for cwd tracking, prompt marks, command completion. Linux users expect this. | LOW | Ghostty shell integration works on Linux; just needs to be wired up |
| Ligatures and font rendering | Ghostty handles font ligatures. Linux users care about this (Cascadia Code, Fira Code). | LOW | Ghostty engine feature |

### Differentiators (Competitive Advantage)

Features that set cmux apart from tmux, zellij, wezterm, kitty on Linux.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| GPU-accelerated terminal via Ghostty | wezterm and kitty are also GPU-accelerated. But Ghostty is newer, has excellent performance, and is growing in reputation. Using Ghostty gives cmux a strong rendering story. | HIGH | Core dependency — already committed. Zig toolchain required for Linux Ghostty build. |
| v2 JSON-RPC socket API with wire compatibility across macOS and Linux | tmux has a control mode; zellij is adding plugins; neither has a rich JSON-RPC socket API with full workspace/pane/surface addressing. Cross-platform socket compatibility is unique. | HIGH | Must maintain v2 protocol wire compatibility with macOS cmux |
| Workspace-centric model (named, persistent, first-class) | tmux sessions/windows hierarchy is confusing to new users. Zellij has sessions. cmux's workspace abstraction is cleaner and more persistent. | MEDIUM | Already designed in macOS cmux |
| Tree-based pane layout (Bonsplit semantics) | tmux uses ad-hoc splits; zellij has layouts but complex; cmux's tree layout is predictable and restorable. | HIGH | Bonsplit Rust port is the hard part |
| SSH remote workspaces via cmuxd-remote | Neither wezterm nor kitty offer this. tmux does survive SSH sessions but doesn't have a relay daemon with structured RPC. This is a significant differentiator for devops users. | HIGH | Go remote daemon may be reusable as-is on Linux |
| Scriptable with `cmux` CLI (send text, manage windows) | tmux has `tmux send-keys`; cmux's socket CLI is richer and more composable. With CMUX_WORKSPACE_ID env var, background agents can control specific workspaces. | HIGH | Socket server is core; CLI is thin wrapper |
| Session persistence with exact layout restore | tmux-resurrect/continuum is a third-party plugin. zellij has session persistence. wezterm/kitty do not persist across restarts. cmux baking this in is a genuine differentiator for GUI terminals. | MEDIUM | session.json port to Linux XDG paths |
| Terminal activity notifications with visual ring animation | Unique UI touch. tmux activity-monitor is text-only. cmux's attention state with ring animation is GUI-native. | MEDIUM | macOS cmux has this; needs GTK4 equivalent |
| Configurable numeric workspace shortcuts | macOS cmux added customizable number shortcuts (recent commit). Power users map workspaces to digits. | LOW | Already shipped on macOS |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Wayland-only support (drop X11) | Wayland is the future; X11 is legacy. | In 2026, major distros still default to X11 (RHEL 9, Ubuntu LTS) or run XWayland for compatibility. Dropping X11 locks out a large user base. GTK4 runs on both via native backends. | GTK4 handles X11 and Wayland natively; no app-level decision needed. |
| Embedded browser panel (WebKit/GTK) | macOS cmux has it; "feature parity" argument. | GTK4 WebKit embedding is significantly more complex than macOS WKWebView. It's a large scope item for a multiplexer port. Deferred in PROJECT.md. | Defer; focus on terminal features first. Add if user demand justifies it. |
| tmux compatibility layer (send/receive tmux commands) | Would let tmux users migrate scripts. | Two competing paradigms — cmux's socket API is better designed than tmux's text protocol. Building a compatibility layer means maintaining two APIs forever. | Publish clear migration guide mapping tmux concepts to cmux socket API. |
| systemd socket activation for socket server | "Professional" / "clean init" approach. | Adds systemd-specific dependency; alienates non-systemd systems (Alpine, Gentoo, Void Linux). cmux's socket server is already process-scoped (dies with app), which is correct behavior for a GUI app. | Keep socket server in-process. Use XDG runtime dir for socket path (`$XDG_RUNTIME_DIR/cmux.sock`) which integrates with session management already. |
| Multiple concurrent socket clients / server daemon | Power users want to connect multiple clients simultaneously. | cmux is a GUI app, not a daemon. Server-mode multiplexer semantics belong to tmux/screen. Adding a separate daemon model splits the product identity. | Single GUI app with socket server. Support concurrent connections but don't split into separate daemon binary for Linux. |
| Built-in SSH client / connection manager | Looks nice; reduces context-switching. | This is a separate product (SSH client UI). cmux already has SSH remote workspaces via cmuxd-remote; building a full SSH client manager blurs scope. | Use cmuxd-remote workspace feature for SSH integration. |
| Plugin / extension system (like zellij plugins) | Extensibility is always requested. | zellij's WASM plugin system added significant complexity and is still not stable. For a port, adding a new plugin API is premature scope expansion. | Expose rich socket API instead. The v2 JSON-RPC + CLI is the "plugin system" for cmux. |
| D-Bus notifications for terminal activity | "Proper" Linux integration. | D-Bus is optional on many systems; not available in all container/WSL environments. Adds fragile optional dependency. | Use GTK4's native notification API which handles D-Bus internally and falls back gracefully. If GTK4 notifications aren't sufficient, a future milestone can add D-Bus directly. |

---

## Linux-Specific Feature Additions

These have no macOS equivalent but are expected on Linux.

| Feature | Why Linux-Specific | Complexity | Priority |
|---------|-------------------|------------|----------|
| XDG Base Directory compliance | Config: `~/.config/cmux/`, Data: `~/.local/share/cmux/`, Runtime: `$XDG_RUNTIME_DIR/cmux.sock` | LOW | Must-have (P1) |
| Wayland clipboard (wl-clipboard) | X11 uses xclip/xsel; Wayland uses wl-copy/wl-paste. GTK4 Clipboard API abstracts this, but OSC 52 clipboard (remote clipboard via Ghostty) may need verification. | MEDIUM | P1 — GTK4 should handle most of it |
| Desktop entry (.desktop file) | Required for application launchers (GNOME, KDE, etc.). Users expect `cmux` to appear in app grid. | LOW | P1 for any GUI Linux app |
| Distribution packaging | .deb (Ubuntu/Debian), .rpm (Fedora/RHEL), AppImage (universal), Flatpak (sandboxed). | MEDIUM | P1 for adoption — at minimum AppImage for broad compatibility |
| GTK4 / GNOME notifications for terminal activity | Linux desktop notification standard (libnotify / org.freedesktop.Notifications). GTK4 wraps this. | LOW | P2 — equivalent to macOS NSUserNotification |
| Systemd user service (optional) | Some users want cmux to start on login. A `cmux.service` unit file in the package. | LOW | P3 — not core; optional packaging extra |
| IME / input method support (IBus, Fcitx) | CJK input on Linux requires IBus or Fcitx integration. GTK4 handles this, but Ghostty surfaces must propagate IM events. | HIGH | P2 — required for CJK users |
| HiDPI / fractional scaling | Linux HiDPI is handled differently per compositor (Wayland: native scaling, X11: XFT DPI env vars). GTK4 handles most cases. | MEDIUM | P1 — broken HiDPI is immediately visible and rage-inducing |
| Flatpak sandbox filesystem access | If distributing as Flatpak, the app needs filesystem portal access for shell spawning. Non-trivial with PTY/subprocess in sandboxed context. | HIGH | P2 — if Flatpak is chosen, this is a blocker |

---

## Feature Dependencies

```
[Unix Socket Server]
    └──required-by──> [CLI cmux tool]
    └──required-by──> [SSH Remote Workspaces]
    └──required-by──> [Scripting / Automation]

[Ghostty Terminal Surface Embedding]
    └──required-by──> [Pane Splitting]
    └──required-by──> [Terminal Notification State]
    └──required-by──> [Scrollback / Shell Integration]
    └──required-by──> [GPU Rendering]

[Pane Split Tree (Bonsplit port)]
    └──required-by──> [Session Persistence]
    └──required-by──> [Resizable Dividers]
    └──required-by──> [Terminal Activity Rings]

[Workspace / Tab Management]
    └──required-by──> [Named Workspaces]
    └──required-by──> [Session Persistence]
    └──required-by──> [Keyboard Shortcut Navigation]

[GTK4 Window]
    └──required-by──> [Ghostty Surface Embedding]
    └──required-by──> [Drag Dividers]
    └──required-by──> [Clipboard Integration]
    └──required-by──> [Desktop Notifications]
    └──required-by──> [HiDPI Support]

[XDG Path Compliance]
    └──required-by──> [Session Persistence] (XDG data dir for session.json)
    └──required-by──> [Socket Server] (XDG runtime dir for socket)
    └──required-by──> [Config Loading] (XDG config dir for cmux/ghostty config)

[Session Persistence]
    └──enhances──> [Workspace Management]
    └──enhances──> [Pane Split Tree]

[SSH Remote Workspaces]
    └──requires──> [Unix Socket Server]
    └──requires──> [Go remote daemon (cmuxd-remote)]
```

### Dependency Notes

- **Ghostty Surface Embedding requires GTK4:** libghostty C API needs a native window/surface handle. On Linux, this is a GTK4 GdkSurface. No Ghostty = no terminal.
- **Pane splits require Ghostty embedding:** Can't lay out multiple terminals until a single terminal is embedded and functional.
- **Session persistence requires pane split tree:** The tree structure is what gets serialized to JSON; must exist before persistence can be meaningful.
- **Socket server is independent of UI:** Can be started early in app init (like macOS cmux does) and accepts connections before the first workspace opens.
- **XDG compliance is infrastructure, not a feature:** Must be decided early because it affects socket path, config path, and session storage path. Changing later breaks existing installs.

---

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed for cmux-linux to be usable by macOS cmux users on Linux.

- [ ] Single GTK4 window with Ghostty terminal surface — proves the embedding works
- [ ] Tab/workspace creation, naming, switching — core user value
- [ ] Horizontal and vertical pane splits with drag dividers — core user value
- [ ] Unix socket server (v2 JSON-RPC, wire-compatible with macOS) — essential for scripting story
- [ ] Session persistence (save/restore layout on relaunch) — key differentiator vs kitty/wezterm
- [ ] XDG Base Directory compliant paths — must not be retrofitted; get it right from day one
- [ ] Keyboard shortcut configuration (config file) — unusable without customizable bindings
- [ ] .desktop file + AppImage packaging — required for Linux users to install and launch it
- [ ] Clipboard integration (X11 + Wayland) — broken clipboard = immediate uninstall

### Add After Validation (v1.x)

- [ ] Terminal notification/attention state — add once core workspace + split is stable
- [ ] SSH remote workspaces (cmuxd-remote on Linux) — high value for devops users; add when socket API is stable
- [ ] Desktop notifications (GTK4 notifications API) — nice polish, not blocking
- [ ] Configurable numeric workspace shortcuts — already shipped macOS; straightforward port
- [ ] .deb / .rpm packages — after AppImage validates demand

### Future Consideration (v2+)

- [ ] IME / input method support (IBus, Fcitx) — important for CJK users but high complexity; defer until GTK4 embedding is stable
- [ ] Browser panel (WebKit/GTK4) — explicitly deferred in PROJECT.md; revisit when there is user demand evidence
- [ ] Flatpak distribution — sandboxing conflicts with PTY/shell spawning; needs dedicated investigation
- [ ] Systemd user service / autostart — convenience feature; packaging extra for v2

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Ghostty surface in GTK4 window | HIGH | HIGH | P1 |
| Tabs / workspace management | HIGH | MEDIUM | P1 |
| Pane splits (H/V) | HIGH | HIGH | P1 |
| Resizable dividers | HIGH | MEDIUM | P1 |
| Unix socket server (v2 JSON-RPC) | HIGH | MEDIUM | P1 |
| Session persistence | HIGH | MEDIUM | P1 |
| XDG paths compliance | HIGH | LOW | P1 |
| Keyboard shortcut config | HIGH | LOW | P1 |
| Clipboard (X11 + Wayland) | HIGH | LOW | P1 |
| Desktop entry + AppImage | HIGH | LOW | P1 |
| Named workspaces | MEDIUM | LOW | P1 |
| Terminal notification state | MEDIUM | MEDIUM | P2 |
| SSH remote workspaces | HIGH | MEDIUM | P2 |
| Desktop notifications | MEDIUM | LOW | P2 |
| Numeric workspace shortcuts | MEDIUM | LOW | P2 |
| HiDPI / fractional scaling | HIGH | MEDIUM | P2 |
| IME support (IBus/Fcitx) | MEDIUM | HIGH | P2 |
| .deb / .rpm packaging | MEDIUM | LOW | P2 |
| Browser panel | LOW | HIGH | P3 |
| Flatpak distribution | MEDIUM | HIGH | P3 |
| Systemd service unit | LOW | LOW | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

---

## Competitor Feature Analysis

| Feature | tmux | zellij | wezterm | kitty | cmux (macOS) | cmux-linux approach |
|---------|------|--------|---------|-------|--------------|---------------------|
| Tabs | Windows | Yes | Yes | Yes | Yes (workspaces) | Yes (workspaces) — same model |
| Pane splits | Yes | Yes | Yes | Yes | Yes (Bonsplit tree) | Yes (Bonsplit Rust port) |
| Session survival (close terminal) | YES — core feature | YES | No | No | No (GUI app) | No — GUI app model, not daemon |
| Session persistence on relaunch | Via resurrect plugin | Yes | No | No | Yes (session.json) | Yes (XDG data dir) |
| GPU rendering | No (escape codes) | No | Yes | Yes | Yes (Ghostty/Metal) | Yes (Ghostty/OpenGL) |
| Scripting / IPC | `tmux` CLI (text protocol) | No stable API | IPC (limited) | Remote control (limited) | v2 JSON-RPC socket | v2 JSON-RPC (wire-compatible) |
| Mouse support | Optional (`mouse on`) | Yes | Yes | Yes | Yes | Yes (GTK4) |
| Keyboard config | `.tmux.conf` | `config.kdl` | `wezterm.lua` | `kitty.conf` | Config file | Config file (format TBD — likely TOML or KDL) |
| SSH remote workspaces | Via SSH + tmux | No | No | No | Yes (cmuxd-remote) | Yes (reuse Go daemon) |
| Cross-platform CLI | Yes | No | No | No | Yes (macOS only today) | Yes (v2 socket shared) |
| Embedded browser | No | No | No | No | Yes (WebKit) | Deferred |
| Notifications | Activity monitor (text) | Yes | No | No | Yes (ring animation) | Yes (GTK4 + attention state) |
| XDG compliance | Yes | Yes | Yes | Yes | No (macOS) | Yes — required from day one |
| Plugin system | No | Yes (WASM) | Lua config | No | No | No — socket API is the extension point |

---

## Confidence Notes

**HIGH confidence (from codebase + macOS feature set):**
- Which features cmux macOS has and what they do
- macOS session.json persistence model
- v2 JSON-RPC socket protocol existence and structure
- Bonsplit tree layout model

**MEDIUM confidence (training knowledge of Linux ecosystem):**
- tmux, zellij, wezterm, kitty feature sets as of ~Aug 2025
- Linux clipboard landscape (xclip vs wl-clipboard)
- XDG Base Directory standard
- GTK4 notification, HiDPI, and IME handling characteristics
- Flatpak PTY/sandboxing constraints

**LOW confidence (flags for live verification):**
- Exact current state of zellij plugin API stability (evolving rapidly)
- Whether GTK4 WebKit embedding complexity has improved since training cutoff
- Current state of Ghostty's Linux GTK4 surface embedding API for multi-surface hosting
- Whether libghostty C API on Linux exposes the same surface embedding hooks as GhosttyKit on macOS
- IME integration status in Ghostty Linux builds

---

## Sources

- Codebase analysis: `/home/twilson/code/cmux-linux/.planning/codebase/` (ARCHITECTURE.md, STACK.md, CONCERNS.md)
- Project requirements: `.planning/PROJECT.md`
- Training knowledge: tmux 3.x, zellij 0.39.x, wezterm 20240203+, kitty 0.35.x, ghostty 1.x feature sets
- XDG Base Directory Specification: https://specifications.freedesktop.org/basedir-spec/latest/
- Note: No live web research performed (tools unavailable). Zellij and wezterm features should be verified against current docs before finalizing roadmap.

---

*Feature research for: Linux terminal multiplexer (cmux-linux port)*
*Researched: 2026-03-23*
