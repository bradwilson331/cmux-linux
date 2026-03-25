---
status: diagnosed
phase: 02-workspaces-pane-splits
source: [02-07-SUMMARY.md, 02-01-SUMMARY.md through 02-05-SUMMARY.md]
started: 2026-03-24T01:00:00Z
updated: 2026-03-24T02:00:00Z
note: Re-test session after 02-07 gap fixes. Testing fixed gaps + previously skipped tests.
---

## Current Test

number: [testing complete]
awaiting: n/a

## Tests

### 7. Drag-to-resize (re-test after fix)
expected: Drag the vertical divider between panes left and right. Panes resize fluidly. After releasing the drag, cursor continues blinking in the active pane and the blue border shows at the correct (new) pane dimensions.
result: issue
reported: "Is not fixed and drag to resize doesn't work on split or normal panes"
severity: major
log_analyzed: target/cmux-fix14.log

### 8. Pane focus navigation (Ctrl+Shift+Arrow — was Ctrl+Alt+Arrow)
expected: With two side-by-side panes open, press Ctrl+Shift+Left — focus moves to the left pane (blue border moves there). Type in the left pane — keystrokes appear there. Press Ctrl+Shift+Right — focus returns to the right pane.
result: issue
reported: "Resizing the terminal caused the cursor to freeze and no input was visible"
severity: major

### 9. Close active pane (no crash — re-test after fix)
expected: With 3 panes open, press Ctrl+Shift+X. Active pane closes, surviving sibling expands to fill space. Focus moves to the surviving pane (has blue border, receives keyboard input). App does NOT crash.
result: issue
reported: "the active pane closes but the cursor disappears and key inputs are not seen"
severity: major
note: No crash (Gap 2 fix working). But focus/render broken after close.

### 10. Sidebar toggle (Ctrl+B)
expected: Press Ctrl+B — sidebar hides, terminal expands full width. Press Ctrl+B again — sidebar reappears.
result: issue
reported: "sidebar hides but terminal does not expand — black/blank area remains where sidebar was. Cursor disappears, input not visible. Ctrl+B again restores sidebar and terminal content reappears."
severity: blocker
note: Two distinct bugs: (1) layout — terminal doesn't expand to fill sidebar space; (2) focus/render — same cursor freeze as tests 7-9.

### 11. Close workspace with confirmation (Ctrl+Shift+W)
expected: With 2+ workspaces, press Ctrl+Shift+W. Dialog appears: title "Close Workspace?", message "All panes in this workspace will be closed. This cannot be undone." Click "Keep Workspace" — dialog closes, workspace stays. Press Ctrl+Shift+W again, click "Close Workspace" — workspace closes, other workspace becomes active.
result: skipped
reason: Blocked by focus regression (tests 7-10) — cannot meaningfully test workspace close when terminal is already broken after any resize/toggle

### 12. Memory sanity (no crash)
expected: Create 5 workspaces, close 4 of them. Split the remaining workspace 3 times, close all split panes. App does not crash or hang.
result: skipped
reason: Blocked by focus regression (tests 7-10) — terminal becomes unusable after first split/close/resize

## Summary

total: 6
passed: 0
issues: 4
pending: 0
skipped: 2

## Gaps

