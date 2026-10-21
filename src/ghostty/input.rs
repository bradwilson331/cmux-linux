use crate::ghostty::ffi;

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

/// Decide whether a key press that went through the input method (IME) should
/// be swallowed instead of being encoded to the terminal. Mirrors Ghostty's GTK
/// apprt (src/apprt/gtk/class/surface.zig, keyEvent):
///
/// - `im_handled`: `filter_keypress` returned true.
/// - `composing`: the IM is composing after the key (preedit active).
/// - `was_composing`: the IM was composing when the key event started.
/// - `committed_len`: bytes the IM committed during this key event.
///
/// Keys the IM did not handle are always encoded. Handled keys are swallowed
/// while composing (e.g. typing a Hangul syllable), when they ended a
/// composition (the commit already sent the text, e.g. Enter after Japanese
/// input), and when they produced no text (e.g. toggling the IM). A handled
/// key whose commit is plain text (e.g. "a" via the simple IM) is encoded
/// together with that text so Ghostty keeps the keycode.
pub fn im_swallows_key(
    im_handled: bool,
    composing: bool,
    was_composing: bool,
    committed_len: usize,
) -> bool {
    im_handled && (composing || was_composing || committed_len == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_im_swallows_key() {
        // Not handled by the IM (no IM active, or a key it ignores): encode.
        assert!(!im_swallows_key(false, false, false, 0));
        assert!(!im_swallows_key(false, true, true, 0));

        // Typing inside a Hangul syllable: preedit only, nothing to encode.
        assert!(im_swallows_key(true, true, false, 0));
        assert!(im_swallows_key(true, true, true, 3));

        // Key that ended a composition (commit already sent the text).
        assert!(im_swallows_key(true, false, true, 6));

        // IM toggle (e.g. Shift+Space with ibus-hangul): handled, no text.
        assert!(im_swallows_key(true, false, false, 0));

        // Plain key the IM committed as text ("a"): encode with that text.
        assert!(!im_swallows_key(true, false, false, 1));
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
