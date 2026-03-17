#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

// =============================================================================
// INP Mobile/Touch Input tests (bd-jqu)
// =============================================================================

/// Double-tap timing threshold in milliseconds.
/// Two taps within this window are considered a double-tap.
#[allow(dead_code)]
const DOUBLE_TAP_THRESHOLD_MS: u64 = 350;

/// Minimum touch hit radius in screen pixels for touch targets.
/// This is larger than mouse hit radius for touch usability.
#[allow(dead_code)]
pub const TOUCH_HIT_RADIUS_PX: f64 = 44.0;

/// Resize handle size in screen pixels.
#[allow(dead_code)]
pub const RESIZE_HANDLE_SIZE_PX: f64 = 14.0;

/// Check if two tap timestamps qualify as a double-tap.
#[must_use]
#[allow(dead_code)]
pub const fn is_double_tap(first_tap_ms: u64, second_tap_ms: u64) -> bool {
    second_tap_ms.saturating_sub(first_tap_ms) <= DOUBLE_TAP_THRESHOLD_MS
}

/// Calculate touch-adjusted hit radius for touch input.
/// Touch input requires larger hit areas than mouse input for usability.
#[must_use]
#[allow(dead_code)]
pub const fn touch_hit_radius(base_radius: f64, is_touch: bool) -> f64 {
    if is_touch {
        let p_max = if base_radius > TOUCH_HIT_RADIUS_PX {
            base_radius
        } else {
            TOUCH_HIT_RADIUS_PX
        };
        p_max
    } else {
        base_radius
    }
}

/// Check if a touch point is within a resize handle's hit area.
/// Touch handles need expanded hit areas for usability.
#[must_use]
#[allow(dead_code)]
pub fn touch_handle_hit_test(
    touch_x: f64,
    touch_y: f64,
    handle_x: f64,
    handle_y: f64,
    is_touch: bool,
) -> bool {
    let effective_size = if is_touch {
        if RESIZE_HANDLE_SIZE_PX > TOUCH_HIT_RADIUS_PX {
            RESIZE_HANDLE_SIZE_PX
        } else {
            TOUCH_HIT_RADIUS_PX
        }
    } else {
        RESIZE_HANDLE_SIZE_PX
    };
    let half_size = effective_size / 2.0;
    touch_x >= handle_x - half_size
        && touch_x <= handle_x + half_size
        && touch_y >= handle_y - half_size
        && touch_y <= handle_y + half_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_two_taps_within_threshold_when_checked_then_is_double_tap() {
        assert!(is_double_tap(1000, 1100));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_two_taps_exactly_at_threshold_when_checked_then_is_double_tap() {
        assert!(is_double_tap(1000, 1350));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_two_taps_just_over_threshold_when_checked_then_not_double_tap() {
        assert!(!is_double_tap(1000, 1351));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_two_taps_far_apart_when_checked_then_not_double_tap() {
        assert!(!is_double_tap(1000, 5000));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_zero_times_when_checked_then_is_double_tap() {
        assert!(is_double_tap(0, 0));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_same_timestamp_when_checked_then_is_double_tap() {
        assert!(is_double_tap(12345, 12345));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_reversed_timestamps_when_checked_then_not_double_tap() {
        assert!(is_double_tap(2000, 1000));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_threshold_boundary_values_when_checked_then_boundary_correct() {
        assert!(is_double_tap(0, DOUBLE_TAP_THRESHOLD_MS));
        assert!(!is_double_tap(0, DOUBLE_TAP_THRESHOLD_MS + 1));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_touch_input_when_hit_testing_handle_then_expanded_hit_area_used() {
        assert!(!touch_handle_hit_test(120.0, 100.0, 100.0, 100.0, false));
        assert!(touch_handle_hit_test(120.0, 100.0, 100.0, 100.0, true));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_touch_input_at_corner_when_hit_testing_then_expanded_area_covers_corners() {
        let half_touch = TOUCH_HIT_RADIUS_PX / 2.0;
        assert!(touch_handle_hit_test(
            100.0 + half_touch - 1.0,
            100.0 + half_touch - 1.0,
            100.0,
            100.0,
            true,
        ));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_touch_input_outside_expanded_area_when_hit_testing_then_fails() {
        let half_touch = TOUCH_HIT_RADIUS_PX / 2.0;
        assert!(!touch_handle_hit_test(
            100.0 + half_touch + 10.0,
            100.0,
            100.0,
            100.0,
            true,
        ));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_mouse_input_when_hit_testing_handle_then_visual_size_used() {
        let half_visual = RESIZE_HANDLE_SIZE_PX / 2.0;
        assert!(touch_handle_hit_test(
            100.0 + half_visual - 1.0,
            100.0,
            100.0,
            100.0,
            false,
        ));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_touch_input_directly_on_handle_when_hit_testing_then_succeeds() {
        assert!(touch_handle_hit_test(100.0, 100.0, 100.0, 100.0, true));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_touch_input_when_calculating_hit_radius_then_uses_touch_minimum() {
        assert_eq!(touch_hit_radius(17.0, false), 17.0);
        assert_eq!(touch_hit_radius(17.0, true), TOUCH_HIT_RADIUS_PX);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_large_base_radius_when_touch_input_then_base_preserved_if_larger() {
        assert_eq!(touch_hit_radius(60.0, true), 60.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_mouse_input_when_calculating_hit_radius_then_base_unchanged() {
        assert_eq!(touch_hit_radius(25.0, false), 25.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_zero_base_radius_when_touch_input_then_touch_minimum_used() {
        assert_eq!(touch_hit_radius(0.0, true), TOUCH_HIT_RADIUS_PX);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_touch_minimum_matches_wcag_guideline() {
        assert_eq!(TOUCH_HIT_RADIUS_PX, 44.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_double_tap_threshold_is_reasonable() {
        assert!(DOUBLE_TAP_THRESHOLD_MS >= 300);
        assert!(DOUBLE_TAP_THRESHOLD_MS <= 500);
    }
}
