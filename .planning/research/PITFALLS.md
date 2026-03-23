# Pitfalls Research

**Domain:** Linux terminal multiplexer — Rust + GTK4 + libghostty C API port of a macOS app
**Researched:** 2026-03-23
**Confidence:** MEDIUM-HIGH (drawn from macOS codebase evidence, ghostty.h API analysis, known Linux GTK4/Rust patterns, and first-hand macOS issue history)

---

## Critical Pitfalls

### Pitfall 1: Calling Ghostty C API Functions Off the Main Thread

**What goes wrong:**
`ghostty_surface_key()`, `ghostty_surface_set_focus()`, `ghostty_surface_set_size()`, `ghostty_app_tick()`, and virtually all surface-mutation functions are not thread-safe. Calling them from a tokio worker thread, a socket handler thread, or a GTK signal callback running on a GLib worker thread causes use-after-free crashes, corrupted terminal state, and silent data races. The macOS codebase enforces this via `@MainActor` on nearly every caller; Rust has no equivalent enforcement — it is a manual discipline problem.

**Why it happens:**
Tokio's async runtime and GTK's GLib main loop are separate event loops. It is tempting to call Ghostty functions directly from a `tokio::spawn` task when handling socket commands ("it's just a function call"). The bug is silent at first because races are non-deterministic.

**How to avoid:**
- Establish a single-threaded "main surface thread" rule at project start: all `ghostty_*` calls go through one dedicated thread (the GTK main thread).
- Socket command handlers must parse and validate on the tokio thread, then send the mutation as a message to the GTK main thread (via `glib::MainContext::default().spawn_local()` or a channel).
- Write a thin Rust wrapper type (`GhosttySurface`) that is `!Send` to get compiler enforcement that it cannot cross thread boundaries.
- Mirror the macOS `SocketCommandPolicy` pattern: socket commands that are not explicit focus-intent commands must not mutate focus state even on the main thread.

**Warning signs:**
- Intermittent segfaults in `ghostty_app_tick` or `ghostty_surface_draw` under socket command load
- Terminal corruption only visible during concurrent CLI usage
- Rust async code that holds a surface pointer across an `await` point

**Phase to address:** Phase 1 (project scaffolding). The threading model must be locked in before any surface code is written. Retrofitting is expensive.

---

### Pitfall 2: Not Implementing the Wakeup Callback Correctly on Linux

**What goes wrong:**
Ghostty's internal event loop calls `wakeup_cb` from an internal background thread to signal that `ghostty_app_tick()` should be called. The macOS implementation uses `DispatchQueue.main.async { GhosttyApp.shared.tick() }` — a fire-and-forget dispatch to the main thread. If the Linux implementation instead calls `ghostty_app_tick()` directly inside the callback (wrong thread) or sets up a polling loop instead of an event-driven callback, input latency explodes or the app crashes.

**Why it happens:**
The callback's threading contract is undocumented in `ghostty.h` ("The documentation for the embedding API is only within the Zig source files"). Developers assume the callback is already on the right thread, or they miss that tick must happen on the same thread as surface creation.

**How to avoid:**
- In `ghostty_runtime_config_s`, set `wakeup_cb` to a function that calls `glib::idle_add_once(|| ghostty_app_tick(app))` (schedules tick on the GLib main loop, not inline).
- Never call any `ghostty_*` function from within the wakeup callback body itself.
- Add a coalescing flag: if a wakeup is already pending, skip scheduling another. The macOS code does NOT do this but the `DispatchQueue` coalesces naturally; GTK `idle_add` does not.

**Warning signs:**
- Typing feels fine but scrolling is janky — tick is happening but too infrequently
- App freezes until the mouse moves — wakeup never fires, only mouse events trigger redraws
- Crash in tick with "called from wrong thread" assertions in Zig

**Phase to address:** Phase 1 (single-surface proof of concept). This is the first integration point with libghostty and must be proven before any multi-pane work.

---

### Pitfall 3: Focus Routing Breaks Keyboard Input to the Correct Terminal Surface

