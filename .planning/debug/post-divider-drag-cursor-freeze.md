---
status: awaiting_human_verify
trigger: "Cursor in split pane freezes permanently after dragging divider to resize"
created: 2026-03-24T12:00:00Z
updated: 2026-03-25T11:00:00Z
---

## Current Focus
<!-- OVERWRITE on each update - reflects NOW -->

hypothesis: The resize path (connect_resize in surface.rs) has no post-settle focus bounce. After resize settles, Ghostty's cursor blink timer is stopped and nothing restarts it. The drag-end path (restore_active_pane_focus) fires for split-pane divider drags but (a) doesn't cover single-pane window resize at all, and (b) for split-pane, the false→true bounce was applied but the timing or scope may have been wrong. The correct fix is a debounced focus bounce directly in connect_resize — it fires for ALL resize scenarios (single-pane window resize and split-pane divider drag), uses the directly-captured surface_cell and gl_area, and uses area.has_focus() to identify the pane that should receive the bounce.
test: Add debounced 150ms timeout in connect_resize that calls set_focus(false)+set_focus(true)+ghostty_surface_refresh on the surface if the GLArea currently has GTK focus
expecting: After resize stops, the bounce fires on the focused pane, restarting Ghostty's cursor blink timer, fixing both single-pane resize freeze AND split-pane divider drag freeze
next_action: Implement fix in surface.rs connect_resize

## Symptoms
<!-- Written during gathering, then IMMUTABLE -->

expected: Cursor should move/render in pane, respond to input, and blink normally in split panes
actual: Cursor freezes after dragging the divider to resize panes
errors: No errors visible — app continues running without crash or warnings
reproduction: Create a split pane (Ctrl+D or Ctrl+Shift+D), drag the divider to resize, cursor in the pane freezes permanently
started: After Plan 07 changes (post-drag cursor freeze fix, pane-close crash fix, Ctrl+Alt+Arrow interception fix)
recovery: Stays frozen permanently until app restart

## Eliminated
<!-- APPEND only - prevents re-investigating -->

- hypothesis: Missing ghostty_surface_set_focus(true) in notify::position handler
  evidence: Fix applied (added GL_TO_SURFACE mapping and set_focus call) but user reports cursor AND blue background still freeze, input still broken. This suggests the issue is more fundamental than Ghostty focus state.
  timestamp: 2026-03-24T12:25:00Z

- hypothesis: grab_focus() called during drag interferes with GtkPaned gesture
  evidence: Debounced approach (idle-based, single call per burst) also did not fix the issue. User confirms cursor still freezes.
  timestamp: 2026-03-24T13:00:00Z

- hypothesis: Focus + render trigger is the problem
  evidence: Fix #3 (grab_focus + ghostty_surface_set_focus + queue_render) also did not work. User confirms cursor still freezes.
  timestamp: 2026-03-24T13:20:00Z

- hypothesis: set_focus(true) not called after re-realize — need to add it to re-realize path
  evidence: Adding set_focus(true) in re-realize was explicitly skipped (commented out). Logs confirm the skip: "re-realize — skipping set_focus(true) for surface 0x5fd7c9c01dc0 (EventControllerFocus handles restoration)". The previous session's fix was for the re-realize path but the real issue is in restore_active_pane_focus.
  timestamp: 2026-03-25T10:00:00Z

- hypothesis: Ghostty's focused flag transitions false→true during drag-end restore
  evidence: Log shows pane 1001 (active, new pane) has has_focus=true during drag-end recovery. This means EventControllerFocus leave never fired — GTK never removed keyboard focus from pane 1001 during the drag. So Ghostty's focused flag was NEVER set to false, meaning restore_active_pane_focus's set_focus(true) call is a no-op (early-return guard: if self.focused == focused { return; }).
  timestamp: 2026-03-25T10:00:00Z

## Evidence
<!-- APPEND only - facts discovered -->

