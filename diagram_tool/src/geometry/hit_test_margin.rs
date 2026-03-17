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

use super::{MAX_ZOOM, MIN_ZOOM};
use crate::geometry::{Point, Rectangle};

/// Errors that can occur during hit test operations
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
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
/// - zoom must be in range [`MIN_ZOOM`, `MAX_ZOOM`]
///
/// # Postconditions
/// - Returns Ok(zoom) if valid
/// - Returns `Err(HitTestError::InvalidZoom)` if invalid
fn validate_zoom(zoom: f64) -> Result<f64, HitTestError> {
    let in_range = zoom.is_finite() && (MIN_ZOOM..=MAX_ZOOM).contains(&zoom);
    in_range
        .then_some(zoom)
        .ok_or(HitTestError::InvalidZoom(zoom, MIN_ZOOM, MAX_ZOOM))
}

/// Validates that a screen margin is positive.
///
/// # Preconditions
/// - `screen_margin` must be > 0
///
/// # Postconditions
/// - Returns `Ok(screen_margin)` if valid
/// - Returns `Err(HitTestError::InvalidMargin)` if invalid
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
/// - Returns `Err(HitTestError::InvalidPoint)` if invalid
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
/// - `screen_margin` must be > 0 (validated)
/// - zoom must be in range [`MIN_ZOOM`, `MAX_ZOOM`] (validated)
///
/// # Postconditions
/// - Returns `screen_margin` / zoom
/// - At `MIN_ZOOM` returns largest world margin
/// - At `MAX_ZOOM` returns smallest world margin
///
/// # Examples
/// ```
/// use diagram_tool::geometry::screen_to_world_margin;
///
/// // At zoom 0.1: world margin = 50.0
/// assert_eq!(screen_to_world_margin(5.0, 0.1).unwrap(), 50.0);
///
/// // At zoom 1.0: world margin = 5.0
/// assert_eq!(screen_to_world_margin(5.0, 1.0).unwrap(), 5.0);
///
/// // At zoom 4.0: world margin = 1.25
/// assert_eq!(screen_to_world_margin(5.0, 4.0).unwrap(), 1.25);
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
/// - zoom must be in range [`MIN_ZOOM`, `MAX_ZOOM`] (validated)
/// - `screen_margin` must be > 0 (validated)
///
/// # Postconditions
/// - Returns true if point is within rect expanded by `hit_margin_world`
/// - `hit_margin_world` = `screen_margin` / zoom
///
/// # Examples
/// ```
/// use diagram_tool::geometry::{Point, Rectangle, hit_test_with_margin};
///
/// let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
/// let point = Point::new(105.0, 50.0); // 5 units from edge
///
/// // At zoom 1.0: margin = 5.0, point is on boundary (hit)
/// assert_eq!(hit_test_with_margin(point, &rect, 1.0, 5.0).unwrap(), true);
///
/// // At zoom 4.0: margin = 1.25, point is outside (no hit)
/// assert_eq!(hit_test_with_margin(point, &rect, 4.0, 5.0).unwrap(), false);
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
