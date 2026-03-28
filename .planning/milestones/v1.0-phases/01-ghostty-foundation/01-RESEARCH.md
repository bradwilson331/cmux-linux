# Phase 1: Ghostty Foundation - Research

**Researched:** 2026-03-23
**Domain:** Ghostty libghostty C API fork extension, GTK4 GtkGLArea embedding, Rust FFI setup, GLib async event loop
**Confidence:** MEDIUM-HIGH (ghostty source read directly from submodule — HIGH; gtk4-rs versions from crates.io registry — HIGH; Zig build internals — MEDIUM)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01 — Repo layout:** Root-level `Cargo.toml` at repo root, with `src/main.rs` as the Rust entry point. The existing macOS Swift files (`Sources/`, etc.) remain at the top level alongside the Rust project. `cargo build` works from the repo root.

**D-02 — Phase 1 window scope:** A bare `GtkApplicationWindow` containing a single `GtkGLArea` hosting the Ghostty surface. No sidebar, no tab bar, no header bar chrome — just a working terminal window. Phase 2 adds the multiplexer UI structure on top of this foundation.

**D-03 — Ghostty fork extension strategy:** ~~Add `GHOSTTY_PLATFORM_GTK4` to `ghostty.h` with `ghostty_platform_gtk4_s` holding `GtkWidget *gl_area`, `GdkGLContext *gl_ctx`, and per-surface wakeup fields.~~ **REVISED (see CONTEXT.md D-03):** `ghostty_platform_gtk4_s` holds only `void* gl_area` (consistent with macOS `void* nsview` pattern). `GdkGLContext` is NOT passed — GTK4 manages it internally. `wakeup_cb` and `userdata` are registered globally via `ghostty_runtime_config_s`, not per-surface. Minimal fork diff; reuses existing `apprt/embedded.zig` infrastructure.

**D-04 — Wakeup callback dispatch:** `wakeup_cb` dispatches to GLib main loop via `glib::idle_add_once()` — never calls any `ghostty_*` API inline from Ghostty's thread (per GHOST-07).

**D-05 — libghostty linkage:** `scripts/setup.sh` pre-builds or downloads `libghostty.a` (static archive) and places it at `ghostty/zig-out/lib/libghostty.a`. `build.rs` then links it statically via `cargo:rustc-link-lib=static=ghostty`. Zig is NOT a Cargo build-time dependency.

**D-06 — Bindgen:** `bindgen` runs at build time in `build.rs` against `ghostty.h` to auto-generate `src/ghostty_sys.rs`. `libclang` must be available in CI. Bindings regenerate automatically when `ghostty.h` changes as the fork evolves.

**D-07 — Defer tokio to Phase 3:** Phase 1 has no socket server and no inter-thread async work. GLib main loop handles all async via `glib::spawn_future_local`. A `tokio` runtime is introduced in Phase 3 alongside the socket server.

**D-08 — Clipboard:** Clipboard callbacks from Ghostty (GHOST-05) bridge to GTK4 via `gdk4::Clipboard::read_text_future()` / `set_text()`, invoked from `glib::spawn_future_local`. GDK handles X11/Wayland divergence transparently.

**D-09 — Error handling:** If Ghostty surface initialization fails (GL context failure, PTY spawn error, bad config): print a clear diagnostic to stderr and exit with code 1. No GUI error dialog in Phase 1.

**D-10 — Phase 1 must not block Phase 3 socket integration:** Phase 1 code organization should make it straightforward to add a tokio-based socket server in Phase 3 — avoid tightly coupling app state to GLib in a way that makes off-main-thread command dispatch impossible.

### Claude's Discretion

- Exact Zig build target flags for producing `libghostty.a` on Linux (researcher to confirm)
- DPI/scale factor update mechanism when window moves between monitors — verify GTK4 signal name (`notify::scale-factor` on `GtkWidget`) and whether Ghostty requires an explicit `ghostty_surface_set_content_scale()` call
- Whether GTK4 requires a custom GDK surface backend or `GtkGLArea` is sufficient for Ghostty's OpenGL renderer

### Deferred Ideas (OUT OF SCOPE)

- In-terminal browser (BROW-01..03): v2 scope; GTK4/Rust WebKit embedding deferred
- Socket API (SOCK-01..06): Core project value prop, explicitly Phase 3 scope
- Tokio/GLib bridge validation: Deferred to Phase 3
- GTK4 error dialogs: Phase 1 uses stderr + exit code
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GHOST-01 | manaflow-ai/ghostty fork extended with `GHOSTTY_PLATFORM_GTK4` Linux platform variant | Fork source read; exact extension points identified; see §Ghostty Fork Extension |
| GHOST-02 | Single Ghostty terminal surface renders in a GTK4 `GtkGLArea` widget from Rust | GtkGLArea render/realize signals documented; OpenGL 4.3 requirement confirmed |
| GHOST-03 | Keyboard input routes to active terminal surface with < 10ms keystroke-to-render latency | Input event flow via `ghostty_surface_key()`; latency pitfalls documented |
| GHOST-04 | Mouse input (selection, scroll, click) routed to correct terminal surface | `ghostty_surface_mouse_button()`, `ghostty_surface_mouse_pos()`, `ghostty_surface_mouse_scroll()` confirmed in ghostty.h |
| GHOST-05 | Clipboard integration works on X11 and Wayland | GDK4 Clipboard API handles both; `gdk4::Clipboard::read_text_future()` / `set_text()` confirmed |
| GHOST-06 | Terminal renders at correct DPI driven from `gtk4::Widget::scale_factor()` | `getScaleFactor()` on GtkWidget confirmed; `ghostty_surface_set_content_scale()` call identified |
| GHOST-07 | `wakeup_cb` dispatches to GLib main loop, never calls `ghostty_*` inline from Ghostty's thread | Confirmed via embedded.zig analysis; `glib::idle_add_once` pattern verified |
</phase_requirements>

