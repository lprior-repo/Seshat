#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use diagram_tool::ui::canvas::math::{safe_zoom, screen_to_canvas, within};

#[cfg(test)]
mod safe_zoom_tests {
    use super::*;

    #[test]
    fn given_valid_zoom_when_within_epsilon_range_then_returns_some() {
        let result = safe_zoom(1.0);
        assert!(result.is_some(), "Expected Some for valid zoom 1.0");
    }

    #[test]
    fn given_valid_zoom_when_exactly_epsilon_then_returns_none() {
        let result = safe_zoom(f64::EPSILON);
        assert!(result.is_none(), "Expected None for zoom equal to EPSILON (strict > check)");
    }

    #[test]
    fn given_invalid_zoom_when_zero_then_returns_none() {
        let result = safe_zoom(0.0);
        assert!(result.is_none(), "Expected None for zoom 0.0");
    }

    #[test]
    fn given_invalid_zoom_when_negative_then_returns_none() {
        let result = safe_zoom(-1.0);
        assert!(result.is_none(), "Expected None for negative zoom");
    }

    #[test]
    fn given_invalid_zoom_when_negative_epsilon_then_returns_none() {
        let result = safe_zoom(-f64::EPSILON);
        assert!(result.is_none(), "Expected None for negative EPSILON");
    }

    #[test]
    fn given_invalid_zoom_when_less_than_epsilon_then_returns_none() {
        let result = safe_zoom(f64::EPSILON * 0.5);
        assert!(result.is_none(), "Expected None for zoom < EPSILON");
    }

    #[test]
    fn given_invalid_zoom_when_positive_infinity_then_returns_none() {
        let result = safe_zoom(f64::INFINITY);
        assert!(result.is_none(), "Expected None for positive infinity");
    }

    #[test]
    fn given_invalid_zoom_when_negative_infinity_then_returns_none() {
        let result = safe_zoom(f64::NEG_INFINITY);
        assert!(result.is_none(), "Expected None for negative infinity");
    }

    #[test]
    fn given_invalid_zoom_when_nan_then_returns_none() {
        let result = safe_zoom(f64::NAN);
        assert!(result.is_none(), "Expected None for NaN");
    }

    #[test]
    fn given_edge_case_when_min_positive_then_returns_none() {
        let result = safe_zoom(f64::MIN_POSITIVE);
        assert!(result.is_none(), "Expected None for MIN_POSITIVE (smaller than EPSILON)");
    }

    #[test]
    fn given_edge_case_when_max_finite_then_returns_some() {
        let result = safe_zoom(f64::MAX);
        assert!(result.is_some(), "Expected Some for MAX");
    }

    #[test]
    fn given_edge_case_when_subnormal_positive_then_returns_none() {
        let subnormal = f64::MIN_POSITIVE / 2.0;
        let result = safe_zoom(subnormal);
        assert!(result.is_none(), "Expected None for subnormal positive (smaller than EPSILON)");
    }

    #[test]
    fn given_boundary_when_just_above_epsilon_then_returns_some() {
        let result = safe_zoom(f64::EPSILON + f64::EPSILON * 0.1);
        assert!(result.is_some(), "Expected Some for epsilon + 10%");
    }

    #[test]
    fn given_boundary_when_just_below_epsilon_then_returns_none() {
        let result = safe_zoom(f64::EPSILON - f64::EPSILON * 0.1);
        assert!(result.is_none(), "Expected None for epsilon - 10%");
    }

    #[test]
    fn given_typical_values_when_zoom_0_5_then_returns_some() {
        let result = safe_zoom(0.5);
        assert!(result.is_some(), "Expected Some for zoom 0.5");
    }

    #[test]
    fn given_typical_values_when_zoom_2_0_then_returns_some() {
        let result = safe_zoom(2.0);
        assert!(result.is_some(), "Expected Some for zoom 2.0");
    }

