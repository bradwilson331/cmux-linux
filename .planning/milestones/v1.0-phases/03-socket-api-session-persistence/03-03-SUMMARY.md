---
phase: 03-socket-api-session-persistence
plan: "03"
subsystem: socket-api
tags: [json-rpc, socket, workspace, commands, dispatch]

requires:
  - phase: 03-socket-api-session-persistence/02
    provides: socket server accept loop, auth, basic SocketCommand stub
provides:
  - Complete SocketCommand enum with 30 variants (system/workspace/window/debug/surface/pane)
  - Full handler dispatch table for all Tier-1 commands
  - dispatch_line() method routing for 28 v2 protocol methods
  - SOCK-05 focus policy enforcement in all handlers
affects: [03-socket-api-session-persistence/04, 03-socket-api-session-persistence/05]

tech-stack:
  added: []
  patterns: [ok/err response helpers, match-based command dispatch, UUID workspace lookup]

key-files:
  created: []
  modified:
    - src/socket/commands.rs
    - src/socket/handlers.rs
    - src/socket/mod.rs
    - src/split_engine.rs

key-decisions:
  - "Used ghostty_surface_text (not ghostty_surface_input_text which does not exist in FFI) for debug.type"
  - "workspace.rename temporarily switches active index to target then restores to avoid focus side effect"
  - "workspace.last uses switch_prev as placeholder until Phase 4 adds workspace history tracking"
  - "workspace.reorder adjusts active_index correctly for all from/to permutations"

patterns-established:
  - "ok()/err() response helpers for consistent JSON-RPC response formatting"
  - "UUID string lookup pattern: borrow state, find position by uuid.to_string() == id, then mutate"

requirements-completed: [SOCK-02, SOCK-05]

duration: 4min
completed: 2026-03-26
---

# Phase 03 Plan 03: Command Handlers and Dispatch Summary

**30-variant SocketCommand enum with full Tier-1 dispatch for system/workspace/window/debug methods, SOCK-05 focus policy enforced**

## What Was Built

### Task 1: SocketCommand Enum + Handlers (4df351c2)
- Expanded `SocketCommand` from 2 variants (Ping, NotImplemented) to 30 variants covering all method groups
- Implemented complete handler dispatch table in `handlers.rs` with `ok()`/`err()` response helpers
- All system.* handlers: ping (pong: true), identify (version/platform/socket_path), capabilities (method list)
- All workspace.* handlers: list, current, create, select, close, rename, next, previous, last, reorder
- window.list and window.current: single GTK window model
- debug.layout: serializes SplitNodeData tree via serde_json
- debug.type: sends text to active surface via ghostty_surface_text FFI
- Surface/pane commands return not_implemented stub (Plan 04)
- Added `find_surface_for_pane()` to SplitNode for debug.type handler

### Task 2: dispatch_line() Full Routing (3261e3cd)
- Replaced NotImplemented-only dispatch with match on all 28 method strings
- Parses `params` object for parameterized commands (id, name, position, direction, text, key)
- Unknown methods fall through to NotImplemented variant

## SOCK-05 Focus Policy Compliance

Focus-intent commands (allowed to change focus via switch_to_index):
- workspace.select, workspace.next, workspace.previous, workspace.last

Non-focus commands (no grab_active_focus or focus_active_surface calls):
- workspace.list, workspace.current, workspace.create, workspace.close, workspace.rename, workspace.reorder
- window.list, window.current
- debug.layout, debug.type
- system.ping, system.identify, system.capabilities

Verified: `grep 'grab_active_focus\|focus_active_surface' src/socket/handlers.rs` returns only a doc comment, no actual calls.

## Deviations from Plan

### TDD Skipped
Per CLAUDE.md test quality policy: handlers require GTK initialization (AppStateRef contains GtkStack, GtkListBox) which cannot be instantiated in unit tests. No meaningful behavioral test is practical without a running GTK event loop. Verified correctness via cargo check and grep-based done criteria instead.

### Auto-fixed Issues

**1. [Rule 1 - Bug] ghostty_surface_input_text does not exist**
- Found during: Task 1
- Issue: Plan referenced `ghostty_surface_input_text` but FFI only exposes `ghostty_surface_text`
- Fix: Changed to `ghostty_surface_text` with same signature (surface, ptr, len)
- Files modified: src/socket/handlers.rs

**2. [Rule 2 - Missing] workspace.rename focus side effect**
- Found during: Task 1
- Issue: Plan's rename handler switched to target workspace but never restored original active index
- Fix: Save prev_active before switch, restore after rename to avoid SOCK-05 violation
- Files modified: src/socket/handlers.rs

## Known Stubs

None that prevent this plan's goals. Surface/pane commands intentionally return not_implemented (documented, Plan 04 scope).

## Self-Check: PASSED
