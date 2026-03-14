//! Viewport operations
//!
//! High-level operations for viewport manipulation including pan, zoom, and fit.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::geometry::AABB;

use super::{
    FitTransform, ViewportError, ViewportState, MAX_ZOOM, MIN_ZOOM, ZOOM_IN_FACTOR, ZOOM_OUT_FACTOR,
};

/// Apply a pan operation to a viewport
///
/// # Arguments
/// * `viewport` - The viewport to modify
/// * `dx` - Pan delta in screen pixels
/// * `dy` - Pan delta in screen pixels
///
/// # Returns
/// true if the viewport was modified
#[must_use]
pub fn apply_pan(viewport: &mut ViewportState, dx: f64, dy: f64) -> bool {
    viewport.pan(dx, dy)
}

/// Apply a zoom in operation
///
/// # Arguments
/// * `viewport` - The viewport to modify
///
/// # Returns
/// true if the viewport was modified
#[must_use]
pub fn apply_zoom_in(viewport: &mut ViewportState) -> bool {
    viewport.zoom_in()
}

/// Apply a zoom out operation
///
/// # Arguments
/// * `viewport` - The viewport to modify
///
/// # Returns
/// true if the viewport was modified
#[must_use]
pub fn apply_zoom_out(viewport: &mut ViewportState) -> bool {
    viewport.zoom_out()
}

/// Apply a zoom to specific level
///
/// # Arguments
/// * `viewport` - The viewport to modify
/// * `zoom` - Target zoom level
///
/// # Returns
/// true if the viewport was modified
#[must_use]
pub fn apply_zoom_to(viewport: &mut ViewportState, zoom: f64) -> bool {
    viewport.set_zoom(zoom)
}

/// Apply a zoom around a specific screen point
///
/// # Arguments
/// * `viewport` - The viewport to modify
/// * `zoom` - Target zoom level
/// * `screen_x` - Screen X coordinate to zoom around
/// * `screen_y` - Screen Y coordinate to zoom around
///
/// # Returns
/// true if the viewport was modified
#[must_use]
pub fn apply_zoom_around_point(
    viewport: &mut ViewportState,
    zoom: f64,
    screen_x: f64,
    screen_y: f64,
) -> bool {
    viewport.zoom_around_point(zoom, screen_x, screen_y)
}

/// Center the viewport on a world point
///
/// # Arguments
/// * `viewport` - The viewport to modify
/// * `world_x` - World X coordinate to center on
/// * `world_y` - World Y coordinate to center on
pub fn apply_center_on(viewport: &mut ViewportState, world_x: f64, world_y: f64) {
    viewport.center_on(world_x, world_y);
}

/// Fit content to viewport
///
/// # Arguments
/// * `viewport` - The viewport to modify
/// * `content` - Content bounds to fit
/// * `padding` - Padding around content
///
/// # Returns
/// The fit transform if successful
///
/// # Errors
/// Returns error if padding is negative, content bounds are invalid, or coordinates overflow
pub fn apply_fit_to_content(
    viewport: &mut ViewportState,
    content: &AABB,
    padding: f64,
) -> Result<FitTransform, ViewportError> {
    let fit = viewport.fit_to_content(content, padding)?;
    viewport.apply_fit(fit);
    Ok(fit)
}

/// Calculate zoom level to fit content
///
/// # Arguments
/// * `content` - Content bounds
/// * `viewport_width` - Viewport width
/// * `viewport_height` - Viewport height
/// * `padding` - Padding around content
///
/// # Returns
/// Zoom level that fits content, clamped to [`MIN_ZOOM`, `MAX_ZOOM`]
#[must_use]
pub fn calculate_fit_zoom(
    content: &AABB,
    viewport_width: f64,
    viewport_height: f64,
    padding: f64,
) -> f64 {
    let content_width = content.width();
    let content_height = content.height();

    if content_width <= 0.0 || content_height <= 0.0 {
        return 1.0;
    }

    let available_width = 2.0f64.mul_add(-padding, viewport_width).max(1.0);
    let available_height = 2.0f64.mul_add(-padding, viewport_height).max(1.0);

    let scale_x = available_width / content_width;
    let scale_y = available_height / content_height;

    scale_x.min(scale_y).clamp(MIN_ZOOM, MAX_ZOOM)
}

/// Reset viewport to default state
///
/// # Arguments
/// * `viewport` - The viewport to reset
///
/// # Returns
/// true if the viewport was modified
#[must_use]
pub fn apply_reset(viewport: &mut ViewportState) -> bool {
    const EPSILON: f64 = 1e-9;
    let changed = (viewport.zoom() - 1.0).abs() > EPSILON
        || (viewport.camera_x() - 0.0).abs() > EPSILON
        || (viewport.camera_y() - 0.0).abs() > EPSILON;

    if changed {
        viewport.set_zoom(1.0);
        viewport.set_camera(0.0, 0.0);
    }

    changed
}

