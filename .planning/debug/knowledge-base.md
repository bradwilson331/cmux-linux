# GSD Debug Knowledge Base

Resolved debug sessions. Used by `gsd-debugger` to surface known-pattern hypotheses at the start of new investigations.

---

## c-d-keys-not-working — Wrong keycode type passed to Ghostty FFI (XKB raw vs enum ordinal collision)
- **Date:** 2026-03-23
- **Error patterns:** keys not working, no output, ghostty_input_key_e, keycode, XKB, surface.rs, key_pressed, Backspace, silent, terminal GTK4
- **Root cause:** src/ghostty/surface.rs assigned `map_keycode_to_ghostty(keycode)` (a `ghostty_input_key_e` enum ordinal) to `input.keycode`, but `ghostty_input_key_s.keycode` expects the raw platform XKB scancode. `GHOSTTY_KEY_C=22` collides with XKB code 0x0016 (Backspace), causing 'c' to be silently suppressed by `key_encode.zig` and 'd' to emit a tab character.
- **Fix:** Changed `input.keycode = map_keycode_to_ghostty(keycode)` to `input.keycode = keycode` in both `key_pressed` and `key_released` handlers. Removed the now-unused `map_keycode_to_ghostty` function and its tests from `input.rs`.
- **Files changed:** src/ghostty/surface.rs, src/ghostty/input.rs
---
