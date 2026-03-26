---
phase: 02-workspaces-pane-splits
plan: 06
status: complete
started: 2026-03-26
completed: 2026-03-26
---

## Summary

Human verification checkpoint for the Phase 2 multiplexer UI. All 12 verification steps exercised.

## Results

| Step | Feature | Result |
|------|---------|--------|
| 1 | Initial state (sidebar, terminal, blue border) | PASS |
| 2 | Workspace create (Ctrl+N) | PASS |
| 3 | Workspace switch (click, Ctrl+[/], Ctrl+1-9) | PASS |
| 4 | Workspace rename (Ctrl+Shift+R) | PASS |
| 5 | Split right (Ctrl+D) | PASS |
| 6 | Split down (Ctrl+Shift+D) | PASS |
| 7 | Drag-to-resize dividers | PASS (known: cursor freeze with 3+ panes) |
| 8 | Focus navigation (Ctrl+Shift+arrows) | PASS |
| 9 | Close pane (Ctrl+Shift+X) | PASS |
| 10 | Sidebar toggle (Ctrl+B) | PASS |
| 11 | Close workspace dialog (Ctrl+Shift+W) | PASS |
| 12 | Memory sanity (create/close stress) | PASS (known: splits not persisted in session) |

## Fixes Applied During Verification

1. **GTK init panic**: Moved `ShortcutMap::from_config` inside `connect_activate` where GTK is initialized (was calling `accelerator_parse` before GTK init)
2. **Ctrl+Shift shortcuts broken**: Added `key.to_lower()` normalization in `ShortcutMap::lookup` — GTK4 gives uppercase keyvals when Shift is held but `accelerator_parse` stores lowercase
3. **Black pane after split/close**: Added `ghostty_surface_display_realized`/`display_unrealized` C API to Ghostty fork — properly reinitializes GL resources (shaders, swap chain) when GLArea is reparented
4. **Close workspace dialog missing**: Uncommented AlertDialog code in `handle_close_workspace`

## Known Limitations

- **3-pane drag cursor freeze**: Post-drag focus restore doesn't fully recover cursor blink timer with 3+ panes. Keyboard input still works; clicking a pane recovers.
- **Session split persistence**: Splits are not saved/restored across sessions. Workspace 2+ restores but without split layout.

## Self-Check: PASSED
