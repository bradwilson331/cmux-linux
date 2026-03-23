# Coding Conventions

**Analysis Date:** 2026-03-23

## Naming Patterns

**Files:**
- Pascal case for Swift files: `TabManager.swift`, `GhosttyConfig.swift`, `BrowserPanel.swift`
- Grouped by feature in subdirectories: `Sources/Panels/`, `Sources/Update/`, `Sources/Find/`
- Test files match source file names with `Tests` suffix: `TabManagerUnitTests.swift`, `SessionPersistenceTests.swift`
- UI test files end with `UITests`: `CloseWindowConfirmDialogUITests.swift`, `BrowserImportProfilesUITests.swift`

**Types (Structs/Classes):**
- Pascal case: `TabManager`, `Workspace`, `GhosttyConfig`, `SidebarStatusEntry`
- Enums use singular form: `NewWorkspacePlacement`, `ColorSchemePreference`, `SidebarMetadataFormat`
- Settings/configuration enums are descriptive: `WorkspaceAutoReorderSettings`, `LastSurfaceCloseShortcutSettings`, `SidebarBranchLayoutSettings`

**Functions:**
- Camel case starting with lowercase: `cmuxSurfaceContextName()`, `cmuxCurrentSurfaceFontSizePoints()`, `cmuxInheritedSurfaceConfig()`
- Static utility functions use prefix pattern: `GhosttyConfig.load()`, `SessionPersistenceStore.save()`
- Test functions use descriptive names with behavior: `testChildExitOnLastPanelClosesSelectedWorkspaceAndKeepsIndexStable()`

**Variables:**
- Camel case: `scrollbackLimit`, `unfocusedSplitOpacity`, `backgroundColor`, `workingDirectory`
- Private variables prefixed with underscore not used; instead use `private let` declarations
- Test helper variables are local and descriptive: `prompts`, `candidates`, `initialTabIds`, `manager`, `workspace`

**Enums with Static Properties:**
- Configuration enums store keys and defaults as static let: `WorkspaceAutoReorderSettings.key`, `WorkspaceAutoReorderSettings.defaultValue`
- Helper functions on configuration enums check UserDefaults: `isEnabled(defaults:)`, `closesWorkspace(defaults:)`, `hidesAllDetails(defaults:)`

## Code Style

**Formatting:**
- No explicit formatter detected; follows Swift conventions
- Indentation: 4 spaces (standard Swift)
- Line length: variable, some lines exceed 120 characters (see `ContentView.swift`)
- Brace style: opening brace on same line (K&R style)

**Linting:**
- No `.swiftlint.yml` or linting configuration detected
- Code follows standard Swift conventions and best practices
- Compilation against Swift 5.9+

**Import Organization:**
- Standard library imports first: `import Foundation`, `import AppKit`, `import SwiftUI`
- Then platform-specific: `import Darwin`, `import CoreVideo`, `import CoreText`
- Then custom frameworks: `import Bonsplit`, `import UniformTypeIdentifiers`, `import WebKit`
- Optional conditional imports: `import ObjectiveC.runtime` (for runtime introspection)
- Example from `Workspace.swift`:
  ```swift
  import Foundation
  import SwiftUI
  import AppKit
  import Bonsplit
  import Combine
  import CryptoKit
  import Darwin
  import Network
  import CoreText
  ```

**Path Aliases:**
- No path aliases detected (no `@_exported import` or swiftpm remapping)
- Full module references used throughout

## Error Handling

**Patterns:**
- Custom error types use enums conforming to `Error` or `LocalizedError`: `RemoteDropUploadError`, `TerminalImageTransferExecutionError`, `FeedbackComposerBridgeError`
- LocalizedError implementations provide `errorDescription` via String(localized:):
  ```swift
  enum RemoteDropUploadError: LocalizedError {
      case unavailable
      case invalidFileURL
      case uploadFailed(String)

      var errorDescription: String? {
          switch self {
          case .unavailable:
              String(localized: "error.remoteDrop.unavailable", defaultValue: "Remote drop is unavailable.")
          case .invalidFileURL:
              String(localized: "error.remoteDrop.invalidFileURL", defaultValue: "Dropped item is not a file URL.")
          case .uploadFailed(let detail):
              String.localizedStringWithFormat(...)
          }
      }
  }
  ```
- Guard statements for early returns with `XCTFail` in tests
- Optional unwrapping with `guard let` pattern preferred over force unwrap except in test setup

## Logging

**Framework:** Custom `dlog()` function (wrapped in `#if DEBUG`)

