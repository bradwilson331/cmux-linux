# Codebase Structure

**Analysis Date:** 2026-03-23

## Directory Layout

```
/home/twilson/code/cmux-linux/
├── Sources/                         # Main Swift source code
│   ├── AppDelegate.swift            # AppKit lifecycle, window management, socket server
│   ├── cmuxApp.swift                # @main SwiftUI app entry, configuration
│   ├── ContentView.swift            # Main UI: sidebar + workspace view
│   ├── TabManager.swift             # Workspace (tab) lifecycle management
│   ├── Workspace.swift              # Individual workspace state and methods
│   ├── WorkspaceContentView.swift   # Workspace split pane UI layer
│   ├── TerminalController.swift     # Unix socket controller, command dispatcher
│   ├── GhosttyTerminalView.swift    # Ghostty C API bridge, clipboard, rendering
│   ├── TerminalView.swift           # SwiftUI terminal surface wrapper
│   ├── TerminalWindowPortal.swift   # AppKit window hosting Ghostty surfaces
│   ├── BrowserWindowPortal.swift    # AppKit window for browser popups
│   ├── GhosttyConfig.swift          # Ghostty config parser (~/.config/ghostty/config)
│   ├── TerminalNotificationStore.swift # Notification state tracking, unread management
│   ├── TerminalImageTransfer.swift  # SIXEL image handling
│   ├── TerminalSSHSessionDetector.swift # SSH prompt parsing for remote detection
│   ├── SessionPersistence.swift     # Workspace layout save/restore
│   ├── SocketControlSettings.swift  # Socket auth mode and password storage
│   ├── KeyboardShortcutSettings.swift # Keybinding configuration
│   ├── KeyboardLayout.swift         # Keyboard layout helpers
│   ├── PortScanner.swift            # Detect listening ports in workspace
│   ├── PostHogAnalytics.swift       # Analytics event tracking
│   ├── SentryHelper.swift           # Error reporting integration
│   ├── AppleScriptSupport.swift     # AppleScript handler stubs
│   ├── NotificationsPage.swift      # Notifications panel view
│   ├── WindowAccessor.swift         # Window introspection utilities
│   ├── WindowDecorationsController.swift # Titlebar customization
│   ├── WindowDragHandleView.swift   # Draggable window area
│   ├── WindowToolbarController.swift # Toolbar setup
│   ├── RemoteRelayZshBootstrap.swift # SSH bootstrap script
│   ├── Backport.swift               # Compatibility shims for older macOS
│   ├── UITestRecorder.swift         # UI test fixture helper
│   │
│   ├── Panels/                      # Panel type implementations
│   │   ├── Panel.swift              # Panel enum, focus intents, attention states
│   │   ├── TerminalPanel.swift      # Terminal pane data model
│   │   ├── TerminalPanelView.swift  # Terminal pane UI
│   │   ├── BrowserPanel.swift       # Browser pane data model + API
│   │   ├── BrowserPanelView.swift   # Browser UI, toolbar, search overlay
│   │   ├── CmuxWebView.swift        # WebKit view wrapper with JS bridge
│   │   ├── BrowserPopupWindowController.swift # Window.open() popup handling
│   │   ├── MarkdownPanel.swift      # Markdown pane data model
│   │   ├── MarkdownPanelView.swift  # Markdown rendering UI
│   │   └── PanelContentView.swift   # Switch statement routing to panel UIs
│   │
│   ├── Find/                        # Terminal find overlay
│   │   ├── SurfaceSearchOverlay.swift # Terminal find UI
│   │   └── BrowserSearchOverlay.swift # Browser find UI
│   │
│   └── Update/                      # Auto-update UI
│       └── (update-related views)
│
├── CLI/                             # CLI stub for app
│   └── cmux.swift                   # Placeholder CLI (real one in daemon/remote)
│
├── daemon/                          # Remote daemon (Go)
│   └── remote/                      # SSH remote relay daemon
│       ├── cmd/                     # Go command packages
│       └── README.md                # RPC protocol documentation
│
├── cmuxTests/                       # Unit tests
│   ├── *Tests.swift                 # Test suites (30+)
│   └── (fixtures for SSH testing)
│
├── cmuxUITests/                     # UI tests
│   └── (AppKit-based UI test harnesses)
│
├── tests/                           # Python/shell test fixtures
│   └── fixtures/
│       └── ssh-remote/              # Mock SSH server for tests
│
├── tests_v2/                        # Python socket command tests
│   └── (pytest scripts connecting to socket)
│
├── vendor/                          # Git submodules
│   └── bonsplit/                    # Split pane layout library (Swift)
│
├── Resources/                       # Bundled resources
│   ├── ghostty/                     # Ghostty theme files, terminfo
│   ├── shell-integration/           # Shell integration scripts
│   ├── terminfo-overlay/            # Terminal capability overrides
│   └── bin/                         # Bundled executables
│
├── GhosttyTabs.xcodeproj/           # Xcode project
│   ├── project.pbxproj              # Build configuration
│   └── xcshareddata/xcschemes/      # Build schemes
│
├── Assets.xcassets/                 # App icons, images
├── AppIcon.icon/                    # Icon source
│
├── ghostty/                         # Ghostty submodule (C/Zig)
│   └── (not directly used in build; libghostty via xcframework)
│
├── scripts/                         # Build and development scripts
│   ├── setup.sh                     # Initialize submodules, build Ghostty
│   ├── reload.sh                    # Build Debug app with tag, launch
│   ├── reloadp.sh                   # Launch Release app
│   ├── reloads.sh                   # Launch staging Release app
│   └── bump-version.sh              # Version update utility
│
├── web/                             # Documentation website (Next.js)
│   ├── app/                         # Page routes
│   ├── i18n/                        # Localization setup
│   └── public/                      # Static assets
│
├── skills/                          # Claude skill definitions
│   ├── cmux/                        # Main cmux skill
│   ├── cmux-browser/                # Browser automation skill
│   ├── cmux-debug-windows/          # Debug window inspection
│   └── release/                     # Release workflow skill
│
├── design/                          # Icon/design source files
├── docs/                            # Markdown documentation
├── .github/                         # GitHub Actions workflows
│
├── cmux.entitlements                # App sandbox entitlements
├── cmux-Bridging-Header.h           # Objective-C bridging
├── ghostty.h                        # Ghostty C API header
│
├── CLAUDE.md                        # Agent instructions (see system context)
├── CHANGELOG.md                     # Release notes
├── PROJECTS.md                      # Project roadmap/status
├── Package.swift                    # Swift Package dependency declarations
├── Package.resolved                 # Locked versions
├── package.json                     # Node.js dependencies (web site)
└── bun.lock                         # Bun lock file (web site)
```

