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
- [ ] 02-06-PLAN.md — Human verification: full Phase 2 feature verification checkpoint
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
**Plans**: TBD

### Phase 4: Notifications + HiDPI + SSH
**Goal**: Users see per-pane activity indicators and desktop notifications; the app renders correctly at any display scale; SSH workspaces connect to remote hosts
**Depends on**: Phase 3
**Requirements**: NOTF-01, NOTF-02, NOTF-03, HDPI-01, HDPI-02, SSH-01, SSH-02, SSH-03, SSH-04
**Success Criteria** (what must be TRUE):
  1. A workspace with terminal bell activity shows a visual indicator in the workspace list; the indicator clears when the workspace is focused
  2. Desktop notification appears when a terminal rings a bell while the app window is not focused
  3. App renders correctly at 1x, 1.5x, and 2x scale factors; moving the window between monitors with different DPI updates rendering without restart
  4. User can configure an SSH workspace, and terminal sessions in that workspace run on the remote host with reconnect after network interruption
**Plans**: TBD
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
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Ghostty Foundation | 8/9 | In Progress|  |
| 2. Workspaces + Pane Splits | 5/8 | In Progress|  |
| 3. Socket API + Session Persistence | 0/? | Not started | - |
| 4. Notifications + HiDPI + SSH | 0/? | Not started | - |
| 5. Config + Distribution | 0/? | Not started | - |
