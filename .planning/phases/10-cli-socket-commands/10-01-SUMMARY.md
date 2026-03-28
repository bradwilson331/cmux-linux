---
phase: 10-cli-socket-commands
plan: 01
subsystem: cli
tags: [clap, unix-socket, json-rpc, cli, rust]

requires:
  - phase: 03-socket-api-session-persistence
    provides: Unix socket JSON-RPC server and SocketCommand protocol
provides:
  - cmux CLI binary (standalone, no GTK4 dependency)
  - Socket discovery chain (CMUX_SOCKET env, XDG_RUNTIME_DIR, marker file, debug fallback, glob)
  - Synchronous Unix socket JSON-RPC client
  - 34+ clap subcommand definitions
  - Dual binary targets (cmux-app for GUI, cmux for CLI)
affects: [10-02-PLAN, release, ci]

tech-stack:
  added: [clap 4]
  patterns: [path-module inclusion for binary-local modules, synchronous socket client]

key-files:
  created:
    - src/bin/cmux.rs
    - src/cli/mod.rs
    - src/cli/discovery.rs
    - src/cli/socket_client.rs
  modified:
    - Cargo.toml
    - .github/workflows/ci.yml
    - .github/workflows/release.yml
    - scripts/cmux-cli
    - resources/cmux.desktop

key-decisions:
  - "Used #[path] attribute to include src/cli/ modules from src/bin/cmux.rs — avoids GTK4 dependency leakage"
  - "Renamed binary from cmux-linux to cmux-app (package name stays cmux-linux)"
  - "Added env feature to clap for CMUX_SOCKET env var support in --socket flag"
  - "Updated resources/cmux.desktop Exec to cmux-app alongside CI references"

patterns-established:
  - "CLI modules under src/cli/ are GTK4-free and imported via #[path] from src/bin/cmux.rs"
  - "Socket discovery chain mirrors Python client ordering for compatibility"

requirements-completed: [D-01, D-02, D-03, D-04, D-06, D-10, D-11, D-12, D-13, D-14]

duration: 5min
completed: 2026-03-28
---

# Phase 10 Plan 01: CLI Scaffold Summary

**Clap-based cmux CLI with dual binary targets, synchronous socket client, full discovery chain, and 34+ subcommand definitions**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-28T03:50:19Z
- **Completed:** 2026-03-28T03:55:25Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Dual binary targets: cmux (CLI) and cmux-app (GUI) compile from same Cargo crate
- CLI binary has zero GTK4 dependency, builds on headless systems
- Socket discovery chain fully ports Python cmux.py logic to Rust
- All 34+ subcommand variants defined in clap (core commands functional, stubs for Plan 02)
- Exit codes follow D-10: 0 success, 1 command error, 2 connection error
- CI/release workflows updated from cmux-linux to cmux-app

## Task Commits

1. **Task 1: Cargo config, binary rename, and CI updates** - `2362bde2` (feat)
2. **Task 2: CLI modules -- socket client, discovery, clap definition, and core commands** - `6472ce27` (feat)

## Files Created/Modified
- `src/bin/cmux.rs` - CLI binary entry point with exit code handling
- `src/cli/mod.rs` - Clap CLI definition, Commands enum, run() dispatch
- `src/cli/discovery.rs` - Socket path discovery chain (env, XDG, marker, debug, glob)
- `src/cli/socket_client.rs` - Synchronous Unix socket JSON-RPC client with CliError
- `Cargo.toml` - Dual [[bin]] sections, clap dependency with derive/color/env
- `.github/workflows/ci.yml` - cargo test --bin cmux-app (was cmux-linux)
- `.github/workflows/release.yml` - target/release/cmux-app (was cmux-linux)
- `scripts/cmux-cli` - Prefers Rust binary, falls back to Python
- `resources/cmux.desktop` - Exec=cmux-app (was cmux-linux)

## Decisions Made
- Used `#[path = "../cli/mod.rs"]` attribute to include cli modules from binary crate, keeping GTK4 isolation clean
- Renamed binary target from `cmux-linux` to `cmux-app`; package name stays `cmux-linux`
- Added `env` feature to clap for `CMUX_SOCKET` env var support on `--socket` flag
- Updated `resources/cmux.desktop` Exec alongside CI references (found via repo-wide search for cmux-linux)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added clap env feature**
- **Found during:** Task 2 (CLI build)
- **Issue:** `#[arg(env = "CMUX_SOCKET")]` requires clap's `env` feature which wasn't in the plan
- **Fix:** Added `env` to clap features list in Cargo.toml
- **Files modified:** Cargo.toml
- **Verification:** Build succeeds
- **Committed in:** 6472ce27

**2. [Rule 1 - Bug] Updated resources/cmux.desktop**
- **Found during:** Task 1 (repo-wide search for cmux-linux references)
- **Issue:** Desktop file Exec still referenced cmux-linux binary name
- **Fix:** Changed Exec=cmux-linux to Exec=cmux-app
- **Files modified:** resources/cmux.desktop
- **Verification:** File updated correctly
- **Committed in:** 2362bde2

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered
- `serde_json::json` is a macro, cannot be assigned to a variable -- used `use serde_json::json;` inside function scope instead

## Known Stubs
None. All stubs in Commands enum are intentional -- they are fully wired to their JSON-RPC method mappings and will get human-friendly output formatting in Plan 02.

## Next Phase Readiness
- Plan 02 can add human-friendly formatters for each command's output
- All subcommand variants are already defined and mapped to JSON-RPC methods
- Socket client and discovery infrastructure is complete

## Self-Check: PASSED

- All 4 created files exist
- Both commit hashes (2362bde2, 6472ce27) verified in git log

---
*Phase: 10-cli-socket-commands*
*Completed: 2026-03-28*