---

## Summary

Phase 1 is fundamentally a fork spike: the `ghostty.h` C API in the manaflow-ai fork (and upstream Ghostty) has no Linux platform variant. The `ghostty_platform_e` enum only has `GHOSTTY_PLATFORM_MACOS` and `GHOSTTY_PLATFORM_IOS`. To embed a Ghostty surface in a Rust GTK4 app, the fork must add `GHOSTTY_PLATFORM_GTK4` along with a new struct in the `ghostty_platform_u` union.

The key technical insight discovered by reading the actual Ghostty source (now that the submodule is initialized): the embedded apprt (`apprt/embedded.zig`) has an OpenGL renderer path that is **explicitly marked as broken** — the `OpenGL.zig` renderer says `"libghostty is strictly broken for rendering on this platform"` for the embedded apprt. The GTK apprt solves this by using `GtkGLArea` with a global OpenGL context loaded via GLAD. The fork extension (D-03) must therefore do more than just add a struct — it must wire `GtkGLArea`'s realize/render signals into Ghostty's OpenGL renderer init path in the embedded apprt. This is the primary technical risk of Phase 1.

The Rust scaffolding (Cargo.toml, build.rs with bindgen, GtkApplicationWindow with GtkGLArea) follows established patterns. The `ghostty_runtime_config_s` callback table (wakeup, action, clipboard, close) maps cleanly to Rust closures captured via `glib::spawn_future_local`. Phase 1 deliberately omits tokio (deferred to Phase 3) — GLib main loop handles all async work.

**Primary recommendation:** Phase 1 is a spike, not a feature sprint. The first wave must establish that the fork extension compiles and produces a rendered terminal frame. All other GHOST-0x requirements are validated atop that foundation.

---

## Standard Stack

### Core (Phase 1 only)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| gtk4 (crate) | 0.11.1 | GtkApplicationWindow, GtkGLArea, event controllers | Official gtk-rs binding; mandatory for Ghostty surface hosting |
| glib (crate) | 0.22.3 | GLib main loop, idle_add, spawn_future_local | Companion crate; GLib is GTK4's event loop |
| gdk4 (crate) | 0.11.x | Clipboard, GLContext, Monitor, scale factor | Companion crate; clipboard and display plumbing |
| bindgen (build dep) | 0.72.1 | Generate src/ghostty_sys.rs from ghostty.h | Automates 1,169-line header; re-runs on header change |

**Note on versions:** `gdk4` is bundled with the `gtk4` crate — pinning `gtk4 = "0.11"` brings in the matching `gdk4`. The `glib` crate at 0.22.x is the dependency of `gtk4 = "0.11"`. These versions are from the crates.io registry as of 2026-03-23 (verified via `cargo search`). The prior research noted 0.9.x — the registry shows 0.11.1 as current; use 0.11.

### Supporting (build/dev)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| cc (build dep) | 1.x | Emit link directives in build.rs | Needed for `cargo:rustc-link-lib=static=ghostty` |
| libclang (system) | 18.1.3 | bindgen runtime (already installed) | Required for bindgen to parse ghostty.h |
| Zig | 0.16.0-dev | Build libghostty.a from fork source | Already installed; ghostty min is 0.15.2 (dev is forward-compat) |

### Phase 1 does NOT include

No tokio, no serde, no socket code, no session persistence — all deferred to later phases per D-07.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| gtk4 crate | raw GTK4 bindgen | gtk4-rs provides type-safe wrappers and signal handling; raw FFI for 1,500+ functions is unworkable |
| bindgen | Manual `extern "C"` | ghostty.h is 1,169 lines with 100+ functions; manual maintenance is error-prone |
| GtkGLArea | GtkDrawingArea + EGL | GtkGLArea is purpose-built for OpenGL; Ghostty's GTK apprt already uses it |

**Installation:**
```toml
# Cargo.toml [dependencies]
gtk4 = "0.11"
glib = "0.22"

# Cargo.toml [build-dependencies]
bindgen = "0.72"
cc = "1"
```

---

## Ghostty Fork Extension — Critical Spike

This is the highest-risk item in Phase 1. The following is based on reading the actual manaflow-ai/ghostty submodule source.

### Current State of embedded.zig

The `Platform` union in `ghostty/src/apprt/embedded.zig` (at commit `bc9be90`) contains:

```zig
pub const Platform = union(PlatformTag) {
    macos: MacOS,
    ios: IOS,
};

pub const PlatformTag = enum(c_int) {
    macos = 1,
    ios = 2,
};
```

The C ABI union exposes `macos` (with `?*anyopaque nsview`) and `ios` (with `?*anyopaque uiview`).

### OpenGL Rendering Blocker

The `OpenGL.zig` renderer (`ghostty/src/renderer/OpenGL.zig`) has this explicit comment at the `surfaceInit` and `threadEnter` entry points:

