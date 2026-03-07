//! Integration tests for math layer migration - canvas coordinate math functions.
//!
//! Tests the mathematical functions used for viewport transformations and
//! coordinate space conversions in the diagram tool.

use diagram_tool::ui::canvas::math::{safe_zoom, screen_to_canvas, within};

#[derive(Debug, Clone, PartialEq)]
pub struct MathLayerTestError {
    pub function: &'static str,
    pub case: &'static str,
    pub details: String,
}

impl std::fmt::Display for MathLayerTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} - {}", self.function, self.case, self.details)
    }
}

impl std::error::Error for MathLayerTestError {}

type TestResult = Result<(), MathLayerTestError>;

fn err(function: &'static str, case: &'static str, details: &str) -> MathLayerTestError {
    MathLayerTestError {
        function,
        case,
        details: details.to_string(),
    }
}

mod safe_zoom_tests {
    use super::*;

    #[test]
    fn given_valid_zoom_when_zoom_is_one_then_returns_some() -> TestResult {
        let result = safe_zoom(1.0);
        if result.is_none() {
            return Err(err("safe_zoom", "valid_zoom_one", "Expected Some(1.0)"));
        }
        Ok(())
    }

    #[test]
    fn given_valid_zoom_when_zoom_is_point_five_then_returns_some() -> TestResult {
        let result = safe_zoom(0.5);
        if result.is_none() {
            return Err(err("safe_zoom", "valid_zoom_half", "Expected Some(0.5)"));
        }
        Ok(())
    }

    #[test]
    fn given_valid_zoom_when_zoom_is_very_large_then_returns_some() -> TestResult {
        let result = safe_zoom(1e10);
        if result.is_none() {
            return Err(err("safe_zoom", "valid_zoom_large", "Expected Some(1e10)"));
        }
        Ok(())
    }

    #[test]
    fn given_zero_zoom_when_zoom_is_zero_then_returns_none() -> TestResult {
        let result = safe_zoom(0.0);
        if result.is_some() {
            return Err(err("safe_zoom", "zero_zoom", "Expected None for 0.0"));
        }
        Ok(())
    }

    #[test]
    fn given_negative_zoom_when_zoom_is_negative_then_returns_none() -> TestResult {
        let result = safe_zoom(-1.0);
        if result.is_some() {
            return Err(err("safe_zoom", "negative_zoom", "Expected None for -1.0"));
        }
        Ok(())
    }

    #[test]
    fn given_negative_zoom_when_zoom_is_negative_epsilon_then_returns_none() -> TestResult {
        let result = safe_zoom(-f64::EPSILON);
        if result.is_some() {
            return Err(err("safe_zoom", "negative_epsilon", "Expected None for -EPSILON"));
        }
        Ok(())
    }

    #[test]
    fn given_positive_infinity_when_zoom_is_infinity_then_returns_none() -> TestResult {
        let result = safe_zoom(f64::INFINITY);
        if result.is_some() {
            return Err(err("safe_zoom", "positive_infinity", "Expected None for INFINITY"));
        }
        Ok(())
    }

    #[test]
    fn given_negative_infinity_when_zoom_is_neg_infinity_then_returns_none() -> TestResult {
        let result = safe_zoom(f64::NEG_INFINITY);
        if result.is_some() {
            return Err(err("safe_zoom", "negative_infinity", "Expected None for NEG_INFINITY"));
        }
        Ok(())
    }

    #[test]
    fn given_nan_zoom_when_zoom_is_nan_then_returns_none() -> TestResult {
        let result = safe_zoom(f64::NAN);
        if result.is_some() {
            return Err(err("safe_zoom", "nan_zoom", "Expected None for NAN"));
        }
        Ok(())
    }

    #[test]
    fn given_subnormal_zoom_when_zoom_is_subnormal_then_returns_none() -> TestResult {
        let subnormal = f64::MIN_POSITIVE / 2.0;
        let result = safe_zoom(subnormal);
        if result.is_some() {
            return Err(err("safe_zoom", "subnormal_zoom", "Expected None for subnormal"));
        }
        Ok(())
    }