- timestamp: 2026-03-24T12:05:00Z
  checked: Plan 07 summary (02-07-SUMMARY.md)
  found: Plan 07 fix uses `notify::position` callback on GtkPaned to restore GTK focus to active pane after drag. The callback iterates GL_AREA_REGISTRY, finds pane with "active-pane" CSS class, calls grab_focus().
  implication: Focus restore mechanism exists — need to check if it's working properly

- timestamp: 2026-03-24T12:06:00Z
  checked: split_engine.rs lines 314-330
  found: notify::position handler connects in replace_leaf_with_split. It only calls gl_area.grab_focus() but does NOT call ghostty_surface_set_focus(true) on the surface. GTK focus != Ghostty focus.
  implication: LIKELY ROOT CAUSE — grabbing GTK widget focus doesn't tell Ghostty to resume cursor blink/input

- timestamp: 2026-03-24T12:08:00Z
  checked: Other focus restore patterns via grep for ghostty_surface_set_focus
  found: focus_next_in_direction (lines 414-416) calls BOTH ghostty_surface_set_focus(true) AND grab_focus(). close_active (lines 387-396) calls BOTH ghostty_surface_set_focus(true) AND grab_focus(). sidebar.rs (line 61) calls ghostty_surface_set_focus when switching workspaces.
  implication: CONFIRMED — consistent pattern is grab_focus() + ghostty_surface_set_focus(true). notify::position only does half the job.

- timestamp: 2026-03-24T12:15:00Z
  checked: Implementation of fix
  found: Added GL_TO_SURFACE HashMap in callbacks.rs, populated in realize callback in surface.rs, used in notify::position handler to look up surface from GLArea and call ghostty_surface_set_focus(true). Added cleanup in close_active.
  implication: Fix follows established pattern used in focus_next_in_direction, close_active, and sidebar.

- timestamp: 2026-03-24T12:30:00Z
  checked: User verification of first fix
  found: First fix did NOT work. User reports "cursor AND blue background freezes, input doesn't work". Blue background = CSS active-pane styling. This suggests ENTIRE GLArea render pipeline is broken, not just Ghostty focus.
  implication: Need to investigate deeper — is queue_render being called? Is the GLArea still in registry? Is there a deadlock?

- timestamp: 2026-03-24T12:35:00Z
  checked: notify::position callback behavior
  found: notify::position fires on EVERY position change DURING drag (continuous, not just at end). Handler calls grab_focus() on every event. Calling grab_focus() during an active GtkPaned drag gesture may interfere with the gesture or cause unexpected state.
  implication: LIKELY ROOT CAUSE — grab_focus() during drag gesture may break GTK state

- timestamp: 2026-03-24T12:45:00Z
  checked: Second fix attempt
  found: Changed to debounced approach — only schedule focus restore idle once per burst of position changes. Also changed connect_notify to connect_notify_local (required for non-Send closures). Added diagnostic logging to trace the flow.
  implication: Need to verify if debounced approach fixes the issue, or if the problem is elsewhere

- timestamp: 2026-03-24T13:05:00Z
  checked: User verification of second fix (debounced approach)
  found: Second fix ALSO did not work. Cursor still freezes after divider drag.
  implication: The problem is not about WHEN we call grab_focus, but something more fundamental

- timestamp: 2026-03-24T13:10:00Z
  checked: Third fix attempt — comprehensive focus restore
  found: Added: (1) ghostty_surface_set_focus(true) to restore Ghostty internal focus, (2) queue_render() to trigger terminal redraw, (3) GL_TO_SURFACE mapping for GLArea→surface lookup, (4) cleanup of GL_TO_SURFACE on pane close
  implication: This fix addresses both GTK focus (grab_focus) AND Ghostty focus (set_focus) AND rendering (queue_render)

- timestamp: 2026-03-24T13:25:00Z
  checked: User verification of fix #3
  found: Fix #3 (grab_focus + set_focus + queue_render) also did not work. This eliminates focus and render trigger as the cause.
  implication: The problem is deeper — likely GL context or surface size mismatch