```zig
apprt.embedded => {
    // TODO(mitchellh): this does nothing today to allow libghostty
    // to compile for OpenGL targets but libghostty is strictly
    // broken for rendering on this platforms.
},
```

The GTK apprt (`apprt.gtk`) works because it loads the OpenGL context with:
```zig
apprt.gtk => try prepareContext(null),  // GTK uses global OpenGL context, load from null
```

**Implication for D-03:** Adding `GHOSTTY_PLATFORM_GTK4` to the platform enum is not sufficient. The Zig fork also needs to:
1. Add a `gtk4` arm to `Platform`, `PlatformTag`, and `Platform.C` in `embedded.zig`
2. Add a `gtk4` arm in `OpenGL.zig`'s `surfaceInit`, `threadEnter`, `threadExit` switch blocks that calls `prepareContext(null)` (same as the GTK apprt does)
3. Wire `GtkGLArea` realize signal → `core_surface.init()` via the surface's `platform` field

### GTK Apprt Reference Pattern

The GTK apprt (`ghostty/src/apprt/gtk/class/surface.zig`) demonstrates the working pattern:
- Each surface holds a `gl_area: *gtk.GLArea` field
- Scale factor comes from `gl_area.as(gtk.Widget).getScaleFactor()`
- The `getContentScale()` call reads `gtk-xft-dpi` GSettings for font DPI scaling on top of the hardware scale factor
- Rendering is driven by `gl_area.queueRender()` from the wakeup callback path

### Minimum Zig Fork Diff

The fork diff must touch:
1. `ghostty/include/ghostty.h` — add `GHOSTTY_PLATFORM_GTK4 = 3` to `ghostty_platform_e`, add `ghostty_platform_gtk4_s` struct, add to `ghostty_platform_u`
2. `ghostty/src/apprt/embedded.zig` — add `gtk4` variant to `Platform` union, `PlatformTag` enum, `Platform.C` union, and `Platform.init()` switch
3. `ghostty/src/renderer/OpenGL.zig` — add `apprt.embedded` (when `platform == .gtk4`) path to `surfaceInit`, `threadEnter`, `threadExit` that calls `prepareContext(null)`

**What to put in `ghostty_platform_gtk4_s`:** Based on how the macOS variant passes an NSView (opaque render target), the GTK4 variant needs:
```c
typedef struct {
    GtkWidget *gl_area;   // The GtkGLArea widget Ghostty renders into
    // Note: GdkGLContext is obtained from gl_area on realize, not passed here
} ghostty_platform_gtk4_s;
```

**Claude's Discretion resolution:** `GtkGLArea` is sufficient — no custom GDK surface backend is needed. The GTK apprt already uses `GtkGLArea` successfully. The `GdkGLContext` is managed by GTK4 internally when `GtkGLArea` is realized; Ghostty just needs to call `glarea.makeCurrent()` before drawing and bind `gl_area.connect_render()` to trigger `ghostty_surface_draw()`.

### Zig Build Command to Produce libghostty.a

Based on reading `ghostty/build.zig.zon` and `ghostty/src/build/Config.zig`:

```bash
cd ghostty
zig build \
  -Dapp-runtime=none \
  -Doptimize=ReleaseFast \
  -Dgtk-x11=true \
  -Dgtk-wayland=true
# Output: zig-out/lib/libghostty.a
```

The `app-runtime=none` flag switches the build to produce `libghostty.a` instead of an executable (confirmed by build.zig: `"Runtime 'none' is libghostty"`). `ReleaseFast` mirrors the macOS `setup.sh` pattern. Both X11 and Wayland targets are needed for GHOST-05.

**Zig version:** The ghostty fork requires `minimum_zig_version = "0.15.2"`. The installed Zig is `0.16.0-dev.2962`. Dev builds are generally forward-compatible for builds that don't use new APIs; MEDIUM confidence this works, LOW risk for the Phase 1 spike.

---

## Architecture Patterns

### Phase 1 Project Structure

```
src/
├── main.rs               # Entry: GtkApplication, window, surface init
├── ghostty/
│   ├── mod.rs            # GhosttyBridge: app/surface lifecycle
│   ├── ffi.rs            # include!(concat!(env!("OUT_DIR"), "/ghostty_sys.rs"))
│   ├── surface.rs        # GhosttyWidget: GtkGLArea subclass + event forwarding
│   └── callbacks.rs      # C callback functions (wakeup, action, clipboard, close)
build.rs                  # bindgen + link directives
Cargo.toml                # gtk4, glib, [build-dependencies] bindgen, cc
ghostty.h                 # (existing) bindgen input
ghostty/zig-out/lib/libghostty.a  # pre-built by setup-linux.sh
scripts/
└── setup-linux.sh        # cd ghostty && zig build -Dapp-runtime=none ...
```

### Pattern 1: GtkGLArea Surface Lifecycle

**What:** Each terminal surface is a `GtkGLArea` widget. Ghostty renders into it via OpenGL. The widget's GTK signals map to Ghostty API calls.

**When to use:** Always — one `GtkGLArea` per `ghostty_surface_t`.

