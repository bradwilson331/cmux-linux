# Codebase Concerns

**Analysis Date:** 2026-03-23

## Tech Debt

**Large monolithic controller classes:**
- Issue: `TerminalController.swift` (15,350 lines), `ContentView.swift` (13,902 lines), and `AppDelegate.swift` (12,753 lines) are among the largest files in the codebase, each handling multiple responsibilities
- Files: `Sources/TerminalController.swift`, `Sources/ContentView.swift`, `Sources/AppDelegate.swift`
- Impact: Hard to test, maintain, and reason about; difficult to isolate concerns; complex refactoring risk
- Fix approach: Break these into smaller focused modules (e.g., separate concerns like socket handling, UI composition, app lifecycle)

**Terminal window portal recovery workaround:**
- Issue: Conditional feature flag `CMUX_ISSUE_483_PORTAL_RECOVERY` wraps a transient recovery mechanism with retry budget (12 retries) and deferred sync scheduling
- Files: `Sources/TerminalWindowPortal.swift` (lines 577-602, and throughout with debug conditional compilation)
- Impact: Fragile recovery logic that may mask underlying instability; not enabled by default, suggesting unresolved stability issues
- Fix approach: Investigate root cause of issue #483 (portal detachment/geometry sync failures); fix underlying issue and remove workaround

**Unsafe memory management in TerminalController:**
- Issue: 12 instances of `nonisolated(unsafe)` properties in `TerminalController` (lines 41-68) for socket state management; stateless thread access to shared mutable state
- Files: `Sources/TerminalController.swift` (lines 41-68, 439-480)
- Impact: Potential data races if socket listener thread and main thread access state simultaneously without proper synchronization
- Fix approach: Audit concurrent access patterns; replace unsafe properties with proper synchronization (locks, actor isolation, or concurrent collections)

**Index-based vs short-ID API inconsistency:**
- Issue: TODO.md item (P0): "Remove all index-based APIs in favor of short ID refs (surface:N, pane:N, workspace:N, window:N)"
- Files: `CLI/cmux.swift`, `Sources/TerminalController.swift` (command parsing), legacy v1 API handlers
- Impact: Two parallel API surfaces increase maintenance burden; clients may break during migration
- Fix approach: Phase out index-based APIs; provide migration path for clients; fully transition to short ID refs

**Workspace-relative command execution missing:**
- Issue: TODO.md item (P0): "CLI commands should be workspace-relative using CMUX_WORKSPACE_ID env var (not focused workspace) so agents in background workspaces don't affect the user's active workspace"
- Files: `Sources/TerminalController.swift` (command routing), `CLI/cmux.swift` (command dispatch)
- Affected commands: `send`, `send-key`, `send-panel`, `send-key-panel`, `new-split`, `new-pane`, `new-surface`, `close-surface`, `list-panes`, `list-pane-surfaces`, `list-panels`, `focus-pane`, `focus-panel`, `surface-health`
- Impact: Background agent workspaces can steal focus or affect user's active workspace unexpectedly
- Fix approach: Thread CMUX_WORKSPACE_ID through all command handlers; default to explicit ID rather than focused workspace

**Missing close-workspace argument validation:**
- Issue: TODO.md item (P0): "`close-workspace` with no args should require explicit workspace short ID or UUID with clear error message if missing"
- Files: `Sources/TerminalController.swift` (close-workspace handler)
- Impact: Accidental workspace closure risk; ambiguous error messages if workspace ID missing
- Fix approach: Add validation to require explicit workspace ID; emit clear error message for missing argument

## Known Bugs

**Terminal title updates suppressed when workspace unfocused:**
- Symptoms: Terminal title changes (e.g., Claude Code loading indicator) don't update in sidebar until workspace is switched to active
- Files: `Sources/TerminalController.swift` (title reporting), `Sources/Workspace.swift` (title update gates)
- Root cause: Likely title update gating based on workspace focus state rather than allowing all updates to propagate
- Workaround: Switch to affected workspace to see updated title
- Priority: P0

**Sidebar tab reorder drag state persistence:**
- Symptoms: After drag operation, tab remains dimmed with blue drop indicator line visible, indicating incomplete drag cleanup
- Files: `Sources/ContentView.swift` (tab view composition), `Sources/Panels/BrowserPanelView.swift` (drag handling)
- Root cause: Drag state not fully cleaned up after drop or cancel
- Workaround: Re-initiate a full drag and drop or switch tabs
- Priority: High