**Patterns:**
- Debug logging only in DEBUG builds: `#if DEBUG ... dlog(...) #endif`
- Log entries include semantic tags: `"typing.delay"`, `"typing.timing"`, `"zoom.inherit"`, `"startup.recovery"`
- Structure: `dlog("tag.subname key=value key2=value2 extra_message")`
- Example from `Workspace.swift`:
  ```swift
  #if DEBUG
  dlog(
      "zoom.inherit context=\(cmuxSurfaceContextName(context)) " +
      "inherited=\(inheritedText) runtime=\(runtimeText) final=\(finalText)"
  )
  #endif
  ```
- Log paths in `AppDelegate.swift` indicate location tracking: `dlog("typing.delay path=\(path) ...")`
- Debug event log stores entries to file via `DebugEventLog.shared.dump()` (see project CLAUDE.md)

## Comments

**When to Comment:**
- MARK sections used extensively to organize code: `// MARK: - Section Name`
- Comments above functions explain non-obvious behavior
- Legacy code marked with explanation: `// The old Tab class is replaced by Workspace`
- Implementation notes inline: `// Make runtime zoom inheritance explicit, even when Ghostty's inherit-font-size config is disabled.`

**JSDoc/TSDoc:**
- Not used in this Swift codebase
- No systematic documentation comments on public functions
- Comments are prose style (not doc-style)

**MARK Organization:**
- Heavy use of `// MARK: - ` sections to segment code:
  - `Sources/TabManager.swift`: Contains marks like "Agent PID Sweep", "Surface/Panel Compatibility Layer", "Panel/Surface ID Access"
  - `Sources/Panels/MarkdownPanel.swift`: Organized as "File watching", "Init", "Panel protocol", "File I/O"
  - Enables code folding in Xcode for large files

## Function Design

**Size:** Functions range from 3-line utilities to 200+ line methods
- Small focused functions for single operations: `cmuxSurfaceContextName()`
- Larger functions for complex state mutations: `didClosePanelAfterChildExit()`, workspace restoration
- Test functions are descriptive but compact: average 20-30 lines

**Parameters:**
- Explicit parameter names: `(tabId:, surfaceId:)`, `(inPane:, filePath:, focus:)`
- Default parameters used for configuration: `func load(preferredColorScheme: ColorSchemePreference? = nil, useCache: Bool = true)`
- Trailing closures for callbacks: `loadFromDisk: (_ preferredColorScheme: ColorSchemePreference) -> GhosttyConfig`

**Return Values:**
- Optional returns for fallible operations: `-> String?`, `-> GhosttyConfig?`
- Void returns for side-effect operations: `closeWorkspace(_:)`, `selectWorkspace(_:)`
- Tuple returns for multiple values: `(title: String, message: String, acceptCmdD: Bool)` in confirmation dialogs

## Module Design

**Exports:**
- Main app entry: `Sources/cmuxApp.swift` contains `@main` entry point
- Workspace model: `Sources/Workspace.swift` exports `typealias Tab = Workspace` for backwards compatibility
- Tab management: `Sources/TabManager.swift` - central orchestration
- Terminal integration: `Sources/TerminalController.swift` - socket protocol implementation
- Configuration: `Sources/GhosttyConfig.swift` - static configuration loading with caching

**Barrel Files:**
- No explicit barrel/index files detected
- Each file is self-contained; imports done at module level

**Extension Organization:**
- Extensions used for test helpers: `CmuxWebView` extended with `cmuxUnitTestInspector()`
- Extensions used for Objective-C interop: `WKWebView` extended with `cmuxSetUnitTestInspector()`

## Localization

**Pattern:** All user-facing strings use `String(localized:defaultValue:)`:
```swift
String(localized: "workspace.placement.top", defaultValue: "Top")
String(localized: "error.remoteDrop.unavailable", defaultValue: "Remote drop is unavailable.")
String.localizedStringWithFormat(
    String(localized: "dialog.closeWorkspaces.message", defaultValue: "..."),
    locale: .current,
    arguments...
)
```
- Localization keys stored in `Resources/Localizable.xcstrings` (referenced in project CLAUDE.md)
- All dialog titles, menu items, tooltips, and error messages must be localized

## Concurrency

**Pattern:**
- Main actor annotation: `@MainActor` on test classes and UI-touching code
- DispatchQueue usage for off-main work: `DispatchQueue.main.async { ... }`
- Test helpers to drain main queue: `drainMainQueue()` function that yields to event loop
- No SwiftUI property wrappers that cause performance issues: TabItemView avoids `@EnvironmentObject` (documented in project CLAUDE.md)

## Testing Convention Extensions

**Test Helpers:**
- Private helper methods prefixed with action names: `clickCancelOnCloseWindowAlert()`, `waitForCloseWindowAlert()`, `isCloseWindowAlertPresent()`
- Spy pattern for behavior verification: `ActionSpy`, `WindowCyclingActionSpy` classes
- Predicate-based waits using `XCTNSPredicateExpectation`

---

*Convention analysis: 2026-03-23*