**Lifecycle:**
```
GtkGLArea::realize
  → gl_area.make_current()
  → ghostty_surface_set_size(surface, width_px, height_px)
  → ghostty_surface_set_content_scale(surface, scale, scale)
  → ghostty_surface_set_focus(surface, true)

GtkGLArea::render (signal)
  → ghostty_surface_draw(surface)   // synchronous OpenGL draw
  → return true (suppress default)

GtkWidget::notify::scale-factor
  → ghostty_surface_set_content_scale(surface, new_scale, new_scale)

GtkGLArea::resize (signal)
  → ghostty_surface_set_size(surface, width_px, height_px)

GtkWidget::destroy / surface close
  → ghostty_surface_request_close(surface)
  → wait for close_surface_cb
  → ghostty_surface_free(surface)
```

**Example (build.rs):**
```rust
// Source: https://rust-lang.github.io/rust-bindgen/tutorial-3.html
fn main() {
    println!("cargo:rustc-link-search=ghostty/zig-out/lib");
    println!("cargo:rustc-link-lib=static=ghostty");
    println!("cargo:rerun-if-changed=ghostty.h");

    let bindings = bindgen::Builder::default()
        .header("ghostty.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ghostty_sys.rs"))
        .expect("Couldn't write bindings");
}
```

### Pattern 2: Wakeup Callback (GHOST-07)

**What:** Ghostty's internal renderer calls `wakeup_cb` from a background thread when it needs the main thread to tick. The callback must schedule `ghostty_app_tick()` on the GLib main loop — never call it inline.

**Critical:** GLib's `idle_add` does NOT coalesce multiple calls (unlike macOS `DispatchQueue.main.async`). Use an atomic pending flag to prevent scheduling multiple ticks per wakeup burst.

```rust
// In callbacks.rs
static WAKEUP_PENDING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

extern "C" fn wakeup_cb(userdata: *mut std::ffi::c_void) {
    // userdata is *mut ghostty_app_t (or a wrapper)
    if WAKEUP_PENDING.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return; // already pending
    }
    let app_ptr = userdata as usize; // stable address
    glib::idle_add_once(move || {
        WAKEUP_PENDING.store(false, std::sync::atomic::Ordering::SeqCst);
        let app = app_ptr as ffi::ghostty_app_t;
        unsafe { ffi::ghostty_app_tick(app); }
    });
}
```

### Pattern 3: Input Routing (GHOST-03, GHOST-04)

**What:** GTK4 input events from `GtkEventControllerKey` and `GtkGestureClick` are translated to Ghostty input structs and forwarded synchronously — no channels, no allocations in the hot path.

```rust
// In surface.rs — key press handler
let key_controller = gtk4::EventControllerKey::new();
key_controller.connect_key_pressed(|_, keyval, keycode, state| {
    let input = map_gtk_key_to_ghostty(keyval, keycode, state);
    unsafe { ffi::ghostty_surface_key(surface_ptr, input); }
    gtk4::Inhibit(true)
});
gl_area.add_controller(&key_controller);
```

**Key mapping:** GDK keyvals do not map 1:1 to `ghostty_input_key_e` (which is W3C UIEvents-based). The `keycode` (hardware scancode) maps to the physical key; `keyval` provides the Unicode codepoint for `text`. Both are needed.

### Pattern 4: Scale Factor Updates (GHOST-06 + Claude's Discretion)

**Verified:** `notify::scale-factor` is the correct GTK4 signal. From reading `ghostty/src/apprt/gtk/class/surface.zig`:

```rust
// In surface.rs — scale factor update
gl_area.connect_notify(Some("scale-factor"), move |widget, _| {
    let scale = widget.scale_factor() as f64;
    unsafe {
        ffi::ghostty_surface_set_content_scale(surface_ptr, scale, scale);
        ffi::ghostty_surface_refresh(surface_ptr);  // trigger redraw at new scale
    }
});
```

**Additional:** The GTK apprt also reads `gtk-xft-dpi` GSettings for font DPI. For Phase 1 simplicity, start with just `getScaleFactor()`. XFT DPI scaling is a Phase 4/5 polish item.

### Pattern 5: Clipboard (GHOST-05)

```rust
// read_clipboard_cb — called by Ghostty to read clipboard
extern "C" fn read_clipboard_cb(
    userdata: *mut c_void,
    clipboard_type: c_int,
    request: *mut ffi::apprt_ClipboardRequest,
) {
    let display = gdk4::Display::default().unwrap();
    let clipboard = if clipboard_type == ffi::GHOSTTY_CLIPBOARD_SELECTION as c_int {
        display.primary_clipboard()  // X11 selection (may be no-op on Wayland)
    } else {
        display.clipboard()
    };
    // Use glib::spawn_future_local — never block
    glib::spawn_future_local(async move {
        if let Ok(text) = clipboard.read_text_future().await {
            let c_str = std::ffi::CString::new(text.as_str()).unwrap();
            unsafe { ffi::ghostty_surface_complete_clipboard_request(surface, c_str.as_ptr(), request, true); }
        }
    });
}

// write_clipboard_cb — called by Ghostty to write to clipboard
extern "C" fn write_clipboard_cb(
    userdata: *mut c_void,
    clipboard_type: c_int,
    content: *const ffi::ghostty_clipboard_content_s,
    len: usize,
    confirm: bool,
) {
    // Find text/plain;charset=utf-8 content
    let text = find_text_content(content, len);
    let display = gdk4::Display::default().unwrap();
    let clipboard = display.clipboard();
    clipboard.set_text(&text);
}
```

**GDK handles X11/Wayland divergence transparently.** On Wayland, `primary_clipboard()` requires the app to have focus; on X11 it's always available. Phase 1 supports both via GDK API without branching.

