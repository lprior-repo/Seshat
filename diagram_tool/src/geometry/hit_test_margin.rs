//! Hit test margin calculations with zoom-aware screen-space behavior.
//!
//! This module provides functions for computing hit test margins that remain
//! constant in screen pixels regardless of zoom level (screen-space behavior).
//!
//! # Screen-Space Behavior
//! - At zoom 0.1: world margin = 50.0 (5.0 / 0.1)
//! - At zoom 1.0: world margin = 5.0 (5.0 / 1.0)
//! - At zoom 4.0: world margin = 1.25 (5.0 / 4.0)
//!
//! This ensures users can reliably click near node edges whether zoomed in or out.

use crate::geometry::{Point, Rectangle};

/// Minimum allowed zoom level (matches viewport::MIN_ZOOM)
const MIN_ZOOM: f64 = 0.1;

/// Maximum allowed zoom level (matches viewport::MAX_ZOOM)
const MAX_ZOOM: f64 = 4.0;

/// Errors that can occur during hit test operations
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum HitTestError {
    #[error("Invalid zoom: {0} is outside valid range [{1}, {2}]")]
    InvalidZoom(f64, f64, f64),

    #[error("Invalid margin: {0} must be positive")]
    InvalidMargin(f64),

    #[error("Invalid point: coordinates must be finite (got x={0}, y={1})")]
    InvalidPoint(f64, f64),
}

/// Validates that a zoom value is within the valid range.
///
/// # Preconditions
/// - zoom must be in range [MIN_ZOOM, MAX_ZOOM]
///
/// # Postconditions
/// - Returns Ok(zoom) if valid
/// - Returns Err(HitTestError::InvalidZoom) if invalid
fn validate_zoom(zoom: f64) -> Result<f64, HitTestError> {
    let in_range = zoom.is_finite() && (MIN_ZOOM..=MAX_ZOOM).contains(&zoom);
    in_range
        .then_some(zoom)
        .ok_or(HitTestError::InvalidZoom(zoom, MIN_ZOOM, MAX_ZOOM))
}

/// Validates that a screen margin is positive.
///
/// # Preconditions
/// - screen_margin must be > 0
///
/// # Postconditions
/// - Returns Ok(screen_margin) if valid
/// - Returns Err(HitTestError::InvalidMargin) if invalid
fn validate_margin(screen_margin: f64) -> Result<f64, HitTestError> {
    let is_positive = screen_margin.is_finite() && screen_margin > 0.0;
    is_positive
        .then_some(screen_margin)
        .ok_or(HitTestError::InvalidMargin(screen_margin))
}

/// Validates that point coordinates are finite.
///
/// # Preconditions
/// - point.x and point.y must be finite
///
/// # Postconditions
/// - Returns Ok(point) if valid
/// - Returns Err(HitTestError::InvalidPoint) if invalid
fn validate_point(point: Point) -> Result<Point, HitTestError> {
    let is_valid = point.x.is_finite() && point.y.is_finite();
    is_valid
        .then_some(point)
        .ok_or(HitTestError::InvalidPoint(point.x, point.y))
}

/// Computes hit margin in world coordinates from screen-space margin.
///
/// Screen-space behavior: margin appears constant in screen pixels regardless of zoom.
/// At higher zoom, world-space margin gets smaller.
///
/// # Preconditions
/// - screen_margin must be > 0 (validated)
/// - zoom must be in range [MIN_ZOOM, MAX_ZOOM] (validated)
///
/// # Postconditions
/// - Returns screen_margin / zoom
/// - At MIN_ZOOM returns largest world margin
/// - At MAX_ZOOM returns smallest world margin
///
/// # Examples
/// ```
/// // At zoom 0.1: world margin = 50.0
/// screen_to_world_margin(5.0, 0.1).unwrap() == 50.0
///
/// // At zoom 1.0: world margin = 5.0
/// screen_to_world_margin(5.0, 1.0).unwrap() == 5.0
///
/// // At zoom 4.0: world margin = 1.25
/// screen_to_world_margin(5.0, 4.0).unwrap() == 1.25
/// ```
#[must_use]
pub fn screen_to_world_margin(screen_margin: f64, zoom: f64) -> Result<f64, HitTestError> {
    validate_margin(screen_margin)?;
    validate_zoom(zoom)?;
    Ok(screen_margin / zoom)
}

