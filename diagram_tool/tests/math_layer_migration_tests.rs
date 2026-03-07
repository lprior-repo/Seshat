use diagram_tool::ui::canvas::math::{safe_zoom, within, screen_to_canvas};

mod safe_zoom_tests {
    use super::*;

    #[test]
    fn given_valid_zoom_when_within_epsilon_boundary_then_returns_some() {
        let result = safe_zoom(f64::EPSILON);
        assert!(result.is_some(), "Expected Some for zoom = EPSILON");
    }

    #[test]
    fn given_zoom_just_above_epsilon_when_validating_then_returns_some() {
        let result = safe_zoom(f64::EPSILON * 2.0);
        assert!(result.is_some(), "Expected Some for zoom > EPSILON");
    }

    #[test]
    fn given_zoom_just_below_epsilon_when_validating_then_returns_none() {
        let result = safe_zoom(f64::EPSILON * 0.5);
        assert!(result.is_none(), "Expected None for zoom < EPSILON");
    }

    #[test]
    fn given_zero_zoom_when_validating_then_returns_none() {
        let result = safe_zoom(0.0);
        assert!(result.is_none(), "Expected None for zero zoom");
    }

    #[test]
    fn given_negative_zero_zoom_when_validating_then_returns_none() {
        let result = safe_zoom(-0.0);
        assert!(result.is_none(), "Expected None for negative zero");
    }

    #[test]
    fn given_positive_zoom_when_validating_then_returns_some() {
        let result = safe_zoom(1.0);
        assert!(result.is_some(), "Expected Some for positive zoom");
    }

    #[test]
    fn given_negative_zoom_when_validating_then_returns_none() {
        let result = safe_zoom(-1.0);
        assert!(result.is_none(), "Expected None for negative zoom");
    }

    #[test]
    fn given_nan_zoom_when_validating_then_returns_none() {
        let result = safe_zoom(f64::NAN);
        assert!(result.is_none(), "Expected None for NaN");
    }

    #[test]
    fn given_positive_infinity_zoom_when_validating_then_returns_none() {
        let result = safe_zoom(f64::INFINITY);
        assert!(result.is_none(), "Expected None for positive infinity");
    }

    #[test]
    fn given_negative_infinity_zoom_when_validating_then_returns_none() {
        let result = safe_zoom(f64::NEG_INFINITY);
        assert!(result.is_none(), "Expected None for negative infinity");
    }

    #[test]
    fn given_min_positive_zoom_when_validating_then_returns_none() {
        let result = safe_zoom(f64::MIN_POSITIVE);
        assert!(result.is_none(), "Expected None for MIN_POSITIVE (smaller than EPSILON)");
    }

    #[test]
    fn given_very_large_zoom_when_validating_then_returns_some() {
        let result = safe_zoom(f64::MAX);
        assert!(result.is_some(), "Expected Some for MAX value");
    }

    #[test]
    fn given_subnormal_zoom_when_validating_then_returns_none() {
        let subnormal = f64::MIN_POSITIVE / 2.0;
        assert!(subnormal.is_subnormal());
        let result = safe_zoom(subnormal);
        assert!(result.is_none(), "Expected None for subnormal value");
    }
}

mod within_tests {
    use super::*;