### Anti-Patterns to Avoid

- **Independent render loop:** Never call `ghostty_surface_draw()` on a timer. Only in `GtkGLArea::render` signal (triggered by `queue_render()` from wakeup path). Adding an independent loop causes input latency per macOS CLAUDE.md pitfall.
- **Tokio in Phase 1:** D-07 locks this out. GLib main loop only. `glib::spawn_future_local` for clipboard async.
- **Direct ghostty calls in wakeup_cb:** The callback fires from Ghostty's thread. Always `glib::idle_add_once`. See Pattern 2.
- **`std::sync::Mutex` in GTK signal handlers:** GTK signals can re-enter on the same thread. Use `RefCell` for single-threaded state in GTK context.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FFI bindings from ghostty.h | Manual `extern "C"` declarations | bindgen in build.rs | 1,169 lines, 100+ functions, 50+ structs/enums — manual maintenance is error-prone |
| GL context management | Custom EGL/GLX setup | GtkGLArea | GtkGLArea handles context creation, buffer swaps, Wayland/X11 backend selection |
| Clipboard X11/Wayland divergence | libxcb + wl_data_device | gdk4::Clipboard | GDK abstracts the backend; one API for both |
| Key event GDK→GTK translation | xkbcommon bindings | gtk4-rs EventControllerKey | GTK4 already handles xkbcommon and composes the keyval |
| GLib idle_add wrappers | Raw `glib_idle_add` FFI | `glib::idle_add_once` from glib crate | Type-safe, handles thread pinning |

**Key insight:** GtkGLArea is purpose-built for exactly this use case (embedding an external OpenGL renderer). Ghostty's own GTK apprt already uses it. Do not attempt a custom EGL/GLX approach — it will not work on Wayland.

---

## Common Pitfalls

### Pitfall 1: OpenGL 4.3 Minimum Requirement

**What goes wrong:** `ghostty/src/renderer/OpenGL.zig` requires OpenGL 4.3 (`MIN_VERSION_MAJOR = 4`, `MIN_VERSION_MINOR = 3`). If the development machine's GPU only supports OpenGL 3.x (or the EGL/GLX context defaults to an older version), Ghostty returns `error.OpenGLOutdated` and the surface fails silently.

**How to avoid:** In `GtkGLArea::realize`, call `gl_area.set_required_version(4, 3)` before the area is realized. If the area cannot create a compatible context, GTK4 fires the `create-context` signal with an error — handle it to print diagnostics and exit per D-09.

**Warning signs:** Black `GtkGLArea` with no error message; check `gl_area.error()` in the realize handler.

### Pitfall 2: libghostty.a Missing System Deps at Link Time

**What goes wrong:** `libghostty.a` (static archive) embeds Ghostty's Zig code but at link time the Rust binary also needs GTK4, GLib, Freetype, Harfbuzz, Fontconfig, and GLAD system libraries. Omitting these from `build.rs` produces link errors like `undefined reference to gtk_gl_area_make_current`.

**How to avoid:** Use `pkg-config` in `build.rs` to find GTK4/GLib libraries. Also link OpenGL:
```rust
// In build.rs
println!("cargo:rustc-link-lib=dylib=GL");
// Use pkg-config crate or system() to find gtk4 flags
```

**Warning signs:** Link errors mentioning `gtk_*` or `gl*` symbols during `cargo build`.

### Pitfall 3: embedded.zig OpenGL Path is Not Wired

**What goes wrong:** Even after adding `GHOSTTY_PLATFORM_GTK4` to the C header and embedded.zig Platform union, calling `ghostty_surface_new()` with the GTK4 platform tag will silently do nothing for rendering. The `OpenGL.zig` renderer's `surfaceInit` has a TODO stub for the embedded apprt. The Ghostty surface will have a PTY and process but no GL state — the GtkGLArea will render a blank frame.

**How to avoid:** The fork diff MUST include changes to `OpenGL.zig` to add the GTK4 embedded path that calls `prepareContext(null)` in `surfaceInit`, same as the `apprt.gtk` path.

**Warning signs:** PTY works (text appears if logged), terminal process responds to input, but no pixels rendered on screen.

### Pitfall 4: wakeup_cb Called Before GtkGLArea is Realized

**What goes wrong:** Ghostty's PTY and renderer start on `ghostty_surface_new()`, which may fire `wakeup_cb` before the `GtkGLArea` has been added to a window and realized (before it has a valid GL context). Calling `ghostty_surface_draw()` before realize crashes in GLAD or produces an `EGL_NOT_INITIALIZED` error.

**How to avoid:** In the wakeup callback path, guard the `queue_render()` call:
```rust
if gl_area.is_realized() {
    gl_area.queue_render();
}
```
The first real frame fires after realize.

**Warning signs:** Crash or EGL error on startup before the window is visible.

### Pitfall 5: Scale Factor Mismatch Between GTK Logical and Physical Pixels

**What goes wrong:** `ghostty_surface_set_size()` expects pixel dimensions. `GtkGLArea::resize` signal provides logical (CSS) pixel dimensions. On a 2x HiDPI display, passing logical dimensions renders at half resolution (blurry). The physical pixel size is `logical * scale_factor`.

**How to avoid:** Always multiply by scale factor:
```rust
gl_area.connect_resize(move |widget, logical_w, logical_h| {
    let scale = widget.scale_factor();
    let phys_w = logical_w * scale;
    let phys_h = logical_h * scale;
    unsafe { ffi::ghostty_surface_set_size(surface, phys_w as u32, phys_h as u32); }
});
```