## Directory Purposes

**Sources/:**
- Purpose: All Swift source code for the main app
- Contains: Views, state management, terminal/browser integration, socket control, configuration
- Key files: AppDelegate.swift (500KB+), ContentView.swift (570KB+), TerminalController.swift (650KB+), Workspace.swift (400KB+), TabManager.swift (210KB+)

**Sources/Panels/:**
- Purpose: Panel type definitions and UI layers
- Contains: Terminal/Browser/Markdown implementations, focus routing, web view bridging
- Key files: BrowserPanel.swift (400KB+), CmuxWebView.swift (85KB), TerminalPanel.swift (10KB)

**Sources/Find/:**
- Purpose: Find/search UI overlays for terminal and browser
- Contains: Regex-based search UI, highlight rendering
- Key files: SurfaceSearchOverlay.swift, BrowserSearchOverlay.swift

**CLI/:**
- Purpose: Stub CLI entry point (actual CLI in daemon/remote for v2 commands)
- Contains: Placeholder Swift CLI (deprecated; remote daemon handles v1/v2 routing)
- Key files: cmux.swift

**daemon/remote/:**
- Purpose: Go-based SSH relay daemon and remote CLI wrapper
- Contains: RPC protocol, SSH tunnel setup, remote command proxy
- Key files: README.md (protocol), cmd/* (Go command packages)

**cmuxTests/:**
- Purpose: Unit test suites
- Contains: 30+ test files covering workspace, notifications, keyboard, browser, drag-drop, socket security
- Key patterns: XCTest framework, snapshot testing, deterministic workspace recreation

**Resources/:**
- Purpose: Bundled resources (themes, shell integration, terminfo)
- Contains: Ghostty themes directory, shell startup scripts, terminal capabilities
- Key files: Resources/ghostty/themes/, Resources/shell-integration/

**vendor/bonsplit/:**
- Purpose: Pane splitting and layout management
- Contains: Split tree data structure, drag-drop routing, divider rendering
- Key files: Sources/Bonsplit/ (Swift library)

**GhosttyTabs.xcodeproj/:**
- Purpose: Xcode build project
- Contains: Target definitions, build phases, scheme definitions
- Key patterns: Debug/Release schemes, test targets, framework embedding

## Key File Locations

**Entry Points:**

- `Sources/cmuxApp.swift`: SwiftUI @main entry, app init, environment setup
- `Sources/AppDelegate.swift`: AppKit window lifecycle, socket server init
- `daemon/remote/cmd/`: Go main functions for version/serve/cli commands

**Configuration:**

- `Sources/GhosttyConfig.swift`: Parse ~/.config/ghostty/config
- `Sources/SocketControlSettings.swift`: Socket auth mode, password store
- `Sources/KeyboardShortcutSettings.swift`: Keybinding persistence
- `GhosttyTabs.xcodeproj/project.pbxproj`: Build settings, framework paths

**Core Logic:**

- `Sources/TabManager.swift`: Workspace collection, auto-save trigger
- `Sources/Workspace.swift`: Individual workspace state, methods (split/close/focus/send)
- `Sources/TerminalController.swift`: Socket accept loop, v1/v2 command dispatch
- `Sources/TerminalNotificationStore.swift`: Unread tracking, attention state machine

**Terminal/Browser:**

- `Sources/GhosttyTerminalView.swift`: Ghostty C API bridge, PTY spawning, rendering
- `Sources/TerminalWindowPortal.swift`: NSView hosting Ghostty surfaces
- `Sources/Panels/BrowserPanel.swift`: WebKit controller, JavaScript API
- `Sources/Panels/CmuxWebView.swift`: WebKit wrapper with message handlers

**Testing:**

- `cmuxTests/TabManagerSessionSnapshotTests.swift`: Workspace persistence
- `cmuxTests/WorkspaceUnitTests.swift`: Workspace methods
- `cmuxTests/TerminalControllerSocketSecurityTests.swift`: Socket command auth
- `tests_v2/`: Python pytest socket command tests

**UI Layer:**

- `Sources/ContentView.swift`: Main view (sidebar + workspace)
- `Sources/WorkspaceContentView.swift`: Workspace split pane layout
- `Sources/Panels/PanelContentView.swift`: Panel type routing

## Naming Conventions

**Files:**

- PascalCase for SwiftUI views and classes: `ContentView.swift`, `TabManager.swift`
- Swift modules match file names exactly: `ContentView.swift` exports `struct ContentView`
- Test files append `Tests`: `TabManagerSessionSnapshotTests.swift` tests `TabManager`
- Suffixes indicate type: `View`, `Panel`, `Controller`, `Store`, `Settings`, `Portal`

**Directories:**

- Lowercase for grouping: `Sources/Panels/`, `Sources/Find/`, `Sources/Update/`
- Submodules in `vendor/`: `vendor/bonsplit/`
- Feature groups in `skills/`: `skills/cmux/`, `skills/cmux-browser/`

**Types (Swift):**

- PascalCase for types: `Workspace`, `TabManager`, `Panel`
- camelCase for properties/functions: `selectedWorkspaceID`, `selectWorkspace(_:)`
- Enum cases lowerCamelCase: `.afterCurrent`, `.splitRight`
- Settings enums group related UserDefaults: `WorkspaceTitlebarSettings`, `SocketControlSettings`

**Protocols:**

- No common suffix convention observed; inferred from context

## Where to Add New Code

**New Feature (e.g., command palette search):**

- Primary code: `Sources/` (new file or modify existing)
- Example: Terminal find → `Sources/Find/SurfaceSearchOverlay.swift`
- Tests: `cmuxTests/CommandPaletteSearchEngineTests.swift` (exists; add test case)
- Localization: Keys in `Resources/Localizable.xcstrings`

**New Component/Module (e.g., new panel type):**

- Implementation: `Sources/Panels/NewTypePanel.swift`
- View: `Sources/Panels/NewTypePanelView.swift`
- Integration: Register enum case in `Sources/Panels/Panel.swift`
- Routing: Add switch case in `Sources/Panels/PanelContentView.swift`
- Localization: Add strings in `Resources/Localizable.xcstrings`

**Utilities/Helpers:**

- Shared helpers: `Sources/` (as extensions or standalone utility files)
- Debug-only helpers: Wrap in `#if DEBUG` / `#endif`
- Example: `Sources/Backport.swift` for OS version compatibility

**Tests:**

- Unit tests: `cmuxTests/` (follow pattern: `FeatureUnitTests.swift`)
- UI tests: `cmuxUITests/` (AppKit-based, snapshot/interaction tests)
- Socket tests: `tests_v2/` (Python pytest, connect to socket)
- Fixtures: `tests/fixtures/` (mock SSH servers, test data)

**Documentation:**

- README translations: `README.*.md` (ISO language codes)
- API docs: `daemon/remote/README.md` (RPC protocol)
- Project status: `PROJECTS.md` (roadmap)

## Special Directories

**Resources/ghostty/:**
- Purpose: Ghostty theme files and terminal capabilities
- Generated: No (committed as-is from Ghostty repo)
- Committed: Yes (required for bundled theme support)

**/tmp/cmux-debug.log:**
- Purpose: Debug event ring buffer (DEBUG builds only)
- Generated: Yes (at runtime)
- Committed: No

**~/Library/Application Support/cmux/:**
- Purpose: Session persistence, Ghostty config user overrides
- Generated: Yes (at runtime)
- Committed: No (user-local data)

**DerivedData/cmux-<tag>/:**
- Purpose: Tagged Debug app build products with isolated socket/bundle ID
- Generated: Yes (by reload.sh --tag)
- Committed: No (build artifacts)

**~/.cmux/relay/:**
- Purpose: SSH relay authentication tokens and metadata
- Generated: Yes (by workspace.remote.configure)
- Committed: No (credentials)

## Import Organization

**Order (observed in codebase):**

1. Framework imports: `import AppKit`, `import SwiftUI`, `import Foundation`
2. System imports: `import Darwin`, `import Network`, `import Metal`
3. Third-party frameworks: `import Bonsplit`, `import Sentry`
4. C headers: `import Darwin` (via bridging header for `ghostty.h`)

**Path Aliases:**

- No custom path aliases detected (standard Swift imports only)

---

*Structure analysis: 2026-03-23*
