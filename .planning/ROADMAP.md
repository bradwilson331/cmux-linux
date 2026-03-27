# Roadmap: cmux Linux Port

## Overview

Six phases deliver a full Linux port of cmux from zero to a distributable, feature-complete GPU-accelerated terminal multiplexer. Phase 1 resolves the critical Ghostty embedding unknown and establishes the foundational architecture. Phase 2 builds the core multiplexer experience (workspaces + pane splits). Phase 3 adds the socket API and session persistence that make cmux scriptable and stateful. Phase 4 completes the feature set (notifications, HiDPI, SSH remote workspaces). Phase 5 adds config file support and closes all remaining Phase 1 config requirements. Phase 6 delivers CI/CD and distribution packaging for Linux installation.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Ghostty Foundation** - Single working Ghostty terminal surface in a GTK4 window with correct threading, input, clipboard, DPI, and XDG paths
- [ ] **Phase 2: Workspaces + Pane Splits** - Full workspace tab management and recursive pane splitting with drag-to-resize dividers
- [ ] **Phase 3: Socket API + Session Persistence** - v2 JSON-RPC socket server wire-compatible with macOS cmux; workspace/pane layout saved and restored across launches
- [ ] **Phase 4: Notifications + HiDPI + SSH** - Per-pane attention state, desktop notifications, correct fractional scaling, and SSH remote workspaces
- [ ] **Phase 5: Config + Distribution** - Config file, keyboard shortcut customization, GitHub Actions CI, and AppImage packaging
- [ ] **Phase 6: Session Layout Restore + Surface Wiring** - Persist/restore split pane layouts and wire SplitNode surface pointers so socket text injection works
- [ ] **Phase 7: SSH Terminal I/O** - Implement proxy.stream bidirectional I/O routing so SSH workspace terminal sessions run on the remote host
- [ ] **Phase 8: Agent-Browser Integration** - Bundle agent-browser, add browser.* socket commands, render CDP screencast frames in preview pane

## Phase Details

### Phase 1: Ghostty Foundation
**Goal**: A GTK4 window runs a single Ghostty terminal surface with correct input, rendering, clipboard, threading, and XDG path compliance
**Depends on**: Nothing (first phase)
**Requirements**: GHOST-01, GHOST-02, GHOST-03, GHOST-04, GHOST-05, GHOST-06, GHOST-07
**Success Criteria** (what must be TRUE):
  1. User can open the app and type into a working terminal — keystrokes appear with < 10ms latency
  2. User can copy text from the terminal and paste it into another application (and vice versa) on both X11 and Wayland
  3. Terminal renders at correct pixel density on HiDPI displays without blurriness or incorrect scale
  4. App scaffold compiles and runs: tokio runtime started, GLib main loop runs, glib::MainContext::channel bridges the two
**Plans**: 9 plans
Plans:
- [x] 01-01-PLAN.md — Rust project scaffold: Cargo.toml, build.rs (bindgen), src/main.rs skeleton, scripts/setup-linux.sh
- [x] 01-02-PLAN.md — Ghostty fork extension: GHOSTTY_PLATFORM_GTK4 in ghostty.h, embedded.zig, OpenGL.zig
- [x] 01-03-PLAN.md — Surface embedding: GtkGLArea lifecycle, wakeup_cb, DPI scaling, clipboard callbacks
- [x] 01-04-PLAN.md — Input routing: GDK keycode mapping, keyboard/mouse/scroll controllers, human verification checkpoint
- [x] 01-05-PLAN.md — Fix linking: Correct build.rs linkage, add missing stubs (gap closure)
- [x] 01-06-PLAN.md — Add tokio runtime: Initialize tokio, bridge to GLib (gap closure)
- [x] 01-07-PLAN.md — Fix initialization crash: Proper Ghostty lifecycle, defensive checks (gap closure)
- [x] 01-08-PLAN.md — Fix rendering: Missing function stubs, GL context debugging (gap closure)
- [ ] 01-09-PLAN.md — Verify functionality: Test input/output/clipboard, achieve Phase 1 goals (gap closure)
**UI hint**: yes

### Phase 2: Workspaces + Pane Splits
**Goal**: Users can manage multiple workspaces (tabs) and split any pane horizontally or vertically with drag-to-resize dividers
**Depends on**: Phase 1
**Requirements**: WS-01, WS-02, WS-03, WS-04, WS-05, WS-06, SPLIT-01, SPLIT-02, SPLIT-03, SPLIT-04, SPLIT-05, SPLIT-06, SPLIT-07
**Success Criteria** (what must be TRUE):
  1. User can create, close, rename, and switch between workspaces via keyboard shortcut and click — each workspace is independent
  2. User can split the active pane horizontally and vertically, navigate between panes with keyboard shortcuts, drag dividers to resize, and close individual panes
  3. Focus routing is correct: keyboard input goes to the active pane after every split, navigation, and close operation
  4. Memory is stable after 50 workspace create/close cycles (no GObject ref-cycle leaks, no Ghostty surface leaks)