    #[test]
    fn given_node_within_subgraph_when_checking_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected node to be within subgraph");
    }

    #[test]
    fn given_node_outside_subgraph_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (90.0, 90.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected node to be outside subgraph");
    }

    #[test]
    fn given_node_exactly_on_subgraph_boundary_when_checking_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 100.0, 100.0);
        let result = within(subgraph, node);
        assert!(result, "Expected node on boundary to be within");
    }

    #[test]
    fn given_node_exceeds_subgraph_by_one_pixel_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 100.0 + f64::EPSILON, 100.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected node exceeding boundary to be outside");
    }

    #[test]
    fn given_node_on_left_boundary_when_checking_then_returns_true() {
        let subgraph = (10.0, 0.0, 100.0, 100.0);
        let node = (10.0, 20.0, 30.0, 30.0);
        let result = within(subgraph, node);
        assert!(result, "Expected node on left boundary to be within");
    }

    #[test]
    fn given_node_on_top_boundary_when_checking_then_returns_true() {
        let subgraph = (0.0, 10.0, 100.0, 100.0);
        let node = (20.0, 10.0, 30.0, 30.0);
        let result = within(subgraph, node);
        assert!(result, "Expected node on top boundary to be within");
    }

    #[test]
    fn given_node_extends_beyond_right_boundary_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 0.0, 60.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected node extending beyond right to be outside");
    }

    #[test]
    fn given_node_extends_beyond_bottom_boundary_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 50.0, 50.0, 60.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected node extending beyond bottom to be outside");
    }

    #[test]
    fn given_degenerate_zero_width_subgraph_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 0.0, 100.0);
        let node = (0.0, 0.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for zero-width subgraph");
    }

    #[test]
    fn given_degenerate_zero_height_subgraph_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 0.0);
        let node = (0.0, 0.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for zero-height subgraph");
    }

    #[test]
    fn given_degenerate_zero_width_node_when_checking_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, 0.0, 10.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true for zero-width node within");
    }

    #[test]
    fn given_degenerate_zero_height_node_when_checking_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, 10.0, 0.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true for zero-height node within");
    }

    #[test]
    fn given_negative_subgraph_dimensions_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, -50.0, -50.0);
        let node = (0.0, 0.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for negative dimensions");
    }

    #[test]
    fn given_negative_node_dimensions_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (-10.0, -10.0, -5.0, -5.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for negative node dimensions");
    }

    #[test]
    fn given_nan_in_subgraph_x_when_checking_then_returns_false() {
        let subgraph = (f64::NAN, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for NaN in subgraph");
    }

    #[test]
    fn given_nan_in_subgraph_y_when_checking_then_returns_false() {
        let subgraph = (0.0, f64::NAN, 100.0, 100.0);
        let node = (10.0, 10.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for NaN in subgraph y");
    }

    #[test]
    fn given_nan_in_subgraph_width_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, f64::NAN, 100.0);
        let node = (10.0, 10.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for NaN in subgraph width");
    }

    #[test]
    fn given_nan_in_subgraph_height_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, f64::NAN);
        let node = (10.0, 10.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for NaN in subgraph height");
    }

    #[test]
    fn given_nan_in_node_x_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (f64::NAN, 10.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for NaN in node x");
    }

    #[test]
    fn given_nan_in_node_y_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, f64::NAN, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for NaN in node y");
    }

    #[test]
    fn given_nan_in_node_width_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, f64::NAN, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for NaN in node width");
    }

    #[test]
    fn given_nan_in_node_height_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 10.0, f64::NAN);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for NaN in node height");
    }

    #[test]
    fn given_infinity_in_subgraph_width_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, f64::INFINITY, 100.0);
        let node = (10.0, 10.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for infinity in subgraph");
    }

    #[test]
    fn given_negative_infinity_in_subgraph_width_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, f64::NEG_INFINITY, 100.0);
        let node = (10.0, 10.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for negative infinity in subgraph");
    }

    #[test]
    fn given_infinity_in_node_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, f64::INFINITY, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for infinity in node");
    }

    #[test]
    fn given_subnormal_dimensions_in_subgraph_when_checking_then_returns_correct() {
        let subnormal = f64::MIN_POSITIVE / 2.0;
        assert!(subnormal.is_subnormal());
        let subgraph = (0.0, 0.0, subnormal, subnormal);
        let node = (0.0, 0.0, subnormal / 2.0, subnormal / 2.0);
        let result = within(subgraph, node);
        assert!(result, "Expected subnormal dimensions to work");
    }

    #[test]
    fn given_overflow_values_in_subgraph_when_checking_then_returns_false() {
        let subgraph = (f64::MAX, f64::MAX, f64::MAX, f64::MAX);
        let node = (0.0, 0.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected overflow values to return false");
    }

    #[test]
    fn given_very_large_finite_dimensions_when_checking_then_returns_correct() {
        let large = f64::MAX / 4.0;
        let subgraph = (0.0, 0.0, large, large);
        let node = (0.0, 0.0, large / 2.0, large / 2.0);
        let result = within(subgraph, node);
        assert!(result, "Expected large finite dimensions to work");
    }

    #[test]
    fn given_node_starts_before_subgraph_when_checking_then_returns_false() {
        let subgraph = (50.0, 50.0, 100.0, 100.0);
        let node = (40.0, 60.0, 20.0, 20.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node starts before subgraph");
    }

    #[test]
    fn given_node_starts_above_subgraph_when_checking_then_returns_false() {
        let subgraph = (50.0, 50.0, 100.0, 100.0);
        let node = (60.0, 40.0, 20.0, 20.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node starts above subgraph");
    }

    #[test]
    fn given_exact_boundary_epsilon_difference_when_checking_then_returns_correct() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 100.0 - f64::EPSILON, 100.0 - f64::EPSILON);
        let result = within(subgraph, node);
        assert!(result, "Expected node within by epsilon to be inside");
    }

    #[test]
    fn given_epsilon_beyond_boundary_when_checking_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 100.0 + f64::EPSILON * 10.0, 100.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected node beyond by epsilon to be outside");
    }

    #[test]
    fn given_offset_coordinates_in_both_axis_when_checking_then_returns_correct() {
        let subgraph = (100.0, 200.0, 300.0, 400.0);
        let node = (150.0, 250.0, 100.0, 100.0);
        let result = within(subgraph, node);
        assert!(result, "Expected node within offset subgraph");
    }
}