    #[test]
    fn given_very_small_finite_zoom_when_zoom_is_below_epsilon_then_returns_none() -> TestResult {
        let result = safe_zoom(f64::EPSILON * 0.5);
        if result.is_some() {
            return Err(err("safe_zoom", "below_epsilon", "Expected None for EPSILON * 0.5"));
        }
        Ok(())
    }

    #[test]
    fn given_max_finite_zoom_when_zoom_is_max_finite_then_returns_some() -> TestResult {
        let result = safe_zoom(f64::MAX);
        if result.is_none() {
            return Err(err("safe_zoom", "max_finite", "Expected Some(MAX)"));
        }
        Ok(())
    }

    #[test]
    fn given_min_positive_normal_when_zoom_is_min_positive_then_returns_none() -> TestResult {
        let result = safe_zoom(f64::MIN_POSITIVE);
        if result.is_some() {
            return Err(err("safe_zoom", "min_positive", "Expected None for MIN_POSITIVE (below EPSILON)"));
        }
        Ok(())
    }

    #[test]
    fn given_valid_zoom_when_zoom_is_epsilon_then_returns_none() -> TestResult {
        let result = safe_zoom(f64::EPSILON);
        if result.is_some() {
            return Err(err("safe_zoom", "valid_zoom_epsilon", "Expected None for exactly EPSILON (must be > EPSILON)"));
        }
        Ok(())
    }

    #[test]
    fn given_valid_zoom_when_zoom_is_two_epsilon_then_returns_some() -> TestResult {
        let result = safe_zoom(f64::EPSILON * 2.0);
        if result.is_none() {
            return Err(err("safe_zoom", "valid_zoom_two_epsilon", "Expected Some for 2*EPSILON"));
        }
        Ok(())
    }
}

mod within_tests {
    use super::*;

