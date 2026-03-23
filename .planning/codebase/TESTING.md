# Testing Patterns

**Analysis Date:** 2026-03-23

## Test Framework

**Runner:**
- XCTest (Apple's native test framework)
- Xcode integration: Run via `xcodebuild -scheme cmux-unit` or Xcode UI
- No external test runner (Jest, Vitest, etc.)

**Assertion Library:**
- XCTest assertion functions: `XCTAssert()`, `XCTAssertEqual()`, `XCTAssertTrue()`, `XCTAssertFalse()`, `XCTAssertNil()`
- Custom predicate expectations: `XCTNSPredicateExpectation()` for async operations
- Waiter utility: `XCTWaiter().wait(for:timeout:)` for timeouts

**Run Commands:**
```bash
xcodebuild -scheme cmux-unit                    # Run all unit tests
xcodebuild -scheme cmux-uiTests                 # Run all UI tests
```

Testing is run via GitHub Actions CI, not locally (see project CLAUDE.md):
```bash
gh workflow run test-e2e.yml                    # Trigger E2E tests on VM
```

## Test File Organization

**Location:**
- Unit tests in `/home/twilson/code/cmux-linux/cmuxTests/` directory
- UI tests in `/home/twilson/code/cmux-linux/cmuxUITests/` directory
- Separate from source files (`Sources/`)

**Naming:**
- Test file names match source or feature: `TabManagerUnitTests.swift`, `SessionPersistenceTests.swift`, `BrowserConfigTests.swift`
- UI test files end with `UITests`: `CloseWindowConfirmDialogUITests.swift`, `CloseWorkspaceCmdDUITests.swift`

**Structure:**
```
cmuxTests/
├── TabManagerUnitTests.swift
├── SessionPersistenceTests.swift
├── BrowserConfigTests.swift
├── ... (80+ test files)
└── cmuxUITests/
    ├── CloseWindowConfirmDialogUITests.swift
    ├── BrowserOmnibarSuggestionsUITests.swift
    ├── BonsplitTabDragUITests.swift
    └── ... (20+ UI test files)
```

## Test Structure

**Suite Organization:**
All test classes use `final class XYZTests: XCTestCase` pattern. Example from `TabManagerUnitTests.swift`:
```swift
import XCTest

#if canImport(cmux_DEV)
@testable import cmux_DEV
#elseif canImport(cmux)
@testable import cmux
#endif

@MainActor
final class TabManagerChildExitCloseTests: XCTestCase {
    func testChildExitOnLastPanelClosesSelectedWorkspaceAndKeepsIndexStable() {
        let manager = TabManager()
        let first = manager.tabs[0]
        let second = manager.addWorkspace()
        let third = manager.addWorkspace()

        manager.selectWorkspace(second)
        XCTAssertEqual(manager.selectedTabId, second.id)

        guard let secondPanelId = second.focusedPanelId else {
            XCTFail("Expected focused panel in selected workspace")
            return
        }

        manager.closePanelAfterChildExited(tabId: second.id, surfaceId: secondPanelId)

        XCTAssertEqual(manager.tabs.map(\.id), [first.id, third.id])
        XCTAssertEqual(
            manager.selectedTabId,
            third.id,
            "Expected selection to stay at the same index after deleting the selected workspace"
        )
    }
}
```

**Patterns:**
- No setup/teardown per test (methods use `override func setUp()` and `override func tearDown()` when needed)
- Arrange-Act-Assert structure: setup objects, perform action, verify results
- Guard statements with `XCTFail` for preconditions
- Descriptive assertion messages as second parameter
- Multiple assertions per test when validating related behavior

**Main Actor:**
- Test classes marked `@MainActor` when testing UI code: `@MainActor final class TabManagerChildExitCloseTests`
- Allows direct access to `TabManager` and `Workspace` without `DispatchQueue.main.async`

## Mocking

**Framework:** Manual mock objects and spy patterns

**Patterns:**
```swift
private final class ActionSpy: NSObject {
    private(set) var invoked: Bool = false

    @objc func didInvoke(_ sender: Any?) {
        invoked = true
    }
}

private final class WindowCyclingActionSpy: NSObject {
    weak var firstWindow: NSWindow?
    weak var secondWindow: NSWindow?
    private(set) var invocationCount = 0

    @objc func cycleWindow(_ sender: Any?) {
        invocationCount += 1
        guard let firstWindow, let secondWindow else { return }

        if NSApp.keyWindow === firstWindow {
            secondWindow.makeKeyAndOrderFront(nil)
        } else {
            firstWindow.makeKeyAndOrderFront(nil)
        }
    }
}
```

**Injection Patterns:**
- Closures for behavior injection: `manager.confirmCloseHandler = { title, message, acceptCmdD in ... }`
- Optional closures for optional behavior: `loadFromDisk: (_ preferredColorScheme: ColorSchemePreference) -> GhosttyConfig`
- Private test properties: `var prompts: [(title: String, message: String, acceptCmdD: Bool)] = []` to capture invocations

**What to Mock:**
- User interaction (button clicks, keyboard events in UI tests)
- External system state (file system paths, temporary directories)
- Dialogs and confirmations (via closure injection)
- Network responses (via URL protocol overrides in `UpdateTestURLProtocol.swift`)

**What NOT to Mock:**
- Core domain objects (`Workspace`, `TabManager`) - test real behavior
- File I/O with temp directories - use `FileManager.default.temporaryDirectory` for isolated tests
- Notifications and delegates - test full flow when meaningful

## Fixtures and Factories

**Test Data:**
```swift
// From SessionPersistenceTests.swift
let tempDir = FileManager.default.temporaryDirectory
    .appendingPathComponent("cmux-session-tests-\(UUID().uuidString)", isDirectory: true)
try? FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
defer { try? FileManager.default.removeItem(at: tempDir) }

let snapshotURL = tempDir.appendingPathComponent("session.json", isDirectory: false)
let snapshot = makeSnapshot(version: SessionSnapshotSchema.currentVersion)

XCTAssertTrue(SessionPersistenceStore.save(snapshot, fileURL: snapshotURL))
```

**Factory Functions:**
- Test setup functions like `makeSnapshot(version:)` used in `SessionPersistenceTests.swift`
- Temporary directories created with `UUID()` for isolation
- `defer` cleanup ensures files removed after test

**Location:**
- Fixtures defined inline in test files
- Shared helper functions at module level (e.g., `drainMainQueue()` in `TabManagerUnitTests.swift`)
- No separate fixture directory (test data generated at runtime)

## Coverage

**Requirements:** Not enforced (no coverage targets in build configuration)

**View Coverage:**
- No built-in coverage reports detected
- Xcode can generate coverage via scheme settings
- Project runs tests via GitHub Actions; coverage not explicitly reported

## Test Types

**Unit Tests:**
- Scope: Single class or function behavior
- Files: `cmuxTests/*.swift` (80+ test files)
- Approach: Direct object instantiation, assertion on state/return values
- Example: `TabManagerUnitTests.testChildExitOnLastPanelClosesSelectedWorkspaceAndKeepsIndexStable()` tests TabManager state mutations
- Characteristics:
  - Tests run fast (no file I/O except isolated temp directories)
  - Focus on observable behavior (public API)
  - Use `@MainActor` for thread safety with UI models

**Integration Tests:**
- Scope: Multiple components interacting
- Files: `cmuxTests/*` (mixed with unit tests by purpose)
- Approach: Set up complex state (multiple workspaces, panels, surfaces) and verify interactions
- Example: `SessionPersistenceTests` combines Workspace snapshots + file I/O
- Characteristics:
  - May test filesystem operations (using temp directories)
  - May test JSON serialization round-trips
  - Verify state restoration across system boundaries

**E2E Tests:**
- Framework: XCTest UI automation (`XCUIApplication`)
- Files: `cmuxUITests/*.swift`
- Approach: Launch actual app, interact via keyboard/mouse, verify UI state
- Example from `CloseWindowConfirmDialogUITests.swift`:
  ```swift
  func testCmdCtrlWShowsCloseWindowConfirmationText() {
      let app = XCUIApplication()
      app.launchEnvironment["CMUX_TAG"] = launchTag
      app.launch()
      XCTAssertTrue(
          ensureForegroundAfterLaunch(app, timeout: 12.0),
          "Expected app to launch for close-window confirmation test. state=\(app.state.rawValue)"
      )

      app.typeKey("w", modifierFlags: [.command, .control])

      XCTAssertTrue(
          waitForCloseWindowAlert(app: app, timeout: 5.0),
          "Expected Cmd+Ctrl+W to show the close window confirmation alert"
      )

      clickCancelOnCloseWindowAlert(app: app)

      XCTAssertFalse(
          isCloseWindowAlertPresent(app: app),
          "Expected close window confirmation alert to dismiss after clicking Cancel"
      )
  }
  ```
- Test environment: Uses `CMUX_TAG` launch environment variable for test isolation
- Timeouts: Generous (5-12 second timeouts for app launch, UI operations)
- App lifecycle: `app.launch()`, `app.activate()`, `app.terminate()`

## Common Patterns

**Async Testing:**
```swift
// From CloseWindowConfirmDialogUITests
private func waitForCloseWindowAlert(app: XCUIApplication, timeout: TimeInterval) -> Bool {
    let expectation = XCTNSPredicateExpectation(
        predicate: NSPredicate { _, _ in
            self.isCloseWindowAlertPresent(app: app)
        },
        object: NSObject()
    )
    return XCTWaiter().wait(for: [expectation], timeout: timeout) == .completed
}
```
- Uses `XCTNSPredicateExpectation` for polling-based waits
- Waiter returns `.completed` or `.timedOut`
- UI tests use `XCUIApplication.wait(for:timeout:)` for app state changes

**Draining Main Queue:**
```swift
func drainMainQueue() {
    let expectation = XCTestExpectation(description: "drain main queue")
    DispatchQueue.main.async {
        expectation.fulfill()
    }
    XCTWaiter().wait(for: [expectation], timeout: 1.0)
}
```
- Used in `TabManagerUnitTests` to ensure pending main-thread work completes
- Called twice in some tests to ensure cascading dispatches settle

**Error Testing:**
```swift
// From BrowserConfigTests
guard let added = class_addMethod(...) else {
    fatalError("Unable to install CmuxWebView _inspector test override")
}
```
- Test setup uses `fatalError()` for impossible preconditions
- Assertion failures in test setup prevent test from running
- Real code uses optional handling; tests use early failure

**Temporary File Testing:**
```swift
// From SessionPersistenceTests
let root = FileManager.default.temporaryDirectory
    .appendingPathComponent("cmux-session-markdown-\(UUID().uuidString)", isDirectory: true)
try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
defer { try? FileManager.default.removeItem(at: root) }
```
- All temp files use `UUID()` for isolation (no test-to-test interference)
- `defer` guarantees cleanup even on failure
- Tests pass `fileURL` to functions under test

**Predicate Testing:**
```swift
// From SessionPersistenceTests - validating invalid state is rejected
func testLoadRejectsSchemaVersionMismatch() {
    // ... create snapshot with wrong version ...
    XCTAssertNil(SessionPersistenceStore.load(fileURL: snapshotURL))
}
```
- Tests negative cases (invalid input rejected, nil returned)
- Validates error handling paths

## Test Conditional Imports

**Pattern:**
```swift
#if canImport(cmux_DEV)
@testable import cmux_DEV
#elseif canImport(cmux)
@testable import cmux
#endif
```
- Used in all test files to handle debug vs. release builds
- `@testable` enables access to internal/private members
- Supports tagged debug builds (e.g., `cmux_DEV` with tag)

## Test Helpers at Module Level

**From TabManagerUnitTests:**
```swift
func drainMainQueue() {
    let expectation = XCTestExpectation(description: "drain main queue")
    DispatchQueue.main.async {
        expectation.fulfill()
    }
    XCTWaiter().wait(for: [expectation], timeout: 1.0)
}
```

**From BrowserConfigTests (global test support):**
```swift
var cmuxUnitTestInspectorAssociationKey: UInt8 = 0
var cmuxUnitTestInspectorOverrideInstalled = false

extension CmuxWebView {
    @objc func cmuxUnitTestInspector() -> NSObject? {
        objc_getAssociatedObject(self, &cmuxUnitTestInspectorAssociationKey) as? NSObject
    }
}

func installCmuxUnitTestInspectorOverride() {
    guard !cmuxUnitTestInspectorOverrideInstalled else { return }
    // ... use Objective-C runtime to install method override ...
}
```

## CI/Test Execution

**Local:** Tests run via `xcodebuild` schemes locally (though project CLAUDE.md notes "Never run tests locally" - all tests expected to run on GitHub Actions)

**CI:** Tests run via GitHub Actions workflows:
- Unit tests: `xcodebuild -scheme cmux-unit`
- UI/E2E tests: `gh workflow run test-e2e.yml` (runs on remote VM with macOS)
- Python socket tests: Connect to running cmux instance via socket

**Environment:** Tagged builds use `CMUX_TAG` environment variable for app isolation during tests

---

*Testing analysis: 2026-03-23*
