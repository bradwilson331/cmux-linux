# Phase 2: Workspaces + Pane Splits - Research

**Researched:** 2026-03-24
**Domain:** Rust + GTK4 (gtk4-rs 0.10) workspace and split-pane multiplexer UI
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Sidebar layout — vertical workspace list on the left using `GtkPaned` (horizontal) with `GtkListBox`/`GtkScrolledWindow` sidebar on the left, terminal area on the right.
- **D-02:** Workspace names always visible — no icon-only collapse in Phase 2.
- **D-03:** Rename via keyboard shortcut only (`Ctrl+Shift+R`). No inline click-to-rename in Phase 2.
- **D-04:** Sidebar toggleable via `Ctrl+B` (show/hide).
- **D-05:** Nested `GtkPaned` widgets represent the split tree. Each split node = one `GtkPaned`; leaf nodes = `GtkGLArea`. Drag-to-resize handled natively by GTK4's GtkPaned separator.
- **D-06:** Recursive nesting: `Root GtkPaned (horizontal) → {GtkGLArea | GtkPaned (vertical) → {GtkGLArea, GtkGLArea}}`.
- **D-07:** On pane close, sibling expands to fill space (GtkPaned's natural behavior).
- **D-08:** New terminal when splitting inherits CWD of active pane via `ghostty_surface_inherited_config()`.
- **D-09:** Initial split ratio always 50/50.
- **D-10:** Linux keyboard shortcuts: Split right: `Ctrl+D`, Split down: `Ctrl+Shift+D`, New workspace: `Ctrl+N`, Close workspace: `Ctrl+Shift+W`, Next workspace: `Ctrl+]`, Prev workspace: `Ctrl+[`, Focus pane left/right/up/down: `Ctrl+Alt+arrows`, Toggle sidebar: `Ctrl+B`, Rename workspace: `Ctrl+Shift+R`, Workspace by number: `Ctrl+1`–`Ctrl+9`.

### Claude's Discretion

- Exact GTK4 widget hierarchy and Rust struct layout for workspace model (`Vec<Workspace>` with active index, or an observable model)
- How `ghostty_surface_set_focus` is called on GtkPaned size-allocate vs. explicit user focus navigation
- Whether GtkPaned position needs to be serialized per-split for session restore in Phase 3
- GObject signal connections for GtkPaned's `notify::position` if divider positions need to be tracked

### Deferred Ideas (OUT OF SCOPE)

- Socket API (Phase 3)
- Session persistence (Phase 3)
- Config file (Phase 5)
- Notification/attention state (Phase 4)
- Sidebar icon-only collapse mode
- Animation on sidebar toggle
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WS-01 | User can create a new workspace (tab) | `src/workspace.rs` model + `GtkStack::add_named` + `GtkListBox` row append |
| WS-02 | User can close a workspace | `GtkStack::remove` + `GtkListBox` row remove + `ghostty_surface_free` for all panes + confirmation dialog |
| WS-03 | User can switch between workspaces via keyboard shortcut and click | `GtkStack::set_visible_child` + `GtkListBox::connect_row_selected` + `EventControllerKey` on window |
| WS-04 | User can rename a workspace | Inline `GtkEntry` swap in sidebar row on `Ctrl+Shift+R` |
| WS-05 | User can switch to workspace by number (1–9) | Window `EventControllerKey` intercept `Ctrl+1`–`Ctrl+9` |
| WS-06 | Workspace list is visible in a sidebar/tab bar | `GtkScrolledWindow` + `GtkListBox` at fixed 160px width |
| SPLIT-01 | User can split the active pane horizontally | `GtkPaned::new(Orientation::Horizontal)` + `ghostty_surface_inherited_config` |
| SPLIT-02 | User can split the active pane vertically | `GtkPaned::new(Orientation::Vertical)` + `ghostty_surface_inherited_config` |
| SPLIT-03 | User can navigate between panes via keyboard shortcut | `Ctrl+Alt+arrows` → directional focus walk of `SplitNode` tree |
| SPLIT-04 | User can drag dividers to resize panes | Native `GtkPaned` separator — no custom code needed |
| SPLIT-05 | User can close the active pane | Remove leaf from `SplitNode` tree, replace parent `GtkPaned` with surviving sibling, call `ghostty_surface_free` |
| SPLIT-06 | Pane layout as immutable tree (SplitEngine Bonsplit Rust port) | `SplitNode` enum in `src/split_engine.rs` |
| SPLIT-07 | Focus routing: correct pane receives keyboard input; `ghostty_surface_set_focus` on focus change | Call `ghostty_surface_set_focus(surface, true)` on focus-in, `false` on focus-out; track active pane pointer |
</phase_requirements>

---

## Summary

Phase 2 transforms the single-surface Phase 1 window into a full multiplexer. The implementation has two parallel tracks: (1) workspace management using `GtkStack` (one page per workspace) with a `GtkListBox` sidebar, and (2) per-workspace pane splitting using a recursive `SplitNode` tree where each split maps to a `GtkPaned` and each leaf is a `GtkGLArea` (Ghostty surface).

All locked decisions map directly to verified gtk4-rs 0.10.3 APIs available in `~/.cargo/registry`. `GtkPaned`, `GtkStack`, `GtkListBox`, `CssProvider`, `add_css_class`/`remove_css_class` are all present and confirmed. The biggest Phase 1 → Phase 2 migration problem is the single-global-surface architecture in `callbacks.rs` and `surface.rs`: `GL_AREA_FOR_RENDER` (thread-local, single `Option<GLArea>`) and `SURFACE_PTR` (`AtomicUsize`) must be replaced with per-surface data structures.

The `close_surface_cb` in the Ghostty C API receives only global app userdata (not the surface that triggered close). Phase 2 must route shell-exit closes via `surface_config.userdata` (a pane ID) that is readable from the closing pane to dispatch the correct close handler.

**Primary recommendation:** Build `workspace.rs`, `split_engine.rs`, `sidebar.rs`, and `shortcuts.rs` as separate Rust modules. Use `Rc<RefCell<AppState>>` for the shared mutable state (all GTK callbacks run on the GLib main thread — no `Arc`/`Mutex` needed). Replace the single-global-GLArea pattern with per-pane closures that capture their own `surface_cell` (already the pattern in `surface.rs` — extend it).

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| gtk4 (gtk4-rs) | 0.10.3 (pinned) | GTK4 widgets: `Paned`, `Stack`, `ListBox`, `CssProvider`, `EventControllerKey` | Project decision — rustc 1.91.1 compatible |
| glib | 0.20 | GLib main loop, `idle_add_once`, `timeout_add_local` | Paired with gtk4 0.10 |
| tokio | 1.x (full) | Async runtime (Phase 3 socket — not used in Phase 2 logic) | Already in Cargo.toml; keep but don't add Phase 2 async |

### Confirmed gtk4-rs 0.10.3 APIs Used in Phase 2

| Widget/API | Method | Confirmed in Source |
|------------|--------|---------------------|
| `gtk4::Paned` | `new(Orientation)`, `set_start_child`, `set_end_child`, `set_position`, `start_child()`, `end_child()` | `~/.cargo/registry/.../gtk4-0.10.3/src/auto/paned.rs` |
| `gtk4::Stack` | `new()`, `add_named`, `remove`, `set_visible_child`, `set_visible_child_name` | `~/.cargo/registry/.../gtk4-0.10.3/src/auto/stack.rs` |
| `gtk4::ListBox` | `append`, `remove`, `select_row`, `connect_row_selected`, `connect_row_activated` | `~/.cargo/registry/.../gtk4-0.10.3/src/auto/list_box.rs` |
| `gtk4::ListBoxRow` | `new()`, widget children via `set_child` | Standard GTK4 |
| `gtk4::CssProvider` | `load_from_data`, `load_from_string` | `~/.cargo/registry/.../gtk4-0.10.3/src/auto/css_provider.rs` |
| `WidgetExt::add_css_class` | `add_css_class("active-pane")` | `~/.cargo/registry/.../gtk4-0.10.3/src/auto/widget.rs:107` |
| `WidgetExt::remove_css_class` | `remove_css_class("active-pane")` | `~/.cargo/registry/.../gtk4-0.10.3/src/auto/widget.rs:1100` |
| `gtk4::functions::style_context_add_provider_for_display` | Register CSS provider globally | `~/.cargo/registry/.../gtk4-0.10.3/src/functions.rs:265` |
| `gtk4::Dialog` or `gtk4::AlertDialog` | Close workspace confirmation | Standard GTK4 |

### Ghostty C API Used in Phase 2

| Function | Purpose | Confirmed in Header |
|----------|---------|---------------------|
| `ghostty_surface_new(app, config*)` | Create new surface per pane | `ghostty.h:1076` |
| `ghostty_surface_free(surface)` | Destroy surface on pane close | `ghostty.h:1078` |
| `ghostty_surface_inherited_config(surface, context)` | Inherit CWD for new split | `ghostty.h:1081` |
| `ghostty_surface_set_focus(surface, bool)` | Route keyboard focus | `ghostty.h:1088` |
| `ghostty_surface_userdata(surface)` | Read per-surface userdata for close routing | `ghostty.h:1079` |
| `GHOSTTY_SURFACE_CONTEXT_SPLIT` | Context enum for splits | `ghostty.h:437` |

**Installation:** No new dependencies needed. All required GTK4 widgets are in `gtk4 = "0.10"` already pinned in `Cargo.toml`.

---

## Architecture Patterns

### Recommended Module Structure

```
src/
├── main.rs              # build_ui() restructured: creates AppState, builds window layout
├── app_state.rs         # Rc<RefCell<AppState>> — workspace list, active workspace index
├── workspace.rs         # Workspace struct: id, name, split_engine root, active_pane_id
├── split_engine.rs      # SplitNode enum + split/close/focus_next operations
├── sidebar.rs           # build_sidebar() → GtkScrolledWindow + GtkListBox
├── shortcuts.rs         # install_shortcuts(window, app_state) — EventControllerKey on window
└── ghostty/
    ├── surface.rs       # create_surface(app, ghostty_app, inherited_config?) → GLArea
    ├── callbacks.rs     # wakeup_cb, close_surface_cb, action_cb — updated for multi-surface
    ├── ffi.rs           # bindgen output (unchanged)
    └── input.rs         # map_mods (unchanged)
```

### Pattern 1: AppState as Rc<RefCell<...>>

**What:** All GTK callbacks on the GLib main thread — use `Rc<RefCell<AppState>>` for shared mutable state, NOT `Arc<Mutex<...>>`.

**When to use:** Any closure that both reads and writes workspace/pane state.

```rust
// Source: GTK4-rs pattern — all closures run on main thread
use std::rc::Rc;
use std::cell::RefCell;

struct AppState {
    workspaces: Vec<Workspace>,
    active_index: usize,
    stack: gtk4::Stack,
    sidebar_list: gtk4::ListBox,
    ghostty_app: ffi::ghostty_app_t,
}

let state: Rc<RefCell<AppState>> = Rc::new(RefCell::new(...));

// Clone for each closure:
let state_clone = state.clone();
button.connect_clicked(move |_| {
    let mut s = state_clone.borrow_mut();
    s.create_workspace();
});
```

**Why Rc not Arc:** `gtk4::GLArea`, `gtk4::Stack`, `gtk4::ListBox` are not `Send`. Sending them across threads would panic. GTK objects are GObject ref-counted and only safe on the main thread.

### Pattern 2: SplitNode Tree

**What:** Immutable-style tree representing pane layout. Each workspace has one root `SplitNode`.

```rust
// src/split_engine.rs
#[derive(Clone)]
pub enum SplitNode {
    Leaf {
        pane_id: u64,           // unique ID per pane
        gl_area: gtk4::GLArea,
        surface: ffi::ghostty_surface_t,
    },
    Split {
        orientation: gtk4::Orientation,
        paned: gtk4::Paned,
        start: Box<SplitNode>,
        end: Box<SplitNode>,
    },
}

pub struct SplitEngine {
    pub root: SplitNode,
    pub active_pane_id: u64,
}
```

**Split operation:** Replace the active `Leaf` with a `Split` node containing the old `Leaf` as `start` and a new `Leaf` as `end`. Build the `GtkPaned`, wire children, return the new pane_id as active.

**Close operation:** Find the parent `Split` of the active `Leaf`. Replace the parent with the surviving sibling. Remove the closed `GtkPaned` from the GTK widget tree, insert the surviving child in its place, call `ghostty_surface_free`.

**Root widget accessor:** Every `SplitNode` has a `widget()` method returning the `GtkWidget` (either `GLArea` or `Paned`) for GTK tree insertion.

### Pattern 3: Per-Surface GLArea Pattern (Extension of Phase 1)

Phase 1 uses a single `Rc<RefCell<Option<ghostty_surface_t>>>` per `create_surface()` call. Phase 2 calls `create_surface()` multiple times. Each call creates its own independent `surface_cell`. No global state needed per-surface.

**Critical change:** `GL_AREA_FOR_RENDER` thread-local and `SURFACE_PTR` global in `callbacks.rs` must be replaced:

- `GL_AREA_FOR_RENDER`: Replace with a `Rc<RefCell<HashMap<u64, gtk4::GLArea>>>` keyed by `pane_id`, or use `glib::idle_add_once` that captures the active GLArea directly. The `wakeup_cb` must trigger `queue_render()` on ALL realized GLAreas (Ghostty wakeup is app-level, not surface-level).
- `SURFACE_PTR`: Replace with a global `Mutex<HashMap<u64, ghostty_surface_t>>` OR store the surface pointer in `GLArea` object data using `glib::object::ObjectExt::set_data`. The latter avoids a global map.

**Recommended approach:** Keep `wakeup_cb` as a global that queue_renders all registered surfaces (a `Vec` protected by `Mutex<Vec<GLArea>>`). This is safe because all queue_render calls are dispatched to idle on the main thread.

### Pattern 4: Window-Level Keyboard Shortcut Interception

**What:** `EventControllerKey` on the `ApplicationWindow` intercepts shortcuts before Ghostty's per-GLArea key controllers.

**Critical:** The window-level controller must use `connect_key_pressed` and return `Propagation::Stop` for handled shortcuts and `Propagation::Proceed` for pass-through. GTK4 propagates events from leaf → root (bubble phase) by default. Adding the controller to the **window** and using `set_propagation_phase(PropagationPhase::Capture)` ensures it fires BEFORE Ghostty's GLArea controllers.

```rust
// src/shortcuts.rs
use gtk4::prelude::*;

pub fn install_shortcuts(window: &gtk4::ApplicationWindow, state: Rc<RefCell<AppState>>) {
    let key_ctrl = gtk4::EventControllerKey::new();
    // Capture phase: fires before child widgets' key handlers
    key_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);

    key_ctrl.connect_key_pressed({
        let state = state.clone();
        move |_ctrl, keyval, _keycode, mods| {
            use gtk4::gdk::ModifierType;
            let ctrl = mods.contains(ModifierType::CONTROL_MASK);
            let shift = mods.contains(ModifierType::SHIFT_MASK);
            let alt = mods.contains(ModifierType::ALT_MASK);

            match (ctrl, shift, alt, keyval) {
                (true, false, false, gtk4::gdk::Key::d) => {
                    state.borrow_mut().split_right();
                    gtk4::glib::Propagation::Stop
                }
                (true, true, false, gtk4::gdk::Key::D) => {
                    state.borrow_mut().split_down();
                    gtk4::glib::Propagation::Stop
                }
                // ... etc
                _ => gtk4::glib::Propagation::Proceed,
            }
        }
    });
    window.add_controller(key_ctrl);
}
```

**Why capture phase:** Without capture phase, Ghostty's `GLArea` EventControllerKey (which returns `Propagation::Stop`) would consume `Ctrl+D` before the window sees it. Capture phase runs parent→child, so the window intercepts first.

### Pattern 5: close_surface_cb — Identifying the Closing Surface

**Problem:** `ghostty_runtime_close_surface_cb` signature is `fn(userdata: *mut c_void, process_alive: bool)`. It receives the global app `userdata`, NOT the surface pointer. When a shell exits in one of multiple panes, the callback cannot identify which surface closed.

**Solution:** Set `surface_config.userdata` to a unique pane ID (cast as `*mut c_void`) when creating each surface. Then in the callback, the global `userdata` field must be used as a context (e.g., a pointer to the `AppState`). The closing surface can be identified because `ghostty_surface_userdata(surface)` returns the per-surface `userdata`.

**BUT:** `close_surface_cb` does not receive the surface pointer. The only way to know which surface is closing is:

1. **Approach A (recommended):** Store `surface_config.userdata = pane_id as *mut c_void` for each surface. When `close_surface_cb` fires, it means *some* surface with shell-exit requested close. Use `ghostty_app_tick` to dispatch: after tick, check all surfaces for a "close requested" flag stored in a shared registry. OR:
2. **Approach B:** Use `action_cb` with `GHOSTTY_ACTION_CLOSE_TAB` — `action_cb` receives a `ghostty_target_s` with `tag = GHOSTTY_TARGET_SURFACE` and `target.surface = surface_ptr`. This IS the correct way to identify which surface triggered a programmatic close from within Ghostty. The `close_surface_cb` handles shell-exit cases (external).

**Recommended for Phase 2:** Use `action_cb` to handle `GHOSTTY_ACTION_CLOSE_TAB` (Ghostty-initiated) and use `close_surface_cb` only for shell-exit, identifying the surface by maintaining a reverse map of `ghostty_surface_t → pane_id` in a global registry.

### Pattern 6: GtkStack for Workspace Pages

Each workspace's root `SplitNode` widget is a page in a `GtkStack`. Switching workspaces = `stack.set_visible_child(workspace_root_widget)`. No animation (`StackTransitionType::None`).

```rust
// Creating a workspace:
let root_gl_area = create_surface(app, ghostty_app, None);
let page = stack.add_named(&root_gl_area, Some(&workspace_id_str));
list_box.append(&new_row_for_workspace(&name));
stack.set_visible_child(&root_gl_area);
ghostty_surface_set_focus(surface, true);
```

### Pattern 7: 50/50 GtkPaned Position

`GtkPaned::set_position` takes logical pixels. At split time, the parent widget's allocated size may be 0 (not yet laid out). Use `paned.connect_realize` or a one-shot `notify::position` signal to set position after allocation:

```rust
// Set 50/50 after the paned widget is allocated
paned.connect_realize(move |p| {
    // At this point the paned has a size from allocation
    let size = if orientation == gtk4::Orientation::Horizontal {
        p.width()
    } else {
        p.height()
    };
    if size > 0 {
        p.set_position(size / 2);
    }
});
```

Alternatively, use `connect_notify_local` on `"max-position"` which fires when the size is determined.

### Anti-Patterns to Avoid

- **Do NOT use `Arc<Mutex<...>>` for GTK widget state.** GTK widgets are not `Send`. Use `Rc<RefCell<...>>`.
- **Do NOT call `ghostty_surface_set_focus` inside `GtkPaned::size-allocate`.** Size allocation fires on every resize — calling focus APIs there will corrupt Ghostty input state. Only call on explicit user focus actions.
- **Do NOT retain the old `GL_AREA_FOR_RENDER` thread-local pattern for multi-surface wakeup.** It only stores one GLArea. Replace with a shared registry.
- **Do NOT call `std::process::exit` in `close_surface_cb` (Phase 1 behavior).** Phase 2 must handle pane close gracefully: destroy the surface, remove the node from the split tree, update focus.
- **Do NOT set `GtkPaned` position before the widget is realized/allocated.** The position will be silently ignored or set to 0.
- **Do NOT intercept Ghostty shortcuts in the window controller at bubble phase.** Use capture phase — otherwise Ghostty eats `Ctrl+D`, `Ctrl+N`, etc.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Divider drag-to-resize | Custom hit-testing, drag gesture | `GtkPaned` native separator | GTK4 handles it; custom code is error-prone and breaks accessibility |
| CSS theming | Inline style attributes on widgets | `GtkCssProvider` + `style_context_add_provider_for_display` | Single source of truth; GTK4 theming model requires this |
| Workspace page switching | Manual widget show/hide | `GtkStack` | `GtkStack` handles child lifecycle, keeps hidden widgets realized |
| Sidebar scrolling | Custom scroll event handling | `GtkScrolledWindow` wrapping `GtkListBox` | GTK4 overlay scrollbars are free |
| Clipboard | Custom X11/Wayland clipboard code | GTK4 `gdk::Display::clipboard()` + `read_text_future()` | Already proven in Phase 1 |
| Keyboard modifier detection | Raw X11 modifier flags | `gtk4::gdk::ModifierType` bitflags | Phase 1 `map_mods` already does this correctly |

**Key insight:** In GTK4, nearly all the "chrome" complexity (splits, scrolling, page switching, CSS) is handled by the widget toolkit. The only custom logic is the `SplitNode` tree that maps logical pane structure to widget operations.

---

## Common Pitfalls

### Pitfall 1: close_surface_cb Cannot Identify the Closing Surface

**What goes wrong:** `close_surface_cb(userdata, process_alive)` is called when a shell exits. It has no surface argument. Phase 1 calls `process::exit(0)`. Phase 2 needs to close only the relevant pane but doesn't know which one.

**Why it happens:** Ghostty's C ABI keeps `close_surface_cb` surface-agnostic at the app level.

**How to avoid:** Maintain a `Mutex<HashMap<usize, u64>>` mapping `surface_ptr as usize → pane_id`. When any surface is created, register it. When `close_surface_cb` fires, use the map to find and remove the pane. The map must be a global `std::sync::Mutex` (not `Rc<RefCell>`) because `close_surface_cb` might theoretically be called from a Ghostty-internal thread during `ghostty_app_tick`.

**Warning signs:** All panes close when one shell exits, or the app panics with "no surface found".

### Pitfall 2: GtkPaned set_position Before Realize

**What goes wrong:** Calling `paned.set_position(width / 2)` immediately after `new()` sets the position to 0 (no allocated size yet). The divider appears stuck at the left edge.

**Why it happens:** GTK4 layout is asynchronous. Widgets have no size until after the first layout pass (realize + allocate).

**How to avoid:** Set position in the `connect_realize` callback or after observing `notify::max-position` going non-zero.

**Warning signs:** Split panes appear with one side at 0px width.

### Pitfall 3: Keyboard Shortcuts Swallowed by Ghostty GLArea

**What goes wrong:** `Ctrl+D` is meant to split right, but it triggers Ghostty's internal action (or passes to the shell as EOF). The shortcut never reaches the window controller.

**Why it happens:** Phase 1 attaches `EventControllerKey` to the `GLArea` with default (bubble) phase and returns `Propagation::Stop`. The window controller, added later in bubble phase, never sees the event.

**How to avoid:** Set window controller to `PropagationPhase::Capture` (runs parent→child, before GLArea controllers). The window intercepts and returns `Propagation::Stop` for known shortcuts. Unknown keys fall through with `Propagation::Proceed`.

**Warning signs:** Shortcuts work when the GLArea is not focused, but not when typing in the terminal.

### Pitfall 4: Focus Loss After Workspace Switch

**What goes wrong:** After switching workspaces, the terminal does not receive keyboard input. Typing does nothing until the user clicks inside the terminal.

**Why it happens:** `GtkStack::set_visible_child` changes the visible page but does not call `ghostty_surface_set_focus`. The newly visible surface is not focused from Ghostty's perspective.

**How to avoid:** After every workspace switch, call `ghostty_surface_set_focus(active_surface, true)` and then `gtk4::Widget::grab_focus()` on the active GLArea.

**Warning signs:** Terminal visible but keyboard input ignored after workspace switch.

### Pitfall 5: Ghost Surfaces After Pane Close (Memory Leak)

**What goes wrong:** Closing a pane removes the `GtkGLArea` from the widget tree, but the Ghostty surface is never freed. Memory grows with each close/create cycle (fails the 50-cycle stability requirement, SPLIT-05).

**Why it happens:** GTK widget removal does not call `ghostty_surface_free`. The `GtkGLArea` is ref-counted by GObject; when it drops to zero, no Rust drop glue calls the FFI free function unless explicitly wired.

**How to avoid:** When removing a `SplitNode::Leaf`, always call `ghostty_surface_free(surface)` immediately after removing the GLArea from its parent. Add a `connect_unrealize` signal on the GLArea to catch any remaining unrealized paths (defensive).

**Warning signs:** `top` / `/proc/meminfo` shows growth after workspace churn; `ghostty_surface_new` count exceeds free count.

### Pitfall 6: wakeup_cb Renders Only One GLArea

**What goes wrong:** `wakeup_cb` calls `queue_render()` on the single `GL_AREA_FOR_RENDER` thread-local. With multiple panes, only the first-created GLArea re-renders. Other panes appear frozen.

**Why it happens:** Phase 1's thread-local stores one `Option<GLArea>`. Phase 2 has N GLAreas.

**How to avoid:** Replace `GL_AREA_FOR_RENDER` with a global `Mutex<Vec<gtk4::GLArea>>` (using `glib::SendWeakRef<GLArea>` to avoid retaining destroyed widgets). `wakeup_cb` iterates and queue_renders all realized entries.

**Warning signs:** Only one pane in a split renders; others appear static/blank.

### Pitfall 7: Rename GtkEntry Focus Loop

**What goes wrong:** Replacing the `GtkLabel` with a `GtkEntry` for rename triggers a focus-out event immediately (because the sidebar row's focus changes), which commits the rename with the original name before the user types.

**Why it happens:** GTK4 focus-out fires when `set_child` replaces the label with an entry, before the entry grabs focus.

**How to avoid:** Use a boolean flag `renaming_in_progress` to suppress focus-out commits until the entry has been explicitly activated. Connect `connect_activate` (Enter key) and `connect_icon_press` (Escape via secondary icon or custom key controller on the entry) to commit/cancel.

**Warning signs:** Rename dialog appears and immediately disappears, restoring the old name.

---

## Code Examples

### CSS Provider Setup

```rust
// Source: gtk4-rs 0.10 functions.rs:265 + css_provider.rs
use gtk4::prelude::*;

pub fn apply_app_css(display: &gtk4::gdk::Display) {
    let css = gtk4::CssProvider::new();
    css.load_from_data("
        .sidebar-list {
            background-color: #242424;
        }
        .sidebar-row {
            min-height: 36px;
            padding: 8px 16px;
            color: #cccccc;
            font-size: 14px;
            font-weight: normal;
        }
        .sidebar-row:selected {
            background-color: #5b8dd9;
            color: #ffffff;
            font-weight: 600;
        }
        .sidebar-row:hover:not(:selected) {
            background-color: #2e2e2e;
        }
        .active-pane {
            border: 1px solid #5b8dd9;
        }
    ");
    gtk4::style_context_add_provider_for_display(
        display,
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
```

### Creating a Split (GtkPaned pattern)

```rust
// Source: gtk4-rs auto/paned.rs
fn split_active_pane(
    engine: &mut SplitEngine,
    orientation: gtk4::Orientation,
    ghostty_app: ffi::ghostty_app_t,
) {
    let active_leaf = engine.find_leaf_mut(engine.active_pane_id);
    let old_gl_area = active_leaf.gl_area.clone();
    let old_surface = active_leaf.surface;

    // Inherit config from active surface (CWD, font size)
    let inherited_config = unsafe {
        ffi::ghostty_surface_inherited_config(
            old_surface,
            ffi::ghostty_surface_context_e_GHOSTTY_SURFACE_CONTEXT_SPLIT,
        )
    };

    // Create new surface with inherited config
    let new_pane_id = next_pane_id();
    let new_gl_area = create_surface_with_config(ghostty_app, &inherited_config, new_pane_id);

    // Build GtkPaned
    let paned = gtk4::Paned::new(orientation);
    paned.set_start_child(Some(&old_gl_area));
    paned.set_end_child(Some(&new_gl_area.gl_area));
    // set_resize on both children ensures proportional resize on window resize
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);

    // Replace old_gl_area in its parent container with paned
    replace_widget_in_parent(&old_gl_area, &paned);

    // Set 50/50 after realize
    let orientation_clone = orientation;
    paned.connect_realize(move |p| {
        let size = if orientation_clone == gtk4::Orientation::Horizontal {
            p.width()
        } else {
            p.height()
        };
        if size > 0 {
            p.set_position(size / 2);
        }
    });

    // Update tree
    engine.replace_leaf_with_split(active_leaf.pane_id, orientation, paned, new_gl_area);
    engine.active_pane_id = new_pane_id;
}
```

### Focus Management

```rust
// Source: ghostty.h:1088
fn set_active_pane(engine: &mut SplitEngine, pane_id: u64) {
    if let Some(prev_id) = Some(engine.active_pane_id) {
        if let Some(prev_leaf) = engine.find_leaf(prev_id) {
            prev_leaf.gl_area.remove_css_class("active-pane");
            unsafe { ffi::ghostty_surface_set_focus(prev_leaf.surface, false); }
        }
    }
    engine.active_pane_id = pane_id;
    if let Some(leaf) = engine.find_leaf(pane_id) {
        leaf.gl_area.add_css_class("active-pane");
        leaf.gl_area.grab_focus();
        unsafe { ffi::ghostty_surface_set_focus(leaf.surface, true); }
    }
}
```

---

## State of the Art

| Old Approach (Phase 1) | Phase 2 Approach | Why Changed |
|------------------------|-----------------|-------------|
| `process::exit(0)` in `close_surface_cb` | Remove pane from split tree, free surface, update focus | Multiple panes must survive individual pane close |
| `GL_AREA_FOR_RENDER` single thread-local | Global `Vec<GLArea>` registry | wakeup_cb must render all panes |
| `SURFACE_PTR` single `AtomicUsize` | Per-pane `surface_cell` + global `HashMap<usize, u64>` | Multiple surfaces require per-pane dispatch |
| `window.set_child(Some(&gl_area))` | `GtkBox(H) { Sidebar, GtkStack { per-workspace root } }` | Workspace pages + sidebar layout |
| No keyboard shortcuts | Capture-phase `EventControllerKey` on window | Intercept before Ghostty eats them |

**Deprecated from Phase 1:**
- `GL_AREA_FOR_RENDER` thread-local (single GLArea): replace with registry
- `SURFACE_PTR` global `AtomicUsize`: replace with per-pane data
- `close_surface_cb` calling `process::exit`: replace with pane-close handler

---

## Open Questions

1. **`ghostty_app_tick` and multi-surface wakeup scope**
   - What we know: `ghostty_app_tick` processes all pending events for the app. `wakeup_cb` is app-level (not per-surface). After tick, Ghostty internally decides which surfaces need redraw via the render action dispatch.
   - What's unclear: Does calling `queue_render()` on ALL GLAreas after every wakeup cause excessive draws, or does Ghostty's `auto_render=false` + action_cb render dispatch handle it correctly?
   - Recommendation: Keep `wakeup_cb` rendering all realized GLAreas. Ghostty's `ghostty_surface_draw` is a no-op when nothing changed (it checks an internal dirty flag). The cost is one redundant `glFlush` per idle tick per extra pane — acceptable.

2. **Directional pane focus navigation (SPLIT-03)**
   - What we know: `Ctrl+Alt+arrows` must find the pane in a given direction. The `SplitNode` tree has orientation info but not pixel positions.
   - What's unclear: The most correct algorithm for "find pane to the right of current" in a recursive split tree is non-trivial when splits nest in different orientations.
   - Recommendation: Use the macOS `BonsplitController.focusPane(direction:)` algorithm as reference. For Phase 2 with limited split depth, a simple left/right/start/end traversal based on tree structure is sufficient.

3. **GtkPaned shrink flags and minimum pane size**
   - What we know: `set_shrink_start_child(false)` prevents a child from shrinking below its natural minimum. `GtkGLArea` has a natural minimum of approximately 40×40px.
   - What's unclear: What GTK4 version introduced `set_shrink_start_child` — is it in gtk4-rs 0.10?
   - Recommendation: Confirmed present in paned.rs at `~/.cargo/registry/.../gtk4-0.10.3/src/auto/paned.rs:146`. Safe to use.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo/rustc | Build | Yes | 1.91.1 | — |
| gtk4 pkg-config | Link flags | Yes | 4.14.5 | — |
| glib-2.0 | GLib main loop | Yes | 2.80.0 | — |
| GTK4 system libs | Runtime | Yes | 4.14.5 | — |

All Phase 2 dependencies are satisfied. No blocking gaps.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` via `cargo test` |
| Config file | `Cargo.toml` (integration tests in `tests/`) |
| Quick run command | `cargo test --lib` (unit tests, no GTK context required) |
| Full suite command | `cargo test` (includes integration tests in `tests/`) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WS-01 | Workspace struct creates correctly with auto-incrementing name | unit | `cargo test --lib workspace::tests::test_create_workspace` | No — Wave 0 |
| WS-02 | Closing a workspace with 1 pane frees the surface, removes from list | unit | `cargo test --lib workspace::tests::test_close_workspace` | No — Wave 0 |
| WS-03 | Active index updates on workspace switch | unit | `cargo test --lib workspace::tests::test_switch_workspace` | No — Wave 0 |
| WS-04 | Rename updates workspace title | unit | `cargo test --lib workspace::tests::test_rename_workspace` | No — Wave 0 |
| WS-05 | `workspace_by_index(n)` returns correct workspace for 1–9 | unit | `cargo test --lib workspace::tests::test_workspace_by_number` | No — Wave 0 |
| WS-06 | Sidebar list has correct row count after create/close | unit | `cargo test --lib sidebar::tests::test_sidebar_row_count` | No — Wave 0 |
| SPLIT-01 | Split right produces Horizontal GtkPaned, two leaf nodes | unit | `cargo test --lib split_engine::tests::test_split_right` | No — Wave 0 |
| SPLIT-02 | Split down produces Vertical GtkPaned, two leaf nodes | unit | `cargo test --lib split_engine::tests::test_split_down` | No — Wave 0 |
| SPLIT-03 | Focus right in H-split moves active_pane_id to right leaf | unit | `cargo test --lib split_engine::tests::test_focus_right` | No — Wave 0 |
| SPLIT-04 | Drag divider — GtkPaned native, no test needed (verified by SPLIT-01 paned construction) | manual | — | — |
| SPLIT-05 | Close leaf removes from tree, surviving sibling is new root of parent | unit | `cargo test --lib split_engine::tests::test_close_pane` | No — Wave 0 |
| SPLIT-06 | SplitNode tree depth-first traversal returns all pane IDs | unit | `cargo test --lib split_engine::tests::test_all_pane_ids` | No — Wave 0 |
| SPLIT-07 | active_pane_id is correct after split/close/switch operations | unit | `cargo test --lib split_engine::tests::test_focus_routing` | No — Wave 0 |

**Note:** Tests for `workspace.rs` and `split_engine.rs` are pure logic tests — they do NOT require a GTK context. `SplitNode` operations on the tree (split, close, find, traverse) can be tested without `GtkPaned` by using a test double or by separating tree logic from widget construction. Phase 2 tests must follow the project testing policy: no tests that only inspect source code structure.

### Sampling Rate

- **Per task commit:** `cargo test --lib` (unit tests only, < 5 seconds)
- **Per wave merge:** `cargo test` (all tests)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src/workspace.rs` — module must exist with `#[cfg(test)]` block for WS-01 through WS-05
- [ ] `src/split_engine.rs` — module must exist with `#[cfg(test)]` block for SPLIT-01 through SPLIT-07
- [ ] `src/sidebar.rs` — module must exist with `#[cfg(test)]` block for WS-06
- [ ] Tests must not require a live GTK display — separate pure-logic tree operations from widget construction in `split_engine.rs`

---

## Project Constraints (from CLAUDE.md)

The following directives from `CLAUDE.md` apply to Phase 2 planning and execution:

- **No `DispatchQueue.main.sync` equivalent (socket threading policy):** Not applicable in Phase 2 (no socket). But the equivalent GTK4 pattern — `glib::MainContext::block_on` inside a GTK callback — is used in clipboard code. Ensure no new blocking calls on the main thread from wakeup_cb hot path.
- **Typing-latency-sensitive paths:** `set_focusable(true)` on `GtkGLArea` is required (already in Phase 1). The window-level shortcut controller in capture phase adds one comparison per keypress to the main thread — acceptable, but ensure no heap allocation in the shortcut dispatch path.
- **`set_focusable(true)` on GLArea required:** Must be set on every new GLArea created for splits and new workspaces.
- **`ApplicationFlags::NON_UNIQUE` required:** Already set in `main.rs`. Do not change.
- **No `tokio` in Phase 2 UI logic:** tokio is in Cargo.toml but Phase 2 is GLib-only. Do not add `tokio::spawn` calls for UI operations.
- **Test policy:** Do NOT add tests that only verify source code text, method signatures, or project file contents. All tests must verify observable runtime behavior through executable paths.
- **Never run tests locally:** All tests run via CI. The plan should specify `cargo test` as the test command but execution is CI-only.
- **Localization:** All user-facing strings must use localized patterns. Phase 2 adds sidebar labels ("Workspace 1", etc.), dialog text ("Close Workspace?"), and rename placeholder text. These must be localizable (at minimum, defined as string constants — full i18n infrastructure is Phase 5).

---

## Sources

### Primary (HIGH confidence)

- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gtk4-0.10.3/` — direct inspection of `paned.rs`, `stack.rs`, `list_box.rs`, `css_provider.rs`, `widget.rs`, `functions.rs`
- `/home/twilson/code/cmux-linux/ghostty/include/ghostty.h` — direct inspection of `ghostty_surface_inherited_config`, `ghostty_surface_free`, `ghostty_surface_set_focus`, `ghostty_surface_userdata`, `close_surface_cb` signature, `ghostty_target_s` structure, `ghostty_surface_context_e` enum
- `/home/twilson/code/cmux-linux/src/ghostty/surface.rs` — confirmed Phase 1 patterns for `surface_cell`, realize lifecycle, and per-GLArea controller attachment
- `/home/twilson/code/cmux-linux/src/ghostty/callbacks.rs` — confirmed `GL_AREA_FOR_RENDER`, `SURFACE_PTR`, `WAKEUP_PENDING` globals that must change in Phase 2
- `/home/twilson/code/cmux-linux/.planning/phases/02-workspaces-pane-splits/02-CONTEXT.md` — all locked decisions (D-01 through D-10)
- `/home/twilson/code/cmux-linux/.planning/phases/02-workspaces-pane-splits/02-UI-SPEC.md` — widget hierarchy, CSS values, keyboard shortcut table, states required

### Secondary (MEDIUM confidence)

- `/home/twilson/code/cmux-linux/Sources/Workspace.swift` — macOS reference implementation; `Workspace` class structure with `bonsplitController`, `focusedPanelId`, `panels` map

### Tertiary (LOW confidence)

- GTK4 PropagationPhase::Capture behavior for event routing — verified by inspection of gtk4-rs source structure; not tested end-to-end on this machine. Flag for validation at first shortcut integration test.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified against pinned registry source
- Architecture patterns: HIGH — derived from Phase 1 code + GTK4 source inspection + Ghostty header
- Pitfalls: HIGH for items 1–6 (derived from code analysis); MEDIUM for pitfall 7 (rename entry focus loop — behavioral, not structurally verified)
- Ghostty API (close routing): MEDIUM — close_surface_cb limitation confirmed in header; recommended approach (HashMap registry) is logical but untested

**Research date:** 2026-03-24
**Valid until:** 2026-06-24 (gtk4-rs 0.10 is pinned; Ghostty header is local — stable until submodule update)
