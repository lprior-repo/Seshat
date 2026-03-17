//! Coordinate transformation utilities
//!
//! Pure functions for coordinate transformations between screen and world space.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::geometry::{Point, AABB};
use canvas_domain::math::safe_zoom;

/// Transform a screen point to world coordinates
///
/// # Arguments
/// * `screen_x` - X coordinate in screen pixels
/// * `screen_y` - Y coordinate in screen pixels
/// * `camera_x` - Camera X position in world coordinates
/// * `camera_y` - Camera Y position in world coordinates
/// * `zoom` - Current zoom level
///
/// # Formula
/// ```text
/// world_x = camera_x + screen_x / zoom
/// world_y = camera_y + screen_y / zoom
/// ```
#[must_use]
pub fn screen_to_world(
    screen_x: f64,
    screen_y: f64,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
) -> Point {
    let safe_zoom = safe_zoom(zoom).unwrap_or(1.0);

    Point::new(
        camera_x + screen_x / safe_zoom,
        camera_y + screen_y / safe_zoom,
    )
}

/// Transform a world point to screen coordinates
///
/// # Arguments
/// * `world_x` - X coordinate in world space
/// * `world_y` - Y coordinate in world space
/// * `camera_x` - Camera X position in world coordinates
/// * `camera_y` - Camera Y position in world coordinates
/// * `zoom` - Current zoom level
///
/// # Formula
/// ```text
/// screen_x = (world_x - camera_x) * zoom
/// screen_y = (world_y - camera_y) * zoom
/// ```
#[must_use]
pub fn world_to_screen(
    world_x: f64,
    world_y: f64,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
) -> Point {
    let safe_zoom = safe_zoom(zoom).unwrap_or(1.0);

    Point::new(
        (world_x - camera_x) * safe_zoom,
        (world_y - camera_y) * safe_zoom,
    )
}

/// Calculate the scale factor to fit content in viewport
///
/// # Arguments
/// * `content` - Content bounds in world coordinates
/// * `viewport_width` - Viewport width in pixels
/// * `viewport_height` - Viewport height in pixels
/// * `padding` - Padding around content in pixels
///
/// # Returns
/// Scale factor that fits content while preserving aspect ratio
#[must_use]
pub fn fit_scale(content: &AABB, viewport_width: f64, viewport_height: f64, padding: f64) -> f64 {
    let content_width = content.width();
    let content_height = content.height();

    if content_width <= 0.0 || content_height <= 0.0 {
        return 1.0;
    }

    let available_width = 2.0f64.mul_add(-padding, viewport_width).max(1.0);
    let available_height = 2.0f64.mul_add(-padding, viewport_height).max(1.0);

    let scale_x = available_width / content_width;
    let scale_y = available_height / content_height;

    scale_x.min(scale_y)
}

/// Calculate camera position to center content in viewport
///
/// # Arguments
/// * `content` - Content bounds in world coordinates
/// * `scale` - Scale factor to apply
/// * `viewport_width` - Viewport width in pixels
/// * `viewport_height` - Viewport height in pixels
///
/// # Returns
/// (`camera_x`, `camera_y`) to center content
#[must_use]
pub fn center_camera_for_content(
    content: &AABB,
    scale: f64,
    viewport_width: f64,
    viewport_height: f64,
) -> (f64, f64) {
    let content_center = content.center();

    // Camera position such that content center is at viewport center
    let camera_x = content_center.x - viewport_width / 2.0 / scale;
    let camera_y = content_center.y - viewport_height / 2.0 / scale;

    (camera_x, camera_y)
}

#[cfg(test)]
mod tests {

