# Phase 9: UI Buttons and Menus - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-27
**Phase:** 09-ui-buttons-and-menus
**Areas discussed:** Sidebar controls, Header bar / toolbar, Right-click context menus, Menu bar / app menu, Config.toml header_bar option, Tooltip hover behavior, New Browser button behavior, Keyboard shortcut discovery

---

## Sidebar Controls

### '+' button for new workspace

| Option | Description | Selected |
|--------|-------------|----------|
| Simple '+' button at bottom | Creates new local workspace (same as Ctrl+N). SSH stays keyboard-only. | ✓ |
| '+' with dropdown menu | Clicking '+' shows popover: New Workspace, New SSH Workspace, New Browser | |
| '+' button at top of sidebar | Same as bottom but positioned above the workspace list | |

**User's choice:** Simple '+' button at bottom

### Close buttons on sidebar rows

| Option | Description | Selected |
|--------|-------------|----------|
| Show 'x' on hover | Small '×' appears at right edge when hovered. Matches iTerm2/Warp/Ghostty | ✓ |
| Always show 'x' | Close button always visible. More discoverable but noisier. | |
| No close button | Keep close as keyboard-only (Ctrl+Shift+W) | |

**User's choice:** Show 'x' on hover

### Sidebar right-click context menu

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, with common actions | Right-click shows: Rename, Close, Split Right, Split Down | ✓ |
| Yes, minimal | Right-click shows only: Rename, Close | |
| No context menu | All sidebar actions stay keyboard-only | |

**User's choice:** Yes, with common actions

---

## Header Bar / Toolbar

### Header bar style

| Option | Description | Selected |
|--------|-------------|----------|
| GTK HeaderBar | Standard GTK4 HeaderBar replaces titlebar. Native GNOME feel. | ✓ |
| Custom toolbar row | Thin GtkBox toolbar below titlebar. Better on KDE/XFCE. | |
| No header bar | Keep current minimal layout | |

**User's choice:** Default to GTK HeaderBar but allow custom and no header bar in config
**Notes:** User wants all three options configurable via config.toml

### Header bar buttons

| Option | Description | Selected |
|--------|-------------|----------|
| New workspace (+) | Quick create new local workspace | ✓ |
| Split horizontal / vertical | Split the active pane | ✓ |
| New browser | Open a new browser pane | ✓ |
| Toggle sidebar | Show/hide the sidebar panel | ✓ |

**User's choice:** All four button types

### Header bar layout

| Option | Description | Selected |
|--------|-------------|----------|
| Left: workspace, Right: pane/view | Left: [+New] [Browser]. Right: [Split H] [Split V] [Sidebar] | ✓ |
| All buttons on right | Right side only with window title on left | |
| You decide | Claude picks layout | |

**User's choice:** Left: workspace actions, Right: pane/view actions

---

## Right-Click Context Menus

### Terminal pane menu contents

| Option | Description | Selected |
|--------|-------------|----------|
| Copy / Paste | Standard clipboard operations | ✓ |
| Split Right / Split Down | Quick split from right-click target pane | ✓ |
| Close Pane | Close the right-clicked pane | ✓ |
| Open Browser Here | Open a browser pane next to this terminal | ✓ |

**User's choice:** All options selected

### Browser pane context menu

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, adapted menu | Close Pane, Open in External Browser, Copy URL | ✓ |
| No context menu | Right-click does nothing on browser preview | |
| You decide | Claude decides | |

**User's choice:** Yes, adapted menu

### Accelerator hints in menus

| Option | Description | Selected |
|--------|-------------|----------|
| Show accelerator hints | Each item shows shortcut (e.g., 'Split Right  Ctrl+D') | ✓ |
| No accelerator hints | Menu items only show action name | |
| You decide | Claude picks based on GTK conventions | |

**User's choice:** Show accelerator hints

### Divider right-click menu

| Option | Description | Selected |
|--------|-------------|----------|
| No | Dividers are drag-only, keep simple | ✓ |
| Yes, with reset/equalize | Right-click shows: Reset to 50/50, Close Left/Right | |

**User's choice:** No

---

## Menu Bar / App Menu

### Menu type

| Option | Description | Selected |
|--------|-------------|----------|
| Hamburger menu in HeaderBar | '☰' button opens popover with sections. Modern GTK4/GNOME. | ✓ |
| Traditional menu bar | GtkMenuBar with File/Edit/View/Help dropdowns | |
| Both (configurable) | Default hamburger, option for traditional | |

**User's choice:** Hamburger menu in HeaderBar

### Menu sections

| Option | Description | Selected |
|--------|-------------|----------|
| File: New/Close/SSH/Browser/Quit | Workspace and pane lifecycle | ✓ |
| Edit: Copy/Paste/Find/Preferences | Clipboard, search, settings link | ✓ |
| View: Sidebar/Split/Zoom | Layout and display controls | ✓ |
| Help: Shortcuts/About | Reference and app info | ✓ |

**User's choice:** All sections

### Shortcuts window

| Option | Description | Selected |
|--------|-------------|----------|
| GtkShortcutsWindow | GTK4 built-in shortcuts overview. Standard GNOME pattern. | ✓ |
| Simple dialog | Plain dialog listing shortcuts as text | |
| You decide | Claude picks | |

**User's choice:** GtkShortcutsWindow

### Preferences action

| Option | Description | Selected |
|--------|-------------|----------|
| Open config.toml in $EDITOR | Launches user's editor. Consistent with Ghostty. | ✓ |
| Basic settings dialog | GTK dialog with key settings. Full config in toml. | |
| Defer settings UI | No Preferences in menu | |

**User's choice:** Open config.toml in $EDITOR

### About dialog

| Option | Description | Selected |
|--------|-------------|----------|
| GtkAboutDialog | Standard GTK4 About with name, version, license, website | ✓ |
| You decide | Claude decides | |

**User's choice:** GtkAboutDialog

---

## Config.toml Header Bar Option

| Option | Description | Selected |
|--------|-------------|----------|
| Simple string field | [ui] header_bar = 'gtk' | 'custom' | 'none' | |
| More granular config | [ui.header_bar] with style, buttons_left, buttons_right | ✓ |
| You decide | Claude picks | |

**User's choice:** More granular config

---

## Tooltip Hover Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, always | Every button shows tooltip like 'New Workspace (Ctrl+N)' | ✓ |
| No tooltips | Keep UI clean | |

**User's choice:** Yes, always

---

## New Browser Button Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Open URL entry dialog | Shows dialog asking for URL, then opens pane | |
| Create blank browser pane | Immediately creates pane with empty URL bar focused | |
| Same as Ctrl+Shift+B | Triggers existing BrowserOpen action | ✓ |

**User's choice:** Same as Ctrl+Shift+B

---

## Keyboard Shortcut Discovery

| Option | Description | Selected |
|--------|-------------|----------|
| Tooltips only | Tooltips + GtkShortcutsWindow + menu accelerators sufficient | ✓ |
| Status bar at bottom | Thin bar showing shortcut for hovered action | |
| You decide | Claude picks | |

**User's choice:** Tooltips only

---

## Claude's Discretion

- Icon choices for header bar buttons
- PopoverMenu construction approach
- GtkShortcutsWindow section organization
- CSS styling details
- External browser open mechanism
- Find action trigger

## Deferred Ideas

- In-app settings dialog
- Status bar for shortcut hints
- Sidebar drag-to-reorder
- Traditional GtkMenuBar
- Tab bar mode