**Plans**: 8 plans
Plans:
- [x] 02-00-PLAN.md — Wave 0 test stubs: #[cfg(test)] modules in workspace.rs and split_engine.rs
- [x] 02-01-PLAN.md — Multi-surface infrastructure: replace single-GLArea globals with GL_AREA_REGISTRY + SURFACE_REGISTRY
- [x] 02-02-PLAN.md — Workspace model: src/workspace.rs + src/app_state.rs with AppState CRUD operations
- [x] 02-03-PLAN.md — Split engine: src/split_engine.rs SplitNode tree with split/close/focus operations
- [x] 02-04-PLAN.md — Window layout + sidebar: full GtkBox(H)+GtkStack layout, sidebar CSS, build_ui restructure
- [x] 02-05-PLAN.md — Keyboard shortcuts: src/shortcuts.rs capture-phase interception, all D-10 shortcuts wired
- [x] 02-06-PLAN.md — Human verification: full Phase 2 feature verification checkpoint
- [x] 02-07-PLAN.md — Gap closure: fix drag-resize cursor freeze, pane-close crash, and Ctrl+Alt+Arrow WM conflict
**UI hint**: yes

### Phase 3: Socket API + Session Persistence
**Goal**: The cmux CLI and `tests_v2/` Python suite can control the Linux app over a Unix socket; workspace and pane layout survives app restarts
**Depends on**: Phase 2
**Requirements**: SOCK-01, SOCK-02, SOCK-03, SOCK-04, SOCK-05, SOCK-06, SESS-01, SESS-02, SESS-03, SESS-04
**Success Criteria** (what must be TRUE):
  1. The `tests_v2/` Python protocol suite passes unmodified against the Linux socket server
  2. Non-focus-intent socket commands (new_split, workspace.list, send) never steal window focus or call gtk_window_present()
  3. App relaunches and fully restores the previous workspace + pane layout
  4. Killing the app mid-save (kill -9) does not corrupt the session file — next launch recovers cleanly
**Plans**: 8 plans
Plans:
- [x] 03-00-PLAN.md — Wave 0 scaffold: Cargo.toml deps (serde/serde_json/uuid/libc), module stubs with test scaffolds
- [x] 03-01-PLAN.md — Data model: UUID fields in Workspace/SplitNode, SplitNodeData serde type, glib channel replaces mpsc polling
- [x] 03-02-PLAN.md — Socket server: SO_PEERCRED auth, XDG path setup, accept loop, cmux.py Linux path discovery
- [x] 03-03-PLAN.md — Tier-1 handlers (system/workspace/window/debug): full dispatch table + Tier-2 stubs
- [x] 03-04-PLAN.md — Tier-1 handlers (surface/pane): send_text/send_key/focus/close with SOCK-05 focus policy
- [x] 03-05-PLAN.md — Session persistence: SessionData serde, atomic save, 500ms debounce, restore on relaunch
- [x] 03-06-PLAN.md — Human verification: socket connectivity, workspace control, session restore, test_ctrl_socket.py
- [x] 03-07-PLAN.md — Gap closure: cmux-cli wrapper script for SOCK-03

### Phase 4: Notifications + HiDPI + SSH
**Goal**: Users see per-pane activity indicators and desktop notifications; the app renders correctly at any display scale; SSH workspaces connect to remote hosts
**Depends on**: Phase 3
**Requirements**: NOTF-01, NOTF-02, NOTF-03, HDPI-01, HDPI-02, SSH-01, SSH-02, SSH-03, SSH-04
**Success Criteria** (what must be TRUE):
  1. A workspace with terminal bell activity shows a visual indicator in the workspace list; the indicator clears when the workspace is focused
  2. Desktop notification appears when a terminal rings a bell while the app window is not focused
  3. App renders correctly at 1x, 1.5x, and 2x scale factors; moving the window between monitors with different DPI updates rendering without restart
  4. User can configure an SSH workspace, and terminal sessions in that workspace run on the remote host with reconnect after network interruption
**Plans**: 5 plans
Plans:
- [x] 04-01-PLAN.md — Bell attention tracking: action_cb RING_BELL handler, per-pane/workspace has_attention, sidebar dot, desktop notifications
- [x] 04-02-PLAN.md — HiDPI verification: audit scale-factor handler, verify CSS at fractional scales, fix any issues
- [x] 04-03-PLAN.md — Notification socket commands: notification.list, notification.clear, surface.health attention field
- [x] 04-04-PLAN.md — SSH remote workspaces: data model, cmuxd-remote deployment, stdio tunnel, reconnection, socket API
- [ ] 04-05-PLAN.md — Human verification: full Phase 4 feature verification checkpoint
**UI hint**: yes

