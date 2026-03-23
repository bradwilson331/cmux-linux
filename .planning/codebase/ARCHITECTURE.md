# Architecture

**Analysis Date:** 2026-03-23

## Pattern Overview

**Overall:** Multi-layered macOS/Linux native app with separation between UI (SwiftUI/AppKit), terminal simulation (Ghostty), browser (WebKit), and background services.

**Key Characteristics:**
- Single-window document interface with workspace-level tab management
- AppKit + SwiftUI hybrid (AppKit for advanced window control, SwiftUI for UI layout)
- Two-process architecture: main app (Swift) + remote daemon (Go) for SSH workspaces
- Unix socket-based programmatic control (v1 text protocol + v2 JSON-RPC)
- GPU-accelerated terminal rendering via libghostty C API
- Bonsplit library for pane splitting and layout management

## Layers

**Application Entry Point:**
- Purpose: Initialize app state, configure environment, set up UI scene
- Location: `Sources/cmuxApp.swift`
- Contains: SwiftUI app root, TabManager initialization, environment setup
- Depends on: AppDelegate, TabManager, TerminalNotificationStore
- Used by: SwiftUI runtime

**AppKit Delegate Layer:**
- Purpose: Manage native window lifecycle, event routing, socket server, AppleScript support
- Location: `Sources/AppDelegate.swift`
- Contains: Window controller management, keyboard event handling, Unix socket listener, CLI wrappers
- Depends on: TerminalController, Ghostty C API, cocoa
- Used by: App life cycle, all windows

**Terminal Engine:**
- Purpose: Host Ghostty C library, render terminal surfaces, manage PTY lifecycle
- Location: `Sources/TerminalController.swift`, `Sources/GhosttyTerminalView.swift`
- Contains: Ghostty C API wrappers, PTY spawning, surface rendering, clipboard bridge
- Depends on: Ghostty.xcframework, Metal, IOSurface, Combine
- Used by: Workspace/Tab rendering

**Tab/Workspace Management:**
- Purpose: Manage workspace (tab) lifecycle, storage, notifications
- Location: `Sources/TabManager.swift`, `Sources/Workspace.swift`
- Contains: Workspace creation/deletion, state persistence, notification coordination
- Depends on: SessionPersistence, TerminalNotificationStore
- Used by: ContentView (UI layer)

**Pane/Split Management:**
- Purpose: Organize panes within a workspace, track layout
- Location: Bonsplit library (vendor/bonsplit)
- Contains: Tree-based pane layout, drag-drop handling, divider management
- Depends on: SwiftUI, AppKit
- Used by: WorkspaceContentView, panel views

**UI Layer:**
- Purpose: Render sidebar, workspace tabs, panes, panels
- Location: `Sources/ContentView.swift`, `Sources/WorkspaceContentView.swift`, `Sources/Panels/`
- Contains: Sidebar (tabs list), workspace view (Bonsplit layout), panel containers
- Depends on: TabManager, Workspace, Panel types, Bonsplit
- Used by: cmuxApp body

**Panel System:**
- Purpose: Abstract container for terminal, browser, or markdown content
- Location: `Sources/Panels/Panel.swift`, `Sources/Panels/TerminalPanel.swift`, `Sources/Panels/BrowserPanel.swift`, `Sources/Panels/MarkdownPanel.swift`
- Contains: Panel enum, type-specific panels, focus intents
- Depends on: SwiftUI, Ghostty, WebKit
- Used by: Workspace panes

**Browser Panel:**
- Purpose: Chromium/WebKit browser embedded in workspace
- Location: `Sources/Panels/BrowserPanel.swift`, `Sources/Panels/BrowserPanelView.swift`, `Sources/Panels/CmuxWebView.swift`
- Contains: WebKit view wrapper, JavaScript console, find UI, download handler
- Depends on: WebKit, JavaScript API bridge
- Used by: Workspace (side-by-side with terminals)

**Socket Control Layer:**
- Purpose: Unix socket server for CLI/programmatic terminal control
- Location: `Sources/TerminalController.swift` (socket methods), `Sources/SocketControlSettings.swift`
- Contains: V1 command parser, V2 JSON-RPC router, client thread handlers
- Depends on: Darwin networking (socket API)
- Used by: External CLI, tests, remote relay

