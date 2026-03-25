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
#[path = "transform_tests.rs"]
mod transform_tests;