    #[test]
    fn given_node_within_subgraph_when_node_is_fully_contained_then_returns_true() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "fully_contained", "Expected true for node within subgraph"));
        }
        Ok(())
    }

    #[test]
    fn given_node_on_subgraph_boundary_when_node_edges_match_subgraph_then_returns_true() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 100.0, 100.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "exact_boundary", "Expected true for exact boundary match"));
        }
        Ok(())
    }

    #[test]
    fn given_node_exceeding_subgraph_when_node_x_exceeds_then_returns_false() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (60.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "exceeds_x", "Expected false when node x exceeds"));
        }
        Ok(())
    }

    #[test]
    fn given_node_exceeding_subgraph_when_node_y_exceeds_then_returns_false() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 60.0, 50.0, 50.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "exceeds_y", "Expected false when node y exceeds"));
        }
        Ok(())
    }

    #[test]
    fn given_node_exceeding_subgraph_when_node_width_exceeds_then_returns_false() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 101.0, 50.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "exceeds_width", "Expected false when node width exceeds"));
        }
        Ok(())
    }

    #[test]
    fn given_node_exceeding_subgraph_when_node_height_exceeds_then_returns_false() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 50.0, 101.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "exceeds_height", "Expected false when node height exceeds"));
        }
        Ok(())
    }

    #[test]
    fn given_node_outside_subgraph_when_node_is_completely_outside_then_returns_false() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (200.0, 200.0, 50.0, 50.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "completely_outside", "Expected false for completely outside"));
        }
        Ok(())
    }

    #[test]
    fn given_negative_coordinates_when_both_are_negative_then_returns_true() -> TestResult {
        let subgraph = (-100.0, -100.0, 200.0, 200.0);
        let node = (-50.0, -50.0, 50.0, 50.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "negative_coords", "Expected true for negative coords"));
        }
        Ok(())
    }

    #[test]
    fn given_large_coordinates_when_values_are_large_then_returns_correct_result() -> TestResult {
        let subgraph = (1e10, 1e10, 1e10, 1e10);
        let node = (1e10 + 1.0, 1e10 + 1.0, 1.0, 1.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "large_coords", "Expected true for large coords"));
        }
        Ok(())
    }

    #[test]
    fn given_degenerate_subgraph_when_width_is_zero_then_returns_correctly() -> TestResult {
        let subgraph = (0.0, 0.0, 0.0, 100.0);
        let node = (0.0, 0.0, 1.0, 1.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "zero_width_subgraph", "Expected false for zero-width subgraph"));
        }
        Ok(())
    }

    #[test]
    fn given_degenerate_subgraph_when_height_is_zero_then_returns_correctly() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 0.0);
        let node = (0.0, 0.0, 1.0, 1.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "zero_height_subgraph", "Expected false for zero-height subgraph"));
        }
        Ok(())
    }

    #[test]
    fn given_degenerate_node_when_width_is_zero_and_inside_then_returns_true() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 0.0, 10.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "zero_width_inside", "Expected true for zero-width node inside"));
        }
        Ok(())
    }

    #[test]
    fn given_degenerate_node_when_width_is_zero_and_outside_then_returns_false() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (150.0, 10.0, 0.0, 10.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "zero_width_outside", "Expected false for zero-width node outside"));
        }
        Ok(())
    }

    #[test]
    fn given_degenerate_node_when_height_is_zero_and_inside_then_returns_true() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 10.0, 0.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "zero_height_inside", "Expected true for zero-height node inside"));
        }
        Ok(())
    }

    #[test]
    fn given_degenerate_node_when_height_is_zero_and_outside_then_returns_false() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (10.0, 150.0, 10.0, 0.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "zero_height_outside", "Expected false for zero-height node outside"));
        }
        Ok(())
    }

    #[test]
    fn given_negative_width_subgraph_when_subgraph_has_negative_dimensions_then_returns_false() -> TestResult {
        let subgraph = (100.0, 100.0, -50.0, -50.0);
        let node = (0.0, 0.0, 10.0, 10.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "negative_dims", "Expected false for negative dimensions"));
        }
        Ok(())
    }

    #[test]
    fn given_infinite_subgraph_when_subgraph_has_infinite_dimensions_then_returns_true() -> TestResult {
        let subgraph = (0.0, 0.0, f64::INFINITY, f64::INFINITY);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "infinite_dims", "Expected true for infinite subgraph"));
        }
        Ok(())
    }

    #[test]
    fn given_subgraph_with_nan_when_subgraph_has_nan_coords_then_returns_false() -> TestResult {
        let subgraph = (f64::NAN, 0.0, 100.0, 100.0);
        let node = (10.0, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "nan_subgraph", "Expected false for NaN in subgraph"));
        }
        Ok(())
    }

    #[test]
    fn given_node_with_nan_when_node_has_nan_coords_then_returns_false() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (f64::NAN, 10.0, 50.0, 50.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "nan_node", "Expected false for NaN in node"));
        }
        Ok(())
    }

    #[test]
    fn given_boundary_with_epsilon_difference_when_node_exceeds_by_one_then_returns_false() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (0.0, 0.0, 101.0, 50.0);
        let result = within(subgraph, node);
        if result {
            return Err(err("within", "one_exceed", "Expected false when node width exceeds by 1.0"));
        }
        Ok(())
    }

    #[test]
    fn given_degenerate_node_when_width_is_zero_and_point_inside_then_returns_true() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, 0.0, 10.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "zero_width_inside", "Expected true for zero-width node inside"));
        }
        Ok(())
    }

    #[test]
    fn given_degenerate_node_when_height_is_zero_and_point_inside_then_returns_true() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, 10.0, 0.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "zero_height_inside", "Expected true for zero-height node inside"));
        }
        Ok(())
    }

    #[test]
    fn given_point_node_when_node_has_zero_dimensions_then_returns_correctly() -> TestResult {
        let subgraph = (0.0, 0.0, 100.0, 100.0);
        let node = (50.0, 50.0, 0.0, 0.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "point_node", "Expected true for point at center"));
        }
        Ok(())
    }

    #[test]
    fn given_max_values_when_coordinates_are_max_then_returns_correctly() -> TestResult {
        let subgraph = (0.0, 0.0, f64::MAX, f64::MAX);
        let node = (1.0, 1.0, f64::MAX - 2.0, f64::MAX - 2.0);
        let result = within(subgraph, node);
        if !result {
            return Err(err("within", "max_values", "Expected true for max values"));
        }
        Ok(())
    }
}