- timestamp: 2026-03-24T13:30:00Z
  checked: Added comprehensive diagnostic logging
  found: Added logging to: (1) resize signal handler, (2) resize idle callback, (3) render callback with render count. This will show exactly what's happening during and after drag.
  implication: Need user to run diagnostic build and report log output to identify where the pipeline breaks

- timestamp: 2026-03-24T13:45:00Z
  checked: Diagnostic log analysis
  found: CRITICAL - THREE ghostty surfaces created (0x61cf88848790, 0x61cf88775170, 0x61cf89b28d80) but only TWO GLAreas receive render callbacks (0x61cf8860b000, 0x61cf897b01e0). The THIRD surface (0x61cf89b28d80) is NEVER rendered!
  implication: ROOT CAUSE FOUND - a third surface/GLArea is being created but never attached to the render pipeline. This orphaned surface is the "frozen" pane.

- timestamp: 2026-03-24T13:55:00Z
  checked: Code analysis for triple surface creation
  found: create_surface is only called from 2 places: (1) create_workspace in app_state.rs, (2) split_active in split_engine.rs. Neither should create extra surfaces. Added detailed logging to trace the source of each surface creation.
  implication: Need to run diagnostic build with enhanced logging to identify which code path creates the orphan surface

- timestamp: 2026-03-24T14:10:00Z
  checked: Enhanced log analysis - FINAL ROOT CAUSE
  found: When split_active() reparents the original GLArea from GtkStack into GtkPaned, GTK UNREALIZES then RE-REALIZES the widget. The realize callback unconditionally creates a new Ghostty surface every time! This orphans the original surface — it's never freed, and the new surface is used for rendering while the orphaned one may still receive input/focus signals.
  implication: ROOT CAUSE CONFIRMED — realize callback must check if surface already exists and reuse it instead of creating a new one

- timestamp: 2026-03-24T14:25:00Z
  checked: User verification of duplicate surface fix
  found: Duplicate surface bug is FIXED — log shows "re-realized — reusing existing surface" and only 2 surfaces created. But cursor still freezes after drag ends. This is a DIFFERENT issue.
  implication: The duplicate surface was ONE bug. There's another bug: renders happen during resize (triggered by resize signals) but stop after drag ends (no wakeup/render loop)

- timestamp: 2026-03-24T14:30:00Z
  checked: Post-resize render behavior
  found: Terminal renders DURING resize (triggered by resize idle callback) but freezes AFTER resize ends. The Ghostty wakeup_cb should trigger renders for cursor blink, shell output, etc. but something may have stopped it.
  implication: Need to check if wakeup_cb is still firing after resize ends

