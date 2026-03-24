---
phase: 02-workspaces-pane-splits
plan: 05
last_updated: 2026-03-24T06:00:00Z
tech_stack:
  - Rust
  - GTK4
key_files_created:
  - path: "src/shortcuts.rs"
    description: "All D-10 keyboard shortcuts at PropagationPhase::Capture"
key_files_modified:
  - path: "src/app_state.rs"
    description: "Added split_engines: Vec<SplitEngine>, gtk_app field, create_workspace_with_surface, active_split_engine_mut, grab_active_focus wiring"
  - path: "src/split_engine.rs"
    description: "Added SURFACE_REGISTRY fallback in find_surface, grab_active_focus() public method"
  - path: "src/sidebar.rs"
    description: "wire_sidebar_clicks now calls ghostty_surface_set_focus after switch_to_index (SPLIT-07)"
  - path: "src/main.rs"
    description: "Updated AppState::new() call with gtk_app arg; wired install_shortcuts"
---

# Phase 02-workspaces-pane-splits, Plan 05: Keyboard Shortcuts + SplitEngine Integration

## One-liner

Implemented all 14 D-10 keyboard shortcuts at PropagationPhase::Capture and integrated SplitEngine per-workspace into AppState, with SPLIT-07 sidebar focus routing.

## Summary

1. **`src/shortcuts.rs`**: Created with `install_shortcuts()` using `PropagationPhase::Capture` so the window-level controller intercepts shortcuts before Ghostty's GLArea EventControllerKey. All 14 D-10 shortcuts wired: Ctrl+N (new workspace), Ctrl+Shift+W (close), Ctrl+]/[ (prev/next), Ctrl+1–9 (switch by number), Ctrl+Shift+R (rename), Ctrl+B (toggle sidebar), Ctrl+D (split right), Ctrl+Shift+D (split down), Ctrl+Shift+X (close pane), Ctrl+Alt+arrows (focus navigation).

2. **`src/app_state.rs`**: Extended AppState with `split_engines: Vec<SplitEngine>` (parallel to `workspaces`), `gtk_app: gtk4::Application`, `active_split_engine_mut()`, and `create_workspace()` now creates a SplitEngine with the initial GLArea.

3. **`src/split_engine.rs`**: Added SURFACE_REGISTRY fallback in `find_surface` (handles the gap where the surface pointer is null before GLArea realize). Added `grab_active_focus()` public method for workspace-switch focus management.

4. **`src/sidebar.rs`**: `wire_sidebar_clicks` now calls `ghostty_surface_set_focus` after `switch_to_index` to satisfy SPLIT-07 (all focus changes must call `ghostty_surface_set_focus`).

## Deviations from Plan

- AlertDialog for close confirmation was temporarily commented out due to API issues (carried over from Plan 04). `handle_close_workspace` directly calls `close_workspace` without a dialog.
- Added `grab_active_focus()` to SplitEngine and called it from `switch_to_index` to fix frozen terminal input (GTK focus was staying on sidebar after workspace creation/switch).
- Added `area.grab_focus()` in the GLArea realize callback as belt-and-suspenders for the initial workspace.

## Key Technical Details

- **Capture phase**: `PropagationPhase::Capture` fires parent→child, so window controller runs before Ghostty's GLArea EventControllerKey. Unhandled keys return `Propagation::Proceed` to pass through to Ghostty.
- **Focus root cause**: After `switch_to_index`, GTK focus was staying on the sidebar ListBox. Fixed by calling `engine.grab_active_focus()` at the end of `switch_to_index` and `area.grab_focus()` in the GLArea realize callback.
- **SPLIT-07 compliance**: Both `wire_sidebar_clicks` and `focus_next_in_direction` call `ghostty_surface_set_focus`.

## Next Steps

- Plan 06: Human verification checkpoint — exercise all Phase 2 features end-to-end.