mod screen_to_canvas_tests {
    use super::*;

    #[test]
    fn given_identity_transform_when_screen_is_origin_then_returns_canvas_origin() -> TestResult {
        let result = screen_to_canvas(0.0, 0.0, 0.0, 0.0, 1.0);
        let expected = (0.0, 0.0);
        match result {
            Some((cx, cy)) => {
                if (cx - expected.0).abs() > f64::EPSILON || (cy - expected.1).abs() > f64::EPSILON {
                    return Err(err("screen_to_canvas", "identity_origin", "Unexpected canvas coords"));
                }
            }
            None => return Err(err("screen_to_canvas", "identity_origin", "Expected Some for valid input")),
        }
        Ok(())
    }

    #[test]
    fn given_identity_transform_when_screen_has_offset_then_returns_correct_canvas() -> TestResult {
        let result = screen_to_canvas(100.0, 50.0, 0.0, 0.0, 1.0);
        let expected = (100.0, 50.0);
        match result {
            Some((cx, cy)) => {
                if (cx - expected.0).abs() > f64::EPSILON || (cy - expected.1).abs() > f64::EPSILON {
                    return Err(err("screen_to_canvas", "identity_offset", "Unexpected canvas coords"));
                }
            }
            None => return Err(err("screen_to_canvas", "identity_offset", "Expected Some for valid input")),
        }
        Ok(())
    }

