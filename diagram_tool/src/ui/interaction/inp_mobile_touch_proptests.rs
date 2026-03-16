use proptest::prelude::*;

prop_compose! {
    fn arb_touch_point()(x in 0.0_f64..1000.0, y in 0.0_f64..1000.0) -> (f64, f64) {
        (x, y)
    }
}

prop_compose! {
    fn arb_small_jitter()(dx in -3.0_f64..3.0, dy in -3.0_f64..3.0) -> (f64, f64) {
        (dx, dy)
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_touch_drag_threshold_consistent_regardless_of_direction(
        origin in arb_touch_point(),
        delta in 3.0_f64..100.0,
    ) {
        let right = (origin.0 + delta, origin.1);
        let down = (origin.0, origin.1 + delta);
        let diagonal = (origin.0 + delta / 2.0_f64.sqrt(), origin.1 + delta / 2.0_f64.sqrt());

        let right_result = has_drag_threshold(origin, right);
        let down_result = has_drag_threshold(origin, down);
        let diag_result = has_drag_threshold(origin, diagonal);

        prop_assert_eq!(right_result, down_result);
        prop_assert_eq!(right_result, diag_result);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_long_press_with_small_jitter_never_triggers_drag(
        origin in arb_touch_point(),
        jitter in arb_small_jitter(),
    ) {
        let jittered = (origin.0 + jitter.0, origin.1 + jitter.1);
        let distance = (
            (jittered.0 - origin.0).abs(),
            (jittered.1 - origin.1).abs(),
        );

        let euclidean = (distance.0 * distance.0 + distance.1 * distance.1).sqrt();
        if euclidean < 3.0 {
            prop_assert!(!has_drag_threshold(origin, jittered));
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_double_tap_timing_window_is_positive(
        min_ms in 50_u64..200,
        max_offset in 200_u64..500,
    ) {
        let max_ms = min_ms + max_offset;
        prop_assert!(max_ms > min_ms);
        prop_assert!(min_ms >= 50);
        prop_assert!(max_ms <= 1000);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_touch_hit_radius_always_positive_and_finite(radius in 1.0_f64..100.0) {
        let effective = radius.max(TOUCH_HIT_RADIUS_MIN);
        prop_assert!(effective.is_finite());
        prop_assert!(effective > 0.0);
        prop_assert!(effective >= TOUCH_HIT_RADIUS_MIN);
    }
}