**Warning signs:** Terminal appears blurry on HiDPI, correct on 1x displays.

### Pitfall 6: GTK4 Key Event Text vs. Keycode Mapping

**What goes wrong:** Ghostty's `ghostty_input_key_s` uses W3C UIEvents keycode strings (e.g., `GHOSTTY_KEY_A`) mapped from hardware scancodes, not GDK keyvals. GDK keyvals are Unicode codepoints (e.g., `GDK_KEY_a = 0x0061`). Passing the GDK keyval integer as the `keycode` field produces incorrect key routing in Ghostty's keybind system.

**How to avoid:** Map GDK hardware keycode (the scancode, not keyval) to `ghostty_input_key_e`. The `keycode` field in `ghostty_input_key_s` is the hardware scancode. The `text` field is the UTF-8 string from `gdk::KeyEvent::keyval().to_unicode()`. Both must be populated.

**Warning signs:** Most keys work but modifier-chord keybindings (e.g., Ctrl+C) don't trigger, or wrong characters appear for non-QWERTY layouts.

---

## Code Examples

### Minimal Cargo.toml

```toml
# Source: verified against crates.io registry 2026-03-23
[package]
name = "cmux-linux"
version = "0.1.0"
edition = "2021"

[dependencies]
gtk4 = "0.11"
glib = "0.22"

[build-dependencies]
bindgen = "0.72"
cc = "1"
```

### Minimal build.rs

```rust
// Source: https://rust-lang.github.io/rust-bindgen/tutorial-3.html + ghostty.h analysis
use std::env;
use std::path::PathBuf;

fn main() {
    // Link libghostty static archive (pre-built by scripts/setup-linux.sh)
    println!("cargo:rustc-link-search=ghostty/zig-out/lib");
    println!("cargo:rustc-link-lib=static=ghostty");
    // libghostty.a needs OpenGL + GTK4 at link time
    println!("cargo:rustc-link-lib=dylib=GL");

    // Re-generate when ghostty.h changes
    println!("cargo:rerun-if-changed=ghostty.h");

    let bindings = bindgen::Builder::default()
        .header("ghostty.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ghostty_sys.rs"))
        .expect("Couldn't write bindings");
}
```

### GtkGLArea Realize + Render Handler

```rust
// Source: GTK4 docs (https://docs.gtk.org/gtk4/class.GLArea.html)
// + ghostty GTK apprt pattern from ghostty/src/apprt/gtk/class/surface.zig

let gl_area = gtk4::GLArea::new();
gl_area.set_required_version(4, 3); // Ghostty requires OpenGL 4.3+
gl_area.set_auto_render(false);     // Manual render via queue_render()

// Realize: GL context is now valid
gl_area.connect_realize({
    let surface = surface.clone();
    move |area| {
        area.make_current();
        if let Some(err) = area.error() {
            eprintln!("GLArea realize error: {err}");
            std::process::exit(1);
        }
        let scale = area.scale_factor() as f64;
        let (w, h) = (area.width(), area.height());
        unsafe {
            ffi::ghostty_surface_set_size(surface, (w * scale as i32) as u32, (h * scale as i32) as u32);
            ffi::ghostty_surface_set_content_scale(surface, scale, scale);
            ffi::ghostty_surface_set_focus(surface, 1);
        }
    }
});

// Render: called by GTK frame clock
gl_area.connect_render({
    let surface = surface.clone();
    move |_area, _ctx| {
        unsafe { ffi::ghostty_surface_draw(surface); }
        glib::Propagation::Stop
    }
});

// Resize: physical pixels
gl_area.connect_resize({
    let surface = surface.clone();
    move |area, logical_w, logical_h| {
        let scale = area.scale_factor();
        unsafe {
            ffi::ghostty_surface_set_size(
                surface,
                (logical_w * scale) as u32,
                (logical_h * scale) as u32,
            );
        }
    }
});
```

### Ghostty App + Surface Init

