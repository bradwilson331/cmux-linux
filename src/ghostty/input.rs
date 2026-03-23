use crate::ghostty::ffi;

/// Maps a GDK hardware keycode (X11 scancode, the raw hardware key number)
/// to a ghostty_input_key_e value.
///
/// GDK hardware keycodes are X11 keycodes which are scancode + 8.
/// The mapping here covers the common QWERTY layout keys that appear in
/// ghostty_input_key_e. Unknown keycodes return GHOSTTY_KEY_UNIDENTIFIED.
///
/// Note: this is NOT the GDK keyval — do not pass GDK_KEY_a (0x0061) here.
/// Pass the hardware `keycode` field from GtkEventControllerKey::connect_key_pressed.
pub fn map_keycode_to_ghostty(keycode: u32) -> ffi::ghostty_input_key_e {
    // X11 keycodes: scancode = keycode - 8
    // Common mappings (QWERTY, US layout — layout-independent at hardware level):
    match keycode {
        9 => ffi::ghostty_input_key_e_GHOSTTY_KEY_ESCAPE,
        10 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DIGIT_1,
        11 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DIGIT_2,
        12 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DIGIT_3,
        13 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DIGIT_4,
        14 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DIGIT_5,
        15 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DIGIT_6,
        16 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DIGIT_7,
        17 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DIGIT_8,
        18 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DIGIT_9,
        19 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DIGIT_0,
        20 => ffi::ghostty_input_key_e_GHOSTTY_KEY_MINUS,
        21 => ffi::ghostty_input_key_e_GHOSTTY_KEY_EQUAL,
        22 => ffi::ghostty_input_key_e_GHOSTTY_KEY_BACKSPACE,
        23 => ffi::ghostty_input_key_e_GHOSTTY_KEY_TAB,
        24 => ffi::ghostty_input_key_e_GHOSTTY_KEY_Q,
        25 => ffi::ghostty_input_key_e_GHOSTTY_KEY_W,
        26 => ffi::ghostty_input_key_e_GHOSTTY_KEY_E,
        27 => ffi::ghostty_input_key_e_GHOSTTY_KEY_R,
        28 => ffi::ghostty_input_key_e_GHOSTTY_KEY_T,
        29 => ffi::ghostty_input_key_e_GHOSTTY_KEY_Y,
        30 => ffi::ghostty_input_key_e_GHOSTTY_KEY_U,
        31 => ffi::ghostty_input_key_e_GHOSTTY_KEY_I,
        32 => ffi::ghostty_input_key_e_GHOSTTY_KEY_O,
        33 => ffi::ghostty_input_key_e_GHOSTTY_KEY_P,
        34 => ffi::ghostty_input_key_e_GHOSTTY_KEY_BRACKET_LEFT,
        35 => ffi::ghostty_input_key_e_GHOSTTY_KEY_BRACKET_RIGHT,
        36 => ffi::ghostty_input_key_e_GHOSTTY_KEY_ENTER,
        37 => ffi::ghostty_input_key_e_GHOSTTY_KEY_CONTROL_LEFT,
        38 => ffi::ghostty_input_key_e_GHOSTTY_KEY_A,
        39 => ffi::ghostty_input_key_e_GHOSTTY_KEY_S,
        40 => ffi::ghostty_input_key_e_GHOSTTY_KEY_D,
        41 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F,
        42 => ffi::ghostty_input_key_e_GHOSTTY_KEY_G,
        43 => ffi::ghostty_input_key_e_GHOSTTY_KEY_H,
        44 => ffi::ghostty_input_key_e_GHOSTTY_KEY_J,
        45 => ffi::ghostty_input_key_e_GHOSTTY_KEY_K,
        46 => ffi::ghostty_input_key_e_GHOSTTY_KEY_L,
        47 => ffi::ghostty_input_key_e_GHOSTTY_KEY_SEMICOLON,
        48 => ffi::ghostty_input_key_e_GHOSTTY_KEY_QUOTE,
        49 => ffi::ghostty_input_key_e_GHOSTTY_KEY_BACKQUOTE,
        50 => ffi::ghostty_input_key_e_GHOSTTY_KEY_SHIFT_LEFT,
        51 => ffi::ghostty_input_key_e_GHOSTTY_KEY_BACKSLASH,
        52 => ffi::ghostty_input_key_e_GHOSTTY_KEY_Z,
        53 => ffi::ghostty_input_key_e_GHOSTTY_KEY_X,
        54 => ffi::ghostty_input_key_e_GHOSTTY_KEY_C,
        55 => ffi::ghostty_input_key_e_GHOSTTY_KEY_V,
        56 => ffi::ghostty_input_key_e_GHOSTTY_KEY_B,
        57 => ffi::ghostty_input_key_e_GHOSTTY_KEY_N,
        58 => ffi::ghostty_input_key_e_GHOSTTY_KEY_M,
        59 => ffi::ghostty_input_key_e_GHOSTTY_KEY_COMMA,
        60 => ffi::ghostty_input_key_e_GHOSTTY_KEY_PERIOD,
        61 => ffi::ghostty_input_key_e_GHOSTTY_KEY_SLASH,
        62 => ffi::ghostty_input_key_e_GHOSTTY_KEY_SHIFT_RIGHT,
        63 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_MULTIPLY, // Keypad *
        64 => ffi::ghostty_input_key_e_GHOSTTY_KEY_ALT_LEFT,
        65 => ffi::ghostty_input_key_e_GHOSTTY_KEY_SPACE,
        66 => ffi::ghostty_input_key_e_GHOSTTY_KEY_CAPS_LOCK,
        67 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F1,
        68 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F2,
        69 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F3,
        70 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F4,
        71 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F5,
        72 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F6,
        73 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F7,
        74 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F8,
        75 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F9,
        76 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F10,
        // 77 => Num Lock
        // 78 => Scroll Lock
        79 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_7,
        80 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_8,
        81 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_9,
        82 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_SUBTRACT,
        83 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_4,
        84 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_5,
        85 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_6,
        86 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_ADD,
        87 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_1,
        88 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_2,
        89 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_3,
        90 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_0,
        91 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_DECIMAL,
        95 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F11,
        96 => ffi::ghostty_input_key_e_GHOSTTY_KEY_F12,
        104 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_ENTER,
        105 => ffi::ghostty_input_key_e_GHOSTTY_KEY_CONTROL_RIGHT,
        106 => ffi::ghostty_input_key_e_GHOSTTY_KEY_NUMPAD_DIVIDE,
        // 107 => Print Screen
        108 => ffi::ghostty_input_key_e_GHOSTTY_KEY_ALT_RIGHT,
        110 => ffi::ghostty_input_key_e_GHOSTTY_KEY_HOME,
        111 => ffi::ghostty_input_key_e_GHOSTTY_KEY_ARROW_UP,
        112 => ffi::ghostty_input_key_e_GHOSTTY_KEY_PAGE_UP,
        113 => ffi::ghostty_input_key_e_GHOSTTY_KEY_ARROW_LEFT,
        114 => ffi::ghostty_input_key_e_GHOSTTY_KEY_ARROW_RIGHT,
        115 => ffi::ghostty_input_key_e_GHOSTTY_KEY_END,
        116 => ffi::ghostty_input_key_e_GHOSTTY_KEY_ARROW_DOWN,
        117 => ffi::ghostty_input_key_e_GHOSTTY_KEY_PAGE_DOWN,
        118 => ffi::ghostty_input_key_e_GHOSTTY_KEY_INSERT,
        119 => ffi::ghostty_input_key_e_GHOSTTY_KEY_DELETE,
        133 => ffi::ghostty_input_key_e_GHOSTTY_KEY_META_LEFT, // Left Windows/Super key
        134 => ffi::ghostty_input_key_e_GHOSTTY_KEY_META_RIGHT, // Right Windows/Super key
        135 => ffi::ghostty_input_key_e_GHOSTTY_KEY_CONTEXT_MENU, // Menu key
        _ => ffi::ghostty_input_key_e_GHOSTTY_KEY_UNIDENTIFIED,
    }
}