**Remote SSH Layer:**
- Purpose: Bootstrap and manage remote daemon on SSH hosts
- Location: `daemon/remote/` (Go), `Sources/Workspace.swift` (remote methods)
- Contains: Remote daemon manifest, SSH tunnel setup, relay authentication
- Depends on: SSH, HTTP/HTTPS
- Used by: SSH workspace initialization

**Configuration Layer:**
- Purpose: Load and apply Ghostty config, app preferences
- Location: `Sources/GhosttyConfig.swift`, `Sources/cmuxApp.swift` (environment setup)
- Contains: Config parser, theme resolver, font loading
- Depends on: FileManager, UserDefaults
- Used by: GhosttyTerminalView, AppKit window setup

**Notification System:**
- Purpose: Track terminal activity, unread states, flash animations
- Location: `Sources/TerminalNotificationStore.swift`, `Sources/Panels/Panel.swift` (WorkspaceAttentionCoordinator)
- Contains: Notification store, attention state machine, UI ring animations
- Depends on: Combine, Terminal output parsing
- Used by: Workspace UI, sidebar rendering

## Data Flow

**Keyboard Input -> Terminal:**

1. AppKit `performKeyEquivalent()` in AppDelegate (timing-sensitive)
2. Route to active window's TerminalWindowPortal
3. TerminalWindowPortal.hitTest() to identify target pane/surface
4. Ghostty C API `ghostty_surface_feed_input()` → PTY stdin
5. Terminal renders via Ghostty renderer → Metal IOSurface
6. SwiftUI refreshes via Ghostty wakeup callback

**Mouse/Divider Drag:**

1. AppKit mouse event via hitTest()
2. Route to Bonsplit divider or panel
3. Bonsplit layout tree update
4. SwiftUI body re-evaluation (constrained by `.equatable()` guards)

**Terminal Output -> Notification Ring:**

1. Ghostty PTY read loop → surface buffer
2. TerminalNotificationStore parses escape sequences (agent attention markers)
3. Notification arrival → update Workspace.attentionState
4. WorkspaceAttentionCoordinator decides animation parameters
5. SwiftUI re-evaluates affected TabItemView + panel ring views

**Socket Command -> Terminal State Change:**

1. Unix socket accept → read command bytes (off-main thread)
2. Parse v1 or v2 command → validate args (off-main thread)
3. Schedule state mutation on DispatchQueue.main
4. Workspace/Tab/TerminalController method executes
5. SwiftUI refreshes bound properties

**State Persistence:**

1. Workspace/Tab closed or app exits
2. SessionPersistence captures layout + pane metadata
3. Write to ~/Library/Application Support/cmux/session.json
4. On next launch: TabManager init() loads session, reconstructs tree

**Remote SSH Connection:**

1. User calls `workspace.remote.configure` socket command
2. Fetch cmuxd-remote manifest from Info.plist
3. Download/verify binary from GitHub releases
4. SSH upload to remote host
5. Start cmuxd-remote serve --stdio over reverse SSH forward
6. Local relay server bridges socket to remote RPC stream
7. Commands proxied through `proxy.*` and `session.*` RPC

## State Management

**Workspace State:**

- Location: `Sources/Workspace.swift`
- Key properties: `panelTree` (Bonsplit root), `selectedPaneID`, `attentionState`, `sidebar` metadata
- Mutations: Via TerminalController methods (socket), TabManager (UI), keyboard shortcuts
- Binding: @StateObject in TabManager, observed by SwiftUI views

**Terminal Notification State:**

- Location: `Sources/TerminalNotificationStore.swift`
- Key properties: Per-surface unread/manual-unread flags, flash timing state
- Updates: From terminal output parsing, focus changes, manual dismissal
- Binding: @StateObject in cmuxApp, @ObservedObject in views

**Tab Manager (Root State):**

- Location: `Sources/TabManager.swift`
- Owns: Ordered array of Workspace objects, current selection index, auto-save trigger
- Mutations: Socket commands, keyboard shortcuts, drag-drop (UI)
- Persists: Via SessionPersistence (debounced on changes)

