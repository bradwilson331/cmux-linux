---
phase: 03-socket-api-session-persistence
plan: "01"
subsystem: data-model-socket-bridge
tags: [uuid, serde, split-engine, workspace, glib, tokio, socket]

requires:
  - phase: 03-00
    provides: socket module stubs (commands.rs, mod.rs, auth.rs), session.rs, Phase 3 Cargo deps

provides:
  - Workspace struct with stable pub uuid: Uuid field (v4, non-nil, unique)
  - SplitNode::Leaf with uuid: Uuid field at every construction site
  - SplitNodeData serde-friendly mirror enum for session persistence (Serialize/Deserialize)
  - SplitNode::to_data() method returning serializable SplitNodeData snapshot
  - Event-driven tokio→GTK bridge replacing 100ms mpsc polling timer
  - src/socket/handlers.rs stub dispatch (handle_socket_command)
  - build_ui signature extended with runtime_handle, cmd_tx, cmd_rx

affects: [03-02, 03-03, 03-04, 03-05, 03-06]

tech-stack:
  added: [uuid serde feature (v4+serde)]
  patterns:
    - SplitNodeData parallel serde type mirrors SplitNode without GTK widget refs
    - tokio::sync::mpsc::unbounded_channel + glib::MainContext::default().spawn_local replaces glib::MainContext::channel (removed in glib 0.18+)
    - Mutex<Option<T>> pattern for moving non-Clone receiver into Fn connect_activate closure

key-files:
  created:
    - src/socket/handlers.rs
  modified:
    - src/workspace.rs
    - src/split_engine.rs
    - src/main.rs
    - src/socket/mod.rs
    - Cargo.toml

key-decisions:
  - "glib::MainContext::channel does not exist in glib 0.21.5 (removed in glib 0.18+); replaced with tokio::sync::mpsc::unbounded_channel + glib::MainContext::default().spawn_local()"
  - "Mutex<Option<UnboundedReceiver<...>>> wraps cmd_rx to allow move into Fn connect_activate closure"
  - "SplitNodeData uses serde tag=type for JSON discriminant matching session restore"
  - "uuid serde feature added to Cargo.toml alongside v4 for Serialize/Deserialize on Uuid fields"

patterns-established:
  - "SplitNode::to_data() produces a serializable mirror — GTK refs stay in SplitNode, data in SplitNodeData"
  - "All SplitNode::Leaf construction sites get uuid: Uuid::new_v4() — never reuse or copy uuid across panes"

requirements-completed: [SOCK-02, SESS-01, SESS-02]

duration: 18min
completed: 2026-03-26
---

# Phase 03 Plan 01: Data Model UUIDs + Socket Bridge Summary

**UUID-bearing Workspace and SplitNode::Leaf with SplitNodeData serde mirror; 100ms mpsc polling bridge replaced by event-driven tokio/glib spawn_local dispatch**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-03-26T00:00:00Z
- **Completed:** 2026-03-26T00:18:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `pub uuid: Uuid` to `Workspace` struct — generated via `Uuid::new_v4()` in `Workspace::new()`, unique per instance, non-nil
- Added `uuid: Uuid` field to `SplitNode::Leaf` at all 3 construction sites in split_engine.rs
- Added `SplitNodeData` serde-friendly mirror enum (`Leaf { pane_id, surface_uuid, shell, cwd }` / `Split { orientation, start, end }`) with `#[serde(tag = "type")]` for JSON roundtrip
- Added `SplitNode::to_data()` method returning a serializable snapshot for session persistence (Plan 05)
- Replaced 100ms polling mpsc bridge with `tokio::sync::mpsc::unbounded_channel` + `glib::MainContext::default().spawn_local()` — zero-latency, event-driven, no timer
- Created `src/socket/handlers.rs` stub with `handle_socket_command` dispatch (expands in Plan 03)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add UUID fields to Workspace and SplitNode** - `90c4c026` (feat)
2. **Task 2: Replace mpsc polling bridge with glib main loop wiring** - `596dd4aa` (feat)