**What goes wrong:**
With multiple terminal surfaces in split panes, GTK4 keyboard events arrive at the window level. If the focus routing logic routes `GdkEvent` to the wrong `GtkWidget` — or worse, no widget — keystrokes are silently dropped. The macOS codebase has a dedicated `hitTest()` path in `TerminalWindowPortal` that explicitly identifies the target surface from pointer/key event coordinates. GTK4's default focus model is widget-centric, not coordinate-based, and does not automatically know which terminal pane should receive keyboard input.

**Why it happens:**
In a single-pane app, keyboard focus just follows the window. In a multi-pane layout with GPU-rendered surfaces embedded inside GtkWidget containers, GTK's focus chain may not match the visual "active pane" concept. Developers often get the first pane working and don't test pane switching + keyboard input together.

**How to avoid:**
- Maintain an explicit "focused surface" reference in the app state, updated on pane selection.
- Override the GTK window's `key_pressed` handler (or use a `GtkEventController`) to forward key events to `ghostty_surface_key()` on the currently focused surface — do not rely on GTK widget focus chain to do this routing.
- When switching panes, call `ghostty_surface_set_focus(old_surface, false)` before calling `ghostty_surface_set_focus(new_surface, true)`. Skipping the defocus call leaves Ghostty's internal cursor blinking and selection state inconsistent.
- Gate the focus call: the macOS CLAUDE.md notes a 50ms debounce (`lastFocusRefreshAt`) on redundant focus refreshes to prevent thrashing.

**Warning signs:**
- Typing in pane B after clicking pane A still sends input to pane A
- Ctrl+C doesn't interrupt the process in the newly focused pane
- Cursor continues blinking in the old pane after switching

**Phase to address:** Phase 2 (multi-pane splitting). Cannot defer — it is the definition of multi-pane correctness.

---

### Pitfall 4: Keyboard Input Latency from GTK Event Loop Overhead

**What goes wrong:**
Every keystroke that passes through GTK4's event processing pipeline (GdkEvent → GtkEventController → signal emission → Rust handler → C FFI → Ghostty) adds latency. On Linux with Wayland, the compositor adds another frame's worth of round-trip. Introducing allocations, channel sends, Mutex locks, or GTK re-layout in the key event path produces visible typing lag (>16ms per keystroke at 60Hz feels sluggish; >30ms is noticeable to most users).

**Why it happens:**
The path looks innocuous: "just a function call through a few layers." The macOS CLAUDE.md warns explicitly about this and lists three specific "typing-latency-sensitive paths" — the Linux port has the same fundamental constraint with even less platform help (GTK4 is not as latency-optimized for embedded GPU rendering as Metal/IOSurface).

**How to avoid:**
- In the key event handler: parse the event, call `ghostty_surface_key()`, return immediately. No channel sends, no mutex acquisition, no GTK widget state queries.
- Do NOT trigger GTK layout passes from the key handler. The macOS port gates all divider/sidebar routing to pointer events only (`isPointerEvent` guard in `hitTest()`).
- Do NOT call `ghostty_surface_draw()` directly from the key handler — let Ghostty's renderer wakeup drive redraws asynchronously.
- Measure latency with `std::time::Instant` probes on the keystroke-to-tick path in debug builds. The macOS debug log tracks `typing.delay` and `typing.timing` explicitly.
- Profile with `perf record` on Linux to verify no unexpected work is happening per keystroke before shipping.

**Warning signs:**
- Visible delay between keypress and character appearing in terminal
- `perf` or `strace` shows allocations or syscalls per keystroke
- Any async `await` in the key event path

**Phase to address:** Phase 1 (single-surface proof of concept). Measure baseline latency before adding panes. Regressions are hard to detect once the codebase grows.

---

### Pitfall 5: GTK4 Widget Lifecycle and Ghostty Surface Lifetime Mismatch

