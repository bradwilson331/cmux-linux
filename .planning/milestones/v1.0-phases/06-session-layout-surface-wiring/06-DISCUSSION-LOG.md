# Phase 6: Session Layout Restore + Surface Wiring - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-26
**Phase:** 06-session-layout-surface-wiring
**Areas discussed:** Tree restoration strategy, Surface lifecycle on restore, CWD/shell capture, Failure/edge cases

---

## Tree Restoration Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Recursive from_data() on SplitEngine | Add SplitNode::from_data() that mirrors to_data() — recursively builds GtkPaned + GLArea tree from the serialized structure | ✓ |
| Replay split operations | Start with one pane, then call split_active() repeatedly to reconstruct the tree | |
| You decide | Claude picks the best approach during planning | |

**User's choice:** Recursive from_data() on SplitEngine
**Notes:** Clean inverse of existing to_data() method

---

| Option | Description | Selected |
|--------|-------------|----------|
| Save and restore positions | Store GtkPaned position as ratio in SplitNodeData | ✓ |
| Always reset to 50/50 | Simpler — just rebuild structure without tracking positions | |
| You decide | Claude picks based on complexity during planning | |

**User's choice:** Save and restore positions
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — restore active pane focus | Save active_pane_uuid in WorkspaceSession | ✓ |
| No — always focus first leaf | Simpler. Active pane resets to first leaf on restore | |

**User's choice:** Yes — restore active pane focus
**Notes:** Field already exists as TODO in WorkspaceSession

---

| Option | Description | Selected |
|--------|-------------|----------|
| All workspaces | Every workspace's full split tree is saved | ✓ |
| Active workspace only | Inactive workspaces restore as single pane | |

**User's choice:** All workspaces
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Ratio (0.0–1.0) | Relative to parent size, works across window resizes | ✓ |
| Absolute pixels | Exact GtkPaned position in pixels | |
| You decide | Claude picks during implementation | |

**User's choice:** Ratio (0.0–1.0)
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Bump to version 2 | Clear signal that old session files need migration or fallback | ✓ |
| Keep version 1, add optional fields | Backward compatible — new fields default to single-pane if absent | |
| You decide | Claude picks based on implementation tradeoffs | |

**User's choice:** Bump to version 2
**Notes:** v1 files auto-upgrade to single-pane per workspace, next save writes v2

---

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-upgrade: treat as single-pane per workspace | If version=1, restore workspace names and create one default pane per workspace | ✓ |
| Discard and start fresh | If version < 2, ignore session file entirely | |

**User's choice:** Auto-upgrade: treat as single-pane per workspace
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Fresh IDs, preserve UUIDs | Pane IDs are internal counters — generate fresh. UUIDs preserved for socket command stability | ✓ |
| Preserve both pane_id and UUID | Restore exact pane_ids from session | |
| Fresh everything | Both pane_id and UUID are regenerated | |

**User's choice:** Fresh IDs, preserve UUIDs
**Notes:** Socket commands reference UUIDs — these must be stable across restarts

---

## Surface Lifecycle on Restore

| Option | Description | Selected |
|--------|-------------|----------|
| Same lifecycle as new splits | GLArea realize callback creates Ghostty surface and calls set_initial_surface() | ✓ |
| Batch creation after tree built | Build entire widget tree first, then iterate leaves and force-realize | |
| You decide | Claude picks based on GTK4 lifecycle constraints | |

**User's choice:** Same lifecycle as new splits
**Notes:** Surfaces wire up naturally as GL contexts become ready

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — pass CWD to surface config | Terminal opens in directory user was last working in | ✓ |
| No — always use default CWD | All restored panes open in $HOME | |
| You decide | Claude determines based on Ghostty API capabilities | |

**User's choice:** Yes — pass CWD to surface config
**Notes:** Use ghostty_surface_inherited_config() or equivalent

---

## CWD/Shell Capture

| Option | Description | Selected |
|--------|-------------|----------|
| /proc/{pid}/cwd readlink | Standard Linux approach. Requires tracking child PID per surface | ✓ |
| OSC 7 escape sequence | Parse from terminal output. Requires shell cooperation | |
| You decide | Claude picks the most reliable approach | |

**User's choice:** /proc/{pid}/cwd readlink
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Default shell only | Always launch $SHELL or /bin/sh on restore | ✓ |
| Capture running command | Read /proc/{pid}/cmdline to save exact command | |
| You decide | Claude picks based on macOS cmux behavior | |

**User's choice:** Default shell only
**Notes:** Simpler and handles 99% of cases

---

## Failure/Edge Cases

| Option | Description | Selected |
|--------|-------------|----------|
| Skip failed pane, collapse parent | Remove failed leaf from tree, sibling expands. Log warning | ✓ |
| Fallback to single pane for workspace | Discard entire split tree on any failure | |
| You decide | Claude picks based on implementation complexity | |

**User's choice:** Skip failed pane, collapse parent
**Notes:** Same behavior as close-pane

---

| Option | Description | Selected |
|--------|-------------|----------|
| Basic validation | Check JSON parses, version present, at least one workspace | ✓ |
| Strict validation | Walk entire tree checking UUID uniqueness, orientation values | |
| No validation — trust the file | Handle errors at widget creation level | |

**User's choice:** Basic validation
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Cap at 16 levels | Reject deeper trees, fall back to single pane | ✓ |
| No limit | Trust the file | |
| You decide | Claude picks a sensible limit | |

**User's choice:** Cap at 16 levels
**Notes:** More than anyone would use intentionally

---

## Claude's Discretion

- How to obtain shell PID from Ghostty surface for /proc readlink
- Exact SplitNodeData schema changes to include ratio field
- from_data() method signature and error handling internals
- Whether to add active_pane_uuid to session save trigger

## Deferred Ideas

- Shell command replay (vim, htop, etc.) — too complex and potentially dangerous
- OSC 7 CWD tracking — could be added later
- Multi-window session restore — single-window only for now
- Divider position animation on restore — unnecessary complexity
