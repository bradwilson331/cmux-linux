# cmux - GPU-Accelerated Terminal Multiplexer

cmux provides tabs, splits, workspaces, and socket CLI control powered by Ghostty.

## Skills

Agent skills are installed at `/usr/share/cmux/skills/`:

- **cmux** (`/usr/share/cmux/skills/cmux/SKILL.md`) -- Core terminal multiplexer skill: workspaces, panes, surfaces, socket CLI
- **cmux-browser** (`/usr/share/cmux/skills/cmux-browser/SKILL.md`) -- Browser automation skill: open sites, interact with pages, wait for state changes, extract data

Read the SKILL.md in each directory for full usage instructions.

## CLI

The `cmux` CLI communicates with the running cmux-app via Unix socket JSON-RPC.

### Key Commands

```bash
# Terminal management
cmux list-workspaces          # List all workspaces
cmux list-surfaces            # List all terminal surfaces
cmux list-panes               # List all panes
cmux split --direction horizontal  # Split current pane

# Browser automation (defaults to JSON output)
cmux browser open <url>       # Open URL, returns surface:N handle
cmux browser surface:N snapshot --interactive  # Accessibility tree with element refs
cmux browser surface:N click e1               # Click element by ref
cmux browser surface:N fill e1 "text"         # Fill input field
cmux browser surface:N wait --selector "#id"  # Wait for element
cmux browser list             # List browser surfaces

# System
cmux identify                 # Show instance info
cmux ping                     # Check connectivity
```

### Socket Path

The cmux socket is at `$XDG_RUNTIME_DIR/cmux/cmux.sock` (typically `/run/user/$UID/cmux/cmux.sock`).

Override with `CMUX_SOCKET` environment variable or `--socket` flag.