    #[test]
    fn given_zoom_factor_when_zoom_is_two_then_returns_scaled_canvas() -> TestResult {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, 2.0);
        let expected = (50.0, 50.0);
        match result {
            Some((cx, cy)) => {
                if (cx - expected.0).abs() > f64::EPSILON || (cy - expected.1).abs() > f64::EPSILON {
                    return Err(err("screen_to_canvas", "zoom_two", "Unexpected canvas coords"));
                }
            }
            None => return Err(err("screen_to_canvas", "zoom_two", "Expected Some for valid input")),
        }
        Ok(())
    }

    #[test]
    fn given_zoom_factor_when_zoom_is_half_then_returns_scaled_canvas() -> TestResult {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, 0.5);
        let expected = (200.0, 200.0);
        match result {
            Some((cx, cy)) => {
                if (cx - expected.0).abs() > f64::EPSILON || (cy - expected.1).abs() > f64::EPSILON {
                    return Err(err("screen_to_canvas", "zoom_half", "Unexpected canvas coords"));
                }
            }
            None => return Err(err("screen_to_canvas", "zoom_half", "Expected Some for valid input")),
        }
        Ok(())
    }

    #[test]
    fn given_camera_offset_when_camera_is_offset_then_returns_shifted_canvas() -> TestResult {
        let result = screen_to_canvas(0.0, 0.0, 100.0, 200.0, 1.0);
        let expected = (100.0, 200.0);
        match result {
            Some((cx, cy)) => {
                if (cx - expected.0).abs() > f64::EPSILON || (cy - expected.1).abs() > f64::EPSILON {
                    return Err(err("screen_to_canvas", "camera_offset", "Unexpected canvas coords"));
                }
            }
            None => return Err(err("screen_to_canvas", "camera_offset", "Expected Some for valid input")),
        }
        Ok(())
    }

    #[test]
    fn given_combined_transform_when_all_params_are_nonzero_then_returns_combined() -> TestResult {
        let result = screen_to_canvas(100.0, 50.0, 10.0, 20.0, 2.0);
        let expected = (60.0, 45.0);
        match result {
            Some((cx, cy)) => {
                if (cx - expected.0).abs() > f64::EPSILON || (cy - expected.1).abs() > f64::EPSILON {
                    return Err(err("screen_to_canvas", "combined_transform", "Unexpected canvas coords"));
                }
            }
            None => return Err(err("screen_to_canvas", "combined_transform", "Expected Some for valid input")),
        }
        Ok(())
    }

    #[test]
    fn given_invalid_zoom_when_zoom_is_zero_then_returns_none() -> TestResult {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, 0.0);
        if result.is_some() {
            return Err(err("screen_to_canvas", "zero_zoom", "Expected None for zero zoom"));
        }
        Ok(())
    }

    #[test]
    fn given_invalid_zoom_when_zoom_is_negative_then_returns_none() -> TestResult {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, -1.0);
        if result.is_some() {
            return Err(err("screen_to_canvas", "negative_zoom", "Expected None for negative zoom"));
        }
        Ok(())
    }

    #[test]
    fn given_invalid_zoom_when_zoom_is_nan_then_returns_none() -> TestResult {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, f64::NAN);
        if result.is_some() {
            return Err(err("screen_to_canvas", "nan_zoom", "Expected None for NaN zoom"));
        }
        Ok(())
    }

    #[test]
    fn given_invalid_zoom_when_zoom_is_infinity_then_returns_none() -> TestResult {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, f64::INFINITY);
        if result.is_some() {
            return Err(err("screen_to_canvas", "infinity_zoom", "Expected None for infinity zoom"));
        }
        Ok(())
    }

    #[test]
    fn given_negative_screen_coords_when_screen_is_negative_then_returns_correct_canvas() -> TestResult {
        let result = screen_to_canvas(-100.0, -50.0, 0.0, 0.0, 1.0);
        let expected = (-100.0, -50.0);
        match result {
            Some((cx, cy)) => {
                if (cx - expected.0).abs() > f64::EPSILON || (cy - expected.1).abs() > f64::EPSILON {
                    return Err(err("screen_to_canvas", "negative_screen", "Unexpected canvas coords"));
                }
            }
            None => return Err(err("screen_to_canvas", "negative_screen", "Expected Some for valid input")),
        }
        Ok(())
    }

    #[test]
    fn given_large_coordinates_when_values_are_large_then_returns_correct_canvas() -> TestResult {
        let result = screen_to_canvas(1e10, 1e10, 1e5, 1e5, 1.0);
        match result {
            Some((cx, cy)) => {
                if cx.is_nan() || cy.is_nan() || cx.is_infinite() || cy.is_infinite() {
                    return Err(err("screen_to_canvas", "large_coords", "Got NaN or Inf for large coords"));
                }
            }
            None => return Err(err("screen_to_canvas", "large_coords", "Expected Some for valid input")),
        }
        Ok(())
    }

    #[test]
    fn given_subnormal_zoom_when_zoom_is_subnormal_then_returns_none() -> TestResult {
        let subnormal = f64::MIN_POSITIVE / 2.0;
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, subnormal);
        if result.is_some() {
            return Err(err("screen_to_canvas", "subnormal_zoom", "Expected None for subnormal zoom"));
        }
        Ok(())
    }

    #[test]
    fn given_very_small_zoom_when_zoom_is_below_epsilon_then_returns_none() -> TestResult {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, f64::EPSILON * 0.5);
        if result.is_some() {
            return Err(err("screen_to_canvas", "below_epsilon_zoom", "Expected None for below epsilon zoom"));
        }
        Ok(())
    }

    #[test]
    fn given_epsilon_zoom_when_zoom_is_exactly_epsilon_then_returns_none() -> TestResult {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, f64::EPSILON);
        if result.is_some() {
            return Err(err("screen_to_canvas", "epsilon_zoom", "Expected None for exactly EPSILON"));
        }
        Ok(())
    }

    #[test]
    fn given_two_epsilon_zoom_when_zoom_is_two_epsilon_then_returns_some() -> TestResult {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, f64::EPSILON * 2.0);
        if result.is_none() {
            return Err(err("screen_to_canvas", "two_epsilon_zoom", "Expected Some for 2*EPSILON"));
        }
        Ok(())
    }

    #[test]
    fn given_nan_screen_coords_when_client_x_is_nan_then_returns_nan_result() -> TestResult {
        let result = screen_to_canvas(f64::NAN, 100.0, 0.0, 0.0, 1.0);
        match result {
            Some((cx, cy)) => {
                if !cx.is_nan() {
                    return Err(err("screen_to_canvas", "nan_client_x", "Expected NaN result for NaN client_x"));
                }
            }
            None => return Err(err("screen_to_canvas", "nan_client_x", "Expected Some with NaN for NaN client_x")),
        }
        Ok(())
    }

    #[test]
    fn given_nan_screen_coords_when_client_y_is_nan_then_returns_nan_result() -> TestResult {
        let result = screen_to_canvas(100.0, f64::NAN, 0.0, 0.0, 1.0);
        match result {
            Some((cx, cy)) => {
                if !cy.is_nan() {
                    return Err(err("screen_to_canvas", "nan_client_y", "Expected NaN result for NaN client_y"));
                }
            }
            None => return Err(err("screen_to_canvas", "nan_client_y", "Expected Some with NaN for NaN client_y")),
        }
        Ok(())
    }

    #[test]
    fn given_max_zoom_when_zoom_is_max_then_returns_some() -> TestResult {
        let result = screen_to_canvas(1.0, 1.0, 0.0, 0.0, f64::MAX);
        if result.is_none() {
            return Err(err("screen_to_canvas", "max_zoom", "Expected Some for max zoom"));
        }
        Ok(())
    }
}

