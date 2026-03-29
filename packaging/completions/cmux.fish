# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_cmux_global_optspecs
	string join \n socket= json v/verbose color= h/help
end

function __fish_cmux_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_cmux_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_cmux_using_subcommand
	set -l cmd (__fish_cmux_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c cmux -n "__fish_cmux_needs_command" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_needs_command" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_needs_command" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_needs_command" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_needs_command" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "ping" -d 'Ping the running cmux instance'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "identify" -d 'Show cmux instance identity (version, platform, pid)'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "capabilities" -d 'List supported socket commands'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "list-workspaces" -d 'List all workspaces'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "current-workspace" -d 'Show the current workspace'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "raw" -d 'Send an arbitrary JSON-RPC method'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "new-workspace" -d 'Create a new workspace'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "select-workspace" -d 'Select a workspace by ID'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "close-workspace" -d 'Close a workspace by ID'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "rename-workspace" -d 'Rename a workspace'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "next-workspace" -d 'Switch to next workspace'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "prev-workspace" -d 'Switch to previous workspace'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "last-workspace" -d 'Switch to last active workspace'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "reorder-workspace" -d 'Reorder a workspace'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "list-surfaces" -d 'List all surfaces'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "split" -d 'Split a surface'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "focus-surface" -d 'Focus a surface by ID'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "close-surface" -d 'Close a surface by ID'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "send-text" -d 'Send text to a surface'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "send-key" -d 'Send a key event to a surface'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "read-text" -d 'Read text from a surface'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "health" -d 'Check surface health'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "refresh" -d 'Refresh a surface'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "list-panes" -d 'List all panes'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "focus-pane" -d 'Focus a pane'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "last-pane" -d 'Switch to last focused pane'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "list-windows" -d 'List all windows'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "current-window" -d 'Show current window info'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "layout" -d 'Show layout tree'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "type" -d 'Type text into the focused terminal'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "list-notifications" -d 'List notifications'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "clear-notification" -d 'Clear a notification'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "browser-open" -d 'Open a URL in the browser pane'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "browser-close" -d 'Close the browser pane'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "browser-stream-enable" -d 'Enable browser streaming'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "browser-stream-disable" -d 'Disable browser streaming'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "browser-snapshot" -d 'Take a browser snapshot'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "browser-screenshot" -d 'Take a browser screenshot'
complete -c cmux -n "__fish_cmux_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c cmux -n "__fish_cmux_using_subcommand ping" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand ping" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand ping" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand ping" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand ping" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand identify" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand identify" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand identify" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand identify" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand identify" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand capabilities" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand capabilities" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand capabilities" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand capabilities" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand capabilities" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand list-workspaces" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand list-workspaces" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand list-workspaces" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand list-workspaces" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand list-workspaces" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand current-workspace" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand current-workspace" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand current-workspace" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand current-workspace" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand current-workspace" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand raw" -l params -d 'JSON params string' -r
complete -c cmux -n "__fish_cmux_using_subcommand raw" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand raw" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand raw" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand raw" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand raw" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand new-workspace" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand new-workspace" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand new-workspace" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand new-workspace" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand new-workspace" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand select-workspace" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand select-workspace" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand select-workspace" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand select-workspace" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand select-workspace" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand close-workspace" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand close-workspace" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand close-workspace" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand close-workspace" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand close-workspace" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand rename-workspace" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand rename-workspace" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand rename-workspace" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand rename-workspace" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand rename-workspace" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand next-workspace" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand next-workspace" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand next-workspace" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand next-workspace" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand next-workspace" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand prev-workspace" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand prev-workspace" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand prev-workspace" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand prev-workspace" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand prev-workspace" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand last-workspace" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand last-workspace" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand last-workspace" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand last-workspace" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand last-workspace" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand reorder-workspace" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand reorder-workspace" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand reorder-workspace" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand reorder-workspace" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand reorder-workspace" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand list-surfaces" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand list-surfaces" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand list-surfaces" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand list-surfaces" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand list-surfaces" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand split" -l direction -d 'Split direction: horizontal or vertical' -r
complete -c cmux -n "__fish_cmux_using_subcommand split" -l id -d 'Target surface ID (default: focused)' -r
complete -c cmux -n "__fish_cmux_using_subcommand split" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand split" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand split" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand split" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand split" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand focus-surface" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand focus-surface" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand focus-surface" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand focus-surface" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand focus-surface" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand close-surface" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand close-surface" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand close-surface" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand close-surface" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand close-surface" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand send-text" -l id -d 'Target surface ID (default: focused)' -r
complete -c cmux -n "__fish_cmux_using_subcommand send-text" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand send-text" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand send-text" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand send-text" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand send-text" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand send-key" -l id -d 'Target surface ID (default: focused)' -r
complete -c cmux -n "__fish_cmux_using_subcommand send-key" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand send-key" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand send-key" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand send-key" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand send-key" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand read-text" -l id -d 'Target surface ID (default: focused)' -r
complete -c cmux -n "__fish_cmux_using_subcommand read-text" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand read-text" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand read-text" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand read-text" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand read-text" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand health" -l id -d 'Target surface ID (default: focused)' -r
complete -c cmux -n "__fish_cmux_using_subcommand health" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand health" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand health" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand health" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand health" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand refresh" -l id -d 'Target surface ID (default: focused)' -r
complete -c cmux -n "__fish_cmux_using_subcommand refresh" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand refresh" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand refresh" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand refresh" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand refresh" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand list-panes" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand list-panes" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand list-panes" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand list-panes" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand list-panes" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand focus-pane" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand focus-pane" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand focus-pane" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand focus-pane" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand focus-pane" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand last-pane" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand last-pane" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand last-pane" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand last-pane" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand last-pane" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand list-windows" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand list-windows" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand list-windows" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand list-windows" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand list-windows" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand current-window" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand current-window" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand current-window" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand current-window" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand current-window" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand layout" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand layout" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand layout" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand layout" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand layout" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand type" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand type" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand type" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand type" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand type" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand list-notifications" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand list-notifications" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand list-notifications" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand list-notifications" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand list-notifications" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand clear-notification" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand clear-notification" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand clear-notification" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand clear-notification" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand clear-notification" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand browser-open" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-open" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-open" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand browser-open" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand browser-open" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand browser-close" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-close" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-close" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand browser-close" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand browser-close" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand browser-stream-enable" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-stream-enable" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-stream-enable" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand browser-stream-enable" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand browser-stream-enable" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand browser-stream-disable" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-stream-disable" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-stream-disable" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand browser-stream-disable" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand browser-stream-disable" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand browser-snapshot" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-snapshot" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-snapshot" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand browser-snapshot" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand browser-snapshot" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand browser-screenshot" -l socket -d 'Path to the cmux socket (overrides discovery)' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-screenshot" -l color -d 'Color mode: always, never, auto' -r
complete -c cmux -n "__fish_cmux_using_subcommand browser-screenshot" -l json -d 'Output raw JSON responses'
complete -c cmux -n "__fish_cmux_using_subcommand browser-screenshot" -s v -l verbose -d 'Verbose output (connection info to stderr)'
complete -c cmux -n "__fish_cmux_using_subcommand browser-screenshot" -s h -l help -d 'Print help'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "ping" -d 'Ping the running cmux instance'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "identify" -d 'Show cmux instance identity (version, platform, pid)'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "capabilities" -d 'List supported socket commands'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "list-workspaces" -d 'List all workspaces'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "current-workspace" -d 'Show the current workspace'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "raw" -d 'Send an arbitrary JSON-RPC method'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "new-workspace" -d 'Create a new workspace'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "select-workspace" -d 'Select a workspace by ID'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "close-workspace" -d 'Close a workspace by ID'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "rename-workspace" -d 'Rename a workspace'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "next-workspace" -d 'Switch to next workspace'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "prev-workspace" -d 'Switch to previous workspace'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "last-workspace" -d 'Switch to last active workspace'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "reorder-workspace" -d 'Reorder a workspace'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "list-surfaces" -d 'List all surfaces'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "split" -d 'Split a surface'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "focus-surface" -d 'Focus a surface by ID'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "close-surface" -d 'Close a surface by ID'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "send-text" -d 'Send text to a surface'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "send-key" -d 'Send a key event to a surface'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "read-text" -d 'Read text from a surface'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "health" -d 'Check surface health'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "refresh" -d 'Refresh a surface'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "list-panes" -d 'List all panes'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "focus-pane" -d 'Focus a pane'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "last-pane" -d 'Switch to last focused pane'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "list-windows" -d 'List all windows'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "current-window" -d 'Show current window info'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "layout" -d 'Show layout tree'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "type" -d 'Type text into the focused terminal'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "list-notifications" -d 'List notifications'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "clear-notification" -d 'Clear a notification'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "browser-open" -d 'Open a URL in the browser pane'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "browser-close" -d 'Close the browser pane'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "browser-stream-enable" -d 'Enable browser streaming'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "browser-stream-disable" -d 'Disable browser streaming'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "browser-snapshot" -d 'Take a browser snapshot'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "browser-screenshot" -d 'Take a browser screenshot'
complete -c cmux -n "__fish_cmux_using_subcommand help; and not __fish_seen_subcommand_from ping identify capabilities list-workspaces current-workspace raw new-workspace select-workspace close-workspace rename-workspace next-workspace prev-workspace last-workspace reorder-workspace list-surfaces split focus-surface close-surface send-text send-key read-text health refresh list-panes focus-pane last-pane list-windows current-window layout type list-notifications clear-notification browser-open browser-close browser-stream-enable browser-stream-disable browser-snapshot browser-screenshot help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