- truth: "After drag-resizing a pane, panes resize and cursor continues blinking; active-pane blue border repaints at new dimensions."
  status: failed
  reason: "User reported: Is not fixed and drag to resize doesn't work on split or normal panes. Log (cmux-fix14.log) confirms mechanical drag works (sizes change) but focus/border recovery fails."
  severity: major
  test: 7
  root_cause: |
    Bug A: restore_active_pane_focus() calls ghostty_surface_set_focus(false) then set_focus(true) on ALL surfaces in GL_AREA_REGISTRY, including the non-active pane. The non-active surface (had focused=false) gets set_focus(true) — giving Ghostty focus to the inactive pane. After recovery, both surfaces have focused=true.
    Bug B: No EventControllerFocus is connected on GLArea — GTK focus-in/out events during the drag (separator stealing focus) never update Ghostty's focused state. During drag, Ghostty thinks the pane is still focused (no automatic false call). restore_active_pane_focus() then calls set_focus(false) to force a toggle — but this STOPS cursor blink. The subsequent set_focus(true) should restart it, but if the async focus message is processed after the render tick, the cursor appears frozen.
    Fix needed: (1) Only call set_focus(true) on the ACTIVE surface; call set_focus(false) on all others. (2) Add EventControllerFocus to each GLArea to auto-sync GTK focus events → ghostty_surface_set_focus.
  artifacts:
    - src/split_engine.rs:804-844 (restore_active_pane_focus loop 1 — set_focus on all surfaces)
    - src/ghostty/surface.rs:52-55 (GLArea setup — missing EventControllerFocus)
  missing:
    - EventControllerFocus on each GLArea to call ghostty_surface_set_focus on focus-in/out
    - restore_active_pane_focus must only call set_focus(true) on the active surface

- truth: "After any widget tree operation (split, pane close, sidebar toggle, divider drag), the active terminal remains fully interactive: cursor blinks, keyboard input is visible."
  status: failed
  reason: "All four tested operations (tests 7-10) leave the terminal with a frozen cursor and invisible input. This is a single unified focus/render regression."
  severity: blocker
  tests: [7, 8, 9, 10]
  root_cause: |
    UNIFIED ROOT CAUSE — three compounding issues:

    1. MISSING EventControllerFocus (primary): No GTK focus controller is wired to ghostty_surface_set_focus(). When GTK focus moves (separator drag, reparent, sidebar hide/show), Ghostty never receives focus-out. Ghostty's surface keeps focused=true internally even when GTK routes keyboard events elsewhere.

    2. restore_active_pane_focus() corrupts focus state (secondary): Called after every GtkPaned drag, it iterates ALL surfaces and calls set_focus(false) then set_focus(true) on each. This leaves ALL surfaces with focused=true in Ghostty — not just the active one. Subsequent legitimate set_focus(true) calls (from re-realize, close_active, etc.) hit Ghostty's early-return guard (if self.focused == focused { return; }) and are no-ops. The cursor blink timer is never explicitly restarted.

    3. re-realize sets size to 0x0 (tertiary): On any reparent (split, close, sidebar hide), the GLArea unrealizes and re-realizes. The realize handler calls ghostty_surface_set_size(0, 0) because area.width()==0 at realize time. This can trigger Ghostty's anti-flicker guard (GL_VIEWPORT != cached renderer size) causing the renderer to re-present the last stale frame indefinitely until a recovery tick fires.

    SIDEBAR-SPECIFIC additional bug: When Ctrl+B hides the sidebar, the terminal widget does not expand to fill the freed space. The layout container (GtkBox or GtkPaned) does not redistribute space. This is a separate layout bug — the terminal's hexpand or widget sizing constraint is not correctly set relative to the sidebar container.
  artifacts:
    - src/ghostty/surface.rs:52-55 (GLArea setup — set_focusable but no EventControllerFocus)
    - src/ghostty/surface.rs:85-124 (re-realize handler — calls set_size(0,0) at realize time)
    - src/split_engine.rs:804-844 (restore_active_pane_focus — set_focus(true) on ALL surfaces)
    - src/split_engine.rs:480-490 (close_active — set_focus(true) is no-op due to Bug 2)
    - src/main.rs or wherever sidebar toggle is implemented (terminal container not expanding)
  missing:
    - EventControllerFocus on each GLArea: enter → ghostty_surface_set_focus(true); leave → ghostty_surface_set_focus(false)
    - restore_active_pane_focus: set_focus(true) only on active surface; set_focus(false) on all others
    - After pane close: schedule same delayed recovery (app_tick + queue_render) as post-drag
    - Sidebar toggle: ensure terminal container has hexpand=true and layout redistributes on sidebar hide/show
