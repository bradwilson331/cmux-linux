---
status: deferred
trigger: "Desktop notification still not appearing after replacing gio::Notification with notify-rust. Bell fires (confirmed via sidebar attention dot) but no desktop notification shows up."
created: 2026-03-27T00:00:00Z
updated: 2026-03-27T00:00:00Z
---

## Current Focus

hypothesis: CONFIRMED — GNOME Shell destroys notifications when D-Bus sender name vanishes; notify-rust's zbus connection drops when thread exits, killing the notification
test: Added 10s sleep after show() to keep D-Bus connection alive
expecting: Notification stays visible with sleep, gets destroyed without it
next_action: Implement proper fix — use a persistent D-Bus connection or use notify-send subprocess

## Symptoms

expected: When terminal bell fires while cmux window is unfocused, a desktop notification should appear via notify-rust calling org.freedesktop.Notifications D-Bus interface
actual: Bell fires (attention dot works) but no desktop notification appears. Same behavior as before the fix.
errors: No error messages visible — eprintln in error handler may not reach stdout/stderr, or notification call silently succeeds but daemon doesn't show it
reproduction: 1. Focus a different window. 2. Trigger terminal bell in cmux (echo -e '\a'). 3. Expected: desktop notification. Actual: nothing.
started: notify-rust was added in plan 04-06 to replace gio::Notification which also silently failed

## Eliminated

## Evidence

- timestamp: 2026-03-27T16:25:00Z
  checked: Full code path from bell → set_pane_attention → send_bell_notification
  found: Path works correctly. Debug logging shows bell triggers, window_focused=false, send_bell_notification called, show() returns Ok
  implication: The code path is NOT broken. notify-rust successfully sends D-Bus message. The issue is at the notification daemon level — it receives the notification but doesn't display it.

- timestamp: 2026-03-27T16:28:00Z
  checked: D-Bus traffic comparison between notify-send (works) and cmux notify-rust (fails)
  found: cmux notification gets NotificationClosed (reason 2) within 5-8ms of Notify call. notify-send does not.
  implication: GNOME Shell immediately destroys the cmux notification. Not a protocol issue.

- timestamp: 2026-03-27T16:30:00Z
  checked: GNOME Shell 46 source code — fdoNotificationDaemon.js and notificationDaemon.js
  found: FdoNotificationDaemonSource._onNameVanished() destroys the source (and all its notifications) when the D-Bus sender name vanishes, IF this.app is set. GNOME Shell resolves the sender PID to an app via WindowTracker. Since cmux PID owns a window, this.app IS set.
  implication: When notify-rust's zbus connection drops (thread exits), D-Bus name vanishes, triggering source destruction.

- timestamp: 2026-03-27T16:33:00Z
  checked: Added 10s sleep after show() to keep D-Bus connection alive
  found: Notification stays visible for the full duration. No NotificationClosed signal within 8s monitoring window.
  implication: ROOT CAUSE CONFIRMED — the thread exits too quickly, dropping the zbus D-Bus connection, which causes GNOME Shell to destroy the notification source and all its notifications.

## Resolution

root_cause: |
  GNOME Shell's FdoNotificationDaemonSource._onNameVanished() destroys the notification
  source (and all its notifications) when the D-Bus sender's unique bus name vanishes,
  IF the source is associated with an app (this.app is set).

  notify-rust uses zbus to create a D-Bus connection in the spawned thread. When show()
  returns and the thread exits, the zbus Connection is dropped, causing the D-Bus unique
  name (e.g., :1.680) to vanish. GNOME Shell detects this via NameOwnerChanged, looks up
  the notification source, finds it has an app (because WindowTracker.get_app_from_pid
  matched the cmux PID to its X11 window), and destroys the source.

  The notification is destroyed with reason DISMISSED (2) within ~5-8ms of being created.

  This does NOT happen for standalone binaries (like test_notify_rust) because they have
  no X11 window, so GNOME Shell's WindowTracker doesn't find an app, this.app is null,
  and _onNameVanished skips the destroy() call.

fix: |
  Replaced notify-rust with a subprocess call to `notify-send`. This avoids the D-Bus
  connection lifetime issue because notify-send is a separate process whose D-Bus
  connection is independent of cmux. The subprocess exits after sending the notification,
  but GNOME Shell's NameOwnerChanged handler won't destroy the notification because
  the sender PID (notify-send's PID) doesn't own any X11 windows.

  Also removed the notify-rust crate dependency from Cargo.toml.

verification: |
  1. D-Bus trace confirmed notify-rust notification was closed with reason 2 within 8ms (before fix)
  2. D-Bus trace confirmed notification stayed alive when thread was kept alive with sleep (confirms root cause)
  3. Code builds successfully with notify-send approach
  4. Awaiting human verification that notification appears in real workflow

files_changed:
  - src/app_state.rs
  - Cargo.toml
