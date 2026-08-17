# Ghostty Fork Changes

This repo uses a fork of Ghostty for local patches that aren't upstream yet.
When we change the fork, update this document and the parent submodule SHA.

## Fork source

The submodule previously pointed at `manaflow-ai/ghostty` pinned to
`4845e82d0713c29071fb78170491682c8b4c8495`. That commit was never published —
the fork was rewritten and the SHA became unfetchable, which broke
`git clone --recurse-submodules` for everyone (issues #1 and #3).

It now points at `tcsenpai/ghostty`, branch `cmux-fork-gtk4-platform`, which
reconstructs the GTK4 embedded-platform patches described in section 8 below.

**Never pin a SHA that is not reachable from a published branch.** A local
commit that builds on this machine is worth nothing to anyone cloning the repo;
that is the exact failure this project already shipped once.

## Fork update checklist

1) Make changes in `ghostty/`.
2) Commit and **push to a public branch** of the fork.
3) Verify the SHA is fetchable: `git -C ghostty fetch origin && git -C ghostty
   merge-base --is-ancestor HEAD origin/<branch>`.
4) Update this file with the new change summary + conflict notes.
5) In the parent repo: `git add ghostty` and commit the submodule SHA.

## Current fork changes

Fork rebased onto upstream `v1.3.0` plus newer `main` commits as of March 12, 2026.

### 1) OSC 99 (kitty) notification parser

- Commit: `a2252e7a9` (Add OSC 99 notification parser)
- Files:
  - `src/terminal/osc.zig`
  - `src/terminal/osc/parsers.zig`
  - `src/terminal/osc/parsers/kitty_notification.zig`
- Summary:
  - Adds a parser for kitty OSC 99 notifications and wires it into the OSC dispatcher.

### 2) macOS display link restart on display changes

- Commit: `c07e6c5a5` (macos: restart display link after display ID change)
- Files:
  - `src/renderer/generic.zig`
- Summary:
  - Restarts the CVDisplayLink when `setMacOSDisplayID` updates the current CGDisplay.
  - Prevents a rare state where vsync is "running" but no callbacks arrive, which can look like a frozen surface until focus/occlusion changes.

### 3) Keyboard copy mode selection C API

- Commit: `a50579bd5` (Add C API for keyboard copy mode selection)
- Files:
  - `src/Surface.zig`
  - `src/apprt/embedded.zig`
- Summary:
  - Restores `ghostty_surface_select_cursor_cell` and `ghostty_surface_clear_selection`.
  - Keeps cmux keyboard copy mode working against the refreshed Ghostty base.

### 4) macOS resize stale-frame mitigation

Sections 3 and 4 are grouped by feature, not by commit order. The section 4 resize commits were
applied earlier than the section 3 copy-mode commit, but they are kept together here because they
touch the same stale-frame mitigation path and tend to conflict in the same files during rebases.

- Commits:
  - `769bbf7a9` (macos: reduce transient blank/scaled frames during resize)
  - `9efcdfdf8` (macos: keep top-left gravity for stale-frame replay)
- Files:
  - `pkg/macos/animation.zig`
  - `src/Surface.zig`
  - `src/apprt/embedded.zig`
  - `src/renderer/Metal.zig`
  - `src/renderer/generic.zig`
  - `src/renderer/metal/IOSurfaceLayer.zig`
- Summary:
  - Replays the last rendered frame during resize and keeps its geometry anchored correctly.
  - Reduces transient blank or scaled frames while a macOS window is being resized.

### 5) zsh prompt redraw markers use OSC 133 P

- Commit: `8ade43ce5` (zsh: use OSC 133 P for prompt redraws)
- Files:
  - `src/shell-integration/zsh/ghostty-integration`
- Summary:
  - Emits one `OSC 133;A` fresh-prompt mark for real prompt transitions.
  - Uses `OSC 133;P` markers for prompt redraws so async zsh themes do not look like extra prompt lines.

