use diagram_tool::ui::canvas::math::{safe_zoom, within, screen_to_canvas};

const EPSILON: f64 = f64::EPSILON;

mod safe_zoom_tests {
    use super::*;

    #[test]
    fn given_valid_positive_zoom_when_safe_zoom_then_returns_some() {
        let result = safe_zoom(1.0);
        assert!(result.is_some());
    }

    #[test]
    fn given_zoom_equal_to_epsilon_when_safe_zoom_then_returns_none() {
        let result = safe_zoom(EPSILON);
        assert!(result.is_none());
    }

    #[test]
    fn given_zoom_slightly_above_epsilon_when_safe_zoom_then_returns_some() {
        let result = safe_zoom(EPSILON * 2.0);
        assert!(result.is_some());
    }

    #[test]
    fn given_zoom_slightly_below_epsilon_when_safe_zoom_then_returns_none() {
        let result = safe_zoom(EPSILON * 0.5);
        assert!(result.is_none());
    }

    #[test]
    fn given_zero_zoom_when_safe_zoom_then_returns_none() {
        let result = safe_zoom(0.0);
        assert!(result.is_none());
    }

    #[test]
    fn given_negative_zero_zoom_when_safe_zoom_then_returns_none() {
        let result = safe_zoom(-0.0);
        assert!(result.is_none());
    }

    #[test]
    fn given_negative_zoom_when_safe_zoom_then_returns_none() {
        let result = safe_zoom(-1.0);
        assert!(result.is_none());
    }

    #[test]
    fn given_positive_infinity_zoom_when_safe_zoom_then_returns_none() {
        let result = safe_zoom(f64::INFINITY);
        assert!(result.is_none());
    }

    #[test]
    fn given_negative_infinity_zoom_when_safe_zoom_then_returns_none() {
        let result = safe_zoom(f64::NEG_INFINITY);
        assert!(result.is_none());
    }

    #[test]
    fn given_nan_zoom_when_safe_zoom_then_returns_none() {
        let result = safe_zoom(f64::NAN);
        assert!(result.is_none());
    }

    #[test]
    fn given_min_positive_zoom_when_safe_zoom_then_returns_none() {
        let result = safe_zoom(f64::MIN_POSITIVE);
        assert!(result.is_none());
    }

    #[test]
    fn given_very_small_finite_zoom_when_safe_zoom_then_returns_none() {
        let result = safe_zoom(1e-320);
        assert!(result.is_none());
    }

    #[test]
    fn given_large_finite_zoom_when_safe_zoom_then_returns_some() {
        let result = safe_zoom(1e100);
        assert!(result.is_some());
    }

    #[test]
    fn given_max_finite_zoom_when_safe_zoom_then_returns_some() {
        let result = safe_zoom(f64::MAX);
        assert!(result.is_some());
    }
}

mod within_tests {
    use super::*;

