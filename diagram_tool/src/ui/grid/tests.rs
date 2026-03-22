#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use super::size::*;
use super::snapping::*;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_value_when_creating_grid_size_then_returns_ok() {
        let result = GridSize::new(50.0);
        assert!(result.is_ok());
        assert!((result.unwrap().inner() - 50.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_minimum_value_when_creating_grid_size_then_returns_ok() {
        let result = GridSize::new(10.0);
        assert!(result.is_ok());
        assert!((result.unwrap().inner() - 10.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_maximum_value_when_creating_grid_size_then_returns_ok() {
        let result = GridSize::new(100.0);
        assert!(result.is_ok());
        assert!((result.unwrap().inner() - 100.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_default_when_getting_default_grid_size_then_returns_20() {
        let default = GridSize::default();
        assert!((default.inner() - 20.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_snap_disabled_when_snapping_value_then_returns_value_unchanged() {
        let value = 37.5;
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(value, false, grid_size);
        assert!((result - 37.5).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_snap_enabled_when_snapping_value_then_returns_grid_multiple() {
        let value = 29.0;
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(value, true, grid_size);
        assert!((result - 20.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_point_when_snapping_then_each_coordinate_snapped_independently() {
        let point = (31.0, 49.0);
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_point(point, true, grid_size);
        assert!((result.0 - 40.0).abs() < f64::EPSILON);
        assert!((result.1 - 40.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_json_number_when_deserializing_grid_size_then_succeeds() {
        let json = "25.0";
        let result: Result<GridSize, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        let grid_size = result.unwrap();
        assert!((grid_size.inner() - 25.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_value_below_minimum_when_creating_grid_size_then_returns_out_of_range_error() {
        let result = GridSize::new(5.0);
        assert!(matches!(result, Err(GridError::OutOfRange { .. })));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_value_above_maximum_when_creating_grid_size_then_returns_out_of_range_error() {
        let result = GridSize::new(150.0);
        assert!(matches!(result, Err(GridError::OutOfRange { .. })));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_negative_value_when_creating_grid_size_then_returns_out_of_range_error() {
        let result = GridSize::new(-20.0);
        assert!(matches!(result, Err(GridError::OutOfRange { .. })));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_zero_value_when_creating_grid_size_then_returns_out_of_range_error() {
        let result = GridSize::new(0.0);
        assert!(matches!(result, Err(GridError::OutOfRange { .. })));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_nan_value_when_creating_grid_size_then_returns_not_finite_error() {
        let result = GridSize::new(f64::NAN);
        assert!(matches!(result, Err(GridError::NotFinite { kind }) if kind == NonFiniteKind::NaN));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_positive_infinity_when_creating_grid_size_then_returns_not_finite_error() {
        let result = GridSize::new(f64::INFINITY);
        assert!(matches!(
            result,
            Err(GridError::NotFinite { kind }) if kind == NonFiniteKind::PositiveInfinity
        ));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_negative_infinity_when_creating_grid_size_then_returns_not_finite_error() {
        let result = GridSize::new(f64::NEG_INFINITY);
        assert!(matches!(
            result,
            Err(GridError::NotFinite { kind }) if kind == NonFiniteKind::NegativeInfinity
        ));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_json_string_when_deserializing_grid_size_then_returns_error() {
        let json = r#""twenty""#;
        let result: Result<GridSize, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_out_of_range_json_number_when_deserializing_then_returns_error() {
        let json = "5.0";
        let result: Result<GridSize, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_json_null_when_deserializing_grid_size_then_returns_error() {
        let json = "null";
        let result: Result<GridSize, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_fractional_value_when_creating_grid_size_then_returns_ok() {
        let result = GridSize::new(50.5);
        assert!(result.is_ok());
        assert!((result.unwrap().inner() - 50.5).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_nan_value_when_snapping_then_returns_nan() {
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(f64::NAN, true, grid_size);
        assert!(result.is_nan());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_infinity_value_when_snapping_then_returns_infinity() {
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(f64::INFINITY, true, grid_size);
        assert!(result.is_infinite());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_exact_grid_multiple_when_snapping_then_returns_same_value() {
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(40.0, true, grid_size);
        assert!((result - 40.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_value_midway_between_grid_lines_when_snapping_then_rounds_to_nearest() {
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(30.0, true, grid_size);
        assert!((result - 40.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_negative_value_when_snapping_then_handles_correctly() {
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(-15.0, true, grid_size);
        assert!((result - (-20.0)).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_precondition_p1_range_validation() {
        assert!(GridSize::new(9.9).is_err());
        assert!(GridSize::new(10.0).is_ok());
        assert!(GridSize::new(50.0).is_ok());
        assert!(GridSize::new(100.0).is_ok());
        assert!(GridSize::new(100.1).is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_precondition_p1_finite_validation() {
        assert!(GridSize::new(f64::NAN).is_err());
        assert!(GridSize::new(f64::INFINITY).is_err());
        assert!(GridSize::new(f64::NEG_INFINITY).is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_q1_inner_value_preserved() {
        let value = 42.5;
        let grid_size = GridSize::new(value).unwrap();
        assert!((grid_size.inner() - value).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_q2_snap_disabled_identity() {
        let values = [0.0, -10.0, 37.5, 100.0, f64::NAN, f64::INFINITY];
        let grid_size = GridSize::default();
        for value in values {
            let result = snap_value(value, false, grid_size);
            if value.is_nan() {
                assert!(result.is_nan());
            } else if value.is_infinite() {
                assert!(
                    result.is_infinite() && result.is_sign_positive() == value.is_sign_positive()
                );
            } else {
                assert!((result - value).abs() < 1e-10);
            }
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_q2_snap_enabled_grid_multiple() {
        let grid_size = GridSize::new(20.0).unwrap();
        let grid = grid_size.inner();
        for value in [0.0, 10.0, 20.0, 30.0, 40.0, 50.0] {
            let result = snap_value(value, true, grid_size);
            let remainder = (result / grid).round() * grid - result;
            assert!(remainder.abs() < f64::EPSILON);
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_q4_serialization_format() {
        let grid_size = GridSize::new(25.0).unwrap();
        let json = serde_json::to_string(&grid_size).unwrap();
        assert_eq!(json, "25.0");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_q5_default_value() {
        let default = GridSize::default();
        assert!((default.inner() - 20.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invariant_i1_range_guaranteed() {
        let test_values = [10.0, 20.0, 50.5, 99.9, 100.0];
        for v in test_values {
            let gs = GridSize::new(v).unwrap();
            assert!(gs.inner() >= 10.0);
            assert!(gs.inner() <= 100.0);
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invariant_i2_finite_guaranteed() {
        let gs = GridSize::new(50.0).unwrap();
        assert!(gs.inner().is_finite());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_grid_size_when_serializing_and_deserializing_then_roundtrips() {
        let original = GridSize::new(42.5).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: GridSize = serde_json::from_str(&json).unwrap();
        assert!((parsed.inner() - 42.5).abs() < f64::EPSILON);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod proptests {

    use proptest::prelude::*;

    prop_compose! {
        fn arb_grid_size_value()(x in 10.0_f64..=100.0) -> f64 { x }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_grid_size_invariant_range(value in arb_grid_size_value()) {
            let gs = GridSize::new(value).unwrap();
            prop_assert!(gs.inner() >= 10.0 && gs.inner() <= 100.0);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_snap_idempotency(value in -1000.0_f64..1000.0, grid in arb_grid_size_value()) {
            let gs = GridSize::new(grid).unwrap();
            let snap1 = snap_value(value, true, gs);
            let snap2 = snap_value(snap1, true, gs);
            if snap1.is_finite() && snap2.is_finite() {
                prop_assert!((snap1 - snap2).abs() < f64::EPSILON);
            }
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_snap_grid_alignment(value in -1000.0_f64..1000.0, grid in arb_grid_size_value()) {
            let gs = GridSize::new(grid).unwrap();
            let result = snap_value(value, true, gs);
            if result.is_finite() {
                let effective_grid = grid.max(1.0);
                let remainder = (result / effective_grid).round() * effective_grid - result;
                prop_assert!(remainder.abs() < f64::EPSILON);
            }
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_snap_disabled_identity(value in -1e6_f64..1e6_f64, grid in arb_grid_size_value()) {
            let gs = GridSize::new(grid).unwrap();
            let result = snap_value(value, false, gs);
            if value.is_nan() {
                prop_assert!(result.is_nan());
            } else {
                prop_assert!((result - value).abs() < f64::EPSILON);
            }
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_snap_point_consistent_with_snap_value(
            x in -1000.0_f64..1000.0,
            y in -1000.0_f64..1000.0,
            grid in arb_grid_size_value()
        ) {
            let gs = GridSize::new(grid).unwrap();
            let snapped = snap_point((x, y), true, gs);
            let expected_x = snap_value(x, true, gs);
            let expected_y = snap_value(y, true, gs);
            if expected_x.is_finite() && snapped.0.is_finite() {
                prop_assert!((snapped.0 - expected_x).abs() < f64::EPSILON);
            }
            if expected_y.is_finite() && snapped.1.is_finite() {
                prop_assert!((snapped.1 - expected_y).abs() < f64::EPSILON);
            }
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_serialization_roundtrip(value in arb_grid_size_value()) {
            let gs = GridSize::new(value).unwrap();
            let json = serde_json::to_string(&gs).unwrap();
            let parsed: GridSize = serde_json::from_str(&json).unwrap();
            // Use relative tolerance for floating point comparison
            let diff = (parsed.inner() - value).abs();
            let tolerance = (value.abs() * 1e-10).max(1e-10);
            prop_assert!(diff < tolerance);
        }
    }
}

// =============================================================================
// SNP Snapping tests (bd-lgh)
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod snp_snapping_tests {

    // SNP-1: Snap threshold engages at correct distance
    // Tests that values within the snap threshold of a grid line snap correctly

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_value_near_grid_line_when_snap_enabled_then_snaps_to_grid() {
        let grid = GridSize::new(20.0).unwrap();

        // Value at 9.0 (less than half grid) should snap to 0.0
        let result = snap_value(9.0, true, grid);
        assert!(
            (result - 0.0).abs() < f64::EPSILON,
            "9.0 should snap to 0.0"
        );

        // Value at 11.0 (more than half grid) should snap to 20.0
        let result = snap_value(11.0, true, grid);
        assert!(
            (result - 20.0).abs() < f64::EPSILON,
            "11.0 should snap to 20.0"
        );

        // Value at 10.0 (exactly half) should snap to 20.0 (round half up)
        let result = snap_value(10.0, true, grid);
        assert!(
            (result - 20.0).abs() < f64::EPSILON,
            "10.0 should snap to 20.0"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_value_at_snap_threshold_when_snap_enabled_then_engages_correctly() {
        let grid = GridSize::new(20.0).unwrap();

        // Test threshold boundaries for multiple grid lines
        let test_cases = [
            (0.0, 0.0),   // At origin
            (10.0, 20.0), // Halfway, rounds up
            (19.9, 20.0), // Just under next grid line
            (20.0, 20.0), // Exactly on grid
            (20.1, 20.0), // Just over grid line
            (29.0, 20.0), // Closer to 20 than 40
            (30.0, 40.0), // Halfway, rounds up
            (31.0, 40.0), // Closer to 40 than 20
        ];

        for (input, expected) in test_cases {
            let result = snap_value(input, true, grid);
            assert!(
                (result - expected).abs() < f64::EPSILON,
                "snap_value({}) should be {} but got {}",
                input,
                expected,
                result
            );
        }
    }

    // SNP-2: Drag near edge snaps with guide
    // Tests boundary conditions where snap should engage

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_drag_near_grid_edge_when_snap_enabled_then_snaps_to_nearest_grid() {
        let grid = GridSize::new(20.0).unwrap();

        // Test dragging near grid edges
        let edge_cases = [
            (-0.1, 0.0),    // Just before origin
            (0.1, 0.0),     // Just after origin
            (19.9, 20.0),   // Just before first grid line
            (20.1, 20.0),   // Just after first grid line
            (39.9, 40.0),   // Just before second grid line
            (40.1, 40.0),   // Just after second grid line
            (-10.0, -20.0), // Negative value near grid
            (-20.1, -20.0), // Just after negative grid line
        ];

        for (input, expected) in edge_cases {
            let result = snap_value(input, true, grid);
            assert!(
                (result - expected).abs() < f64::EPSILON,
                "snap_value({}) should be {} but got {}",
                input,
                expected,
                result
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_point_near_grid_intersection_when_snap_enabled_then_snaps_both_axes() {
        let grid = GridSize::new(20.0).unwrap();

        // Test points near grid intersections
        let point_cases = [
            ((9.0, 9.0), (0.0, 0.0)),
            ((11.0, 11.0), (20.0, 20.0)),
            ((10.0, 30.0), (20.0, 40.0)),
            ((-5.0, 15.0), (0.0, 20.0)),
            ((25.0, -8.0), (20.0, 0.0)),
        ];

        for (input, expected) in point_cases {
            let result = snap_point(input, true, grid);
            assert!(
                (result.0 - expected.0).abs() < f64::EPSILON,
                "snap_point({:?}).0 should be {} but got {}",
                input,
                expected.0,
                result.0
            );
            assert!(
                (result.1 - expected.1).abs() < f64::EPSILON,
                "snap_point({:?}).1 should be {} but got {}",
                input,
                expected.1,
                result.1
            );
        }
    }

    // SNP-3: Grid snap multi-select
    // Tests that multiple selected nodes snap correctly when dragged
    // Note: The actual multi-select drag logic is in interaction.rs, but we test
    // the snap_point function that underlies it

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_multiple_offsets_when_snap_enabled_then_all_snap_consistently() {
        let grid = GridSize::new(20.0).unwrap();

        // Simulate dragging multiple nodes with different offsets
        let offsets = [(0.0, 0.0), (100.0, 50.0), (-30.0, 75.0), (15.0, -15.0)];

        // When all are dragged by the same delta (14.0, 26.0), each should snap consistently
        // Delta (14.0, 26.0) with grid 20.0:
        // - 14/20 = 0.7, rounds to 1, so 1*20 = 20.0
        // - 26/20 = 1.3, rounds to 1, so 1*20 = 20.0
        let delta = (14.0, 26.0);
        let snapped_delta = snap_point(delta, true, grid);

        // Delta (14.0, 26.0) should snap to (20.0, 20.0) with grid 20.0
        assert!((snapped_delta.0 - 20.0).abs() < f64::EPSILON);
        assert!((snapped_delta.1 - 20.0).abs() < f64::EPSILON);

        // All offsets should have the same snapped delta applied
        for (ox, oy) in offsets {
            let new_pos = snap_point((ox + delta.0, oy + delta.1), true, grid);
            // The snapped positions should be grid-aligned
            let remainder_x = (new_pos.0 / grid.inner()).round() * grid.inner() - new_pos.0;
            let remainder_y = (new_pos.1 / grid.inner()).round() * grid.inner() - new_pos.1;
            assert!(remainder_x.abs() < f64::EPSILON, "X should be grid-aligned");
            assert!(remainder_y.abs() < f64::EPSILON, "Y should be grid-aligned");
        }
    }

    // SNP-4: Disable snapping free movement
    // Tests that when snap_to_grid is false, values pass through unchanged

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_snap_disabled_when_snapping_then_returns_value_unchanged() {
        let grid = GridSize::new(20.0).unwrap();

        let test_values = [0.0, 10.0, 15.5, 20.0, 37.5, 100.0, -15.0, -50.5, 999.999];

        for value in test_values {
            let result = snap_value(value, false, grid);
            assert!(
                (result - value).abs() < f64::EPSILON,
                "snap_value({}, false) should return {} but got {}",
                value,
                value,
                result
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_snap_disabled_when_snapping_point_then_returns_point_unchanged() {
        let grid = GridSize::new(20.0).unwrap();

        let test_points = [
            (0.0, 0.0),
            (15.5, 27.3),
            (100.0, -50.0),
            (-25.5, -75.5),
            (0.001, 999.999),
        ];

        for (x, y) in test_points {
            let result = snap_point((x, y), false, grid);
            assert!(
                (result.0 - x).abs() < f64::EPSILON,
                "snap_point(({}, {}), false).0 should return {} but got {}",
                x,
                y,
                x,
                result.0
            );
            assert!(
                (result.1 - y).abs() < f64::EPSILON,
                "snap_point(({}, {}), false).1 should return {} but got {}",
                x,
                y,
                y,
                result.1
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_snap_disabled_with_nan_then_returns_nan() {
        let grid = GridSize::new(20.0).unwrap();

        let result = snap_value(f64::NAN, false, grid);
        assert!(
            result.is_nan(),
            "NaN should pass through when snap disabled"
        );

        let result = snap_point((f64::NAN, 10.0), false, grid);
        assert!(result.0.is_nan(), "NaN x should pass through");
        assert!(
            (result.1 - 10.0).abs() < f64::EPSILON,
            "y should pass through"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_snap_disabled_with_infinity_then_returns_infinity() {
        let grid = GridSize::new(20.0).unwrap();

        let result = snap_value(f64::INFINITY, false, grid);
        assert!(result.is_infinite() && result.is_sign_positive());

        let result = snap_value(f64::NEG_INFINITY, false, grid);
        assert!(result.is_infinite() && result.is_sign_negative());
    }

    // SNP-11: Snap tie-break deterministic
    // Tests that values exactly midway between grid lines snap consistently

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_value_exactly_midway_when_snap_enabled_then_deterministic_tie_break() {
        let grid = GridSize::new(20.0).unwrap();

        // Value exactly at 10.0 (midway between 0 and 20)
        // Rust's round() uses "round half to even" (banker's rounding)
        // 10.0 / 20.0 = 0.5, rounds to 0.0 (even), so result is 0.0
        // But wait: (10.0 / 20.0).round() = 1.0 (0.5 rounds to nearest even, which is 0)
        // Actually f64::round() rounds 0.5 away from zero to 1.0
        let result = snap_value(10.0, true, grid);

        // Verify the result is deterministic (either 0 or 20, consistently)
        let expected = if (10.0_f64 / 20.0_f64).round() == 1.0 {
            20.0
        } else {
            0.0
        };
        assert!(
            (result - expected).abs() < f64::EPSILON,
            "Midway value should snap deterministically to {} but got {}",
            expected,
            result
        );

        // Test that the same input always produces the same output
        for _ in 0..100 {
            let r = snap_value(10.0, true, grid);
            assert!(
                (r - result).abs() < f64::EPSILON,
                "Snap should be deterministic"
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_multiple_midway_values_when_snap_enabled_then_all_tie_break_consistently() {
        let grid = GridSize::new(20.0).unwrap();

        // Test midway values at different grid positions
        let midway_values = [10.0, 30.0, 50.0, 70.0, 90.0, -10.0, -30.0];

        for value in midway_values {
            let result1 = snap_value(value, true, grid);
            let result2 = snap_value(value, true, grid);
            let result3 = snap_value(value, true, grid);

            assert!(
                (result1 - result2).abs() < f64::EPSILON
                    && (result2 - result3).abs() < f64::EPSILON,
                "Snap should be deterministic for midway value {}: got {}, {}, {}",
                value,
                result1,
                result2,
                result3
            );

            // Verify result is a grid multiple
            let remainder = (result1 / grid.inner()).round() * grid.inner() - result1;
            assert!(
                remainder.abs() < f64::EPSILON,
                "Result should be grid multiple"
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_point_at_grid_center_when_snap_enabled_then_deterministic() {
        let grid = GridSize::new(20.0).unwrap();

        // Point exactly at grid cell center
        let center_points = [(10.0, 10.0), (30.0, 10.0), (10.0, 30.0), (30.0, 30.0)];

        for (x, y) in center_points {
            let result1 = snap_point((x, y), true, grid);
            let result2 = snap_point((x, y), true, grid);

            assert!(
                (result1.0 - result2.0).abs() < f64::EPSILON,
                "X snap should be deterministic for ({}, {})",
                x,
                y
            );
            assert!(
                (result1.1 - result2.1).abs() < f64::EPSILON,
                "Y snap should be deterministic for ({}, {})",
                x,
                y
            );
        }
    }

    // Additional edge case tests for completeness

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_very_small_grid_when_snap_enabled_then_still_snaps() {
        let grid = GridSize::new(10.0).unwrap(); // Minimum grid size

        let result = snap_value(5.0, true, grid);
        assert!((result - 10.0).abs() < f64::EPSILON, "Should snap to 10.0");

        let result = snap_value(4.9, true, grid);
        assert!((result - 0.0).abs() < f64::EPSILON, "Should snap to 0.0");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_very_large_grid_when_snap_enabled_then_still_snaps() {
        let grid = GridSize::new(100.0).unwrap(); // Maximum grid size

        let result = snap_value(50.0, true, grid);
        assert!(
            (result - 100.0).abs() < f64::EPSILON,
            "Should snap to 100.0"
        );

        let result = snap_value(49.0, true, grid);
        assert!((result - 0.0).abs() < f64::EPSILON, "Should snap to 0.0");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_zero_value_when_snap_enabled_then_returns_zero() {
        let grid = GridSize::new(20.0).unwrap();

        let result = snap_value(0.0, true, grid);
        assert!((result - 0.0).abs() < f64::EPSILON);

        let result = snap_point((0.0, 0.0), true, grid);
        assert!((result.0 - 0.0).abs() < f64::EPSILON);
        assert!((result.1 - 0.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_very_large_value_when_snap_enabled_then_snaps_correctly() {
        let grid = GridSize::new(20.0).unwrap();

        let result = snap_value(1_000_010.0, true, grid);
        assert!(
            (result - 1_000_020.0).abs() < f64::EPSILON,
            "Large value should snap correctly"
        );

        let result = snap_value(-1_000_010.0, true, grid);
        assert!(
            (result - (-1_000_020.0)).abs() < f64::EPSILON,
            "Large negative value should snap correctly"
        );
    }
}