/// Determines if a point hits a rectangle with margin adjusted for zoom.
///
/// Uses screen-space behavior: same screen distance always triggers hit.
///
/// # Preconditions
/// - point.x and point.y must be finite (validated)
/// - rect must be valid (width > 0, height > 0)
/// - zoom must be in range [MIN_ZOOM, MAX_ZOOM] (validated)
/// - screen_margin must be > 0 (validated)
///
/// # Postconditions
/// - Returns true if point is within rect expanded by hit_margin_world
/// - hit_margin_world = screen_margin / zoom
///
/// # Examples
/// ```
/// let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
/// let point = Point::new(105.0, 50.0); // 5 units from edge
///
/// // At zoom 1.0: margin = 5.0, point is on boundary (hit)
/// hit_test_with_margin(point, &rect, 1.0, 5.0).unwrap() == true
///
/// // At zoom 4.0: margin = 1.25, point is outside (no hit)
/// hit_test_with_margin(point, &rect, 4.0, 5.0).unwrap() == false
/// ```
#[must_use]
pub fn hit_test_with_margin(
    point: Point,
    rect: &Rectangle,
    zoom: f64,
    screen_margin: f64,
) -> Result<bool, HitTestError> {
    // Validate all inputs
    let valid_point = validate_point(point)?;
    let valid_margin = validate_margin(screen_margin)?;
    let valid_zoom = validate_zoom(zoom)?;

    // Compute world-space margin from screen-space margin
    let hit_margin_world = valid_margin / valid_zoom;

    // Get AABB and test with margin
    let aabb = rect.aabb();
    let is_within_horizontal = valid_point.x >= aabb.min_x - hit_margin_world
        && valid_point.x <= aabb.max_x + hit_margin_world;
    let is_within_vertical = valid_point.y >= aabb.min_y - hit_margin_world
        && valid_point.y <= aabb.max_y + hit_margin_world;

    Ok(is_within_horizontal && is_within_vertical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rectangle;

    // Helper to create point
    const fn p(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    // Helper to create rectangle
    const fn r(x: f64, y: f64, w: f64, h: f64) -> Rectangle {
        Rectangle::new(x, y, w, h)
    }

    /// GEO-020-T010: Reject zoom below minimum
    #[test]
    fn test_reject_zoom_below_minimum() {
        let result = screen_to_world_margin(5.0, 0.05);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HitTestError::InvalidZoom(_, _, _)));
    }

    /// GEO-020-T011: Reject zoom above maximum
    #[test]
    fn test_reject_zoom_above_maximum() {
        let result = screen_to_world_margin(5.0, 5.0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HitTestError::InvalidZoom(_, _, _)));
    }

    /// GEO-020-T012: Reject negative zoom
    #[test]
    fn test_reject_negative_zoom() {
        let result = screen_to_world_margin(5.0, -1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HitTestError::InvalidZoom(_, _, _)
        ));
    }

    /// GEO-020-T013: Reject zero margin
    #[test]
    fn test_reject_zero_margin() {
        let result = screen_to_world_margin(0.0, 1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HitTestError::InvalidMargin(0.0)
        ));
    }

    /// GEO-020-T014: Reject negative margin
    #[test]
    fn test_reject_negative_margin() {
        let result = screen_to_world_margin(-5.0, 1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HitTestError::InvalidMargin(-5.0)
        ));
    }

    /// GEO-020-T015: Reject NaN point coordinates
    #[test]
    fn test_reject_nan_point() {
        let rect = r(0.0, 0.0, 100.0, 100.0);
        let result = hit_test_with_margin(p(f64::NAN, 50.0), &rect, 1.0, 5.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HitTestError::InvalidPoint(_, _)
        ));
    }

    /// GEO-020-T016: Reject infinite point coordinates
    #[test]
    fn test_reject_infinite_point() {
        let rect = r(0.0, 0.0, 100.0, 100.0);
        let result = hit_test_with_margin(p(f64::INFINITY, 50.0), &rect, 1.0, 5.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HitTestError::InvalidPoint(_, _)
        ));
    }

    /// GEO-020-T030: Verify postcondition Q1 at min zoom
    #[test]
    fn test_postcondition_q1_min_zoom() {
        let result = screen_to_world_margin(5.0, 0.1).unwrap();
        assert!((result - 50.0).abs() < f64::EPSILON);
    }

    /// GEO-020-T031: Verify postcondition Q2 at max zoom
    #[test]
    fn test_postcondition_q2_max_zoom() {
        let result = screen_to_world_margin(5.0, 4.0).unwrap();
        assert!((result - 1.25).abs() < f64::EPSILON);
    }

    /// GEO-020-T032: Verify postcondition Q3 at unit zoom
    #[test]
    fn test_postcondition_q3_unit_zoom() {
        let result = screen_to_world_margin(5.0, 1.0).unwrap();
        assert!((result - 5.0).abs() < f64::EPSILON);
    }

    /// GEO-020-T020: Zoom at exact minimum boundary
    #[test]
    fn test_zoom_at_minimum_boundary() {
        let result = screen_to_world_margin(5.0, MIN_ZOOM);
        assert!(result.is_ok());
        assert!((result.unwrap() - 50.0).abs() < f64::EPSILON);
    }

    /// GEO-020-T021: Zoom at exact maximum boundary
    #[test]
    fn test_zoom_at_maximum_boundary() {
        let result = screen_to_world_margin(5.0, MAX_ZOOM);
        assert!(result.is_ok());
        assert!((result.unwrap() - 1.25).abs() < f64::EPSILON);
    }

    /// GEO-020-T022: Very small screen margin
    #[test]
    fn test_very_small_screen_margin() {
        let result = screen_to_world_margin(0.001, 1.0).unwrap();
        assert!((result - 0.001).abs() < f64::EPSILON);
    }

    /// GEO-020-T023: Very large screen margin at min zoom
    #[test]
    fn test_very_large_screen_margin() {
        let result = screen_to_world_margin(10000.0, MIN_ZOOM).unwrap();
        assert!((result - 100000.0).abs() < f64::EPSILON);
    }

    /// GEO-020-T024: Point exactly on margin boundary
    #[test]
    fn test_point_on_margin_boundary() {
        let rect = r(0.0, 0.0, 100.0, 100.0);
        let point = p(105.0, 50.0); // exactly 5 units from edge
        let result = hit_test_with_margin(point, &rect, 1.0, 5.0).unwrap();
        assert!(result);
    }

    /// GEO-020-T025: Point just outside margin
    #[test]
    fn test_point_just_outside_margin() {
        let rect = r(0.0, 0.0, 100.0, 100.0);
        let point = p(105.1, 50.0); // just outside 5-unit margin
        let result = hit_test_with_margin(point, &rect, 1.0, 5.0).unwrap();
        assert!(!result);
    }

    /// GEO-020-T001: Easy node selection when zoomed out
    #[test]
    fn test_easy_selection_zoomed_out() {
        let rect = r(0.0, 0.0, 100.0, 100.0);
        let point = p(105.0, 50.0); // 5 pixels from edge in screen space
                                    // At zoom 0.1, margin = 50.0 world units, so 5 screen pixels = 50 world units
        let result = hit_test_with_margin(point, &rect, 0.1, 5.0).unwrap();
        assert!(result, "Should select node when zoomed out");
    }

    /// GEO-020-T002: Precise node selection when zoomed in
    #[test]
    fn test_precise_selection_zoomed_in() {
        let rect = r(0.0, 0.0, 100.0, 100.0);
        let point = p(105.0, 50.0); // 5 pixels from edge in screen space
                                    // At zoom 4.0, margin = 1.25 world units, so 5 screen pixels = 1.25 world units
        let result = hit_test_with_margin(point, &rect, 4.0, 5.0).unwrap();
        assert!(!result, "Should NOT select node when zoomed in");
    }

    /// GEO-020-T003: Consistent selection at default zoom
    #[test]
    fn test_consistent_selection_default_zoom() {
        let rect = r(0.0, 0.0, 100.0, 100.0);
        let point = p(105.0, 50.0); // 5 pixels from edge in screen space
        let result = hit_test_with_margin(point, &rect, 1.0, 5.0).unwrap();
        assert!(result, "Should select node at default zoom");
    }

    /// GEO-020-T004: Hit test margin scales with zoom inversely
    #[test]
    fn test_margin_scales_with_zoom_inversely() {
        let margin_01 = screen_to_world_margin(5.0, 0.1).unwrap();
        let margin_10 = screen_to_world_margin(5.0, 1.0).unwrap();
        let margin_40 = screen_to_world_margin(5.0, 4.0).unwrap();

        assert!((margin_01 - 50.0).abs() < f64::EPSILON);
        assert!((margin_10 - 5.0).abs() < f64::EPSILON);
        assert!((margin_40 - 1.25).abs() < f64::EPSILON);

        // Monotonically decreasing
        assert!(margin_01 > margin_10);
        assert!(margin_10 > margin_40);
    }

    /// GEO-020-T033: Verify invariant I1 - screen-space consistency
    #[test]
    fn test_invariant_screen_space_consistency() {
        let rect = r(0.0, 0.0, 100.0, 100.0);
        // Point at 5 screen pixels from edge: at different zooms, this hits differently
        // because world margin changes to maintain same screen hit area

        // At zoom 0.1: world margin = 50, point at 5 world units from edge is WITHIN margin
        let hit_01 = hit_test_with_margin(p(105.0, 50.0), &rect, 0.1, 5.0).unwrap();

        // At zoom 1.0: world margin = 5, point at 5 world units from edge is ON boundary
        let hit_10 = hit_test_with_margin(p(105.0, 50.0), &rect, 1.0, 5.0).unwrap();

        // At zoom 4.0: world margin = 1.25, point at 5 world units from edge is OUTSIDE
        let hit_40 = hit_test_with_margin(p(105.0, 50.0), &rect, 4.0, 5.0).unwrap();

        // Screen-space consistency: lower zoom = larger hit area
        assert!(hit_01);
        assert!(hit_10);
        assert!(!hit_40);
    }

    /// GEO-020-T034: Verify invariant I2 - world margin decreases with zoom
    #[test]
    fn test_invariant_world_margin_decreases_with_zoom() {
        let margin_01 = screen_to_world_margin(5.0, 0.1).unwrap();
        let margin_10 = screen_to_world_margin(5.0, 1.0).unwrap();
        let margin_40 = screen_to_world_margin(5.0, 4.0).unwrap();

        assert!(margin_01 > margin_10);
        assert!(margin_10 > margin_40);
    }
}