### Phase 5: Config + Distribution
**Goal**: Keyboard shortcuts are configurable via a TOML config file; GitHub Actions CI validates every commit; AppImage artifact ships on release tags
**Depends on**: Phase 4
**Requirements**: CFG-01, CFG-02, CFG-03, CFG-04, DIST-01, DIST-02, DIST-03, DIST-04
**Success Criteria** (what must be TRUE):
  1. User can edit `~/.config/cmux/config.toml` to remap keyboard shortcuts and the app respects those bindings on next launch
  2. GitHub Actions CI pipeline builds, runs clippy, and executes unit tests on ubuntu-latest for every push
  3. An AppImage artifact is produced and downloadable from each release tag
  4. App launches and runs correctly on both Wayland and X11/XWayland sessions
**Plans**: 2 plans
Plans:
- [x] 05-01-PLAN.md — TOML config system: config.rs with XDG path, ShortcutMap HashMap lookup, refactored shortcuts.rs
- [x] 05-02-PLAN.md — Linux CI and AppImage: ci.yml linux-build job, release.yml AppImage job, .desktop file and icon

### Phase 6: Session Layout Restore + Surface Wiring
**Goal**: Split pane layouts persist and restore on relaunch; SplitNode surface pointers are wired so socket commands (surface.send_text, surface.send_key, debug.type) work
**Depends on**: Phase 3
**Requirements**: SESS-02, SOCK-02, SOCK-03
**Gap Closure:** Closes gaps from v1.0 milestone audit
**Success Criteria** (what must be TRUE):
  1. App saves full split tree topology (not just workspace names) to session.json
  2. On relaunch, each workspace restores its exact split layout with new Ghostty surfaces
  3. `set_initial_surface()` is called for every SplitNode::Leaf, making socket text injection functional
  4. `surface.send_text` via socket actually types into the target pane
**Plans**: 2 plans
Plans:
- [x] 06-01-PLAN.md — Session save upgrade: ratio field, CWD capture, real tree serialization, version 2
- [x] 06-02-PLAN.md — Session restore: from_data() tree rebuild, recursive surface wiring, main.rs version-aware restore

### Phase 7: SSH Terminal I/O
**Goal**: SSH workspace terminal sessions execute on the remote host via bidirectional I/O proxying through the SSH tunnel
**Depends on**: Phase 4
**Requirements**: SSH-03
**Gap Closure:** Closes gaps from v1.0 milestone audit
**Success Criteria** (what must be TRUE):
  1. `proxy.stream` in tunnel.rs routes terminal I/O between the local Ghostty surface and the remote cmuxd shell
  2. Keystrokes typed in an SSH workspace pane appear in the remote shell
  3. Remote shell output renders in the local terminal surface

### Phase 8: Agent-Browser Integration
**Goal**: Integrate agent-browser headless Chrome automation into cmux with bundled binary, browser.* socket commands, and CDP screencast preview pane
**Depends on**: Phase 3 (socket infrastructure), Phase 2 (split engine)
**Requirements**: BROW-01
**Success Criteria** (what must be TRUE):
  1. `browser.open <url>` socket command spawns agent-browser daemon and navigates to URL
  2. `browser.stream.enable` starts CDP screencast and frames render in a GTK4 preview pane at ~5fps
  3. `browser.close` shuts down daemon cleanly with no orphaned Chrome processes
  4. All 6 browser.* socket commands are listed in system.capabilities
**Plans**: 3 plans
Plans:
- [ ] 08-01-PLAN.md — Foundation: BrowserManager module, SplitNode::Preview variant, Cargo deps
- [ ] 08-02-PLAN.md — Socket commands: browser.* enum variants, dispatch routing, handler implementations
- [ ] 08-03-PLAN.md — Preview rendering: WebSocket stream pipeline, preview pane widget factory, shutdown cleanup

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Ghostty Foundation | 8/9 | In Progress|  |
| 2. Workspaces + Pane Splits | 5/8 | In Progress|  |
| 3. Socket API + Session Persistence | 2/7 | In Progress|  |
| 4. Notifications + HiDPI + SSH | 0/5 | Not started | - |
| 5. Config + Distribution | 0/2 | Not started | - |
| 6. Session Layout Restore + Surface Wiring | 0/2 | Not started | - |
| 7. SSH Terminal I/O | 0/0 | Not started | - |
| 8. Agent-Browser Integration | 0/3 | Not started | - |