- timestamp: 2026-03-24T14:40:00Z
  checked: Wakeup callback logging
  found: CONFIRMED - wakeup_cb stops firing after split! Only 2 wakeup logs total (#1 at startup, #61 during resize). After split/resize, NO MORE WAKEUPS. Without wakeups, there's nothing to trigger cursor blink or shell output rendering.
  implication: The Ghostty wakeup mechanism breaks after split. Need to understand how wakeup is registered and what breaks it.

- timestamp: 2026-03-24T14:50:00Z
  checked: Re-realize path analysis
  found: When GLArea is re-realized after reparenting, the GL context changes. But the Ghostty surface was set up with the old GL context. The re-realize code only called set_size, set_content_scale, and queue_render — but NOT ghostty_surface_refresh() which may be needed to kick Ghostty's internal render loop.
  implication: Added ghostty_surface_refresh() call in re-realize path to restart Ghostty's wakeup mechanism

- timestamp: 2026-03-24T15:10:00Z
  checked: Focus state during split operation
  found: Line 217 in split_engine.rs calls ghostty_surface_set_focus(inherited_surface, false) BEFORE reparenting. After reparent, the re-realize callback calls refresh() but NOT set_focus(true). In Ghostty's render thread (Thread.zig lines 390-403), when focus is false, the cursor blink timer is CANCELLED. Timer is only restarted when focus transitions to true (lines 404-417). Since focus was never restored, cursor blink timer stays stopped!
  implication: ROOT CAUSE — must call ghostty_surface_set_focus(existing_surface, true) in re-realize path to restart cursor blink timer

- timestamp: 2026-03-25T10:00:00Z
  checked: New diagnostic logs — drag-end recovery state
  found: pane 1001 (active, new pane) shows has_focus=true in drag-end recovery. The GtkPaned separator drag does NOT steal GTK keyboard focus from the terminal GLArea. EventControllerFocus leave NEVER fires during a divider drag. So Ghostty's focused flag on pane 1001 stays true throughout. The set_focus(true) in restore_active_pane_focus hits Ghostty's early-return guard (if self.focused == focused { return; }) → cursor blink timer is NEVER restarted.
  implication: ROOT CAUSE CONFIRMED — need to force a false→true focus toggle to bypass the guard and restart the cursor blink timer

- timestamp: 2026-03-25T10:00:00Z
  checked: re-realize skip comment in surface.rs lines 92-97
  found: The re-realize path intentionally skips set_focus(true) with the comment "EventControllerFocus handles restoration automatically". This assumption is WRONG — EventControllerFocus enter only fires when GTK explicitly routes focus to the widget. During a split+reparent where the pane never lost GTK focus, enter doesn't fire again.
  implication: The skip comment is based on a false assumption for the case where the pane retains GTK focus through the reparent. However, the restore is needed in drag-end recovery, not re-realize.

- timestamp: 2026-03-25T11:00:00Z
  checked: SplitEngine::new and restore_active_pane_focus CSS class logic
  found: SplitEngine::new does NOT call update_focus_css — so the initial single-pane GLArea never gets "active-pane" CSS class. restore_active_pane_focus looks for has_css_class("active-pane") to identify the active surface — for single-pane it finds nothing and skips the bounce. For window resize (no GtkPaned), restore_active_pane_focus is never called at all. The resize signal itself (connect_resize in surface.rs) is the only code path common to both scenarios.
  implication: The fix must be in connect_resize using area.has_focus() (not CSS class lookup) to identify the focused pane.

- timestamp: 2026-03-25T11:00:00Z
  checked: connect_resize handler in surface.rs
  found: Each GLArea has its own connect_resize callback with direct access to surface_cell. Adding a debounced 150ms settle timeout here covers all resize scenarios (single-pane window resize and split-pane divider drag). area.has_focus() correctly identifies the focused pane without relying on CSS classes.
  implication: Added debounced post-settle focus bounce in connect_resize. Build passes.

## Resolution
<!-- OVERWRITE as understanding evolves -->

root_cause: ghostty_surface_set_size (called on every resize event) causes Ghostty's IO thread to rebuild cells, which cancels the cursor blink timer as a side-effect. After rebuild, the timer is only restarted when focusCallback sees a false→true transition. Because GTK never removes keyboard focus from the terminal during a resize (no EventControllerFocus leave), Ghostty's focused flag stays true throughout. A plain set_focus(true) is a no-op (early-return guard: if self.focused == focused { return; }). With no timer restart and no wakeups, the cursor freezes. This affects BOTH single-pane window resize (no GtkPaned at all) and split-pane divider drag (both fire connect_resize). The previous drag-end fix (restore_active_pane_focus) only ran for split-pane and only fired after the gesture ended — too late for the general case.
fix: Added a debounced 150ms settle timeout in connect_resize (surface.rs). Each resize event resets the debounce. When the stream settles, if the GLArea currently has GTK focus (area.has_focus()), do set_focus(false)+set_focus(true)+ghostty_surface_refresh. The false→true bounce bypasses the early-return guard and restarts the cursor blink timer. Using area.has_focus() correctly identifies the focused pane without requiring CSS class ("active-pane" is not set on single-pane workspaces).
verification: Build passes. Awaiting human verification.
files_changed: [src/ghostty/surface.rs]
