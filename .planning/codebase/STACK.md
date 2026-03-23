# Technology Stack

**Analysis Date:** 2026-03-23

## Languages

**Primary:**
- Swift 5.9 - Application layer (UI, terminal management, window control)
- Objective-C - Bridge layer for AppKit integration and OS APIs

**Secondary:**
- Go 1.22 - Remote daemon for SSH relay functionality (`daemon/remote/`)
- TypeScript/JavaScript - Web documentation and tooling

## Runtime

**Environment:**
- macOS 13+ (iOS-compatible framework available in future)

**Package Manager:**
- SPM (Swift Package Manager) - Dependency management for Swift packages
- Bun 1.1+ - Package manager for JavaScript/TypeScript dependencies
- Lockfile: `Package.resolved` (Swift), `bun.lock` (Node/Bun)

## Frameworks

**Core:**
- SwiftUI - Modern declarative UI framework for application interface
- AppKit - Native macOS APIs for window management, notifications, AppleScript
- Combine - Reactive programming for data flow
- Network - Low-level network APIs for socket operations

**Graphics & Rendering:**
- Metal - GPU-accelerated graphics for terminal rendering
- CoreGraphics - Drawing primitives
- QuartzCore - Animation and visual effects
- ImageIO - Image processing and codec support
- IOSurface - Efficient buffer sharing between processes

**Terminal Emulation:**
- GhosttyKit.xcframework - Custom Ghostty terminal emulator (vendored)
  - Built from `ghostty/` submodule (fork: `manaflow-ai/ghostty`)
  - C header: `ghostty.h`
  - XCFRAMEWORK binary (Release optimized build with Zig)

**Browser & Web:**
- WebKit (WKWebView) - Embedded browser for internal web UI and web content rendering
- SwiftTerm 1.5.1 - Terminal emulation library (SPM dependency)

**UI Components:**
- MarkdownUI - Markdown rendering in SwiftUI
- Bonsplit - Tab/pane split management (vendored via `vendor/bonsplit/` submodule)

**Update Management:**
- Sparkle - macOS app auto-update framework (SPM dependency)

**Testing:**
- XCTest - Native Swift testing framework (standard with Xcode)

**Build/Dev:**
- Xcode 15+ - Primary IDE and build system
- Zig 0.15.2 - Build system for GhosttyKit compilation
- Xcode Build System (native to `.xcodeproj`)

## Key Dependencies

**Critical:**
- GhosttyKit.xcframework - Terminal rendering engine (core feature)
  - Resolution: `./scripts/download-prebuilt-ghosttykit.sh`
  - Checksum verification in CI (`.sha256` file validation)
  - Built with Zig from submodule in Release configuration

- Bonsplit - Pane/tab splitting UI logic
  - Location: `vendor/bonsplit/` submodule
  - Source: `github.com/manaflow-ai/bonsplit.git`

**Observability & Analytics:**
- Sentry (via SPM) - Error tracking and crash reporting
  - Configuration: `SentryHelper.swift`
  - DSN: Embedded in `AppDelegate.swift` (hardcoded production DSN)
  - Breadcrumb tracking enabled when telemetry is enabled

- PostHog (via SPM) - Product analytics (daily/hourly active users)
  - Configuration: `PostHogAnalytics.swift`
  - Public API key embedded (non-sensitive): `phc_opOVu7oFzR9wD3I6ZahFGOV2h3mqGpl5EHyQvmHciDP`
  - Host: `https://us.i.posthog.com`
  - Events: `cmux_daily_active`, `cmux_hourly_active`
  - Controlled by `TelemetrySettings.enabledForCurrentLaunch` (user-configurable)

**Telemetry Control:**
- Opt-out available: `sendAnonymousTelemetry` setting (default: enabled)
- Environment overrides: `CMUX_POSTHOG_ENABLE`, `CMUX_POSTHOG_DEBUG` (DEBUG builds only)
- Telemetry state frozen per app launch for consistency

**Infrastructure:**
- Foundation - File system, UserDefaults, Date/Time utilities
- Security - Cryptographic APIs (`CryptoKit`)
- ObjectiveC.runtime - Dynamic method dispatching for window events

**Scripting & Automation:**
- AppleScript support via `cmux.sdef` (scripting dictionary)
- AppleScriptSupport.swift - Handler for AppleScript commands

## Configuration

**Environment:**
- macOS app defaults (UserDefaults) - Settings persistence
- Environment variables for development:
  - `CMUX_DISABLE_SESSION_RESTORE` - Disable session restoration
  - `CMUX_RESTORE_SCROLLBACK_FILE` - Scrollback history file path
  - `CMUX_FEEDBACK_API_URL` - Custom feedback endpoint (default: `https://cmux.com/api/feedback`)
  - `CMUX_SOCKET` - Unix socket path for CLI communication
  - `CMUX_POSTHOG_ENABLE`, `CMUX_POSTHOG_DEBUG` - Analytics override (DEBUG only)

**Build:**
- Xcode project: `GhosttyTabs.xcodeproj`
- Schemes: `cmux`, `cmux-cli`, `cmuxTests`, `cmuxUITests`
- Configuration files:
  - `Info.plist` - App metadata, URL handlers, scripting definition, ATSecurity rules
  - `cmux.entitlements` - App sandbox/capability settings
  - `.xcconfig` files (not found; settings in pbxproj)

**Localization:**
- `Localizable.xcstrings` - Multi-language strings (English, Japanese)
- `InfoPlist.xcstrings` - Info.plist localizations

## Platform Requirements

**Development:**
- macOS 13+ (deployment target)
- Xcode 15+ with Swift 5.9 toolchain
- Zig 0.15.2 (for GhosttyKit compilation if building from source)
- Optional: Go 1.22 (for remote daemon development)

**Production:**
- macOS 13+ (Big Sur)
- Auto-updates via Sparkle: `SUFeedURL = "https://github.com/manaflow-ai/cmux/releases/latest/download/appcast.xml"`
- Code signing required for distribution

**CI/CD:**
- WarpBuild runners: `warp-macos-15-arm64-6x` (GitHub Actions)
- Ubuntu runners for linting and validation
- macOS Monterey 15 ARM64 for main builds

---

*Stack analysis: 2026-03-23*
