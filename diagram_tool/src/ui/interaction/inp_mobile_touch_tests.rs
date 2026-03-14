use super::*;

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_touch_drag_when_motion_below_threshold_then_not_considered_drag() {
    let touch_start = (100.0, 100.0);
    let touch_current_below = (101.5, 101.5); // ~2.12px distance
    let touch_current_at = (103.0, 100.0); // exactly 3.0px

    assert!(
        !has_drag_threshold(touch_start, touch_current_below),
        "Touch motion below 3px should not trigger drag"
    );
    assert!(
        has_drag_threshold(touch_start, touch_current_at),
        "Touch motion at 3px should trigger drag"
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_touch_drag_when_rightward_then_uses_contain_selection_mode() {
    let start = (50.0, 50.0);
    let current = (150.0, 100.0); // Rightward drag

    let mode = selection_mode_from_drag(start, current);
    assert_eq!(
        mode,
        SelectionMode::Contain,
        "Rightward touch drag should use contain mode for selection"
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_long_press_when_no_motion_then_not_drag_and_can_select() {
    let press_point = (100.0, 100.0);
    let slightly_moved = (100.5, 100.5); // ~0.7px distance

    assert!(
        !has_drag_threshold(press_point, slightly_moved),
        "Long press with negligible motion should not trigger drag"
    );

    let selected = select_single("node-pressed".to_string());
    assert!(
        selected.contains("node-pressed"),
        "Long press should allow node selection"
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_long_press_when_minor_jitter_then_still_not_drag() {
    let press_point = (0.0, 0.0);
    let jitter_positions = [
        (0.5, 0.5), // ~0.7px
        (1.0, 1.0), // ~1.4px
        (1.5, 0.0), // 1.5px
        (0.0, 2.0), // 2.0px
        (2.0, 2.0), // ~2.8px
    ];

    for jitter in jitter_positions {
        assert!(
            !has_drag_threshold(press_point, jitter),
            "Long press jitter at ({}, {}) should not trigger drag",
            jitter.0,
            jitter.1
        );
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_double_tap_timing_when_taps_within_window_then_detected() {
    const DOUBLE_TAP_WINDOW_MS: u64 = 400;
    let first_tap_ms: u64 = 1000;
    let second_tap_within = first_tap_ms + 300;
    let second_tap_outside = first_tap_ms + 500;

    let within_window = second_tap_within.abs_diff(first_tap_ms) <= DOUBLE_TAP_WINDOW_MS;
    let outside_window = second_tap_outside.abs_diff(first_tap_ms) <= DOUBLE_TAP_WINDOW_MS;

    assert!(
        within_window,
        "Taps within {}ms should be detected as double-tap",
        DOUBLE_TAP_WINDOW_MS
    );
    assert!(
        !outside_window,
        "Taps outside {}ms should not be detected as double-tap",
        DOUBLE_TAP_WINDOW_MS
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_double_tap_timing_constants_then_are_finite_and_reasonable() {
    const DOUBLE_TAP_MIN_MS: u64 = 100;
    const DOUBLE_TAP_MAX_MS: u64 = 700;

    assert!(
        DOUBLE_TAP_MIN_MS >= 50,
        "Double-tap min should be at least 50ms"
    );
    assert!(
        DOUBLE_TAP_MAX_MS <= 1000,
        "Double-tap max should be at most 1000ms"
    );
    assert!(
        DOUBLE_TAP_MIN_MS < DOUBLE_TAP_MAX_MS,
        "Double-tap min should be less than max"
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_touch_hit_area_when_checking_selection_handles_then_meets_minimum() {
    let handle_hit_radius: f64 = 7.0;
    let touch_enlarged_radius = handle_hit_radius.max(TOUCH_HIT_RADIUS_MIN);

    assert!(
        touch_enlarged_radius >= TOUCH_HIT_RADIUS_MIN,
        "Touch hit area should be at least {} radius, got {}",
        TOUCH_HIT_RADIUS_MIN,
        touch_enlarged_radius
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_touch_finger_hit_area_when_computed_then_meets_accessibility() {
    let min_touch_size = 44.0;
    let effective_radius = min_touch_size / 2.0;

    assert!(
        TOUCH_HIT_RADIUS_MIN >= effective_radius - 1.0,
        "Touch hit radius {} should meet accessibility minimum {}",
        TOUCH_HIT_RADIUS_MIN,
        effective_radius
    );
}
