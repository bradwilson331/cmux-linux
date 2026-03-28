---
phase: 02-workspaces-pane-splits
plan: "01"
summary: "This plan refactored the Ghostty surface creation and callback mechanisms to support multiple concurrent terminal surfaces. It replaced single-surface static globals with thread-safe registries (`GL_AREA_REGISTRY` and `SURFACE_REGISTRY`) and updated the `create_surface` function to be re-entrant, taking a `ghostty_app` handle and a `pane_id`. This is a foundational change for implementing pane splits and workspaces."
key_files_created: []
key_files_modified:
  - src/ghostty/callbacks.rs
  - src/ghostty/surface.rs
  - src/main.rs
  - Cargo.toml
decisions:
  - Used a newtype wrapper `GtkGLAreaPtr` around a raw pointer `*mut ffi::GtkGLArea` with `unsafe impl Send + Sync` to store GTK objects in a global static `Mutex`. This is safe because the pointer is only ever dereferenced on the main GTK thread.
  - Used `std::sync::LazyLock` to initialize the `SURFACE_REGISTRY` `HashMap` in a static context.
  - Updated the `glib` dependency from `0.20.12` to `0.21.5` to resolve a version conflict that was causing trait implementation errors.
  - Made clipboard callback functions in `surface.rs` public (`pub(crate)`) so they could be accessed from `main.rs` after refactoring.
deviations:
  - "Rule 3 (Auto-fix blocking issue): The initial implementation in Task 1 failed to compile due to `gtk4::GLArea` not being `Send` or `Sync`, and `HashMap::new()` not being a `const` function. This was fixed by using a raw pointer wrapper and `LazyLock`."
  - "Rule 3 (Auto-fix blocking issue): The build continued to fail due to a version mismatch in the `glib` dependency. This was fixed by updating `Cargo.toml`."
  - "Rule 3 (Auto-fix blocking issue): The build failed due to syntax errors from incorrect chained `Edit` calls. This was fixed by resetting the file and applying the full changes with `Write`."
  - "Rule 3 (Auto-fix blocking issue): The build failed due to incorrect `use` statements and visibility issues after refactoring. This was fixed by making clipboard callbacks public and moving `use` statements to the correct scope."
  - "Rule 3 (Auto-fix blocking issue): The build failed due to a corrupted file state, likely from parallel agent operations. The file was reset to a known good state and changes were re-applied."
commits:
  - "c6cec143: refactor(02-workspaces-pane-splits-01): replace single-surface globals with multi-surface registries"
  - "5f4a6eba: refactor(02-workspaces-pane-splits-01): refactor create_surface for multi-surface use"
---

# Phase 02-workspaces-pane-splits, Plan 01 Summary

## Summary
This plan refactored the Ghostty surface creation and callback mechanisms to support multiple concurrent terminal surfaces. It replaced single-surface static globals with thread-safe registries (`GL_AREA_REGISTRY` and `SURFACE_REGISTRY`) and updated the `create_surface` function to be re-entrant, taking a `ghostty_app` handle and a `pane_id`. This is a foundational change for implementing pane splits and workspaces.

## Files Modified
- `src/ghostty/callbacks.rs`
- `src/ghostty/surface.rs`
- `src/main.rs`
- `Cargo.toml`

## Key Decisions
- Used a newtype wrapper `GtkGLAreaPtr` around a raw pointer `*mut ffi::GtkGLArea` with `unsafe impl Send + Sync` to store GTK objects in a global static `Mutex`. This is safe because the pointer is only ever dereferenced on the main GTK thread.
- Used `std::sync::LazyLock` to initialize the `SURFACE_REGISTRY` `HashMap` in a static context.
- Updated the `glib` dependency from `0.20.12` to `0.21.5` to resolve a version conflict that was causing trait implementation errors.
- Made clipboard callback functions in `surface.rs` public (`pub(crate)`) so they could be accessed from `main.rs` after refactoring.

## Deviations from Plan
- **Rule 3 (Auto-fix blocking issue):** The initial implementation in Task 1 failed to compile due to `gtk4::GLArea` not being `Send` or `Sync`, and `HashMap::new()` not being a `const` function. This was fixed by using a raw pointer wrapper and `LazyLock`.
- **Rule 3 (Auto-fix blocking issue):** The build continued to fail due to a version mismatch in the `glib` dependency. This was fixed by updating `Cargo.toml`.
- **Rule 3 (Auto-fix blocking issue):** The build failed due to syntax errors from incorrect chained `Edit` calls. This was fixed by resetting the file and applying the full changes with `Write`.
- **Rule 3 (Auto-fix blocking issue):** The build failed due to incorrect `use` statements and visibility issues after refactoring. This was fixed by making clipboard callbacks public and moving `use` statements to the correct scope.
- **Rule 3 (Auto-fix blocking issue):** The build failed due to a corrupted file state, likely from parallel agent operations. The file was reset to a known good state and changes were re-applied.

## Commits
- c6cec143: refactor(02-workspaces-pane-splits-01): replace single-surface globals with multi-surface registries
- 5f4a6eba: refactor(02-workspaces-pane-splits-01): refactor create_surface for multi-surface use