**File/image drag-and-drop renders URL instead of path:**
- Symptoms: Dragging files or images into terminal pastes the file:// URL instead of local file path
- Files: `Sources/TerminalView.swift` (paste handling), `Sources/Panels/TerminalPanelView.swift` (drop target)
- Root cause: Ghostty drop handler may not be converting file URLs to paths before display
- Expected behavior: Ghostty supports dropping files as paths (per TODO comment)
- Priority: Medium

**Keyboard shortcuts stop working after browser tab open:**
- Symptoms: Up/down arrow keys and possibly other keyboard shortcuts become non-responsive in terminal after opening a browser tab
- Files: `Sources/Panels/BrowserPanelView.swift` (focus handling), `Sources/GhosttyTerminalView.swift` (key routing)
- Root cause: Browser panel may be capturing keyboard events or not releasing first responder properly
- Workaround: Switch focus between panels to reset keyboard routing
- Priority: High

**Notification unread marker doesn't sort to top:**
- Symptoms: Marking a notification as unread doesn't move it to the top of the notification list
- Files: `Sources/TerminalNotificationStore.swift` (notification ordering logic)
- Root cause: Unread-to-top sort not implemented or triggered on state change
- Priority: Low

**Browser cmd+shift+H ring flashes only once:**
- Symptoms: Browser history navigation ring animation (cmd+shift+H) flashes only once instead of twice
- Files: `Sources/Panels/BrowserPanelView.swift` (history animation)
- Root cause: Animation trigger condition may not be firing on second invocation
- Priority: Low

## Fragile Areas

**Portal-based terminal view hosting:**
- Files: `Sources/TerminalWindowPortal.swift` (1,960 lines), `Sources/BrowserWindowPortal.swift` (3,791 lines)
- Why fragile: Complex geometry synchronization between SwiftUI splits (Bonsplit) and AppKit portal views hosting Ghostty surfaces; transient recovery logic with retry budget suggests frequent detachment; heavy reliance on frame tracking and constraint installation
- Safe modification: Do not remove or significantly alter geometry sync scheduling without understanding issue #483; test portal recovery behavior when adding new split operations; avoid removing `#if DEBUG` logging
- Test coverage: Portal geometry bugs likely caught by UI tests; limited unit test coverage of frame calculations

**Socket listener and thread-based command dispatch:**
- Files: `Sources/TerminalController.swift` (socket listener, accept loop, per-client thread handlers)
- Why fragile: Multi-threaded socket handling with `nonisolated(unsafe)` state; backoff/retry logic with complex state machines; per-client thread spawning without bounded pool
- Safe modification: Do not simplify thread spawning without understanding accept loop generation tracking; do not remove state snapshot pattern without adding explicit synchronization; test socket listener health checks after changes
- Test coverage: Python socket tests (tests_v2/) verify socket behavior at protocol level; UI tests exercise socket commands; limited unit testing of socket state transitions

**Terminal focus steal prevention system:**
- Files: `Sources/TerminalController.swift` (socket command policy depth/allowance stack), `Sources/AppDelegate.swift` (window activation gates)
- Why fragile: Stack-based focus allowance tracking across nested command execution; policy context must be properly unwound to avoid leaks or unintended focus mutations
- Safe modification: Do not modify socketCommandPolicyDepth or socketCommandFocusAllowanceStack without wrapping in withSocketCommandPolicy; test that non-focus-intent commands don't steal focus (documented in socket-focus-steal-audit.todo.md)
- Test coverage: Focus steal audit tests present; socket command routing tests verify policy enforcement

**Browser automation API and selector execution:**
- Files: `Sources/Panels/BrowserPanel.swift` (10,182 lines), `Sources/Panels/BrowserPanelView.swift` (6,392 lines)
- Why fragile: WKWebView JavaScript evaluation with complex selector matching; timeout-based retries for transient not_found races; snapshot capture and diagnostics with bounded output
- Safe modification: Do not modify selector failure diagnostics without testing retry/timeout behavior; do not add new selector families without comprehensive v2 test coverage; test both supported and unsupported selector types
- Test coverage: Comprehensive v2 test suite (tests_v2/test_browser_api_*.py); unsupported matrix tests; placement policy regression tests

**Remote workspace and daemon communication:**
- Files: `Sources/Workspace.swift` (10,297 lines), `Sources/TerminalSSHSessionDetector.swift` (807 lines)
- Why fragile: SSH session detection, daemon bootstrap over stdio, relay token verification, local proxy port allocation with deterministic test hooks, and reconnect/disconnect orchestration
- Safe modification: Do not change relay token validation or proxy port logic without understanding test hook override (localProxyPort configuration); test remote daemon startup timeouts; verify SSH session detection with various shell configurations
- Test coverage: Workspace remote connection tests (WorkspaceRemoteConnectionTests.swift); manual unread tests; likely VM-level integration tests