**What goes wrong:**
GTK4 widgets are reference-counted (GObject). Ghostty surfaces are manually managed with `ghostty_surface_new` / `ghostty_surface_free`. If a `GtkWidget` is dropped (ref count reaches zero) while a Ghostty surface is still rendering to it, or if `ghostty_surface_free` is called while a pending GTK idle callback still holds a raw surface pointer, the result is a use-after-free. The macOS CLAUDE.md documents this exact class of bug: "Re-read self.surface before each ghostty call to guard against the surface being freed during wake-from-sleep geometry reconciliation."

**Why it happens:**
Two independent lifetime systems (GObject ref counting vs. manual C memory) must be kept in sync, but there is no compiler enforcement. Rust's ownership system helps within Rust code but does not protect raw `ghostty_surface_t` pointers passed into GLib closures.

**How to avoid:**
- Wrap `ghostty_surface_t` in a Rust struct with a `Drop` impl that calls `ghostty_surface_free`. Make the struct `!Copy` so surfaces cannot be silently duplicated.
- Never store a raw `ghostty_surface_t` in a GLib closure (e.g., `g_idle_add` lambda). Store it only in the owning Rust struct; closures should hold a weak reference to the owner and re-borrow the surface pointer inside the closure body, checking for None.
- When a pane is closed: (1) call `ghostty_surface_request_close()`, (2) wait for the `close_surface_cb` callback, (3) only then drop the owning Rust struct and call `ghostty_surface_free`. Do not reverse this order.
- Implement the `close_surface_cb` in `ghostty_runtime_config_s` — if this callback is left null or unimplemented, Ghostty cannot signal to the host that a surface should be freed.

**Warning signs:**
- Segfault in `ghostty_surface_draw` only after closing a pane
- Surface renders into a blank/black rectangle after a split close
- Valgrind or AddressSanitizer reports use-after-free in libghostty

**Phase to address:** Phase 1 (surface lifecycle fundamentals) and Phase 2 (split close/reopen stress testing).

---

### Pitfall 6: gtk4-rs Reference Cycles Causing Memory Leaks

**What goes wrong:**
gtk4-rs uses `glib::clone!` macros to capture `glib::WeakRef` or strong `glib::Object` references in signal closures. If a widget holds a strong reference to a closure, and the closure holds a strong reference back to the widget (or to its parent), neither is ever freed. Terminal multiplexers are especially prone to this because panes, workspaces, and the main window all hold references to each other for layout and focus routing. Long-running sessions accumulate leaked workspaces.

**Why it happens:**
The gtk4-rs book recommends `glib::clone!(@weak self => ...)` patterns but it is easy to accidentally capture `@strong` references (or just raw clones of `glib::Object` subtypes, which are strong by default). There is no compile-time warning for cycles.

**How to avoid:**
- Default to `@weak` captures in all signal closures. Only use `@strong` when there is an explicit reason and that reason is documented in a comment.
- Run the application under Valgrind's massif heap profiler after a session of opening/closing 20+ workspaces to check for leaks before any release.
- In Rust, `Rc<RefCell<T>>` for pane/workspace state creates similar cycle risks — prefer an arena or ID-based lookup table (similar to how the macOS code uses UUID-based short IDs) over nested `Rc` trees.

**Warning signs:**
- RSS memory grows monotonically as workspaces are created and closed
- RAII Drop impls for workspace state are never called (add logging to verify)
- `glib::Object::ref_count()` shows unexpected counts on closed workspaces

**Phase to address:** Phase 2 (workspace/tab management). Establish a leak test before multi-workspace support is "done."

---

### Pitfall 7: Ghostty Surface Rendering Artifacts During Split Resize

**What goes wrong:**
When pane dividers are dragged, the terminal surface must be resized via `ghostty_surface_set_size()`. If the size update races with an in-progress frame render, or if the size update is not followed by `ghostty_surface_refresh()`, the terminal displays a stretched/blank frame for a visible duration. The macOS fork has two dedicated commits specifically addressing this ("macos: reduce transient blank/scaled frames during resize," "macos: keep top-left gravity for stale-frame replay") — these are macOS/Metal-specific but the problem class exists on Linux too.

