---
status: diagnosed
trigger: "SSH workspace creation failure: cmuxd-remote binary not found + infinite reconnect loop"
created: 2026-03-27T14:00:00Z
updated: 2026-03-27T14:00:00Z
---

## Current Focus

hypothesis: Two separate bugs - (1) missing build/install step for cmuxd-remote binary, (2) reconnect loop never gives up even on permanent failures like missing binary
test: Code review of deploy.rs and tunnel.rs run_ssh_lifecycle
expecting: Confirm no install step places binary at expected path; confirm no max-retry exit condition
next_action: Return diagnosis

## Symptoms

expected: SSH workspace creation deploys cmuxd-remote to remote host and connects
actual: "cmuxd-remote binary not found at ~/.local/share/cmux/bin/cmuxd-remote-linux-amd64" then infinite reconnect loop
errors: "cmuxd-remote binary not found at ~/.local/share/cmux/bin/cmuxd-remote-linux-amd64"
reproduction: Create SSH workspace
started: First attempt

## Eliminated

(none)

## Evidence

- timestamp: 2026-03-27T14:00:00Z
  checked: deploy.rs local_daemon_path()
  found: Expects binary at $XDG_DATA_HOME/cmux/bin/cmuxd-remote-linux-amd64 (defaults to ~/.local/share/cmux/bin/cmuxd-remote-linux-amd64)
  implication: Binary must be placed there by some build/install step

- timestamp: 2026-03-27T14:00:00Z
  checked: ~/.local/share/cmux/ directory
  found: Only contains session.json - no bin/ directory exists
  implication: No build or install step has ever placed the binary there

- timestamp: 2026-03-27T14:00:00Z
  checked: daemon/remote/ directory
  found: Contains pre-built cmuxd-remote Go binary (4.6MB) and Go source in cmd/cmuxd-remote/
  implication: Binary exists in repo but not at the path deploy.rs expects

- timestamp: 2026-03-27T14:00:00Z
  checked: scripts/build_remote_daemon_release_assets.sh
  found: Builds cmuxd-remote-linux-amd64 etc. into an output directory for release packaging
  implication: This is a release build script, not a dev install script. No script copies binary to ~/.local/share/cmux/bin/

- timestamp: 2026-03-27T14:00:00Z
  checked: Cargo.toml, Makefiles, all shell scripts
  found: No install/setup step copies cmuxd-remote to ~/.local/share/cmux/bin/
  implication: The path in deploy.rs is a packaging convention that has no corresponding install step

- timestamp: 2026-03-27T14:00:00Z
  checked: run_ssh_lifecycle() in tunnel.rs lines 26-136
  found: Outer loop is `loop { ... }` with NO exit condition. When deploy fails (attempt==0), it logs error, increments attempt, sleeps with backoff, then continues the loop. On attempt > 0 it skips deploy entirely and goes straight to start_ssh(), which will also fail because binary was never deployed.
  implication: Deploy failure is treated as transient - retried forever. After first attempt, deploy is skipped, so SSH connects but cmuxd-remote is missing on remote, SSH exits, reconnect loop continues forever at 30s cap.

- timestamp: 2026-03-27T14:00:00Z
  checked: backoff_duration() in tunnel.rs
  found: Caps at 30 seconds (MAX_BACKOFF_SECS). Backoff is 1,2,4,8,16,30,30,30... forever
  implication: Backoff exists but has no max attempt count - loops indefinitely

## Resolution

root_cause: |
  TWO BUGS:

  1. MISSING BINARY - No install/setup step: deploy.rs expects cmuxd-remote-linux-amd64 at
     ~/.local/share/cmux/bin/cmuxd-remote-linux-amd64, but nothing in the build system places
     it there. The release script (build_remote_daemon_release_assets.sh) builds it into a
     release output directory, but there is no dev-mode install that copies daemon/remote/cmuxd-remote
     to the expected XDG data path. The error message tells you to run
     `cd daemon/remote && GOOS=linux GOARCH=amd64 go build -o <path> ./cmd/cmuxd-remote/`
     manually, but this is never automated.

  2. INFINITE RECONNECT LOOP: run_ssh_lifecycle() (tunnel.rs:28) is an unconditional `loop {}`
     with no max-retry exit. When deploy fails on attempt 0, it increments attempt and continues.
     On subsequent attempts (attempt > 0), deploy is SKIPPED (line 36: `if attempt == 0`), so
     the binary is never re-deployed. SSH connects to the remote but cmuxd-remote doesn't exist
     there, so it exits immediately. This triggers the reconnect loop which retries every 30s
     forever with no way to stop.

fix: (not yet applied)
verification: (not yet done)
files_changed: []