mod integration_tests {
    use super::*;

    #[test]
    fn given_round_trip_conversion_when_convert_to_canvas_and_back_then_preserves_offset() -> TestResult {
        let screen_x = 250.0;
        let screen_y = 150.0;
        let camera_x = 100.0;
        let camera_y = 50.0;
        let zoom = 2.0;

        let canvas = screen_to_canvas(screen_x, screen_y, camera_x, camera_y, zoom);
        match canvas {
            Some((cx, cy)) => {
                let reprojected_x = (cx - camera_x) * zoom;
                let reprojected_y = (cy - camera_y) * zoom;
                if (reprojected_x - screen_x).abs() > f64::EPSILON * 100.0 {
                    return Err(err("integration", "round_trip_x", "X coordinate not preserved"));
                }
                if (reprojected_y - screen_y).abs() > f64::EPSILON * 100.0 {
                    return Err(err("integration", "round_trip_y", "Y coordinate not preserved"));
                }
            }
            None => return Err(err("integration", "round_trip", "Expected Some from screen_to_canvas")),
        }
        Ok(())
    }

    #[test]
    fn given_viewport_query_when_checking_node_in_viewport_then_correct() -> TestResult {
        let viewport = (0.0, 0.0, 800.0, 600.0);
        let visible_node = (100.0, 100.0, 200.0, 150.0);
        let invisible_node = (1000.0, 1000.0, 100.0, 100.0);

        let visible_result = within(viewport, visible_node);
        let invisible_result = within(viewport, invisible_node);

        if !visible_result {
            return Err(err("integration", "viewport_visible", "Expected visible node to be within viewport"));
        }
        if invisible_result {
            return Err(err("integration", "viewport_invisible", "Expected invisible node to be outside viewport"));
        }
        Ok(())
    }

    #[test]
    fn given_zoom_range_when_testing_zoom_boundaries_then_correct() -> TestResult {
        let valid_zooms = [0.1, 0.5, 1.0, 2.0, 10.0, f64::EPSILON * 2.0, f64::MAX / 2.0];
        let invalid_zooms = [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::EPSILON * 0.5];

        for z in valid_zooms.iter() {
            let result = safe_zoom(*z);
            if result.is_none() {
                return Err(err("integration", "valid_zoom", &format!("Expected {:?} to be valid", z)));
            }
        }

        for z in invalid_zooms.iter() {
            let result = safe_zoom(*z);
            if result.is_some() {
                return Err(err("integration", "invalid_zoom", &format!("Expected {:?} to be invalid", z)));
            }
        }
        Ok(())
    }
}