**Why it happens:**
GPU-rendered surfaces have a pipeline: set size → wait for GPU to reallocate → first frame at new size renders. The host application sees the new size immediately but the GPU has not caught up. On Linux with OpenGL/Vulkan, this manifests as a black or stretched frame during resize.

**How to avoid:**
- After calling `ghostty_surface_set_size()`, immediately call `ghostty_surface_refresh()` to trigger a synchronous size acknowledgment.
- During live resize (user dragging a divider), coalesce size updates: do not call `ghostty_surface_set_size()` more often than once per frame (16ms at 60Hz). Batch the final size update on drag completion.
- Do not remove the coalescing/throttle logic once it is in place, even if it seems unnecessary in testing — resize artifacts are hardware-dependent.

**Warning signs:**
- Black flashes in terminal panes during divider drag
- Terminal content appears scaled/stretched while resizing
- Resize works on one GPU vendor but not another

**Phase to address:** Phase 3 (pane splitting and divider drag).

---

### Pitfall 8: Session Persistence Corruption on Partial Write / Crash Mid-Save

**What goes wrong:**
If the session JSON file (`~/.config/cmux/session.json` or equivalent) is written non-atomically (open → truncate → write → close), a crash during the write phase leaves a partially-written JSON file. On next launch, JSON parsing fails and the session appears empty — all workspaces are lost. This is a data-loss bug that users will report loudly.

**Why it happens:**
Naive file writes in Rust (`std::fs::write()`) truncate before writing, creating a window where the file is empty. Developers test the happy path (write completes successfully) but not the crash path.

**How to avoid:**
- Always write session state atomically: write to a `.tmp` file in the same directory, then `rename()` to the final path. On Linux, `rename()` is atomic within the same filesystem.
- Validate that the JSON parses correctly before committing the rename (serialize → parse round-trip check).
- Keep the previous session file as `.session.json.bak` and fall back to it on parse failure.
- The macOS code uses a debounced save (triggered on state changes) — preserve this pattern in Rust to avoid write storms during rapid workspace creation.

**Warning signs:**
- Empty or truncated `session.json` on disk
- Any `std::fs::write()` call to the session file (replace with atomic write pattern)
- Tests that only test successful restore, not restore-after-corrupt

**Phase to address:** Phase 4 (session persistence). Atomic writes must be a success criterion for the phase.

---

### Pitfall 9: Socket Protocol Drift Between macOS and Linux

**What goes wrong:**
The v2 JSON-RPC protocol must be wire-compatible across both platforms so shared tooling (CLI scripts, agents, tests) works on both. If the Linux port silently changes field names, error code strings, response shapes, or the ordering of IDs in list responses, cross-platform scripts break. The macOS codebase already has an established `tests_v2/` Python test suite that validates protocol behavior — if these tests are not run against the Linux socket server, drift accumulates invisibly.

**Why it happens:**
The protocol is defined implicitly in Swift handler code, not in a schema file. Porting to Rust means reimplementing from reading the macOS source and making assumptions about intent. Edge cases (empty workspace lists, null fields, error codes for "not_found" vs "invalid_args") are easy to get wrong.

**How to avoid:**
- Define the v2 protocol in a machine-readable schema (JSON Schema or similar) before implementing the Rust server, derived from reading the macOS handlers + `tests_v2/` assertions.
- Port the macOS `tests_v2/` Python test suite to run against the Linux socket server as-is — it should pass without modification. If it does not pass, that is a protocol incompatibility to fix.
- Pay special attention to: short ID format (`surface:abc123`, `pane:abc123`), error code strings (`"not_found"`, `"invalid_args"`), the newline-delimited framing protocol, and the `"ok": true/false` response envelope.
- The `CMUX_WORKSPACE_ID` environment variable for workspace-relative commands must behave identically on Linux.

**Warning signs:**
- Tests that pass when run against macOS cmux but fail against Linux cmux
- Field name differences (e.g., `workspace_id` vs `workspaceId`) in responses
- Any place in Rust code where a JSON field name is hardcoded without a cross-reference to the macOS source