## Files Created/Modified

- `src/workspace.rs` - Added `pub uuid: Uuid` field + `Uuid::new_v4()` in `new()`; added unit tests for non-nil and uniqueness
- `src/split_engine.rs` - Added `uuid: Uuid` to `SplitNode::Leaf`; updated 3 construction sites; added `SplitNodeData` enum and `SplitNode::to_data()`; added 3 unit tests for serde roundtrip
- `src/main.rs` - Removed mpsc/Arc/thread imports; replaced polling loop with unbounded channel + spawn_local bridge; updated `build_ui` signature
- `src/socket/mod.rs` - Added `pub mod handlers;`
- `src/socket/handlers.rs` - NEW: stub `handle_socket_command` dispatch for Ping and NotImplemented
- `Cargo.toml` - Added `serde` feature to `uuid` dependency

## Decisions Made

- **glib::MainContext::channel unavailable in glib 0.21**: The plan specified this API but it was removed in glib 0.18+. Used `tokio::sync::mpsc::unbounded_channel` + `glib::MainContext::default().spawn_local()` which provides identical semantics: Sender is `Send+Clone`, Receiver is consumed by a non-Send future on the GTK main thread.
- **Mutex<Option<...>> for connect_activate closure**: `UnboundedReceiver` is not Clone, but `connect_activate` requires `Fn`. Wrapped in `Mutex<Option<...>>` so it can be taken once from the closure.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] glib::MainContext::channel API does not exist in glib 0.21.5**
- **Found during:** Task 2 (Replace mpsc polling bridge)
- **Issue:** `glib::MainContext::channel::<T>()` was removed in glib 0.18+. The plan's `must_haves` and RESEARCH.md both specified this API but it compiles with 3 errors (cannot find type `Sender` in crate `glib`, `Receiver`, `channel` on `MainContext`).
- **Fix:** Used `tokio::sync::mpsc::unbounded_channel::<SocketCommand>()` + `glib::MainContext::default().spawn_local(async move { while let Some(cmd) = cmd_rx.recv().await { ... } })` — identical semantics, compiles cleanly.
- **Files modified:** `src/main.rs`
- **Verification:** `cargo check` exits 0; `grep timeout_add_local src/main.rs` returns empty; `grep mpsc::channel src/main.rs` returns empty; event-driven bridge attached in `build_ui`.
- **Committed in:** `596dd4aa` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - API incompatibility)
**Impact on plan:** The fix achieves the same architectural goal (event-driven tokio→GTK bridge) using a different mechanism. All downstream plans (02-05) that use `cmd_tx: tokio::sync::mpsc::UnboundedSender<SocketCommand>` remain unaffected — they just call `.send()` on the sender, which works identically.

## Issues Encountered

- Worktree branch was at wrong commit (macOS cmux repo history). Reset to `321aad5d` (Phase 03 main branch HEAD) before beginning. No code was lost.

## Known Stubs

| File | Function | Stub Type | Plan that implements |
|------|----------|-----------|---------------------|
| src/socket/handlers.rs | handle_socket_command | partial (Ping + NotImplemented only) | Plan 03 |
| src/socket/mod.rs | start_socket_server | todo!() | Plan 02 |
| src/split_engine.rs | SplitNode::to_data shell/cwd | empty strings | Plan 05 |

## Next Phase Readiness

- Plan 02 can now implement `start_socket_server` using the `runtime_handle` and `cmd_tx: UnboundedSender<SocketCommand>` already passed through `build_ui`
- Plan 03 can expand `handle_socket_command` in handlers.rs with full Tier 1 dispatch
- Plan 05 can implement session persistence using `SplitNode::to_data()` and `Workspace.uuid`
- `cargo check` exits 0 with zero errors (16 pre-existing warnings, none new)

## Self-Check: PASSED