```rust
// Source: ghostty.h C API analysis + macOS GhosttyTerminalView.swift reference

unsafe {
    // One-time init
    let argv = std::env::args().collect::<Vec<_>>();
    let c_args: Vec<_> = argv.iter().map(|a| CString::new(a.as_str()).unwrap()).collect();
    let mut ptrs: Vec<_> = c_args.iter().map(|a| a.as_ptr() as *mut _).collect();
    ffi::ghostty_init(ptrs.len(), ptrs.as_mut_ptr());

    // Config
    let config = ffi::ghostty_config_new();
    ffi::ghostty_config_load_default_files(config);
    ffi::ghostty_config_finalize(config);  // CRITICAL: must call before ghostty_app_new

    // Runtime config with callbacks
    let mut runtime_config = ffi::ghostty_runtime_config_s {
        userdata: app_ptr,
        supports_selection_clipboard: 1,
        wakeup_cb: Some(wakeup_cb),
        action_cb: Some(action_cb),
        read_clipboard_cb: Some(read_clipboard_cb),
        confirm_read_clipboard_cb: Some(confirm_read_clipboard_cb),
        write_clipboard_cb: Some(write_clipboard_cb),
        close_surface_cb: Some(close_surface_cb),
    };

    let app = ffi::ghostty_app_new(&runtime_config, config);
    ffi::ghostty_config_free(config);

    // Surface config
    let mut surface_config = ffi::ghostty_surface_config_new();
    surface_config.platform_tag = ffi::GHOSTTY_PLATFORM_GTK4 as c_int;  // NEW
    surface_config.platform = ffi::ghostty_platform_u {
        gtk4: ffi::ghostty_platform_gtk4_s {
            gl_area: gl_area.as_ptr() as *mut _,  // GtkWidget*
        },
    };
    surface_config.userdata = surface_userdata_ptr;
    surface_config.scale_factor = 1.0;  // updated on realize

    let surface = ffi::ghostty_surface_new(app, &surface_config);
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| gtk4 crate 0.9.x (training data) | gtk4 crate 0.11.1 | ~late 2025 | Use 0.11.x in Cargo.toml |
| glib crate 0.20.x (training data) | glib crate 0.22.3 | ~late 2025 | Use 0.22.x; pulled in by gtk4 dep |
| bindgen 0.70.x (training data) | bindgen 0.72.1 | ~2025 | API stable; use latest |
| Ghostty GTK apprt: ad-hoc GObject | Ghostty GTK apprt: full GObject type system (gtk-ng) | Ghostty 1.2 (Aug 2025) | Ghostty's standalone GTK app is now fully GObject-based; embedded apprt unchanged |
| libghostty: macOS-only | libghostty-vt: cross-platform VT lib (no GPU) | Ghostty 1.3-dev | libghostty-vt is a separate no-GPU library; the GPU-rendering libghostty is still macOS-only without the fork extension |

**Deprecated/outdated:**
- `gtk4 = "0.9"`: Prior project research used this; registry shows 0.11.1 current
- `glib = "0.20"`: Prior project research used this; registry shows 0.22.3 current

---

## Open Questions

1. **Zig 0.16-dev compatibility with ghostty fork**
   - What we know: ghostty fork requires `minimum_zig_version = "0.15.2"`; installed Zig is `0.16.0-dev.2962`
   - What's unclear: Whether the `0.16.0-dev` API surface (new Zig 0.16 changes) breaks any ghostty build code
   - Recommendation: Attempt `zig build -Dapp-runtime=none` early in the spike; if it fails, install Zig 0.15.x alongside

2. **Which pkg-config libraries does libghostty.a need at link time?**
   - What we know: Ghostty uses GTK4, GLib, Freetype, Harfbuzz, Fontconfig, GLAD; all are bundled statically by Zig build when `app-runtime=none` + `ReleaseFast`
   - What's unclear: Whether `app-runtime=none` produces a fully self-contained `.a` or still requires system GTK4/GLib link flags
   - Recommendation: Run `nm ghostty/zig-out/lib/libghostty.a | grep gtk_` after build; if undefined symbols exist, add pkg-config lookup in build.rs

3. **XFT DPI font scaling for Phase 1**
   - What we know: Ghostty's GTK apprt reads `gtk-xft-dpi` GSettings on top of hardware scale factor
   - What's unclear: Whether Phase 1 (single surface, developer tool) needs this, or if `getScaleFactor()` alone is sufficient
   - Recommendation: Defer XFT DPI to Phase 4 polish (HDPI-01/02); use `getScaleFactor()` only in Phase 1

4. **ghostty_config_finalize() requirement**
   - What we know: `ghostty_config_finalize()` must be called before `ghostty_app_new()` (confirmed from macOS reference code in GhosttyTerminalView.swift). Values are not applied until finalized.
   - What's unclear: Whether there are any Linux-specific config defaults that need explicit setting
   - Recommendation: Call `ghostty_config_load_default_files()` then `ghostty_config_finalize()` — this picks up Ghostty's own `~/.config/ghostty/config` on Linux automatically

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo | Rust builds | Yes | 1.91.1 | — |
| Zig | Build libghostty.a | Yes | 0.16.0-dev.2962 | Install 0.15.x if 0.16-dev fails |
| libgtk-4-1 (runtime) | GTK4 widgets at runtime | Yes | 4.14.5 | — |
| libgtk-4-dev (headers) | gtk4-rs crate build | No | — | Install: `sudo apt install libgtk-4-dev` |
| libglib2.0-dev | glib crate build | Yes | 2.80.0 | — |
| libclang-dev | bindgen parse ghostty.h | Yes | 18.1.3 | — |
| pkg-config | Detect GTK4/GLib flags | Yes | 1.8.1 | — |
| GTK4 dev headers (pkg-config) | `pkg-config gtk4` | No | — | Install: `sudo apt install libgtk-4-dev` |
| OpenGL 4.3 GPU | Ghostty rendering | Unknown | — | Software rasterizer (Mesa llvmpipe) |

**Missing dependencies with no fallback:**
- `libgtk-4-dev`: Required for `gtk4-rs` to find GTK4 headers during `cargo build`. Install with `sudo apt install libgtk-4-dev` before Wave 1.

**Missing dependencies with fallback:**
- OpenGL 4.3 GPU: If the development machine lacks a hardware GL 4.3 GPU, Mesa's llvmpipe (software rasterizer) can provide OpenGL 4.5 but at reduced performance. Acceptable for development.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust's built-in `cargo test` |
| Config file | none needed (no special config) |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

**Note:** GHOST-01 (fork extension) and GHOST-02 (surface renders) cannot be tested with unit tests — they require a live GTK4 window and GPU context. Tests for Phase 1 validate pure-Rust logic only (key mapping, scale calculation, wakeup coalescing logic).

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GHOST-01 | Fork compiles as libghostty.a | Build/manual | `zig build -Dapp-runtime=none` | Wave 0 setup |
| GHOST-02 | Surface renders in GTK4 window | Manual smoke | Launch app, visually verify | N/A (integration) |
| GHOST-03 | Keystroke < 10ms latency | Manual measure | `perf record` + timing probe | N/A (runtime) |
| GHOST-04 | Mouse events route correctly | Manual smoke | Click/scroll in terminal | N/A (integration) |
| GHOST-05 | Clipboard copy/paste X11+Wayland | Manual smoke | Copy from terminal, paste elsewhere | N/A (integration) |
| GHOST-06 | HiDPI scale factor correct | Manual + unit | `cargo test test_scale_calculation` | Wave 0 |
| GHOST-07 | Wakeup never calls ghostty inline | Unit | `cargo test test_wakeup_coalescing` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo build` (compile check) + `cargo test --lib`
- **Per wave merge:** `cargo test` + manual smoke of visible behavior
- **Phase gate:** Terminal visible, keyboard input works, clipboard works, no crash in 5-minute typing session