**Phase to address:** Phase 4 (socket server). Protocol compatibility should be a gate before the phase is called done.

---

### Pitfall 10: Wayland vs X11 Input Handling Divergence

**What goes wrong:**
On Wayland, GTK4 uses `libxkbcommon` for keyboard input; on X11 (via XWayland), it uses a different path. Key event structs, modifier handling, compose key behavior, and IME (input method editor) preedit flow differ between backends. The `ghostty_surface_preedit()` call for IME must be wired correctly for both paths, or CJK input (Japanese, Chinese, Korean) is completely broken. Additionally, clipboard access semantics differ: Wayland has no global clipboard without focus.

**Why it happens:**
Developers test on one backend (usually X11 or a specific Wayland compositor like GNOME/Mutter) and do not test the other. Wayland/X11 divergence in GTK4 is partially abstracted but not fully — surface-level rendering and focus behavior differ at the GTK level.

**How to avoid:**
- Test on both Wayland (GNOME/Mutter) and X11 (plain Xorg or XWayland) before any phase is called complete.
- Wire `ghostty_surface_preedit()` from the GTK `InputMethod` API from day one — do not defer IME support.
- For clipboard: GTK4's `gdk_clipboard_read_async()` API handles Wayland's focus requirement; do not bypass it with raw X11 clipboard reads.
- Test modifier key behavior: Super, Alt, Ctrl must all pass correctly to `ghostty_surface_key()` with the correct `ghostty_input_mods_e` bitmask.

**Warning signs:**
- Keyboard shortcuts work under GNOME/Wayland but not under i3/XWayland
- Compose key sequences produce wrong characters
- Paste doesn't work when terminal is not focused (Wayland selection clipboard limitation)

**Phase to address:** Phase 1 (basic input), verified again in Phase 5 (packaging/distribution).

---

### Pitfall 11: Focus Steal from Socket Commands

**What goes wrong:**
Socket commands (e.g., `workspace.select`, `new_split`, `send`) must not cause the application to steal window focus from whatever the user is currently doing. On macOS, this required a dedicated `socketCommandPolicy` depth stack and explicit gates on every handler — it took significant effort to audit and fix (see `socket-focus-steal-audit.todo.md`, which lists 70+ commands). On Linux, the equivalent risk is that GTK window `present()` calls triggered by socket commands raise the window, and `ghostty_surface_set_focus()` calls triggered by non-focus-intent commands move input focus unexpectedly.

**Why it happens:**
The intuitive implementation of "switch to workspace X" via socket is to call `gtk_window_present()` and `ghostty_surface_set_focus(new_surface, true)`. This is correct for explicit focus commands but wrong for informational or automation commands (e.g., `workspace.list`, `send`, `new_split` for a background agent workspace).

**How to avoid:**
- Port the macOS `socketCommandPolicy` concept to Rust from the start: every socket command handler must be tagged as "focus-intent" or "not focus-intent."
- Non-focus-intent commands: never call `gtk_window_present()`, never call `ghostty_surface_set_focus()`.
- Focus-intent allowlist (same as macOS): `window.focus`, `workspace.select`, `workspace.next`, `workspace.previous`, `workspace.last`, `surface.focus`, `pane.focus`.
- The `CMUX_WORKSPACE_ID` env-var-relative command targeting is specifically designed to let background agents work without touching the user's active workspace — implement this correctly on Linux.

**Warning signs:**
- Terminal window jumps to focus when an agent sends a socket command in the background
- Active pane changes unexpectedly during background workspace operations
- Tests show workspace selection side-effects from non-focus commands

**Phase to address:** Phase 4 (socket server implementation). This is a policy decision that must be built in, not patched on.

---

### Pitfall 12: macOS-Specific Assumptions Ported to Linux

**What goes wrong:**
The macOS codebase contains several platform-specific assumptions that are easy to accidentally port:

1. **`ghostty_surface_set_display_id()`** — macOS only (CGDisplay). This function is guarded `#ifdef __APPLE__` in `ghostty.h`. Calling it on Linux is a compile error or a no-op depending on the build, but trying to replicate its behavior (restart vsync after display topology change) has a different solution on Linux (monitor hotplug via `GdkMonitor` signals).

2. **IOSurface / Metal rendering** — The macOS port shares surfaces between Ghostty renderer and AppKit via `IOSurface`. Linux uses OpenGL or Vulkan. Do not attempt to replicate the `IOSurface` sharing pattern; use GTK's `GtkGLArea` or equivalent.

3. **`~/Library/Application Support/`** — Session/config paths must be `~/.config/cmux/` on Linux following XDG Base Directory spec, not `~/Library/...`.

4. **`UserDefaults`** — macOS settings storage. Linux equivalent is either a simple TOML/JSON config file in `~/.config/cmux/config.toml` or `gsettings`. Do not use `UserDefaults`-equivalent "registry-style" APIs.

5. **Sparkle auto-update** — Already scoped out, but ensure no Sparkle API calls leak into shared code paths.

6. **HiDPI content scale** — macOS uses `backingScaleFactor` (2.0 for Retina). Linux uses `ghostty_surface_set_content_scale()` driven by `GdkSurface`'s scale factor, which may be 1.0, 1.5, 2.0, or fractional. Hardcoding 2.0 breaks non-HiDPI Linux displays.

**Why it happens:**
Developers look at macOS code for reference and port patterns without checking whether the underlying API exists on Linux.

**How to avoid:**
- Flag every macOS API call in the Swift codebase with a "Linux equivalent" note during the porting planning phase.
- Use `dirs` crate in Rust for XDG-compliant path resolution.
- Drive content scale from `gtk4::Widget::scale_factor()` and `gdk4::Surface::scale()` — do not hardcode.
- Test on a non-HiDPI (1.0 scale) display before every phase completion.

**Warning signs:**
- Hardcoded `/home/user/.config` paths (use XDG crate instead)
- Hardcoded `content_scale = 2.0`
- Any `~/Library` path reference in Rust code