mod screen_to_canvas_tests {
    use super::*;

    #[test]
    fn given_valid_inputs_when_converting_then_returns_canvas_coordinates() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 1.0);
        assert_eq!(result, Some((100.0, 200.0)));
    }

    #[test]
    fn given_zoom_of_two_when_converting_then_scales_coordinates() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 2.0);
        assert_eq!(result, Some((50.0, 100.0)));
    }

    #[test]
    fn given_camera_offset_when_converting_then_applies_offset() {
        let result = screen_to_canvas(0.0, 0.0, 50.0, 75.0, 1.0);
        assert_eq!(result, Some((50.0, 75.0)));
    }

    #[test]
    fn given_zoom_and_camera_when_converting_then_combines_both() {
        let result = screen_to_canvas(100.0, 200.0, 50.0, 75.0, 2.0);
        assert_eq!(result, Some((100.0, 175.0)));
    }

    #[test]
    fn given_zero_zoom_when_converting_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 0.0);
        assert!(result.is_none(), "Expected None for zero zoom");
    }

    #[test]
    fn given_negative_zoom_when_converting_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, -1.0);
        assert!(result.is_none(), "Expected None for negative zoom");
    }

    #[test]
    fn given_nan_zoom_when_converting_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::NAN);
        assert!(result.is_none(), "Expected None for NaN zoom");
    }

    #[test]
    fn given_infinity_zoom_when_converting_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::INFINITY);
        assert!(result.is_none(), "Expected None for infinity zoom");
    }

    #[test]
    fn given_nan_client_x_when_converting_then_returns_none() {
        let result = screen_to_canvas(f64::NAN, 200.0, 0.0, 0.0, 1.0);
        assert!(result.is_none(), "Expected None for NaN client x");
    }

    #[test]
    fn given_nan_client_y_when_converting_then_returns_none() {
        let result = screen_to_canvas(100.0, f64::NAN, 0.0, 0.0, 1.0);
        assert!(result.is_none(), "Expected None for NaN client y");
    }

    #[test]
    fn given_infinity_client_x_when_converting_then_returns_none() {
        let result = screen_to_canvas(f64::INFINITY, 200.0, 0.0, 0.0, 1.0);
        assert!(result.is_none(), "Expected None for infinity client x");
    }

    #[test]
    fn given_negative_infinity_client_x_when_converting_then_returns_none() {
        let result = screen_to_canvas(f64::NEG_INFINITY, 200.0, 0.0, 0.0, 1.0);
        assert!(result.is_none(), "Expected None for negative infinity client x");
    }

    #[test]
    fn given_infinity_client_y_when_converting_then_returns_none() {
        let result = screen_to_canvas(100.0, f64::INFINITY, 0.0, 0.0, 1.0);
        assert!(result.is_none(), "Expected None for infinity client y");
    }

    #[test]
    fn given_zoom_just_above_epsilon_when_converting_then_returns_some() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::EPSILON * 2.0);
        assert!(result.is_some(), "Expected Some for zoom > EPSILON");
    }

    #[test]
    fn given_zoom_just_below_epsilon_when_converting_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::EPSILON * 0.5);
        assert!(result.is_none(), "Expected None for zoom < EPSILON");
    }

    #[test]
    fn given_zero_client_coordinates_when_converting_then_returns_camera() {
        let result = screen_to_canvas(0.0, 0.0, 50.0, 75.0, 1.0);
        assert_eq!(result, Some((50.0, 75.0)));
    }

    #[test]
    fn given_subnormal_zoom_when_converting_then_returns_none() {
        let subnormal = f64::MIN_POSITIVE / 2.0;
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, subnormal);
        assert!(result.is_none(), "Expected None for subnormal zoom");
    }

    #[test]
    fn given_very_large_camera_values_when_converting_then_returns_correct() {
        let large = f64::MAX / 4.0;
        let result = screen_to_canvas(100.0, 200.0, large, large, 1.0);
        assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        assert!(cx.is_finite() && cy.is_finite());
    }

    #[test]
    fn given_fractional_zoom_when_converting_then_scales_correctly() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 0.5);
        assert_eq!(result, Some((200.0, 400.0)));
    }

    #[test]
    fn given_zoom_of_point_one_when_converting_then_scales_by_ten() {
        let result = screen_to_canvas(10.0, 20.0, 0.0, 0.0, 0.1);
        assert_eq!(result, Some((100.0, 200.0)));
    }
}