    const TOLERANCE: f64 = 1e-9;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_screen_to_world_origin() {
        let result = screen_to_world(0.0, 0.0, 0.0, 0.0, 1.0);
        assert!((result.x - 0.0).abs() < TOLERANCE);
        assert!((result.y - 0.0).abs() < TOLERANCE);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_screen_to_world_with_camera() {
        // Given: camera at (100, 200), zoom 2.0
        // When: convert screen (400, 300)
        // Then: world = (100 + 400/2, 200 + 300/2) = (300, 350)
        let result = screen_to_world(400.0, 300.0, 100.0, 200.0, 2.0);
        assert!((result.x - 300.0).abs() < TOLERANCE);
        assert!((result.y - 350.0).abs() < TOLERANCE);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_world_to_screen_origin() {
        let result = world_to_screen(0.0, 0.0, 0.0, 0.0, 1.0);
        assert!((result.x - 0.0).abs() < TOLERANCE);
        assert!((result.y - 0.0).abs() < TOLERANCE);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_world_to_screen_with_camera() {
        // Given: camera at (100, 200), zoom 2.0
        // When: convert world (300, 350)
        // Then: screen = ((300-100)*2, (350-200)*2) = (400, 300)
        let result = world_to_screen(300.0, 350.0, 100.0, 200.0, 2.0);
        assert!((result.x - 400.0).abs() < TOLERANCE);
        assert!((result.y - 300.0).abs() < TOLERANCE);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_roundtrip_transform() {
        let screen_x = 400.0;
        let screen_y = 300.0;
        let camera_x = 100.0;
        let camera_y = 200.0;
        let zoom = 2.0;

        let world = screen_to_world(screen_x, screen_y, camera_x, camera_y, zoom);
        let screen_back = world_to_screen(world.x, world.y, camera_x, camera_y, zoom);

        assert!((screen_back.x - screen_x).abs() < TOLERANCE);
        assert!((screen_back.y - screen_y).abs() < TOLERANCE);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_screen_to_world_invalid_zoom() {
        // Invalid zoom should use default of 1.0
        let result = screen_to_world(100.0, 100.0, 0.0, 0.0, 0.0);
        assert!((result.x - 100.0).abs() < TOLERANCE);

        let result = screen_to_world(100.0, 100.0, 0.0, 0.0, -1.0);
        assert!((result.x - 100.0).abs() < TOLERANCE);

        let result = screen_to_world(100.0, 100.0, 0.0, 0.0, f64::NAN);
        assert!((result.x - 100.0).abs() < TOLERANCE);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_fit_scale_basic() {
        let content = AABB::new(0.0, 0.0, 100.0, 100.0);
        let scale = fit_scale(&content, 200.0, 200.0, 0.0);
        assert!((scale - 2.0).abs() < TOLERANCE);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_fit_scale_with_padding() {
        let content = AABB::new(0.0, 0.0, 100.0, 100.0);
        let scale = fit_scale(&content, 120.0, 120.0, 10.0);
        // Available: 100x100, Content: 100x100, Scale: 1.0
        assert!((scale - 1.0).abs() < TOLERANCE);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_fit_scale_preserves_aspect() {
        let content = AABB::new(0.0, 0.0, 200.0, 100.0); // 2:1 aspect
        let scale = fit_scale(&content, 100.0, 100.0, 0.0);
        // Should fit width: 100/200 = 0.5
        // Should fit height: 100/100 = 1.0
        // Use minimum to fit both: 0.5
        assert!((scale - 0.5).abs() < TOLERANCE);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_screen_to_world_uses_epsilon_threshold() {
        let result = screen_to_world(100.0, 100.0, 0.0, 0.0, f64::EPSILON / 2.0);
        assert!((result.x - 100.0).abs() < TOLERANCE);

        let result = screen_to_world(100.0, 100.0, 0.0, 0.0, f64::EPSILON * 2.0);
        assert!((result.x - (100.0 / (f64::EPSILON * 2.0))).abs() < TOLERANCE);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_world_to_screen_uses_epsilon_threshold() {
        let result = world_to_screen(100.0, 100.0, 0.0, 0.0, f64::EPSILON / 2.0);
        assert!((result.x - 100.0).abs() < TOLERANCE);

        let result = world_to_screen(100.0, 100.0, 0.0, 0.0, f64::EPSILON * 2.0);
        assert!((result.x - (100.0 * f64::EPSILON * 2.0)).abs() < TOLERANCE);
    }
}
