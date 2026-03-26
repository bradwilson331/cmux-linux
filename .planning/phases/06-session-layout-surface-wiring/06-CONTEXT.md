# Phase 6: Session Layout Restore + Surface Wiring - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Split pane layouts persist and restore on relaunch. Session save captures full split tree topology (not just workspace names) to session.json. On restore, the app rebuilds the exact GtkPaned + GLArea widget tree with live Ghostty surfaces wired to every leaf pane, so socket commands like `surface.send_text` and `debug.type` work against restored panes. Gap closure phase — closes SESS-02, SOCK-02, SOCK-03 gaps from v1.0 milestone audit.

</domain>

<decisions>
## Implementation Decisions

### Tree Serialization & Schema
- **D-01:** Bump session version from 1 to 2. Version=2 includes full split tree topology with divider ratios. Version=1 files auto-upgrade: restore workspace names with single default pane per workspace, next save writes version=2.
- **D-02:** Save full split tree for ALL workspaces (not just active). Every workspace's `SplitNodeData` tree is serialized via `split_engines[i].root.to_data()` in `trigger_session_save()`.
- **D-03:** Store divider positions as ratios (0.0–1.0 relative to parent size), not absolute pixels. Ratios work correctly when window is resized between sessions.
- **D-04:** Save `active_pane_uuid` per workspace in `WorkspaceSession` (field already exists as TODO). Restored workspace focuses the same pane the user was last working in.

### Tree Restoration
- **D-05:** Add `SplitNode::from_data()` as the inverse of `to_data()`. Recursively builds the GtkPaned + GLArea widget tree from serialized `SplitNodeData`. This is the primary restoration method.
- **D-06:** Fresh pane IDs on restore (generated from `NEXT_PANE_ID` counter), but preserve UUIDs from session.json. Socket commands reference UUIDs — these must be stable across restarts.
- **D-07:** After restoring tree structure, set active pane focus using saved `active_pane_uuid`.

### Surface Lifecycle on Restore
- **D-08:** Restored panes follow the same surface creation lifecycle as new splits: GLArea's realize callback creates the Ghostty surface and calls `set_initial_surface()`. No batch creation — surfaces wire up naturally as GL contexts become ready.
- **D-09:** Pass saved CWD to new Ghostty surface config on restore. Terminal opens in the directory the user was last working in. Use `ghostty_surface_inherited_config()` or equivalent API to set CWD before surface creation.
- **D-10:** After each restored leaf's GLArea realize, wire surface into `SURFACE_REGISTRY` and `GL_TO_SURFACE` maps — same registration path as existing splits.

### CWD/Shell Capture
- **D-11:** Capture CWD via `readlink("/proc/{pid}/cwd")` where pid is the shell process PID from the Ghostty surface. Store in `SplitNodeData::Leaf.cwd` during `to_data()`.
- **D-12:** Always launch the user's default shell (`$SHELL` or `/bin/sh`) on restore. Do not capture or replay running commands — only CWD matters.

### Failure Handling
- **D-13:** Basic session validation only: check JSON parses, version field present, at least one workspace. If invalid, log warning and fall back to default single workspace (existing SESS-04 behavior).
- **D-14:** Tree depth cap at 16 levels. Reject trees deeper than 16 and fall back to single pane for that workspace.
- **D-15:** If a GLArea fails to realize during restore, skip the failed pane and collapse its parent (same as close-pane behavior). Log warning. Rest of layout stays intact.

### Claude's Discretion
- How to obtain the shell PID from Ghostty surface for `/proc` readlink (may need Ghostty API or track from surface creation)
- Exact `SplitNodeData` schema changes to include `ratio: f64` field
- `from_data()` method signature and error handling internals
- Whether to add `active_pane_uuid` to session save trigger (pane focus change events)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Session Persistence — SESS-02 (layout fully restored on next launch)
- `.planning/REQUIREMENTS.md` §Socket API — SOCK-02 (v2 JSON-RPC wire-compatible), SOCK-03 (CLI controls Linux app)

