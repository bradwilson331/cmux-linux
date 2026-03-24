---
status: resolved
trigger: "c and d keys do not work in Ghostty terminal GTK4 app"
created: 2026-03-23T00:00:00Z
updated: 2026-03-23T00:00:00Z
---

## Current Focus

hypothesis: CONFIRMED — surface.rs assigns ghostty_input_key_e enum value to keycode field instead of raw XKB keycode
test: traced full pipeline from GTK event → Rust FFI → Ghostty Zig lookup → key_encode
expecting: fix by passing raw GTK hardware keycode directly to input.keycode
next_action: apply fix in surface.rs and remove dead map_keycode_to_ghostty usage

## Symptoms

expected: Pressing 'c' or 'd' produces those characters in the terminal (or triggers ctrl+c / ctrl+d when combined with Ctrl)
actual: The 'c' and 'd' keys produce no output / do nothing
errors: None reported
reproduction: Run the app with `cargo run`, click terminal window to focus, type 'c' or 'd'
started: Discovered during Phase 1 human verification — keys were never confirmed working

## Eliminated

- hypothesis: GDK key intercept (Ctrl+C/D handled by GTK before key_pressed)
  evidence: key_pressed handler returns Propagation::Stop for all keys; GTK cannot intercept
  timestamp: 2026-03-23

- hypothesis: Missing entry in keycode map for c/d keycodes
  evidence: Keycodes 54 (C) and 40 (D) ARE present in input.rs map; the map itself is not the bug
  timestamp: 2026-03-23

## Evidence

- timestamp: 2026-03-23
  checked: ghostty/src/input/keycodes.zig — XKB native keycodes for letter keys
  found: KeyC native=0x0036=54, KeyD native=0x0028=40 on Linux (native_idx=2 for xkb)
  implication: Ghostty expects raw XKB keycodes in the keycode field, not enum values

- timestamp: 2026-03-23
  checked: ghostty/src/apprt/embedded.zig App.KeyEvent.core() function
  found: loops over keycodes.entries looking for entry.native == self.keycode to identify physical_key
  implication: keycode field MUST be the raw platform native code, not a translated enum

- timestamp: 2026-03-23
  checked: src/ghostty/surface.rs key_pressed handler — input.keycode assignment
  found: input.keycode = map_keycode_to_ghostty(keycode) assigns ghostty_input_key_e (e.g. GHOSTTY_KEY_C=22) instead of raw keycode (54)
  implication: 'C' press sends keycode=22 → Ghostty looks up XKB 22 → finds Backspace key

- timestamp: 2026-03-23
  checked: ghostty/src/input/key_encode.zig kitty() encoder, lines 161-164
  found: when event.key=.backspace AND utf8 is non-control text: `if comptime tag == .backspace) return;` — returns without writing any output
  implication: 'C' press (mis-identified as Backspace key) produces no output, exactly matching the symptom

- timestamp: 2026-03-23
  checked: ghostty/src/input/key_encode.zig kitty() encoder line 184
  found: when event.key=.tab with no mods: outputs '\t' (tab character)
  implication: 'D' press (mis-identified as Tab key, keycode=23=0x0017) outputs a tab instead of 'd'

- timestamp: 2026-03-23
  checked: ghostty_input_key_s struct definition (ghostty.h + bindgen output)
  found: keycode field is uint32_t / u32 — same type as ghostty_input_key_e — so Rust compiles without type error
  implication: the wrong assignment is invisible to the type checker

## Resolution

root_cause: src/ghostty/surface.rs assigns the result of map_keycode_to_ghostty() (a ghostty_input_key_e enum ordinal) to input.keycode, but ghostty_input_key_s.keycode expects the raw platform XKB keycode. GHOSTTY_KEY_C=22 happens to equal the XKB code for Backspace (0x0016), so pressing 'C' is processed as a Backspace keystroke and produces no output (key_encode.zig explicitly suppresses Backspace with non-control text).
fix: changed `input.keycode = map_keycode_to_ghostty(keycode)` to `input.keycode = keycode` in both key_pressed and key_released handlers. Removed the now-unused map_keycode_to_ghostty function and its tests from input.rs.
verification: cargo build succeeds cleanly. Human confirmed fix works at runtime — 'c' and 'd' keys now produce correct output.
files_changed: [src/ghostty/surface.rs, src/ghostty/input.rs]