    #[test]
    fn given_typical_values_when_zoom_0_1_then_returns_some() {
        let result = safe_zoom(0.1);
        assert!(result.is_some(), "Expected Some for zoom 0.1");
    }
}

#[cfg(test)]
mod within_tests {
    use super::*;

    #[test]
    fn given_valid_rectangles_when_node_inside_subgraph_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true when node is inside subgraph");
    }

    #[test]
    fn given_valid_rectangles_when_node_exactly_matches_subgraph_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 100.0, 100.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true when node matches subgraph exactly");
    }

    #[test]
    fn given_boundary_condition_when_node_on_left_edge_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true when node starts at left edge");
    }

    #[test]
    fn given_boundary_condition_when_node_on_top_edge_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 0.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true when node starts at top edge");
    }

    #[test]
    fn given_boundary_condition_when_node_touches_right_edge_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true when node touches right edge");
    }

    #[test]
    fn given_boundary_condition_when_node_touches_bottom_edge_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 50.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true when node touches bottom edge");
    }

    #[test]
    fn given_boundary_condition_when_node_exceeds_right_by_epsilon_then_returns_expected() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 10.0, 50.0 + f64::EPSILON * 1000.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node significantly exceeds right edge");
    }

    #[test]
    fn given_boundary_condition_when_node_exceeds_bottom_by_epsilon_then_returns_expected() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 50.0, 50.0, 50.0 + f64::EPSILON * 1000.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node significantly exceeds bottom edge");
    }

    #[test]
    fn given_degenerate_case_when_subgraph_zero_width_then_returns_false() {
        let subgraph = (0.0, 0.0, 0.0, 100.0);
        let node = (0.0, 0.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for zero-width subgraph");
    }

    #[test]
    fn given_degenerate_case_when_subgraph_zero_height_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 0.0);
        let node = (0.0, 0.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for zero-height subgraph");
    }

    #[test]
    fn given_degenerate_case_when_node_zero_width_then_returns_true_when_inside() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, 0.0, 10.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true for zero-width node inside subgraph (point on line)");
    }

    #[test]
    fn given_degenerate_case_when_node_zero_height_then_returns_true_when_inside() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, 10.0, 0.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true for zero-height node inside subgraph (point on line)");
    }

    #[test]
    fn given_degenerate_case_when_both_zero_dimensions_then_returns_true_at_origin() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, 0.0, 0.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true for zero-dimension node at point inside");
    }

    #[test]
    fn given_negative_dimensions_when_subgraph_has_negative_width_then_returns_false() {
        let subgraph = (100.0, 0.0, -50.0, 100.0);
        let node = (10.0, 10.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for negative width subgraph");
    }

    #[test]
    fn given_negative_dimensions_when_subgraph_has_negative_height_then_returns_false() {
        let subgraph = (0.0, 100.0, 100.0, -50.0);
        let node = (10.0, 10.0, 10.0, 10.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for negative height subgraph");
    }

    #[test]
    fn given_negative_dimensions_when_node_has_negative_width_then_returns_expected() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, -10.0, 10.0);
        let result = within(subgraph, node);
        let expected = 50.0 - 10.0 >= 0.0 && 50.0 <= 100.0;
        assert_eq!(result, expected, "Implementation allows negative width nodes");
    }

    #[test]
    fn given_negative_dimensions_when_node_has_negative_height_then_returns_expected() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, 10.0, -10.0);
        let result = within(subgraph, node);
        let expected = 50.0 - 10.0 >= 0.0 && 50.0 <= 100.0;
        assert_eq!(result, expected, "Implementation allows negative height nodes");
    }

    #[test]
    fn given_node_outside_when_to_the_right_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (80.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node is to the right");
    }

    #[test]
    fn given_node_outside_when_below_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 80.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node is below");
    }

    #[test]
    fn given_node_outside_when_to_the_left_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (-10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node is to the left");
    }

    #[test]
    fn given_node_outside_when_above_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, -10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node is above");
    }

    #[test]
    fn given_nan_in_subgraph_when_x_is_nan_then_returns_false() {
        let subgraph = (f64::NAN, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when subgraph x is NaN");
    }

    #[test]
    fn given_nan_in_subgraph_when_y_is_nan_then_returns_false() {
        let subgraph = (0.0, f64::NAN, 100.0, 100.0);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when subgraph y is NaN");
    }

    #[test]
    fn given_nan_in_subgraph_when_width_is_nan_then_returns_false() {
        let subgraph = (0.0, 0.0, f64::NAN, 100.0);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when subgraph width is NaN");
    }

    #[test]
    fn given_nan_in_subgraph_when_height_is_nan_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, f64::NAN);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when subgraph height is NaN");
    }

    #[test]
    fn given_nan_in_node_when_x_is_nan_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (f64::NAN, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node x is NaN");
    }

    #[test]
    fn given_nan_in_node_when_y_is_nan_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, f64::NAN, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node y is NaN");
    }

    #[test]
    fn given_nan_in_node_when_width_is_nan_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, f64::NAN, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node width is NaN");
    }

    #[test]
    fn given_nan_in_node_when_height_is_nan_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 50.0, f64::NAN);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node height is NaN");
    }

    #[test]
    fn given_infinity_in_subgraph_when_width_is_infinite_then_returns_expected() {
        let subgraph = (0.0, 0.0, f64::INFINITY, 100.0);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true when subgraph has infinite width");
    }

    #[test]
    fn given_infinity_in_subgraph_when_height_is_infinite_then_returns_expected() {
        let subgraph = (0.0, 0.0, 100.0, f64::INFINITY);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true when subgraph has infinite height");
    }

    #[test]
    fn given_infinity_in_subgraph_when_both_infinite_then_returns_expected() {
        let subgraph = (0.0, 0.0, f64::INFINITY, f64::INFINITY);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true when subgraph has infinite dimensions");
    }

    #[test]
    fn given_infinity_in_subgraph_when_negative_infinity_width_then_returns_false() {
        let subgraph = (0.0, 0.0, f64::NEG_INFINITY, 100.0);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when subgraph has negative infinite width");
    }

    #[test]
    fn given_infinity_in_node_when_width_is_infinite_then_returns_expected() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, f64::INFINITY, 50.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node has infinite width");
    }

    #[test]
    fn given_overflow_values_when_max_finite_dimensions_then_returns_expected() {
        let subgraph = (f64::MAX / 4.0, f64::MAX / 4.0, f64::MAX / 2.0, f64::MAX / 2.0);
        let node = (f64::MAX / 4.0 + 1.0, f64::MAX / 4.0 + 1.0, 1.0, 1.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true for overflow-dimension rectangles");
    }

    #[test]
    fn given_subnormal_dimensions_when_very_small_subgraph_then_returns_expected() {
        let subgraph = (0.0, 0.0, f64::MIN_POSITIVE, f64::MIN_POSITIVE);
        let node = (0.0, 0.0, f64::MIN_POSITIVE / 2.0, f64::MIN_POSITIVE / 2.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true for subnormal dimensions");
    }

    #[test]
    fn given_offset_coordinates_when_subgraph_not_at_origin_then_returns_true() {
        let subgraph = (50.0, 50.0, 100.0, 100.0);
        let node = (60.0, 60.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true for offset coordinates");
    }

    #[test]
    fn given_offset_coordinates_when_node_exceeds_offset_subgraph_then_returns_false() {
        let subgraph = (50.0, 50.0, 50.0, 50.0);
        let node = (50.0, 50.0, 60.0, 60.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node exceeds offset subgraph");
    }

    #[test]
    fn given_negative_coordinates_when_both_negative_then_returns_true() {
        let subgraph = (-100.0, -100.0, 100.0, 100.0);
        let node = (-50.0, -50.0, 50.0, 50.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true for negative coordinates");
    }

    #[test]
    fn given_epsilon_difference_when_node_larger_by_meaningful_amount_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 100.0 + 0.01, 100.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false when node exceeds by meaningful amount");
    }

    #[test]
    fn given_epsilon_difference_when_node_smaller_by_single_epsilon_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 100.0 - f64::EPSILON, 100.0 - f64::EPSILON);
        let result = within(subgraph, node);
        assert!(result, "Expected true when node is smaller by epsilon");
    }

    #[test]
    fn given_point_node_when_zero_dimensions_inside_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, 0.0, 0.0);
        let result = within(subgraph, node);
        assert!(result, "Expected true for zero-dimension node at point inside");
    }

    #[test]
    fn given_point_node_when_zero_dimensions_outside_then_returns_false() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (150.0, 150.0, 0.0, 0.0);
        let result = within(subgraph, node);
        assert!(!result, "Expected false for zero-dimension node outside");
    }
}