**Phase to address:** Phase 1 (scaffolding). Path handling and content scale must be correct from the first working build.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Single global `Mutex<GhosttyApp>` for all surface access | Easy to reason about concurrency | Per-keystroke lock contention kills latency; deadlock risk when GTK callbacks re-enter | Never — use message passing instead |
| Storing `ghostty_surface_t` as `*mut c_void` in Arc | Compiles with zero-cost | Loses `!Send` enforcement; surfaces will cross threads accidentally | Never — wrap in `!Send` struct |
| Calling `gtk_window_present()` in all workspace switches | Correct for explicit navigation | Focus steal in background agent workflows | Only in explicit focus-intent commands |
| Porting macOS Bonsplit Swift library verbatim using FFI | Skip Rust rewrite | Pulling in a Swift dylib on Linux is impractical (no Swift runtime guarantee) | Never — rewrite in Rust |
| Polling `ghostty_surface_process_exited()` in a loop | Simple child exit detection | Burns CPU; adds latency vs. callback-driven approach | Never — use close_surface_cb |
| Writing session JSON with `std::fs::write()` | One-liner | Data loss on crash during write | Never — always use atomic rename |
| Using `std::sync::Mutex` in GTK signal handlers | Simple | GTK signals can re-enter, causing deadlock on same-thread mutex | Never — use `RefCell` in GTK context |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| libghostty C API | Calling `ghostty_app_tick()` from wakeup_cb directly | Schedule via `glib::idle_add_once()` back to GTK main thread |
| libghostty C API | Forgetting `ghostty_config_finalize()` before `ghostty_app_new()` | Always call `finalize` — config values are not applied until then |
| libghostty C API | Calling `ghostty_surface_free()` before `close_surface_cb` fires | Wait for the callback; the callback is Ghostty's signal that free is safe |
| gtk4-rs signals | Using strong captures in all closures | Default to `@weak` / `Weak::clone()` to avoid reference cycles |
| gtk4-rs / GLib | Calling GTK methods from tokio threads | All GTK calls must be on the GLib main thread; use `glib::MainContext::spawn_local()` |
| Wayland clipboard | Reading clipboard without focus | Use `gdk_clipboard_read_async()`; warn users that selection clipboard requires focus |
| tokio + GTK | Running tokio on the GTK main thread | Run tokio runtime on a separate OS thread; communicate back via GLib channel |
| Session JSON | Parsing user-modified JSON | Handle `serde_json` errors gracefully; fall back to empty session, do not crash |
| v2 socket protocol | Implementing new error codes | Cross-check all error code strings against `tests_v2/` expectations |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| GTK layout triggered by key events | Typing lag > 16ms | Never mutate widget sizes/visibility in key handler | Immediately, every keystroke |
| Channel send in key event path | Typing lag spikes under load | Key handler calls ghostty_surface_key() directly, no channel | Under socket command load (>10 concurrent commands) |
| `ghostty_surface_set_size()` on every mouse-move during resize | Black frames, high CPU | Throttle to 1 update per frame (16ms) during live resize | Any divider drag operation |
| Spawning OS threads per socket client | Thread exhaustion at ~128 concurrent connections | Use tokio async I/O with bounded task pool | >50 concurrent socket connections |
| Re-evaluating full pane tree layout on every state change | UI jank during workspace switches | Diff-based layout updates; only re-layout changed subtrees | Workspaces with >8 panes |
| `ghostty_app_tick()` called more than once per wakeup | Extra CPU usage, no visual benefit | Coalesce wakeups with a pending flag | High terminal output rate (e.g., `cat /dev/urandom`) |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Socket created with world-readable permissions (0666) | Any local user can control terminal | Create socket with 0600; validate peer uid via `SO_PEERCRED` |
| Socket path in `/tmp/` without user-specific subdirectory | Symlink attack, socket hijacking | Use `$XDG_RUNTIME_DIR/cmux/cmux.sock` (user-private, tmpfs) |
| Accepting socket commands without uid validation | Root-owned process can send commands | Check `SO_PEERCRED` uid matches process owner on every accept |
| Session JSON readable by other users | Workspace commands + working dirs leaked | Write session file with 0600 permissions |
| Ghostty fork diverging from upstream security patches | Terminal escape injection, CVE exposure | Maintain quarterly upstream sync schedule; subscribe to Ghostty advisories |

---

## "Looks Done But Isn't" Checklist

