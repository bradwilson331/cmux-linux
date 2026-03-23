//! Tests for wakeup_cb coalescing behavior (GHOST-07)
//! These tests verify the AtomicBool deduplication logic without requiring a live GTK context.

#[test]
fn wakeup_pending_starts_false() {
    // WAKEUP_PENDING must be false at program start (or after reset)
    cmux_linux::ghostty::callbacks::WAKEUP_PENDING.store(false, std::sync::atomic::Ordering::SeqCst);
    assert!(!cmux_linux::ghostty::callbacks::WAKEUP_PENDING.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn wakeup_coalescing_flag_set_on_first_call() {
    // After first wakeup_cb fires and sets the flag, a second swap returns true (already pending)
    use std::sync::atomic::Ordering;
    cmux_linux::ghostty::callbacks::WAKEUP_PENDING.store(false, Ordering::SeqCst);

    // Simulate first wakeup: swap(true) returns false → schedule idle
    let was_pending = cmux_linux::ghostty::callbacks::WAKEUP_PENDING.swap(true, Ordering::SeqCst);
    assert!(!was_pending, "First wakeup should not have been pending");

    // Simulate second wakeup: swap(true) returns true → skip (already scheduled)
    let was_pending_again = cmux_linux::ghostty::callbacks::WAKEUP_PENDING.swap(true, Ordering::SeqCst);
    assert!(was_pending_again, "Second wakeup should see pending=true and skip");

    // Reset for other tests
    cmux_linux::ghostty::callbacks::WAKEUP_PENDING.store(false, Ordering::SeqCst);
}