    #[test]
    fn given_node_inside_subgraph_when_within_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 20.0, 20.0);
        let result = within(subgraph, node);
        assert!(result);
    }

    #[test]
    fn given_node_exactly_matches_subgraph_when_within_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 100.0, 100.0);
        let result = within(subgraph, node);
        assert!(result);
    }

    #[test]
    fn given_degenerate_zero_height_subgraph_when_within_then_returns_correctly() {
        let subgraph = (0.0, 0.0, 100.0, 0.0);
        let node = (10.0, 0.0, 20.0, 0.0);
        let result = within(subgraph, node);
        assert!(result);
    }

    #[test]
    fn given_degenerate_zero_width_subgraph_when_within_then_returns_correctly() {
        let subgraph = (0.0, 0.0, 0.0, 100.0);
        let node = (0.0, 10.0, 0.0, 20.0);
        let result = within(subgraph, node);
        assert!(result);
    }

    #[test]
    fn given_nan_in_subgraph_coords_when_within_then_returns_false() {
        let subgraph = (f64::NAN, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 20.0, 20.0);
        let result = within(subgraph, node);
        assert!(!result);
    }

    #[test]
    fn given_nan_in_node_coords_when_within_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (f64::NAN, 10.0, 20.0, 20.0);
        let result = within(subgraph, node);
        assert!(!result);
    }

    #[test]
    fn given_positive_infinity_subgraph_dims_when_within_then_returns_true_for_contained_node() {
        let subgraph = (0.0, 0.0, f64::INFINITY, f64::INFINITY);
        let node = (10.0, 10.0, 20.0, 20.0);
        let result = within(subgraph, node);
        assert!(result);
    }

    #[test]
    fn given_negative_infinity_subgraph_dims_when_within_then_returns_false() {
        let subgraph = (0.0, 0.0, f64::NEG_INFINITY, f64::INFINITY);
        let node = (10.0, 10.0, 20.0, 20.0);
        let result = within(subgraph, node);
        assert!(!result);
    }

    #[test]
    fn given_epsilon_difference_on_right_boundary_when_within_then_returns_correctly() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let exceed_amount = 0.01 * 100.0;
        let node = (0.0, 0.0, 100.0 + exceed_amount, 100.0);
        let result = within(subgraph, node);
        assert!(!result);
    }

    #[test]
    fn given_epsilon_difference_on_bottom_boundary_when_within_then_returns_correctly() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let exceed_amount = 0.01 * 100.0;
        let node = (0.0, 0.0, 100.0, 100.0 + exceed_amount);
        let result = within(subgraph, node);
        assert!(!result);
    }

    #[test]
    fn given_node_exceeds_by_one_ulp_when_within_then_returns_false() {
        let subgraph = (0.0, 0.0, 1.0, 1.0);
        let next_after = f64::from_bits(f64::to_bits(1.0) + 1);
        let node = (0.0, 0.0, next_after, 1.0);
        let result = within(subgraph, node);
        assert!(!result);
    }

    #[test]
    fn given_large_coords_overflow_scenario_when_within_then_no_panic() {
        let subgraph = (f64::MAX / 2.0, f64::MAX / 2.0, f64::MAX / 4.0, f64::MAX / 4.0);
        let node = (f64::MAX / 2.0 + 1.0, f64::MAX / 2.0 + 1.0, 1.0, 1.0);
        let _ = within(subgraph, node);
    }

    #[test]
    fn given_subnormal_dimensions_when_within_then_handles_correctly() {
        let subnormal = f64::MIN_POSITIVE;
        let subgraph = (0.0, 0.0, subnormal * 2.0, subnormal * 2.0);
        let node = (0.0, 0.0, subnormal, subnormal);
        let result = within(subgraph, node);
        assert!(result);
    }

    #[test]
    fn given_negative_subgraph_origin_when_within_then_handles_correctly() {
        let subgraph = (-100.0, -100.0, 50.0, 50.0);
        let node = (-100.0, -100.0, 25.0, 25.0);
        let result = within(subgraph, node);
        assert!(result);
    }

    #[test]
    fn given_node_partially_outside_left_when_within_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (-10.0, 10.0, 50.0, 20.0);
        let result = within(subgraph, node);
        assert!(!result);
    }

    #[test]
    fn given_node_partially_outside_top_when_within_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, -10.0, 20.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result);
    }
}

mod screen_to_canvas_tests {
    use super::*;

    #[test]
    fn given_valid_inputs_when_screen_to_canvas_then_returns_correct_coords() {
        let result = screen_to_canvas(100.0, 200.0, 50.0, 75.0, 2.0);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert_eq!(cx, 100.0);
        assert_eq!(cy, 175.0);
    }

    #[test]
    fn given_zoom_of_one_when_screen_to_canvas_then_returns_client_plus_camera() {
        let result = screen_to_canvas(10.0, 20.0, 5.0, 10.0, 1.0);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert_eq!(cx, 15.0);
        assert_eq!(cy, 30.0);
    }

    #[test]
    fn given_zoom_of_two_when_screen_to_canvas_then_returns_scaled_coords() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, 2.0);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert_eq!(cx, 50.0);
        assert_eq!(cy, 50.0);
    }

    #[test]
    fn given_zero_zoom_when_screen_to_canvas_then_returns_none() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, 0.0);
        assert!(result.is_none());
    }

    #[test]
    fn given_negative_zoom_when_screen_to_canvas_then_returns_none() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, -1.0);
        assert!(result.is_none());
    }

    #[test]
    fn given_nan_zoom_when_screen_to_canvas_then_returns_none() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, f64::NAN);
        assert!(result.is_none());
    }

    #[test]
    fn given_infinite_zoom_when_screen_to_canvas_then_returns_none() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, f64::INFINITY);
        assert!(result.is_none());
    }

    #[test]
    fn given_epsilon_zoom_when_screen_to_canvas_then_returns_none() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, EPSILON);
        assert!(result.is_none());
    }

    #[test]
    fn given_zero_client_coords_when_screen_to_canvas_then_returns_camera() {
        let result = screen_to_canvas(0.0, 0.0, 10.0, 20.0, 1.0);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert_eq!(cx, 10.0);
        assert_eq!(cy, 20.0);
    }

    #[test]
    fn given_negative_camera_coords_when_screen_to_canvas_then_returns_correct_result() {
        let result = screen_to_canvas(100.0, 100.0, -50.0, -50.0, 1.0);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert_eq!(cx, 50.0);
        assert_eq!(cy, 50.0);
    }

    #[test]
    fn given_fractional_zoom_when_screen_to_canvas_then_returns_correct_scaling() {
        let result = screen_to_canvas(50.0, 75.0, 0.0, 0.0, 0.5);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert_eq!(cx, 100.0);
        assert_eq!(cy, 150.0);
    }

    #[test]
    fn given_very_large_zoom_when_screen_to_canvas_then_returns_scaled_coords() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, 1e100);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert!((cx - 0.0).abs() < 1e-50);
        assert!((cy - 0.0).abs() < 1e-50);
    }

    #[test]
    fn given_very_small_valid_zoom_when_screen_to_canvas_then_returns_scaled_coords() {
        let result = screen_to_canvas(1.0, 1.0, 0.0, 0.0, 1e-10);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert_eq!(cx, 1e10);
        assert_eq!(cy, 1e10);
    }

    #[test]
    fn given_subnormal_zoom_when_screen_to_canvas_then_returns_none() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, f64::MIN_POSITIVE);
        assert!(result.is_none());
    }

    #[test]
    fn given_nan_client_coords_when_screen_to_canvas_then_returns_nan() {
        let result = screen_to_canvas(f64::NAN, 100.0, 0.0, 0.0, 1.0);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert!(cx.is_nan());
        assert_eq!(cy, 100.0);
    }

    #[test]
    fn given_nan_camera_coords_when_screen_to_canvas_then_returns_nan() {
        let result = screen_to_canvas(100.0, 100.0, f64::NAN, 0.0, 1.0);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert!(cx.is_nan());
        assert_eq!(cy, 100.0);
    }

    #[test]
    fn given_infinite_camera_coords_when_screen_to_canvas_then_returns_infinite() {
        let result = screen_to_canvas(100.0, 100.0, f64::INFINITY, 0.0, 1.0);
        assert!(result.is_some());
        let (cx, _cy) = result.unwrap();
        assert!(cx.is_infinite());
    }
}

