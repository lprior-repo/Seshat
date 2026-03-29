//! Viewport operations
//!
//! High-level operations for viewport manipulation including zoom utilities and reset.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::geometry::AABB;

use super::{ViewportState, MAX_ZOOM, MIN_ZOOM, ZOOM_IN_FACTOR, ZOOM_OUT_FACTOR};

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
    fn test_clamp_zoom() {
        assert!((clamp_zoom(1.0) - 1.0).abs() < f64::EPSILON);
        assert!((clamp_zoom(0.05) - 0.1).abs() < f64::EPSILON);
        assert!((clamp_zoom(10.0) - 4.0).abs() < f64::EPSILON);
        assert!((clamp_zoom(f64::NAN) - 1.0).abs() < f64::EPSILON);
        assert!((clamp_zoom(0.0) - 1.0).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    fn test_calculate_fit_zoom() {
        let content = AABB::new(0.0, 0.0, 500.0, 400.0);
        let zoom = calculate_fit_zoom(&content, 800.0, 600.0, 20.0);

        // Available: 760 x 560, Content: 500 x 400
        // Scale: min(760/500, 560/400) = min(1.52, 1.4) = 1.4
        assert!((zoom - 1.4).abs() < 0.01);
    }

    #[test]
    fn given_viewport_when_applying_reset_then_sets_to_default_values() {
        let mut viewport = ViewportState::new(800.0, 600.0);
        viewport.set_camera(100.0, 200.0);
        viewport.set_zoom(2.0);

        let result = apply_reset(&mut viewport);

        assert!(result);
        assert!((viewport.camera_x()).abs() < f64::EPSILON);
        assert!((viewport.camera_y()).abs() < f64::EPSILON);
        assert!((viewport.zoom() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_valid_and_invalid_zooms_when_checking_validity_then_returns_correct_results() {
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

    #[test]
    fn given_zoom_values_when_clamping_then_respects_bounds() {
        assert!((clamp_zoom(1.0) - 1.0).abs() < f64::EPSILON);
        assert!((clamp_zoom(0.05) - 0.1).abs() < f64::EPSILON);
        assert!((clamp_zoom(10.0) - 4.0).abs() < f64::EPSILON);
        assert!((clamp_zoom(f64::NAN) - 1.0).abs() < f64::EPSILON);
        assert!((clamp_zoom(0.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_content_when_calculating_fit_zoom_then_returns_correct_scale() {
        let content = AABB::new(0.0, 0.0, 500.0, 400.0);
        let zoom = calculate_fit_zoom(&content, 800.0, 600.0, 20.0);
        assert!((zoom - 1.4).abs() < 0.01);
    }

    #[test]
    fn given_zoom_when_zooming_in_and_out_then_applies_factors() {
        assert!((next_zoom_in(1.0) - ZOOM_IN_FACTOR).abs() < f64::EPSILON);
        assert!((next_zoom_out(1.0) - ZOOM_OUT_FACTOR).abs() < f64::EPSILON);
    }
}