### 6) zsh Pure-style multiline prompt redraws

- Commits:
  - `0cf559581` (zsh: fix Pure-style multiline prompt redraws)
  - `312c7b23a` (zsh: avoid extra Pure continuation markers)
  - `404a3f175` (Fix Pure prompt redraw markers)
- Files:
  - `src/shell-integration/zsh/ghostty-integration`
- Summary:
  - Handles multiline prompts that use `\n%{\r%}` to return to column 0 before the visible prompt line.
  - Keeps redraw-safe prompt-start markers for async themes.
  - Avoids inserting an explicit continuation marker after Pure's hidden carriage return, because Ghostty already tracks the newline as prompt continuation and the extra marker duplicates the preprompt row.
  - Restores that prompt-marker behavior on top of the current Ghostty `main` base after the older redraw fix drifted out during later submodule updates.

The fork branch HEAD is now the section 6 zsh redraw follow-up commit.

### 7) cmux theme picker helper hooks

- Commit: `0c52c987b` (Add cmux theme picker helper hooks)
- Files:
  - `build.zig`
  - `src/cli/list_themes.zig`
  - `src/main_ghostty.zig`
- Summary:
  - Adds a `zig build cli-helper` step so cmux can bundle Ghostty's CLI helper binary on macOS.
  - Lets `+list-themes` switch into a cmux-managed mode via env vars, writing the cmux theme override file and posting the existing cmux reload notification for live app-wide preview.
  - Fixes the helper-only `app-runtime=none` stdout path so the Ghostty CLI binary builds with the current Zig toolchain.

The fork branch HEAD is now the section 7 cmux theme picker helper commit.

### 8) GHOSTTY_PLATFORM_GTK4 — Linux GTK4 embedded variant

- Files:
  - `include/ghostty.h` (at repo root: `ghostty.h`)
  - `src/apprt/embedded.zig`
  - `src/renderer/OpenGL.zig`
- Summary:
  - Adds `GHOSTTY_PLATFORM_GTK4 = 3` to `ghostty_platform_e` enum in ghostty.h.
  - Adds `ghostty_platform_gtk4_s` struct (single `void* gl_area` field) and `gtk4` member to `ghostty_platform_u` union.
  - Adds `gtk4 = 3` to `PlatformTag` enum in embedded.zig.
  - Adds `GTK4` type (linux-only, `gl_area: *anyopaque`) and `gtk4` arm to the `Platform` union.
  - Adds `gtk4` member to `Platform.C` extern union and `gtk4` arm to `Platform.init()`.
  - Replaces the "strictly broken" TODO stub in `OpenGL.zig surfaceInit` with a GTK4 dispatch: calls `prepareContext(null)` when the embedded surface's platform is GTK4 (same path as the GTK apprt).
  - Adds no-op GTK4 arms in `threadEnter` and `threadExit` (GtkGLArea manages context lifecycle on the Rust side).
- Design notes:
  - `void* gl_area` avoids requiring GTK4 headers in ghostty.h — consistent with `void* nsview` (macOS) pattern.
  - `GdkGLContext` is NOT passed; GTK4 manages it internally via `GtkGLArea`. Rust side calls `gl_area.make_current()` in the realize signal handler.
  - `wakeup_cb` and `userdata` are registered globally via `ghostty_runtime_config_s`, not per-surface.
  - `GTK4` type is `void` on non-Linux targets so the type compiles cross-platform.

### 9) Embedded display realize/unrealize C API

- Files:
  - `src/apprt/embedded.zig`
  - `src/renderer/OpenGL.zig`
- Summary:
  - Adds `ghostty_surface_display_realized` / `ghostty_surface_display_unrealized`
    exports plus the `Surface.displayRealized` / `Surface.displayUnrealized`
    methods they wrap. These forward to the core renderer's existing
    `displayRealized` / `displayUnrealized`.
  - Adds `apprt.embedded` to the `displayRealized` switch in `OpenGL.zig`, which
    previously `@compileError`'d for any runtime other than `apprt.gtk`. The
    embedded runtime needs the same `prepareContext(null)` GLAD reload because it
    renders into a caller-owned `GtkGLArea` exactly like the gtk apprt.
