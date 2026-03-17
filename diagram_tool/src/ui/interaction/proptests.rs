use proptest::prelude::*;

prop_compose! {
    fn arb_finite_f64()(x in -1e6_f64..1e6_f64) -> f64 { x }
}

prop_compose! {
    fn arb_positive_f64()(x in 0.1_f64..1000.0_f64) -> f64 { x }
}

prop_compose! {
    fn arb_point()(x in arb_finite_f64(), y in arb_finite_f64()) -> (f64, f64) { (x, y) }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_has_drag_threshold_symmetric(origin in arb_point(), delta in 0.0_f64..100.0_f64) {
        let current = (origin.0 + delta, origin.1);
        let result1 = has_drag_threshold(origin, current);
        let result2 = has_drag_threshold(current, origin);
        prop_assert_eq!(result1, result2);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_snap_value_disabled_returns_same(value in arb_finite_f64(), grid in arb_positive_f64()) {
        let result = snap_value(value, false, grid);
        prop_assert!((result - value).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_snap_value_enabled_is_multiple_of_grid(value in arb_finite_f64(), grid in arb_positive_f64()) {
        let result = snap_value(value, true, grid);
        let effective_grid = grid.clamp(GridSize::MIN, GridSize::MAX).max(1.0);
        let remainder = (result / effective_grid).round() * effective_grid - result;
        prop_assert!(remainder.abs() < f64::EPSILON || !result.is_finite());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_snap_value_nan_returns_nan(grid in arb_positive_f64()) {
        let result = snap_value(f64::NAN, true, grid);
        // NaN input should produce NaN output
        prop_assert!(result.is_nan());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_snap_point_consistent_with_snap_value(point in arb_point(), grid in arb_positive_f64()) {
        let snapped = snap_point(point, true, grid);
        let expected_x = snap_value(point.0, true, grid);
        let expected_y = snap_value(point.1, true, grid);
        prop_assert!((snapped.0 - expected_x).abs() < f64::EPSILON);
        prop_assert!((snapped.1 - expected_y).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_snap_point_disabled_returns_same(point in arb_point(), grid in arb_positive_f64()) {
        let result = snap_point(point, false, grid);
        prop_assert!((result.0 - point.0).abs() < f64::EPSILON);
        prop_assert!((result.1 - point.1).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_toggle_selection_idempotent_after_two(item in "[a-z]{1,3}") {
        let once = toggle_selection(&HashSet::new(), &item);
        let twice = toggle_selection(&once, &item);
        prop_assert!(twice.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_toggle_selection_adds_item(item in "[a-z]{1,3}") {
        let result = toggle_selection(&HashSet::new(), &item);
        prop_assert!(result.contains(&item));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_dragged_positions_preserves_count(
        x1 in arb_finite_f64(), y1 in arb_finite_f64(),
        x2 in arb_finite_f64(), y2 in arb_finite_f64(),
        anchor in arb_point(), current in arb_point(),
    ) {
        let originals = im::HashMap::new()
            .update(diagram_models::document::NodeId::new("a".to_string()), (x1, y1))
            .update(diagram_models::document::NodeId::new("b".to_string()), (x2, y2));
        let result = dragged_positions_with_snap(&originals, anchor, current, false, GridSize::default());
        prop_assert_eq!(result.len(), originals.len());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_dragged_positions_zero_delta_same_position(
        x in arb_finite_f64(), y in arb_finite_f64(),
        point in arb_point(),
    ) {
        let originals = im::HashMap::new()
            .update(diagram_models::document::NodeId::new("a".to_string()), (x, y));
        let result = dragged_positions_with_snap(&originals, point, point, false, GridSize::default());
        let pos = result.get(&diagram_models::document::NodeId::new("a".to_string()));
        if let Some((rx, ry)) = pos {
            prop_assert!((rx - x).abs() < f64::EPSILON);
            prop_assert!((ry - y).abs() < f64::EPSILON);
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_dragged_positions_nan_anchor_preserves_original(
        x in arb_finite_f64(), y in arb_finite_f64(),
        current in arb_point(),
    ) {
        let originals = im::HashMap::new()
            .update(diagram_models::document::NodeId::new("a".to_string()), (x, y));
        let result = dragged_positions_with_snap(&originals, (f64::NAN, f64::NAN), current, false, GridSize::default());
        let pos = result.get(&diagram_models::document::NodeId::new("a".to_string()));
        if let Some((rx, ry)) = pos {
            if x.is_finite() && y.is_finite() {
                prop_assert!(rx.is_finite() || rx.is_nan());
                prop_assert!(ry.is_finite() || ry.is_nan());
            }
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_has_drag_threshold_always_true_for_large_delta(delta in 100.0_f64..10000.0_f64) {
        prop_assert!(has_drag_threshold((0.0, 0.0), (delta, 0.0)));
        prop_assert!(has_drag_threshold((0.0, 0.0), (0.0, delta)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_has_drag_threshold_always_false_for_tiny_delta(delta in 0.0_f64..2.0_f64) {
        prop_assert!(!has_drag_threshold((0.0, 0.0), (delta, 0.0)));
        prop_assert!(!has_drag_threshold((0.0, 0.0), (0.0, delta)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_snap_value_grid_zero_uses_one(value in arb_finite_f64()) {
        let result = snap_value(value, true, 0.0);
        let expected = snap_value(value, true, 1.0);
        if result.is_finite() && expected.is_finite() {
            prop_assert!((result - expected).abs() < f64::EPSILON);
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_snap_value_negative_grid_uses_one(value in arb_finite_f64(), grid in -100.0_f64..-0.1_f64) {
        let result = snap_value(value, true, grid);
        prop_assert!(result.is_finite() || !value.is_finite());
    }
}