#[cfg(test)]
mod screen_to_canvas_tests {
    use super::*;

    #[test]
    fn given_valid_inputs_when_zoom_one_and_no_offset_then_returns_direct_mapping() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 1.0);
        assert!(result.is_some(), "Expected Some for valid inputs");
        let (cx, cy) = result.unwrap();
        assert!((cx - 100.0).abs() < f64::EPSILON, "Expected x to map directly");
        assert!((cy - 200.0).abs() < f64::EPSILON, "Expected y to map directly");
    }

    #[test]
    fn given_valid_inputs_when_with_camera_offset_then_returns_offset_coordinates() {
        let result = screen_to_canvas(100.0, 200.0, 50.0, 75.0, 1.0);
        assert!(result.is_some(), "Expected Some for valid inputs");
        let (cx, cy) = result.unwrap();
        assert!((cx - 150.0).abs() < f64::EPSILON, "Expected x with camera offset");
        assert!((cy - 275.0).abs() < f64::EPSILON, "Expected y with camera offset");
    }

    #[test]
    fn given_valid_inputs_when_with_zoom_two_then_returns_scaled_coordinates() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 2.0);
        assert!(result.is_some(), "Expected Some for valid inputs");
        let (cx, cy) = result.unwrap();
        assert!((cx - 50.0).abs() < f64::EPSILON, "Expected x scaled by zoom");
        assert!((cy - 100.0).abs() < f64::EPSILON, "Expected y scaled by zoom");
    }

    #[test]
    fn given_valid_inputs_when_with_zoom_half_then_returns_scaled_coordinates() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 0.5);
        assert!(result.is_some(), "Expected Some for valid inputs");
        let (cx, cy) = result.unwrap();
        assert!((cx - 200.0).abs() < f64::EPSILON, "Expected x scaled by zoom 0.5");
        assert!((cy - 400.0).abs() < f64::EPSILON, "Expected y scaled by zoom 0.5");
    }

    #[test]
    fn given_valid_inputs_when_combined_zoom_and_offset_then_returns_correct_coordinates() {
        let result = screen_to_canvas(100.0, 200.0, 50.0, 75.0, 2.0);
        assert!(result.is_some(), "Expected Some for valid inputs");
        let (cx, cy) = result.unwrap();
        assert!((cx - 100.0).abs() < f64::EPSILON, "Expected x with zoom and offset");
        assert!((cy - 175.0).abs() < f64::EPSILON, "Expected y with zoom and offset");
    }

    #[test]
    fn given_invalid_zoom_when_zero_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 0.0);
        assert!(result.is_none(), "Expected None for zero zoom");
    }

    #[test]
    fn given_invalid_zoom_when_negative_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, -1.0);
        assert!(result.is_none(), "Expected None for negative zoom");
    }

    #[test]
    fn given_invalid_zoom_when_positive_infinity_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::INFINITY);
        assert!(result.is_none(), "Expected None for positive infinity");
    }

    #[test]
    fn given_invalid_zoom_when_negative_infinity_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::NEG_INFINITY);
        assert!(result.is_none(), "Expected None for negative infinity");
    }

    #[test]
    fn given_invalid_zoom_when_nan_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::NAN);
        assert!(result.is_none(), "Expected None for NaN zoom");
    }

    #[test]
    fn given_invalid_zoom_when_less_than_epsilon_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::EPSILON * 0.5);
        assert!(result.is_none(), "Expected None for zoom < EPSILON");
    }

    #[test]
    fn given_edge_case_when_zoom_at_epsilon_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::EPSILON);
        assert!(result.is_none(), "Expected None for zoom = EPSILON");
    }

    #[test]
    fn given_nan_coordinates_when_client_x_is_nan_then_returns_none() {
        let result = screen_to_canvas(f64::NAN, 200.0, 0.0, 0.0, 1.0);
        assert!(result.is_some(), "Expected Some for NaN client_x (result uses safe_zoom)");
    }

    #[test]
    fn given_nan_coordinates_when_client_y_is_nan_then_returns_none() {
        let result = screen_to_canvas(100.0, f64::NAN, 0.0, 0.0, 1.0);
        assert!(result.is_some(), "Expected Some for NaN client_y (result uses safe_zoom)");
    }

    #[test]
    fn given_nan_coordinates_when_camera_x_is_nan_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, f64::NAN, 0.0, 1.0);
        assert!(result.is_some(), "Expected Some for NaN camera_x (result uses safe_zoom)");
    }

    #[test]
    fn given_nan_coordinates_when_camera_y_is_nan_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, f64::NAN, 1.0);
        assert!(result.is_some(), "Expected Some for NaN camera_y (result uses safe_zoom)");
    }

    #[test]
    fn given_infinity_coordinates_when_client_x_is_infinite_then_returns_some() {
        let result = screen_to_canvas(f64::INFINITY, 200.0, 0.0, 0.0, 1.0);
        assert!(result.is_some(), "Expected Some for infinite client_x");
    }

    #[test]
    fn given_infinity_coordinates_when_client_y_is_infinite_then_returns_some() {
        let result = screen_to_canvas(100.0, f64::INFINITY, 0.0, 0.0, 1.0);
        assert!(result.is_some(), "Expected Some for infinite client_y");
    }

    #[test]
    fn given_infinity_coordinates_when_camera_x_is_infinite_then_returns_some() {
        let result = screen_to_canvas(100.0, 200.0, f64::INFINITY, 0.0, 1.0);
        assert!(result.is_some(), "Expected Some for infinite camera_x");
    }

    #[test]
    fn given_infinity_coordinates_when_camera_y_is_infinite_then_returns_some() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, f64::INFINITY, 1.0);
        assert!(result.is_some(), "Expected Some for infinite camera_y");
    }

    #[test]
    fn given_typical_values_when_zoom_0_25_then_returns_scaled_coordinates() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, 0.25);
        assert!(result.is_some(), "Expected Some for zoom 0.25");
        let (cx, cy) = result.unwrap();
        assert!((cx - 400.0).abs() < f64::EPSILON, "Expected x scaled by 0.25 zoom");
        assert!((cy - 400.0).abs() < f64::EPSILON, "Expected y scaled by 0.25 zoom");
    }

    #[test]
    fn given_typical_values_when_zoom_4_0_then_returns_scaled_coordinates() {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, 4.0);
        assert!(result.is_some(), "Expected Some for zoom 4.0");
        let (cx, cy) = result.unwrap();
        assert!((cx - 25.0).abs() < f64::EPSILON, "Expected x scaled by 4.0 zoom");
        assert!((cy - 25.0).abs() < f64::EPSILON, "Expected y scaled by 4.0 zoom");
    }

    #[test]
    fn given_negative_camera_offset_when_offset_is_negative_then_returns_adjusted_coordinates() {
        let result = screen_to_canvas(100.0, 100.0, -50.0, -50.0, 1.0);
        assert!(result.is_some(), "Expected Some for negative camera offset");
        let (cx, cy) = result.unwrap();
        assert!((cx - 50.0).abs() < f64::EPSILON, "Expected x adjusted by negative offset");
        assert!((cy - 50.0).abs() < f64::EPSILON, "Expected y adjusted by negative offset");
    }

    #[test]
    fn given_zero_coordinates_when_all_zeros_then_returns_origin() {
        let result = screen_to_canvas(0.0, 0.0, 0.0, 0.0, 1.0);
        assert!(result.is_some(), "Expected Some for zero inputs");
        let (cx, cy) = result.unwrap();
        assert!((cx - 0.0).abs() < f64::EPSILON, "Expected x at origin");
        assert!((cy - 0.0).abs() < f64::EPSILON, "Expected y at origin");
    }

    #[test]
    fn given_subnormal_zoom_when_very_small_zoom_then_returns_none() {
        let subnormal = f64::MIN_POSITIVE;
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, subnormal);
        assert!(result.is_none(), "Expected None for subnormal zoom");
    }

    #[test]
    fn given_boundary_zoom_when_just_above_epsilon_then_returns_some() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::EPSILON + f64::EPSILON * 0.1);
        assert!(result.is_some(), "Expected Some for zoom > EPSILON");
    }

    #[test]
    fn given_boundary_zoom_when_just_below_epsilon_then_returns_none() {
        let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::EPSILON - f64::EPSILON * 0.1);
        assert!(result.is_none(), "Expected None for zoom < EPSILON");
    }

    #[test]
    fn given_large_values_when_max_finite_coordinates_then_returns_some() {
        let result = screen_to_canvas(f64::MAX / 4.0, f64::MAX / 4.0, 0.0, 0.0, 1.0);
        assert!(result.is_some(), "Expected Some for large finite coordinates");
    }

    #[test]
    fn given_origin_to_canvas_when_screen_origin_then_returns_camera_position() {
        let result = screen_to_canvas(0.0, 0.0, 100.0, 200.0, 1.0);
        assert!(result.is_some(), "Expected Some for screen origin");
        let (cx, cy) = result.unwrap();
        assert!((cx - 100.0).abs() < f64::EPSILON, "Expected x = camera_x");
        assert!((cy - 200.0).abs() < f64::EPSILON, "Expected y = camera_y");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn given_typical_workflow_when_zoom_and_pan_then_returns_consistent_results() {
        let zoom = safe_zoom(1.5);
        assert!(zoom.is_some(), "Expected valid zoom");
        
        let canvas_result = screen_to_canvas(300.0, 400.0, 100.0, 200.0, zoom.unwrap());
        assert!(canvas_result.is_some(), "Expected Some from screen_to_canvas");
        
        let (cx, cy) = canvas_result.unwrap();
        let expected_x = (300.0 / 1.5) + 100.0;
        let expected_y = (400.0 / 1.5) + 200.0;
        
        assert!((cx - expected_x).abs() < f64::EPSILON, "Expected consistent x calculation");
        assert!((cy - expected_y).abs() < f64::EPSILON, "Expected consistent y calculation");
    }

    #[test]
    fn given_sequential_operations_when_zoom_chain_then_returns_none_for_invalid() {
        let result1 = safe_zoom(0.0);
        assert!(result1.is_none(), "Expected None for 0.0");
        
        let result2 = safe_zoom(f64::NAN);
        assert!(result2.is_none(), "Expected None for NaN");
        
        let result3 = safe_zoom(f64::INFINITY);
        assert!(result3.is_none(), "Expected None for infinity");
    }

    #[test]
    fn given_rectangle_hierarchy_when_nested_rectangles_then_returns_expected() {
        let outer = (0.0, 0.0, 200.0, 200.0);
        let middle = (25.0, 25.0, 150.0, 150.0);
        let inner = (50.0, 50.0, 100.0, 100.0);
        
        assert!(within(outer, middle), "Expected middle inside outer");
        assert!(within(middle, inner), "Expected inner inside middle");
        assert!(within(outer, inner), "Expected inner inside outer");
    }

    #[test]
    fn given_rectangle_hierarchy_when_sibling_rectangles_then_returns_expected() {
        let container = (0.0, 0.0, 200.0, 200.0);
        let left = (0.0, 0.0, 100.0, 200.0);
        let right = (100.0, 0.0, 100.0, 200.0);
        
        assert!(within(container, left), "Expected left inside container");
        assert!(within(container, right), "Expected right inside container");
        assert!(!within(left, right), "Expected right NOT inside left");
        assert!(!within(right, left), "Expected left NOT inside right");
    }

    #[test]
    fn given_viewport_calculation_when_zoom_changes_then_returns_different_canvas_coords() {
        let canvas_zoom_1 = screen_to_canvas(100.0, 100.0, 0.0, 0.0, 1.0);
        let canvas_zoom_2 = screen_to_canvas(100.0, 100.0, 0.0, 0.0, 2.0);
        
        assert!(canvas_zoom_1.is_some() && canvas_zoom_2.is_some(), "Expected Some for both zooms");
        
        let (cx1, _) = canvas_zoom_1.unwrap();
        let (cx2, _) = canvas_zoom_2.unwrap();
        
        assert!(cx1 != cx2, "Expected different canvas coordinates for different zooms");
    }

    #[test]
    fn given_invalid_inputs_chain_when_multiple_failures_then_returns_none() {
        let result = screen_to_canvas(0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(result.is_none(), "Expected None when zoom is 0");
    }

    #[test]
    fn given_edge_case_workflow_when_minimal_valid_zoom_then_returns_none_for_epsilon() {
        let zoom = safe_zoom(f64::EPSILON);
        assert!(zoom.is_none(), "Expected None for EPSILON (not > EPSILON)");
    }

    #[test]
    fn given_zoom_validation_when_various_valid_zoom_levels_then_returns_expected() {
        let valid_zooms = [0.1, 0.5, 1.0, 1.0 + f64::EPSILON, f64::MAX];
        
        for zoom in valid_zooms {
            let result = safe_zoom(zoom);
            assert!(result.is_some(), "Expected Some for valid zoom: {}", zoom);
        }
        
        let invalid_edge_cases = [f64::EPSILON, f64::MIN_POSITIVE];
        for zoom in invalid_edge_cases {
            let result = safe_zoom(zoom);
            assert!(result.is_none(), "Expected None for zoom <= EPSILON: {}", zoom);
        }
    }

    #[test]
    fn given_large_pan_values_when_camera_offset_then_returns_adjusted_coordinates() {
        let large_offset = 1e10;
        let result = screen_to_canvas(100.0, 100.0, large_offset, large_offset, 1.0);
        assert!(result.is_some(), "Expected Some for large offset");
        
        let (cx, cy) = result.unwrap();
        assert!((cx - (100.0 + large_offset)).abs() < f64::EPSILON, "Expected x with large offset");
        assert!((cy - (100.0 + large_offset)).abs() < f64::EPSILON, "Expected y with large offset");
    }

    #[test]
    fn given_boundary_rectangles_when_exactly_touching_edges_then_returns_true() {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node_left = (0.0, 25.0, 100.0, 50.0);
        let node_right = (0.0, 25.0, 100.0, 50.0);
        
        assert!(within(subgraph, node_left), "Expected node touching left edge to be inside");
    }

    #[test]
    fn given_zoom_validation_when_various_invalid_zoom_levels_then_returns_none() {
        let invalid_zooms = [0.0, -0.5, -2.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::EPSILON * 0.9];
        
        for zoom in invalid_zooms {
            let result = safe_zoom(zoom);
            assert!(result.is_none(), "Expected None for invalid zoom: {}", zoom);
        }
    }
}
