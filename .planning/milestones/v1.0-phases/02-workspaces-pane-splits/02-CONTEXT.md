# Phase 2: Workspaces + Pane Splits - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Transform the single-surface Phase 1 window into a full multiplexer UI. Phase 2 delivers workspace (tab) management and recursive pane splitting with drag-to-resize dividers. No socket API, no session persistence (Phase 3), no config file (Phase 5) — just the core multiplexer interaction layer.

Starting point: `main.rs` has bare `ApplicationWindow` → single `GLArea`. Phase 2 adds a sidebar workspace list and a recursive GtkPaned split engine on top.

</domain>

<decisions>
## Implementation Decisions

### Workspace Chrome
- **D-01:** Sidebar layout — vertical workspace list on the left side of the window, matching macOS cmux exactly. GTK4 implementation: `GtkPaned` (horizontal) with a `GtkListBox`/`GtkScrolledWindow` sidebar on the left and the terminal area on the right.
- **D-02:** Workspace names always visible in the sidebar — no icon-only collapse mode in Phase 2. Simple text list.
- **D-03:** Rename via keyboard shortcut only (Ctrl+Shift+R to match macOS Cmd+Shift+R default). No inline click-to-rename in Phase 2.
- **D-04:** Sidebar is toggleable (show/hide) via keyboard shortcut, matching macOS Cmd+B → Ctrl+B on Linux.

### Split Divider Widget
- **D-05:** Use nested `GtkPaned` widgets to represent the split tree. Each split node = one `GtkPaned` (horizontal or vertical); leaf nodes = `GtkGLArea` (Ghostty surface). Drag-to-resize handled natively by GTK4's GtkPaned separator — no custom hit-testing needed.
- **D-06:** Tree structure: `Root GtkPaned (horizontal) → {GtkGLArea | GtkPaned (vertical) → {GtkGLArea, GtkGLArea}}`. Recursive nesting follows the Bonsplit tree model (SPLIT-06 requirement).
- **D-07:** When a pane is closed, the sibling expands to fill all available space (GtkPaned's natural behavior when one child is removed and replaced by the surviving child). No special "collapse" logic needed.
- **D-08:** New terminal when splitting inherits the CWD of the active pane via `ghostty_surface_inherited_config()` — matches macOS cmux behavior.
- **D-09:** Initial split ratio is always 50/50 (GtkPaned default position = half available space).

### Keyboard Shortcuts (Linux defaults)
- **D-10:** Linux shortcuts map macOS Cmd → Ctrl for all cmux actions. Rationale: cross-platform consistency — the same muscle memory works on both platforms when remapped at the config level. Defaults for Phase 2:
  - Split right: `Ctrl+D`
  - Split down: `Ctrl+Shift+D`
  - New workspace: `Ctrl+N`
  - Close workspace: `Ctrl+Shift+W`
  - Next workspace: `Ctrl+]` (matching macOS Ctrl+Cmd+])
  - Prev workspace: `Ctrl+[`
  - Focus pane left/right/up/down: `Ctrl+Alt+arrows`
  - Toggle sidebar: `Ctrl+B`
  - Rename workspace: `Ctrl+Shift+R`
  - Workspace by number: `Ctrl+1` through `Ctrl+9`

### Claude's Discretion
- Exact GTK4 widget hierarchy and Rust struct layout for workspace model (`Vec<Workspace>` with active index, or an observable model)
- How `ghostty_surface_set_focus` is called on GtkPaned size-allocate vs. explicit user focus navigation
- Whether GtkPaned position needs to be serialized per-split for session restore in Phase 3 (researcher should verify Bonsplit tree snapshot API)
- GObject signal connections for GtkPaned's `notify::position` if divider positions need to be tracked

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 1 Foundation
- `src/main.rs` — Current entry point: bare `ApplicationWindow` + single `create_surface()` call. Phase 2 restructures `build_ui()`.
- `src/ghostty/surface.rs` — `create_surface()` returns a `GtkGLArea`. Phase 2 calls this per-pane. Understand the realize/render/resize lifecycle before adding multiple surfaces.
- `.planning/phases/01-ghostty-foundation/01-CONTEXT.md` §Decisions — D-02 (bare window scope for Phase 1), D-09 (close_surface_cb uses process::exit — Phase 2 must override this with per-pane close logic).

### Requirements
- `.planning/REQUIREMENTS.md` §Workspace Management — WS-01 through WS-06
- `.planning/REQUIREMENTS.md` §Pane Splitting — SPLIT-01 through SPLIT-07. Especially SPLIT-06 (immutable tree = Bonsplit Rust port) and SPLIT-07 (focus routing via ghostty_surface_set_focus).

### macOS Reference Implementation
- `Sources/Workspace.swift` — Full workspace model with bonsplitController, tab management, pane focus, session serialization. The Rust port mirrors this data model.
- `Sources/KeyboardShortcutSettings.swift` §defaultShortcut — All macOS default shortcuts. Linux defaults = Ctrl for Cmd, Ctrl+Alt for Cmd+Opt.

### Project Constraints
- `.planning/PROJECT.md` §Constraints — GTK4 mandatory, tokio arrives in Phase 3 (Phase 2 is GLib-only async).
- `.planning/STATE.md` §Decisions — `set_focusable(true)` on GLArea required; `ApplicationFlags::NON_UNIQUE` required for DBus environments.

</canonical_refs>

<code_context>
## Existing Code Insights

### Integration Points
- `build_ui()` in `src/main.rs`: Currently sets `window.set_child(Some(&gl_area))`. Phase 2 replaces this with a `GtkBox` or top-level `GtkPaned` that holds sidebar + terminal area.
- `ghostty::surface::create_surface()`: Returns `GtkGLArea` — Phase 2 calls this once per pane creation, not just once at startup.
- `GL_AREA_FOR_RENDER` thread-local in `surface.rs`: Currently stores a single `Option<GLArea>`. Phase 2 needs per-surface wakeup routing — this global needs to become per-surface or use the surface pointer to look up the correct GLArea.
- `SURFACE_PTR` in `callbacks.rs`: Currently a single `AtomicUsize`. Phase 2 needs per-pane surface pointers — likely a `Mutex<HashMap<pane_id, ghostty_surface_t>>` or per-GLArea user-data.

### New Modules Phase 2 Will Create
- `src/workspace.rs` — Workspace model: `Vec<Workspace>`, active index, create/close/rename/switch
- `src/split_engine.rs` — Bonsplit tree: `SplitNode` enum (Leaf(GLArea) | Split { orientation, left, right, paned }), split/close/focus operations
- `src/sidebar.rs` — GTK4 sidebar widget: `GtkListBox` of workspace names, click-to-switch, keyboard navigation
- `src/shortcuts.rs` — Global `EventControllerKey` on the window; intercept cmux shortcuts before Ghostty sees them

</code_context>

<discussion_log_ref>
Discussion log: `.planning/phases/02-workspaces-pane-splits/02-DISCUSSION-LOG.md`
</discussion_log_ref>
