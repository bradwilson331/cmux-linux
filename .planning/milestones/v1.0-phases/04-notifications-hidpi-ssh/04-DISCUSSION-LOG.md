# Phase 4: Notifications + HiDPI + SSH - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-26
**Phase:** 04-notifications-hidpi-ssh
**Areas discussed:** Bell & attention tracking, Desktop notifications, HiDPI / fractional scaling, SSH remote workspaces

---

## Bell & Attention Tracking

### Attention source

| Option | Description | Selected |
|--------|-------------|----------|
| Bell-only tracking | Track via terminal bell (\a) only. Simple, matches tmux/screen. OSC 99 deferred. | ✓ |
| Bell + OSC 99 markers | Track both bell and OSC 99 progress markers. Richer but more complex. | |
| Bell + any output activity | Track bell AND any new terminal output in unfocused panes. Noisier. | |

**User's choice:** Bell-only tracking
**Notes:** None

### Indicator style

| Option | Description | Selected |
|--------|-------------|----------|
| Colored dot | Small colored dot next to workspace name. Matches macOS cmux. | ✓ |
| Name highlight/bold | Change workspace name styling when it has unread activity. | |
| You decide | Claude picks approach fitting GTK4 sidebar. | |

**User's choice:** Colored dot
**Notes:** None

### Clear behavior

| Option | Description | Selected |
|--------|-------------|----------|
| On workspace focus | Clears when user switches to workspace. Matches tmux/macOS cmux. | ✓ |
| On pane focus | Only clears when specific bell pane is focused. More granular. | |
| You decide | Claude picks clearing behavior. | |

**User's choice:** On workspace focus
**Notes:** None

---

## Desktop Notifications

### When to fire

| Option | Description | Selected |
|--------|-------------|----------|
| Only when app unfocused | Desktop notification only when GTK4 window not focused. | ✓ |
| Always on bell | Desktop notification on every bell regardless of focus. | |
| Configurable | Default unfocused-only, config option for always/never. | |

**User's choice:** Only when app unfocused
**Notes:** None

### API choice

| Option | Description | Selected |
|--------|-------------|----------|
| GNotification | GTK4 built-in via GApplication. No extra dependency. | ✓ |
| libnotify (notify-rust) | notify-rust crate. More control but adds dependency. | |
| You decide | Claude picks based on GTK4 integration. | |

**User's choice:** GNotification
**Notes:** None

---

## HiDPI / Fractional Scaling

### Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Verify & fix | Test existing handler at 1x/1.5x/2x. Fix issues in sidebar/dividers. | ✓ |
| Full rework | Rewrite DPI handling from scratch. Overkill unless fundamentally broken. | |
| Sidebar + chrome only | Focus only on non-terminal chrome scaling. | |

**User's choice:** Verify & fix
**Notes:** None

### Fractional scaling

| Option | Description | Selected |
|--------|-------------|----------|
| GTK4 default handling | GTK4 4.12+ handles fractional scaling natively. Just verify. | ✓ |
| Custom fractional logic | Custom handling for edge cases. Unlikely needed. | |
| You decide | Claude verifies and adds custom logic only if gaps found. | |

**User's choice:** GTK4 default handling
**Notes:** None

---

## SSH Remote Workspaces

### Daemon strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse Go daemon | Go daemon/remote/ is platform-agnostic. Compile for Linux. | ✓ |
| Rewrite in Rust | Full Rust rewrite. More consistent stack but significant effort. | |
| Defer SSH entirely | Push to later phase. Reduces Phase 4 scope. | |

**User's choice:** Reuse Go daemon
**Notes:** None

### Configuration

| Option | Description | Selected |
|--------|-------------|----------|
| Socket command only | Configure via socket API. No GUI config in Phase 4. | ✓ |
| Config file + socket | Both config.toml and socket commands. Config file is Phase 5 though. | |
| You decide | Claude picks based on Phase 4 scope. | |

**User's choice:** Socket command only
**Notes:** None

### Reconnect strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-reconnect with backoff | Exponential backoff (1s→30s cap). Show disconnected state. | ✓ |
| Manual reconnect | Show disconnected state, user must explicitly reconnect. | |
| You decide | Claude picks reconnect strategy. | |

**User's choice:** Auto-reconnect with backoff
**Notes:** None

---

## Claude's Discretion

- Exact Ghostty action tag for bell events
- Sidebar dot color and GTK4 widget choice
- GNotification category/priority settings
- cmuxd-remote deployment mechanism
- SSH tunnel port selection and multiplexing details
- Whether surface.health includes attention state

## Deferred Ideas

- OSC 99 progress markers
- GUI SSH configuration dialog (Phase 5)
- Rust rewrite of cmuxd-remote
- Browser panel integration