mod integration_edge_cases {
    use super::*;

    #[test]
    fn given_zoom_just_above_epsilon_screen_to_canvas_returns_some() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, EPSILON * 2.0);
        assert!(result.is_some());
    }

    #[test]
    fn given_multiple_operations_chained_then_correct_results() {
        let zoom = safe_zoom(2.0);
        assert!(zoom.is_some());
        
        let coords = screen_to_canvas(100.0, 100.0, 0.0, 0.0, zoom.unwrap());
        assert!(coords.is_some());
        
        let (cx, cy) = coords.unwrap();
        let subgraph = (cx - 10.0, cy - 10.0, 100.0, 100.0);
        let node = (cx, cy, 50.0, 50.0);
        
        let result = within(subgraph, node);
        assert!(result);
    }

    #[test]
    fn given_all_invalid_zoom_values_then_all_return_none() {
        let invalid_zooms = [0.0, -0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, EPSILON, f64::MIN_POSITIVE];
        
        for &zoom in &invalid_zooms {
            let safe_result = safe_zoom(zoom);
            assert!(safe_result.is_none(), "zoom {} should be rejected by safe_zoom", zoom);
            
            let canvas_result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, zoom);
            assert!(canvas_result.is_none(), "zoom {} should be rejected by screen_to_canvas", zoom);
        }
    }

    #[test]
    fn given_all_valid_zoom_values_then_all_return_some() {
        let valid_zooms = [0.1, 0.5, 1.0, 2.0, 10.0, 1e10, f64::MAX];
        
        for &zoom in &valid_zooms {
            let safe_result = safe_zoom(zoom);
            assert!(safe_result.is_some(), "zoom {} should be accepted by safe_zoom", zoom);
            
            let canvas_result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, zoom);
            assert!(canvas_result.is_some(), "zoom {} should be accepted by screen_to_canvas", zoom);
        }
    }

    #[test]
    fn given_very_large_coords_then_no_overflow_in_within() {
        let max_val = f64::MAX / 4.0;
        let subgraph = (0.0, 0.0, max_val, max_val);
        let node = (max_val / 2.0, max_val / 2.0, max_val / 4.0, max_val / 4.0);
        
        let result = within(subgraph, node);
        assert!(result);
    }

    #[test]
    fn given_exact_boundary_conditions_then_within_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        
        let node = (0.0, 0.0, 100.0, 100.0);
        assert!(within(subgraph, node));
        
        let node = (50.0, 50.0, 50.0, 50.0);
        assert!(within(subgraph, node));
        
        let node = (99.999, 99.999, 0.001, 0.001);
        assert!(within(subgraph, node));
    }

    #[test]
    fn given_near_boundary_exceed_then_within_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let exceed_amount = 0.01 * 100.0;
        
        let node = (0.0, 0.0, 100.0 + exceed_amount, 100.0);
        assert!(!within(subgraph, node));
        
        let node = (0.0, 0.0, 100.0, 100.0 + exceed_amount);
        assert!(!within(subgraph, node));
        
        let node = (-0.01, 0.0, 100.0, 100.0);
        assert!(!within(subgraph, node));
        
        let node = (0.0, -0.01, 100.0, 100.0);
        assert!(!within(subgraph, node));
    }
}