- [ ] **Single-surface keyboard input:** Verify input reaches terminal when GTK window has no explicit widget focus set — GTK4 does not default to routing key events to embedded GL surfaces.
- [ ] **Pane switching focus:** After switching panes, verify the OLD surface received `ghostty_surface_set_focus(false)` and the NEW surface received `ghostty_surface_set_focus(true)` — check both.
- [ ] **Session restore:** Verify session restores correctly after a crash (partially written session file), not just after a clean exit.
- [ ] **Socket protocol parity:** Run macOS `tests_v2/` Python suite against Linux socket server before calling socket protocol "done."
- [ ] **Content scale:** Test on a 1.0-scale (non-HiDPI) display. Many Linux machines are not HiDPI.
- [ ] **Workspace close:** Verify `ghostty_surface_free()` is called exactly once per closed pane — not zero times (leak) and not twice (use-after-free).
- [ ] **Socket focus steal:** Verify non-focus-intent commands do not change the active workspace or pane when called from a background agent.
- [ ] **XDG paths:** Verify config/session/socket paths follow XDG spec (`$XDG_CONFIG_HOME`, `$XDG_RUNTIME_DIR`) with fallback to `~/.config/cmux/`.
- [ ] **Atomic session writes:** Verify that killing the process mid-save does not corrupt the session file (test with `kill -9` during save).
- [ ] **Wayland + X11:** Test both backends before any phase is complete. Key handling diverges.

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Threading model wrong (Ghostty API called off main thread) | HIGH | Audit all `ghostty_*` call sites; add `!Send` wrapper type; refactor all socket handlers to message-passing pattern |
| wakeup_cb implemented incorrectly | LOW | Fix the callback registration in the runtime config; can be done without changing other code |
| Focus routing broken for multi-pane | MEDIUM | Add explicit focused-surface tracking in app state; rewrite key event handler to use it |
| Input latency regression | MEDIUM | Profile with `perf`; identify allocation/lock in hot path; remove it |
| Session file corruption discovered in production | MEDIUM | Add migration code to detect and ignore corrupt JSON; switch to atomic writes |
| Socket protocol drift found after release | HIGH | Schema-first redesign + version field in responses; run compatibility tests against both platforms |
| GTK reference cycles (memory leak) | MEDIUM | Audit all signal closures with Valgrind; replace strong captures with weak |
| Wayland/X11 input divergence found late | MEDIUM | Add conditional code path per GDK backend; test matrix both backends |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Ghostty API threading model | Phase 1: Scaffolding | `!Send` wrapper type compiles; no `ghostty_*` calls outside main thread under TSAN |
| Wakeup callback correctness | Phase 1: Single-surface PoC | Single surface renders and responds to wakeup without extra polling |
| Keyboard input latency | Phase 1: Single-surface PoC | Keystroke-to-render measured < 10ms in debug build; no allocations in hot path |
| Surface/widget lifetime mismatch | Phase 1 + Phase 2 | AddressSanitizer clean after 100 pane open/close cycles |
| Focus routing multi-pane | Phase 2: Multi-pane splitting | Each pane receives exactly its keystrokes after switching; defocus verified |
| Split resize rendering artifacts | Phase 3: Divider drag | No black frames during 60-second live resize test |
| GTK reference cycles | Phase 2: Workspace management | RSS stable after 50 workspace create/close cycles |
| macOS API assumptions | Phase 1: Scaffolding | XDG paths verified; content scale 1.0 tested; no `~/Library` paths |
| Session persistence corruption | Phase 4: Session persistence | Atomic write verified; `kill -9` during save leaves valid or previous backup |
| Socket protocol parity | Phase 4: Socket server | `tests_v2/` suite passes against Linux server without modification |
| Socket focus steal | Phase 4: Socket server | Non-focus commands verified to not change active workspace/pane |
| Wayland vs X11 divergence | Phase 1 (input) + Phase 5 (packaging) | Both backends tested; modifier keys correct on both |
| Linux packaging (Flatpak/AppImage) | Phase 5: Distribution | Flatpak bundle includes all deps; Wayland portal sandbox permission verified |

---

## Sources

- `/home/twilson/code/cmux-linux/ghostty.h` — libghostty C API, callback structs, threading contract (inferred from structure)
- `/home/twilson/code/cmux-linux/Sources/GhosttyTerminalView.swift` — macOS wakeup_cb implementation, forceRefresh pattern, latency probes
- `/home/twilson/code/cmux-linux/Sources/TerminalWindowPortal.swift` — hitTest focus routing, geometry sync, issue #483 portal recovery pattern
- `/home/twilson/code/cmux-linux/.planning/codebase/CONCERNS.md` — macOS tech debt, fragile areas, known bugs directly applicable to Linux port
- `/home/twilson/code/cmux-linux/docs/ghostty-fork.md` — fork changes revealing platform-specific surface rendering issues (display link restart, resize stale frames)
- `/home/twilson/code/cmux-linux/docs/socket-focus-steal-audit.todo.md` — complete focus steal audit; focus-intent allowlist
- `/home/twilson/code/cmux-linux/docs/v2-api-migration.md` — v2 protocol spec, framing, error format
- `/home/twilson/code/cmux-linux/CLAUDE.md` — typing-latency-sensitive paths, socket threading policy, socket focus policy
- `/home/twilson/code/cmux-linux/.planning/PROJECT.md` — port constraints and known technical unknowns

---
*Pitfalls research for: cmux Linux port (Rust + GTK4 + libghostty)*
*Researched: 2026-03-23*
