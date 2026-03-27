---
status: diagnosed
trigger: "terminal bell sounds but no desktop notification appears when cmux window is unfocused"
created: 2026-03-27T00:00:00Z
updated: 2026-03-27T00:00:00Z
---

## Current Focus

hypothesis: GNotification requires matching .desktop file and D-Bus activation entry; cmux has none
test: check .desktop filename and D-Bus service file against APP_ID
expecting: mismatch between APP_ID "io.cmux.App" and desktop/dbus identifiers
next_action: report root cause

## Symptoms

expected: Desktop notification appears when bell fires and window is unfocused
actual: Bell sound plays, attention dot appears, but no desktop notification shown
errors: (none visible — GNotification silently fails)
reproduction: 1. Unfocus cmux window. 2. Run `echo -e '\a'` in terminal. 3. No notification.
started: Since Phase 04 implementation

## Eliminated

(none)

## Evidence

- timestamp: 2026-03-27
  checked: send_bell_notification in src/app_state.rs:565-572
  found: Uses gio::Notification + app.send_notification() — this is GNotification API
  implication: GNotification requires the app to be a D-Bus activatable application

- timestamp: 2026-03-27
  checked: APP_ID in src/main.rs:20
  found: APP_ID = "io.cmux.App"
  implication: GNotification will look for desktop file named "io.cmux.App.desktop"

- timestamp: 2026-03-27
  checked: resources/cmux.desktop
  found: Filename is "cmux.desktop", no StartupNotify, no DBusActivatable entry
  implication: Desktop file name does not match APP_ID — GNotification cannot find it

- timestamp: 2026-03-27
  checked: D-Bus service files in repo
  found: No D-Bus service file for cmux (only ghostty submodule has one)
  implication: D-Bus activation is impossible; GNotification silently drops notifications

- timestamp: 2026-03-27
  checked: Application flags in src/main.rs:96
  found: flags = NON_UNIQUE — app does not register as primary instance
  implication: NON_UNIQUE may prevent D-Bus registration needed for GNotification

## Resolution

root_cause: |
  GNotification (gio::Notification via app.send_notification) requires THREE things on Linux:
  1. A .desktop file whose filename matches the application ID (io.cmux.App.desktop)
  2. The .desktop file must be installed in a standard XDG location ($XDG_DATA_DIRS/applications/)
  3. The application must be registered on D-Bus (either via D-Bus activation service file or by running as primary instance)

  cmux has NONE of these:
  - The desktop file is named "cmux.desktop" but APP_ID is "io.cmux.App" (filename mismatch)
  - The desktop file is not installed to any XDG applications directory
  - There is no D-Bus service file (resources/ only has cmux.desktop and cmux.svg)
  - The app uses NON_UNIQUE flag which may skip D-Bus name registration

  GNotification silently fails when these prerequisites aren't met — no error, no warning.

fix: |
  Two options:

  Option A (proper GNotification):
  1. Rename cmux.desktop to io.cmux.App.desktop
  2. Add "DBusActivatable=true" to the desktop file
  3. Create resources/io.cmux.App.service D-Bus service file
  4. Ensure desktop file is installed to XDG applications dir
  5. Consider removing NON_UNIQUE flag or ensuring D-Bus name is claimed

  Option B (use libnotify instead — simpler, more reliable):
  1. Add notify-rust crate to Cargo.toml
  2. Replace send_bell_notification to use notify_rust::Notification instead of gio::Notification
  3. notify-rust uses libnotify/D-Bus org.freedesktop.Notifications directly — no .desktop file needed
  4. Works immediately without installation requirements

verification: (not yet verified)
files_changed: []