/// Check if a zoom level is valid
///
/// # Arguments
/// * `zoom` - Zoom level to check
///
/// # Returns
/// true if zoom is finite, positive, and within bounds
#[must_use]
pub const fn is_valid_zoom(zoom: f64) -> bool {
    zoom.is_finite() && zoom >= MIN_ZOOM && zoom <= MAX_ZOOM
}

/// Clamp a zoom value to valid bounds
#[must_use]
pub fn clamp_zoom(zoom: f64) -> f64 {
    if zoom.is_finite() && zoom > 0.0 {
        zoom.clamp(MIN_ZOOM, MAX_ZOOM)
    } else {
        1.0
    }
}

/// Get the next zoom level when zooming in
#[must_use]
pub fn next_zoom_in(current: f64) -> f64 {
    clamp_zoom(current * ZOOM_IN_FACTOR)
}

/// Get the next zoom level when zooming out
#[must_use]
pub fn next_zoom_out(current: f64) -> f64 {
    clamp_zoom(current * ZOOM_OUT_FACTOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_pan_basic() {
        let mut viewport = ViewportState::new(800.0, 600.0);
        let result = apply_pan(&mut viewport, 100.0, 50.0);

        assert!(result);
        assert!((viewport.camera_x() - (-100.0)).abs() < f64::EPSILON);
        assert!((viewport.camera_y() - (-50.0)).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_zoom_in() {
        let mut viewport = ViewportState::new(800.0, 600.0);
        let result = apply_zoom_in(&mut viewport);

        assert!(result);
        assert!((viewport.zoom() - ZOOM_IN_FACTOR).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_zoom_out() {
        let mut viewport = ViewportState::new(800.0, 600.0);
        let result = apply_zoom_out(&mut viewport);

        assert!(result);
        assert!((viewport.zoom() - ZOOM_OUT_FACTOR).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_zoom_to() {
        let mut viewport = ViewportState::new(800.0, 600.0);
        let result = apply_zoom_to(&mut viewport, 2.0);

        assert!(result);
        assert!((viewport.zoom() - 2.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_zoom_to_bounds() {
        let mut viewport = ViewportState::new(800.0, 600.0);

        // Try to set zoom beyond max
        let result = apply_zoom_to(&mut viewport, 10.0);
        assert!(result); // Changed because clamped
        assert!((viewport.zoom() - MAX_ZOOM).abs() < f64::EPSILON);

        // Try to set zoom below min
        let result = apply_zoom_to(&mut viewport, 0.01);
        assert!(result); // Changed because clamped
        assert!((viewport.zoom() - MIN_ZOOM).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_reset() {
        let mut viewport = ViewportState::new(800.0, 600.0);
        viewport.set_camera(100.0, 200.0);
        viewport.set_zoom(2.0);

        let result = apply_reset(&mut viewport);

        assert!(result);
        assert!((viewport.camera_x()).abs() < f64::EPSILON);
        assert!((viewport.camera_y()).abs() < f64::EPSILON);
        assert!((viewport.zoom() - 1.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_is_valid_zoom() {
        assert!(is_valid_zoom(1.0));
        assert!(is_valid_zoom(0.1));
        assert!(is_valid_zoom(4.0));
        assert!(!is_valid_zoom(0.0));
        assert!(!is_valid_zoom(-1.0));
        assert!(!is_valid_zoom(0.05));
        assert!(!is_valid_zoom(5.0));
        assert!(!is_valid_zoom(f64::NAN));
        assert!(!is_valid_zoom(f64::INFINITY));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_clamp_zoom() {
        assert!((clamp_zoom(1.0) - 1.0).abs() < f64::EPSILON);
        assert!((clamp_zoom(0.05) - 0.1).abs() < f64::EPSILON);
        assert!((clamp_zoom(10.0) - 4.0).abs() < f64::EPSILON);
        assert!((clamp_zoom(f64::NAN) - 1.0).abs() < f64::EPSILON);
        assert!((clamp_zoom(0.0) - 1.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_calculate_fit_zoom() {
        let content = AABB::new(0.0, 0.0, 500.0, 400.0);
        let zoom = calculate_fit_zoom(&content, 800.0, 600.0, 20.0);

        // Available: 760 x 560, Content: 500 x 400
        // Scale: min(760/500, 560/400) = min(1.52, 1.4) = 1.4
        assert!((zoom - 1.4).abs() < 0.01);
    }
}