## Performance Bottlenecks

**Terminal controller command dispatch thread spawning:**
- Problem: Each socket client connection spawns a new thread for command handling (per `clientHandlers` dictionary)
- Files: `Sources/TerminalController.swift` (socket accept loop, thread creation)
- Cause: No bounded thread pool; high-frequency socket connections can exhaust system resources
- Improvement path: Implement bounded thread pool or async/await dispatch instead of per-connection threads; measure socket connection frequency under load

**Large view hierarchies in ContentView:**
- Problem: ContentView.swift (13,902 lines) may be rendering large tab and pane hierarchies without view pooling or lazy loading
- Files: `Sources/ContentView.swift` (TabItemView ForEach with .equatable() pattern suggests awareness of perf issues)
- Cause: SwiftUI body re-evaluation on every state change if not carefully isolated
- Improvement path: Profile body re-evaluation count; ensure all tab/pane views use .equatable() and avoid capturing @State/@ObservedObject unnecessarily; consider lazy containers for very large workspaces

**Portal geometry synchronization overhead:**
- Problem: WindowTerminalPortal performs full sync on geometry changes, frame updates, and constraint installation
- Files: `Sources/TerminalWindowPortal.swift` (geometry observer installation, sync scheduling, divider region collection)
- Cause: Collecting all split divider regions and syncing views on every layout change
- Improvement path: Batch geometry updates; implement dirty-flag tracking to skip unnecessary syncs; profile sync frequency during split/resize operations

**JavaScript evaluation latency in browser panels:**
- Problem: Selector execution (find.role, find.text, etc.) requires WKWebView JS evaluation with timeout-based retries
- Files: `Sources/Panels/BrowserPanel.swift` (selector execution and retry logic)
- Cause: JS context isolation, frame serialization, and lack of native selector APIs in WebKit
- Improvement path: Cache frequently-used selector results; implement early timeout for unsupported selectors; profile JS evaluation latency under load

## Scaling Limits

**Socket listener backlog limit:**
- Current capacity: Backlog set to 128 (Sources/TerminalController.swift line 58)
- Limit: If > 128 concurrent socket connections arrive before accept loop processes them, additional connections will be dropped
- Scaling path: Increase backlog constant; implement connection pooling; profile actual socket connection rates in production

**Terminal surface handle management:**
- Current capacity: V2 browser element refs (V2BrowserElementRefEntry) stored in unbounded dictionary
- Files: `Sources/TerminalController.swift` (private var browserElementRefs: [ObjectIdentifier: V2BrowserElementRefEntry])
- Limit: Long-running sessions with thousands of browser elements could exhaust memory
- Scaling path: Implement TTL-based eviction for element refs; add metrics for dictionary size

**Transient recovery retry budget:**
- Current capacity: 12 retries per portal entry (Sources/TerminalWindowPortal.swift line 576)
- Limit: Geometry recovery attempts exhausted after 12 failures; portals become permanently detached
- Scaling path: Make retry budget configurable; implement exponential backoff; add diagnostics to detect persistent geometry issues

## Security Considerations

**Socket permission and access control:**
- Risk: Unix socket at `~/.config/cmux/cmux.sock` (or custom path) is created with default permissions; any user who can access the socket can control terminal
- Files: `Sources/TerminalController.swift` (socket binding), `Sources/SocketControlSettings.swift` (socket path configuration)
- Current mitigation: Socket in user home directory; cmuxOnly mode for local access
- Recommendations: Audit socket permission bits on creation; document security model for shared machines; consider user-id validation in socket handshake

**Ghostty submodule fork security:**
- Risk: Custom fork of Ghostty (manaflow-ai/ghostty) used instead of upstream; fork could drift and miss critical security updates
- Files: `.gitmodules`, `ghostty/` submodule
- Current mitigation: Fork kept in sync with upstream main (per CONTRIBUTING.md)
- Recommendations: Establish automated fork sync schedule; subscribe to upstream security advisories; audit fork changes for security implications

**SSH relay and remote daemon trust:**
- Risk: Remote daemon (`cmuxd-remote`) runs on SSH target with stdio transport; relay token verification could be weak
- Files: `Sources/Workspace.swift` (relay token verification, remote daemon bootstrap)
- Current mitigation: Relay token generated from hex string validation
- Recommendations: Document token generation and validation strength; verify SSH key-based authentication is enforced; audit daemon input validation