### Wave 0 Gaps

- [ ] `scripts/setup-linux.sh` — build libghostty.a via Zig
- [ ] `Cargo.toml` — initial dependencies
- [ ] `build.rs` — bindgen + link directives
- [ ] `src/main.rs` — GtkApplication skeleton
- [ ] `src/ghostty/ffi.rs` — include generated bindings
- [ ] `tests/test_scale.rs` — unit test for physical pixel calculation (REQ GHOST-06)
- [ ] `tests/test_wakeup.rs` — unit test for wakeup coalescing (REQ GHOST-07)

---

## Project Constraints (from CLAUDE.md)

The following directives from `./CLAUDE.md` govern implementation:

- **Testing policy — no fake tests:** Tests must verify observable runtime behavior, not source code text or file contents. No tests that grep source files or check if a method exists. If a behavior cannot be tested end-to-end, add a runtime seam first.
- **Socket command threading:** Non-focus socket commands must never use `DispatchQueue.main.sync` equivalents. Phase 1 has no socket — establish threading pattern correctly now for Phase 3.
- **Socket focus policy:** Socket commands must not steal macOS app focus. Linux equivalent: no `gtk_window_present()` from non-focus socket commands (enforced in Phase 3).
- **No bare `xcodebuild`:** macOS-only; irrelevant. Linux equivalent: no bare `cargo build` without confirming `libghostty.a` is built.
- **Submodule safety:** Before committing updated submodule pointer, push the submodule commit to `manaflow-ai/ghostty` remote. Never commit on detached HEAD.
- **Ghostty submodule workflow:** Fork changes go to `manaflow-ai/ghostty`. Keep `docs/ghostty-fork.md` updated with all fork changes.

---

## Sources

### Primary (HIGH confidence)
- `ghostty/src/apprt/embedded.zig` (manaflow-ai fork, commit bc9be90) — Platform union, App.Options, Surface struct, wakeup/action callbacks
- `ghostty/src/renderer/OpenGL.zig` — OpenGL 4.3 requirement, embedded apprt TODO stub, GTK global context pattern
- `ghostty/src/apprt/gtk/class/surface.zig` — GtkGLArea usage pattern, scale factor, getContentScale, queueRender
- `ghostty.h` (repo root, 1,169 lines) — Full C API: ghostty_platform_e, ghostty_runtime_config_s, ghostty_surface_config_s, all surface functions
- `Sources/GhosttyTerminalView.swift` — macOS reference: platform_tag = GHOSTTY_PLATFORM_MACOS, ghostty_surface_new call, wakeup dispatch pattern
- `ghostty/build.zig.zon` — Minimum Zig version = 0.15.2
- crates.io registry (via `cargo search`) — gtk4 = 0.11.1, glib = 0.22.3, bindgen = 0.72.1 (verified 2026-03-23)
- System packages: libgtk-4-1 4.14.5, libclang-dev 18.1.3, zig 0.16.0-dev (verified via `dpkg` and `zig version`)

### Secondary (MEDIUM confidence)
- GTK4 official docs (https://docs.gtk.org/gtk4/class.GLArea.html) — GtkGLArea signals (render, realize, resize, create-context), make_current, attach_buffers
- bindgen user guide (https://rust-lang.github.io/rust-bindgen/tutorial-3.html) — build.rs pattern for cargo library linking + bindgen
- GTK4-rs book (https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html) — glib::spawn_future_local, idle_add, cross-thread communication
- Mitchell Hashimoto's gtk-ng rewrite post — GTK apprt context for Ghostty 1.2+

### Tertiary (LOW confidence)
- `zig build -Dapp-runtime=none` produces self-contained libghostty.a with all GTK deps bundled — inferred from build.zig analysis; not verified by actually running the build

---

## Metadata

**Confidence breakdown:**
- Ghostty fork extension points: HIGH — source read directly
- OpenGL embedded path blocker: HIGH — explicit TODO comment in source
- Standard stack versions: HIGH — verified via crates.io registry
- Zig 0.16-dev compatibility: LOW — untested; ghostty requires 0.15.2 minimum
- libghostty.a link-time deps: MEDIUM — inferred from Zig build system

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (gtk4-rs stable; Ghostty fork slow-moving)