### Roadmap
- `.planning/ROADMAP.md` §Phase 6 — success criteria, phase goal, requirement IDs, gap closure context

### Session Persistence Implementation
- `src/session.rs` — `SessionData`, `WorkspaceSession`, `SplitNodeData` structs; `save_session_atomic()` and `load_session()` functions
- `src/app_state.rs:432-460` — `trigger_session_save()` snapshots session data; currently uses dummy single-leaf layout (gap zone)

### Split Tree & Surface Wiring
- `src/split_engine.rs:14-37` — `SplitNode` enum (runtime: GLArea + surface pointer per leaf)
- `src/split_engine.rs:1065-1105` — `SplitNodeData` enum (serializable) + `to_data()` method
- `src/split_engine.rs:254-265` — `set_initial_surface()` wiring method

### Surface Creation & Registration
- `src/ghostty/surface.rs:21-232` — GLArea lifecycle, `create_surface()`, registry insertion
- `src/ghostty/callbacks.rs` — `SURFACE_REGISTRY`, `GL_AREA_REGISTRY`, `GL_TO_SURFACE` global maps

### Session Restore (current gap)
- `src/main.rs:235-254` — Current restore logic: workspace names only, splits not restored

### Socket Command Routing (verification targets)
- `src/socket/handlers.rs:259-279` — `debug.type` handler (uses `find_surface_for_pane`)
- `src/socket/handlers.rs:383-409` — `surface.send_text` handler (UUID lookup)

### Prior Phase Context
- `.planning/phases/03-socket-api-session-persistence/03-CONTEXT.md` — D-12 through D-16 (session save/restore decisions from Phase 3)
- `.planning/phases/02-workspaces-pane-splits/02-CONTEXT.md` — D-05 through D-09 (split tree widget structure decisions)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `SplitNodeData` enum and `to_data()` method already exist — Phase 6 adds the inverse `from_data()`
- `set_initial_surface()` already wires a surface to a SplitNode leaf — Phase 6 calls it for every restored leaf
- `SURFACE_REGISTRY` + `GL_TO_SURFACE` maps already handle surface lookup — no new registry needed
- `save_session_atomic()` and `load_session()` already handle file I/O with atomic rename — Phase 6 changes the data, not the mechanism

### Established Patterns
- GLArea realize callback → `create_surface()` → registry insertion: standard lifecycle for surface creation
- `trigger_session_save()` snapshots on GTK main thread, sends via mpsc to tokio debounce: established pattern
- `split_engines[ws_idx]` maps workspace index to SplitEngine: standard access pattern

### Integration Points
- `app_state.rs:447-452`: Replace dummy leaf with `split_engines[i].root.to_data()` call
- `main.rs:235-254`: Replace name-only restore with full tree rebuild using `from_data()`
- `split_engine.rs`: Add `from_data()` constructor, add ratio field to SplitNodeData
- `session.rs`: Update version to 2, add ratio field to SplitNodeData::Split

</code_context>

<specifics>
## Specific Ideas

- The `to_data()` → `from_data()` pair should be a clean inverse: serialization round-trips perfectly
- Surface wiring must complete before socket commands work — ensure `set_initial_surface()` runs during GLArea realize for restored panes, not deferred
- `/proc/{pid}/cwd` readlink may fail for sandboxed or remote processes — fall back to `$HOME` gracefully
- `active_pane_uuid` field in `WorkspaceSession` is already defined but set to `None` — Phase 6 fills it

</specifics>

<deferred>
## Deferred Ideas

- Shell command replay (capturing and relaunching vim, htop, etc.) — too complex and potentially dangerous
- OSC 7 CWD tracking as alternative to /proc — could be added later as enhancement
- Multi-window session restore — current scope is single-window only
- Divider position animation on restore — unnecessary complexity

</deferred>

---

*Phase: 06-session-layout-surface-wiring*
*Context gathered: 2026-03-26 via discuss-phase*
