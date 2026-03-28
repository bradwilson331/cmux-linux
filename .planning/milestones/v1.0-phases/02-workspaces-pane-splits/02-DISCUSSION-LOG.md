# Phase 2: Discussion Log

**Date:** 2026-03-23
**Areas discussed:** Workspace chrome, Split divider widget

---

## Workspace Chrome

**Q: How should workspaces be displayed?**
Options: Sidebar (match macOS), Top tab bar
Selected: **Sidebar (match macOS)**

**Q: When the sidebar is visible, should workspace names be shown inline or only on hover/icon-mode?**
Options: Names always visible, Collapsible to icons
Selected: **Names always visible**

**Q: Should the active workspace name be editable inline or only via keyboard shortcut?**
Options: Keyboard shortcut only, Click to rename inline
Selected: **Keyboard shortcut only**

---

## Split Divider Widget

**Q: How should pane split dividers be implemented?**
Options: GtkPaned nested, Custom drawing widget
Selected: **GtkPaned nested**
Preview confirmed: Root GtkPaned (horizontal) → Left: GLArea, Right: GtkPaned (vertical) → Top: GLArea, Bottom: GLArea

**Q: When a pane is closed, what should happen to the space it occupied?**
Options: Sibling expands to fill, Parent split collapses
Selected: **Sibling expands to fill**

**Q: When splitting a pane, where should the new terminal open?**
Options: Same CWD as active pane, Default shell directory
Selected: **Same CWD as active pane** (via ghostty_surface_inherited_config)

**Q: Should the initial split ratio be 50/50, or weight toward the active pane?**
Options: 50/50 always, Active pane keeps 2/3
Selected: **50/50 always**