**Session Persistence:**

- Location: `Sources/SessionPersistence.swift`
- Captures: Workspace order, pane tree layout, working dirs, shell commands
- Writes: JSON to ~/Library/Application Support/cmux/session.json
- Reads: On app launch before UI appears

## Key Abstractions

**Panel:**
- Purpose: Unified type for terminal/browser/markdown content within a pane
- Examples: `Sources/Panels/Panel.swift`, `Sources/Panels/TerminalPanel.swift`, `Sources/Panels/BrowserPanel.swift`
- Pattern: Enum `PanelType` with associated data, switch-based rendering in `PanelContentView.swift`

**Workspace (formerly Tab):**
- Purpose: Container for multiple panels in split layout with unified shell context
- Examples: `Sources/Workspace.swift`
- Pattern: Single @ObservedObject per workspace, owns Bonsplit tree and metadata

**TerminalController Singleton:**
- Purpose: Global Unix socket server + programmatic command dispatcher
- Examples: `Sources/TerminalController.swift`
- Pattern: @MainActor class, static `shared` instance, thread-safe socket accept loop

**GhosttyTerminalView Bridge:**
- Purpose: SwiftUI wrapper for Ghostty C surface rendering
- Examples: `Sources/GhosttyTerminalView.swift`
- Pattern: NSViewRepresentable hosting Metal view, wakeup callback for re-renders

**Bonsplit Split Tree:**
- Purpose: Immutable layout tree for pane organization
- Examples: vendor/bonsplit (Swift library)
- Pattern: Recursive enum (branch/leaf), update returns new tree

## Entry Points

**App Launch:**
- Location: `Sources/cmuxApp.swift` (cmuxApp struct + init)
- Triggers: OS launches app from Dock/Cmd+Space
- Responsibilities: Parse environment, initialize TabManager/NotificationStore, configure Ghostty, start socket listener

**Window Creation:**
- Location: `Sources/ContentView.swift` (WindowGroup)
- Triggers: New window shortcut (Cmd+N) or app startup
- Responsibilities: Create NSWindow via AppDelegate, wire up views, start session restore

**Socket Command:**
- Location: `Sources/TerminalController.swift` (accept loop, command dispatcher)
- Triggers: CLI `cmux` command connects to ~/.cmux/socket
- Responsibilities: Parse v1/v2 command, route to handler, return response

**Remote SSH Session:**
- Location: `Sources/Workspace.swift` (remoteBootstrapIfNeeded)
- Triggers: User calls `workspace.remote.configure` with SSH target
- Responsibilities: Download cmuxd-remote, SSH upload, start reverse forward, authenticate relay

## Error Handling

**Strategy:** Separate error paths for terminal/browser/system operations.

**Patterns:**

- Terminal PTY spawn failure: Log to system, show error notification
- Socket command invalid args: Return JSON error response with message
- SSH connection timeout: Display error in workspace sidebar, allow retry
- Ghostty surface initialization: Retry with backoff, log timing data (DEBUG mode)
- Browser network failure: WebKit handles inline (no app-level intervention)

## Cross-Cutting Concerns

**Logging:**

- **Production:** Sentry integration for crashes, PostHog for analytics
- **Debug:** File-based ring buffer (`/tmp/cmux-debug.log`) via `dlog()` function, timing probes for keystroke latency

**Validation:**

- Socket command args validated off-main thread before state mutation
- SSH config merged with defaults using case-insensitive precedence checks
- File URLs on drag-drop validated before remote upload

**Authentication:**

- Socket commands: Optional password (HMAC-SHA256) stored in Keychain
- Remote relay: HMAC-SHA256 challenge-response, token written to `~/.cmux/relay/<port>.auth` (0600)
- SSH: Standard SSH key auth, custom StrictHostKeyChecking + control socket injection

**Security:**

- Untagged DEBUG app refuses to launch (prevents socket conflict with production)
- Socket access mode: `cmuxOnly` (default) vs. `off` vs. `insecure` (password-protected)
- Remote daemon SHA-256 pinned in Info.plist, verified on download

---

*Architecture analysis: 2026-03-23*