**Browser automation and untrusted web content:**
- Risk: `browser.*` commands can execute JavaScript in WKWebView contexts; no sandboxing of untrusted sites
- Files: `Sources/Panels/BrowserPanel.swift` (JS evaluation), `Sources/Panels/BrowserPanelView.swift` (web content handling)
- Current mitigation: WKWebView default security policies
- Recommendations: Audit evaluate(javascript:) calls for injection risks; document that browser panels should not be used with untrusted content; consider content security policy headers

**File drag-and-drop and path traversal:**
- Risk: Drag-and-drop file handling could potentially allow path traversal or unintended file access
- Files: `Sources/TerminalView.swift` (paste handling), `Sources/Panels/TerminalPanelView.swift` (drop target)
- Current mitigation: Ghostty's native file drop handler
- Recommendations: Validate file paths before passing to terminal; audit Ghostty's file drop implementation; test with symlinks and special file types

## Dependencies at Risk

**Ghostty fork maintenance burden:**
- Risk: Maintaining a fork of Ghostty requires continuous sync with upstream and conflict resolution
- Impact: New upstream versions require merge/rebase work; fork changes could be lost in large upstream rewrites
- Migration plan: Document all fork changes in `docs/ghostty-fork.md` (already done); establish quarterly upstream sync schedule; consider upstreaming cmux-specific features

**WebKit version dependencies:**
- Risk: WKWebView behavior varies across macOS versions; new macOS releases may break browser automation
- Impact: Browser panel features may require new WKWebView API availability checks; selector APIs unavailable on older macOS
- Migration plan: Maintain compatibility matrix for macOS versions; test browser API against all supported macOS versions; implement graceful degradation for unavailable APIs

**Xcode and Swift version requirements:**
- Risk: Project requires Xcode 15+ and macOS 14+; future Xcode versions may deprecate APIs
- Impact: Older machines cannot build; Swift concurrency/actor model requires recent Xcode
- Migration plan: Monitor Xcode release notes for deprecations; plan Swift 6.0 migration (non-copyable types, complete concurrency checking)

## Test Coverage Gaps

**Terminal title update gating behavior:**
- What's not tested: Specific conditions that suppress title updates when workspace unfocused; title update propagation to sidebar
- Files: `Sources/TerminalController.swift` (title reporting), `Sources/Workspace.swift` (title update logic)
- Risk: Regression in title update visibility; CI may not catch title suppression bugs
- Priority: High

**Socket listener health and recovery transitions:**
- What's not tested: State transitions in socket listener accept loop (running → failing → recovering); edge cases in backoff/retry logic
- Files: `Sources/TerminalController.swift` (AcceptFailureRecoveryAction, recovery scheduling)
- Risk: Socket listener may enter broken state and not recover; health checks may give false positives
- Priority: High

**Portal geometry sync edge cases:**
- What's not tested: Rapid resize/split/join operations; portal detachment and reattachment; geometry conflicts between Bonsplit and Ghostty
- Files: `Sources/TerminalWindowPortal.swift` (geometry sync, transient recovery)
- Risk: Transient recovery workaround may mask underlying geometry bugs; new split/resize patterns could trigger portal detachment
- Priority: High

**Browser keyboard event routing after panel switch:**
- What's not tested: Keyboard focus restoration after switching from browser panel back to terminal; event routing integrity across panel transitions
- Files: `Sources/Panels/BrowserPanelView.swift` (focus handling), `Sources/GhosttyTerminalView.swift` (key routing)
- Risk: Arrow key regression after browser tab open; similar bugs on future keyboard integration changes
- Priority: High

**Remote daemon bootstrap timeout and reconnection:**
- What's not tested: SSH startup delays; cmuxd-remote availability probing; reconnect behavior after brief network loss
- Files: `Sources/Workspace.swift` (remote daemon probe, retry logic)
- Risk: Remote workspace recovery may fail silently; timeout values may be too short/long for various network conditions
- Priority: Medium

**Notification state transitions and sorting:**
- What's not tested: Unread state changes triggering list reordering; notification sorting stability; concurrent notification updates
- Files: `Sources/TerminalNotificationStore.swift` (notification model), `Sources/NotificationsPage.swift` (UI)
- Risk: Notification list ordering bugs may not be caught until user-visible failures
- Priority: Medium

**UI test flakiness and VM-only execution:**
- What's not tested: Local reproducibility of UI test failures; timing-sensitive assertions in test suite
- Files: `cmuxUITests/*.swift` (all UI test files)
- Risk: Tests may pass locally but fail on CI VM due to timing differences; difficult to debug intermittent failures
- Recommendations: Document which tests are VM-only and require specific hardware; consider adding per-test timeout tuning; improve test diagnostics output

---

*Concerns audit: 2026-03-23*