/// Maps GDK modifier state (gdk4::ModifierType bits) to ghostty_input_mods_e.
/// Returns 0 if no modifiers.
pub fn map_mods(state: gtk4::gdk::ModifierType) -> ffi::ghostty_input_mods_e {
    let mut mods: ffi::ghostty_input_mods_e = 0;
    use gtk4::gdk::ModifierType;
    if state.contains(ModifierType::SHIFT_MASK) {
        mods |= ffi::ghostty_input_mods_e_GHOSTTY_MODS_SHIFT;
    }
    if state.contains(ModifierType::CONTROL_MASK) {
        mods |= ffi::ghostty_input_mods_e_GHOSTTY_MODS_CTRL;
    }
    if state.contains(ModifierType::ALT_MASK) {
        mods |= ffi::ghostty_input_mods_e_GHOSTTY_MODS_ALT;
    }
    if state.contains(ModifierType::SUPER_MASK) {
        mods |= ffi::ghostty_input_mods_e_GHOSTTY_MODS_SUPER;
    }
    mods
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keycode_a_maps_to_ghostty_a() {
        let result = map_keycode_to_ghostty(38);
        // Must not be GHOSTTY_KEY_UNIDENTIFIED
        assert_ne!(
            result,
            ffi::ghostty_input_key_e_GHOSTTY_KEY_UNIDENTIFIED,
            "keycode 38 (hardware 'a') must map to a valid key"
        );
        // Should map to A specifically
        assert_eq!(
            result,
            ffi::ghostty_input_key_e_GHOSTTY_KEY_A,
            "keycode 38 must map to GHOSTTY_KEY_A"
        );
    }

    #[test]
    fn keycode_return_maps_to_enter() {
        let result = map_keycode_to_ghostty(36);
        assert_eq!(
            result,
            ffi::ghostty_input_key_e_GHOSTTY_KEY_ENTER,
            "keycode 36 (Return) must map to GHOSTTY_KEY_ENTER"
        );
    }

    #[test]
    fn keycode_escape_maps_correctly() {
        let result = map_keycode_to_ghostty(9);
        assert_eq!(
            result,
            ffi::ghostty_input_key_e_GHOSTTY_KEY_ESCAPE,
            "keycode 9 (Escape) must map to GHOSTTY_KEY_ESCAPE"
        );
    }

    #[test]
    fn unknown_keycode_returns_invalid() {
        let result = map_keycode_to_ghostty(0);
        assert_eq!(
            result,
            ffi::ghostty_input_key_e_GHOSTTY_KEY_UNIDENTIFIED,
            "unknown keycode must map to GHOSTTY_KEY_UNIDENTIFIED without panic"
        );
    }

    #[test]
    fn test_map_mods() {
        use gtk4::gdk::ModifierType;

        // Test shift
        let shift = map_mods(ModifierType::SHIFT_MASK);
        assert_eq!(shift, ffi::ghostty_input_mods_e_GHOSTTY_MODS_SHIFT);

        // Test control
        let ctrl = map_mods(ModifierType::CONTROL_MASK);
        assert_eq!(ctrl, ffi::ghostty_input_mods_e_GHOSTTY_MODS_CTRL);

        // Test combined
        let combined = map_mods(ModifierType::SHIFT_MASK | ModifierType::CONTROL_MASK);
        assert_eq!(
            combined,
            ffi::ghostty_input_mods_e_GHOSTTY_MODS_SHIFT
                | ffi::ghostty_input_mods_e_GHOSTTY_MODS_CTRL,
            "Combined modifiers must have both bits set"
        );
    }
}
