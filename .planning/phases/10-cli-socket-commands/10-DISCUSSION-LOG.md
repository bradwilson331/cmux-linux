# Phase 10: CLI Socket Commands - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 10-cli-socket-commands
**Areas discussed:** CLI architecture, Command coverage, Output format, Socket discovery

---

## CLI Architecture

### Q1: Implementation language

| Option | Description | Selected |
|--------|-------------|----------|
| Rust binary (Recommended) | Separate [[bin]] target using clap. Zero runtime deps, fast startup, ships in AppImage. | ✓ |
| Keep Python wrapper | Enhance cmux.py's main() with proper subcommands. Requires Python 3. | |
| You decide | Claude's discretion | |

**User's choice:** Rust binary
**Notes:** None

### Q2: Subcommand structure

| Option | Description | Selected |
|--------|-------------|----------|
| Noun-verb (Recommended) | cmux workspace list, cmux surface split. Mirrors socket namespace. | |
| Flat verbs | cmux list-workspaces, cmux split. Flatter but harder to discover. | ✓ |
| Method passthrough | cmux call workspace.list --params '{}'. Raw socket access. | |

**User's choice:** Flat verbs
**Notes:** "flat verbs to match the cmux implementation"

### Q3: Raw passthrough mode

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, add `cmux raw` subcommand | Direct socket method access for scripting/testing | ✓ |
| No, just flat verbs | Scripts use cmux.py for raw access | |

**User's choice:** Yes, add cmux raw
**Notes:** None

### Q4: Crate layout

| Option | Description | Selected |
|--------|-------------|----------|
| [[bin]] in same crate (Recommended) | src/bin/cmux.rs alongside app binary. Shares types. | ✓ |
| Separate crate in workspace | New cmux-cli/ crate. Cleaner deps but more config. | |

**User's choice:** [[bin]] in same crate
**Notes:** None

### Q5: Binary name

| Option | Description | Selected |
|--------|-------------|----------|
| cmux (Recommended) | Same name as macOS. Consistent cross-platform. | ✓ |
| cmux-cli | Avoids collision with GUI app binary. | |
| cmuxctl | Control-style naming like kubectl. | |

**User's choice:** cmux
**Notes:** None

### Q6: GUI app binary rename

| Option | Description | Selected |
|--------|-------------|----------|
| Rename app to cmux-app | GUI binary becomes cmux-app, CLI is cmux. | ✓ |
| Keep both as cmux | Two binaries, potential confusion. | |
| You decide | Claude's discretion | |

**User's choice:** Rename app to cmux-app
**Notes:** None

---

## Command Coverage

### Q1: Which command groups get CLI subcommands

| Option | Description | Selected |
|--------|-------------|----------|
| Core (workspace/surface/pane) | Essential multiplexer commands | ✓ |
| Browser commands | browser.* socket commands | ✓ |
| Notification commands | notification.* socket commands | ✓ |
| Debug/system commands | ping, identify, capabilities, layout, type | ✓ |

**User's choice:** All groups selected
**Notes:** None

---

## Output Format

### Q1: Default output format

| Option | Description | Selected |
|--------|-------------|----------|
| Human-readable + --json flag | Tables/text default, JSON with flag | ✓ |
| JSON only | All output is JSON | |
| Human-readable only | No JSON option | |

**User's choice:** Human-readable + --json flag
**Notes:** None

### Q2: Color output

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, with auto-detect (Recommended) | Color on TTY, plain when piped. --color flag. | ✓ |
| No color | Plain text only | |
| You decide | Claude's discretion | |

**User's choice:** Yes with auto-detect
**Notes:** None

### Q3: List display style

| Option | Description | Selected |
|--------|-------------|----------|
| Simple lines with markers | `* 1: Main (3 panes)`. Like tmux list-sessions. | ✓ |
| Formatted table | Column-aligned with headers | |
| You decide | Claude's discretion | |

**User's choice:** Simple lines with markers
**Notes:** None

### Q4: Mutation command output

| Option | Description | Selected |
|--------|-------------|----------|
| Print result ID/name | `Created workspace: Dev (uuid)`. Useful for scripting. | ✓ |
| Silent on success | No output, exit code only. Unix convention. | |
| You decide | Claude's discretion | |

**User's choice:** Print result ID/name
**Notes:** None

### Q5: Error display

| Option | Description | Selected |
|--------|-------------|----------|
| Stderr with exit code (Recommended) | `Error: workspace not found: foo`. Standard CLI. | ✓ |
| JSON errors always | Even in human mode, JSON on stderr. | |
| You decide | Claude's discretion | |

**User's choice:** Stderr with exit code
**Notes:** None

---

## Socket Discovery

### Q1: Discovery strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Port cmux.py logic (Recommended) | Same discovery order as Python client. Consistent. | ✓ |
| Simplified Linux-only | Only CMUX_SOCKET and XDG path. Leaner. | |
| You decide | Claude's discretion | |

**User's choice:** Port cmux.py logic
**Notes:** None

### Q2: Socket flag

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, --socket flag (Recommended) | Global flag before subcommand. Also respects CMUX_SOCKET env. | ✓ |
| Env var only | CMUX_SOCKET only, no flag. | |
| You decide | Claude's discretion | |

**User's choice:** Yes, --socket flag
**Notes:** None

### Q3: Multiple instances

| Option | Description | Selected |
|--------|-------------|----------|
| Connect to first found (Recommended) | Discovery order picks first. Use --socket to target. | ✓ |
| List and prompt | Show choices if multiple found. | |
| You decide | Claude's discretion | |

**User's choice:** Connect to first found
**Notes:** None

### Q4: Connection verbosity

| Option | Description | Selected |
|--------|-------------|----------|
| Only with --verbose/-v | Silent by default. -v shows socket path. | ✓ |
| Always show on stderr | Always print socket path to stderr. | |
| You decide | Claude's discretion | |

**User's choice:** Only with --verbose/-v
**Notes:** None

---

## Claude's Discretion

- Exact clap subcommand naming for each socket method
- Internal Rust module structure for CLI binary
- Connection timeout and retry behavior
- Help text wording and examples

## Deferred Ideas

None — discussion stayed within phase scope