- Why this is required:
  - `src/ghostty/surface.rs` calls both functions when a `GtkGLArea` is
    re-realized after a reparent (split/move), which recreates the GL context.
    Without them cmux does not link at all, and the declarations were already
    present in the repo-root `ghostty.h`.
- Contract:
  - The caller must have made its GL context current before calling either, and
    each `display_unrealized` must be paired with a later `display_realized`.
    `displayRealized` asserts the swap chain was left defunct by a preceding
    `displayUnrealized`.

## Non-ghostty consequences of the fork bump

Bumping off the lost `4845e82d` pin changed what libghostty provides, which
required matching changes in the parent repo. If you bump the fork again,
re-check these:

- Ghostty now vendors real glslang, SPIRV-Cross and Dear ImGui. The old
  `stubs.c` (deleted) fake-implemented all three and now collides with them
  (~87 duplicate symbols).
- The embedded static library installs as `ghostty-internal.a`, not
  `libghostty.a`. `scripts/setup-linux.sh` symlinks the conventional name so
  `-lghostty` resolves.
- `simdutf.o` and `libhighway_zcu.o` are members of the archive now, so the old
  `.zig-cache` hash-scanning link args in `build.rs` were removed.
- The archive does not carry libc++/libc++abi (the vendored C++ is compiled
  against Zig's libc++, not the system libstdc++) or libxml2 (needed by the
  vendored fontconfig). `setup-linux.sh` copies all three out of the Zig caches
  into `ghostty/zig-out/lib` and `build.rs` links them statically.
- Ghostty pins an exact Zig version (`minimum_zig_version` in `build.zig.zon`)
  and `@compileError`s on anything else, so `setup-linux.sh` provisions that
  exact Zig rather than trusting the distro's.

## Upstreamed fork changes

### cursor-click-to-move respects OSC 133 click-to-move

- Was local in the fork as `10a585754`.
- Landed upstream as `bb646926f`, so it is no longer carried as a fork-only patch.

## Merge conflict notes

These files change frequently upstream; be careful when rebasing the fork:

- `src/terminal/osc/parsers.zig`
  - Upstream uses `std.testing.refAllDecls(@This())` in `test {}`.
  - Ensure `iterm2` import stays, and keep `kitty_notification` import added by us.

- `src/terminal/osc.zig`
  - OSC dispatch logic moves often. Re-check the integration points for the OSC 99 parser.

- `src/shell-integration/zsh/ghostty-integration`
  - Prompt marker handling is easy to regress when upstream adjusts zsh redraw behavior. Keep the
    `OSC 133;A` vs `OSC 133;P` split intact for redraw-heavy themes. Pure-style `\n%{\r%}`
    prompt newlines should not get an extra explicit continuation marker after the hidden CR.

- `src/cli/list_themes.zig`
  - cmux now relies on the upstream picker UI plus local env-driven hooks for live preview and restore.
    If upstream reorganizes the preview loop or key handling, re-check the cmux mode path and keep the
    stock Ghostty behavior unchanged when the cmux env vars are absent.

- `src/apprt/embedded.zig` (Platform/PlatformTag)
  - `ghostty_platform_e` enum will conflict if upstream adds new platform values at position 3.
  - `Platform` union and `PlatformTag` enum conflict on any change to these types.

- `src/renderer/OpenGL.zig` (surfaceInit/threadEnter/threadExit)
  - The embedded arm switch blocks are conflict-prone during upstream renderer updates.
  - Re-check GTK4 dispatch if upstream changes the apprt.embedded arm structure.

If you resolve a conflict, update this doc with what changed.
