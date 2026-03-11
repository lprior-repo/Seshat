//! Tests for Viewport/Camera operations (CAM-001 to CAM-012)
//!
//! This module implements all 12 viewport test cases as specified in the
//! contract specification and Martin Fowler BDD tests.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::geometry::AABB;

const TOLERANCE: f64 = 1e-9;

// ============================================================================
// CAM-001: Pan Viewport Basic
// ============================================================================

#[test]
fn cam_001_pan_viewport_basic() {
    // Given: a viewport with camera at origin (0, 0) and zoom 1.0
    let mut viewport = ViewportState::new(800.0, 600.0);
    assert!((viewport.camera_x() - 0.0).abs() < TOLERANCE);
    assert!((viewport.camera_y() - 0.0).abs() < TOLERANCE);
    assert!((viewport.zoom() - 1.0).abs() < TOLERANCE);

    // When: the user pans by screen delta (100, 50)
    let result = viewport.pan(100.0, 50.0);

    // Then: the camera position becomes (-100, -50)
    assert!(result, "Pan should return true when changed");
    assert!((viewport.camera_x() - (-100.0)).abs() < TOLERANCE);
    assert!((viewport.camera_y() - (-50.0)).abs() < TOLERANCE);
}

#[test]
fn cam_001_pan_viewport_basic_negative() {
    // Given: a viewport at origin
    let mut viewport = ViewportState::new(800.0, 600.0);

    // When: pan left and up (negative deltas)
    let result = viewport.pan(-100.0, -50.0);

    // Then: camera moves positive
    assert!(result);
    assert!((viewport.camera_x() - 100.0).abs() < TOLERANCE);
    assert!((viewport.camera_y() - 50.0).abs() < TOLERANCE);
}

// ============================================================================
// CAM-002: Pan with Bounds Checking
// ============================================================================

#[test]
fn cam_002_pan_with_bounds_checking_max() {
    // Given: a viewport near max bounds
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_camera(9500.0, 9500.0);

    // When: pan left (negative dx/dy means camera increases)
    // Pan negative = camera moves positive
    let _ = viewport.pan(-1000.0, -1000.0);

    // Then: clamped to max
    assert!((viewport.camera_x() - MAX_PAN_DISTANCE).abs() < TOLERANCE);
    assert!((viewport.camera_y() - MAX_PAN_DISTANCE).abs() < TOLERANCE);
}

#[test]
fn cam_002_pan_with_bounds_checking_min() {
    // Given: a viewport near min bounds
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_camera(-9500.0, -9500.0);

    // When: pan right (positive dx/dy means camera decreases)
    // Pan positive = camera moves negative
    let _ = viewport.pan(1000.0, 1000.0);

    // Then: clamped to min
    assert!((viewport.camera_x() - (-MAX_PAN_DISTANCE)).abs() < TOLERANCE);
    assert!((viewport.camera_y() - (-MAX_PAN_DISTANCE)).abs() < TOLERANCE);
}

#[test]
fn cam_002_pan_with_nan_delta() {
    // Given: a viewport at origin
    let mut viewport = ViewportState::new(800.0, 600.0);

    // When: pan with NaN
    let result = viewport.pan(f64::NAN, 50.0);

    // Then: no change
    assert!(!result);
    assert!((viewport.camera_x()).abs() < TOLERANCE);
}

// ============================================================================
// CAM-003: Zoom In Operation
// ============================================================================

#[test]
fn cam_003_zoom_in_operation() {
    // Given: a viewport with zoom 1.0
    let mut viewport = ViewportState::new(800.0, 600.0);
    assert!((viewport.zoom() - 1.0).abs() < TOLERANCE);

    // When: zoom in with factor 1.25
    let result = viewport.zoom_in();

    // Then: zoom becomes 1.25
    assert!(result);
    assert!((viewport.zoom() - ZOOM_IN_FACTOR).abs() < TOLERANCE);
}

#[test]
fn cam_003_zoom_in_multiple_times() {
    // Given: a viewport at zoom 1.0
    let mut viewport = ViewportState::new(800.0, 600.0);

    // When: zoom in multiple times
    viewport.zoom_in();
    viewport.zoom_in();
    viewport.zoom_in();

    // Then: zoom compounds
    let expected = 1.0 * ZOOM_IN_FACTOR * ZOOM_IN_FACTOR * ZOOM_IN_FACTOR;
    assert!((viewport.zoom() - expected).abs() < TOLERANCE);
}

// ============================================================================
// CAM-004: Zoom Out Operation
// ============================================================================

#[test]
fn cam_004_zoom_out_operation() {
    // Given: a viewport with zoom 1.0
    let mut viewport = ViewportState::new(800.0, 600.0);
    assert!((viewport.zoom() - 1.0).abs() < TOLERANCE);

    // When: zoom out with factor 0.8
    let result = viewport.zoom_out();

    // Then: zoom becomes 0.8
    assert!(result);
    assert!((viewport.zoom() - ZOOM_OUT_FACTOR).abs() < TOLERANCE);
}

#[test]
fn cam_004_zoom_out_multiple_times() {
    // Given: a viewport at zoom 1.0
    let mut viewport = ViewportState::new(800.0, 600.0);

    // When: zoom out multiple times
    viewport.zoom_out();
    viewport.zoom_out();

    // Then: zoom compounds but stays above minimum
    let expected = 1.0 * ZOOM_OUT_FACTOR * ZOOM_OUT_FACTOR;
    assert!((viewport.zoom() - expected).abs() < TOLERANCE);
    assert!(viewport.zoom() >= MIN_ZOOM);
}

// ============================================================================
// CAM-005: Zoom to Specific Level
// ============================================================================

#[test]
fn cam_005_zoom_to_specific_level() {
    // Given: a viewport with zoom 1.0
    let mut viewport = ViewportState::new(800.0, 600.0);

    // When: set zoom to 2.0
    let result = viewport.set_zoom(2.0);

    // Then: zoom becomes 2.0
    assert!(result);
    assert!((viewport.zoom() - 2.0).abs() < TOLERANCE);
}

#[test]
fn cam_005_zoom_to_same_level() {
    // Given: a viewport with zoom 2.0
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_zoom(2.0);

    // When: set zoom to same value
    let result = viewport.set_zoom(2.0);

    // Then: no change reported
    assert!(!result);
}

// ============================================================================
// CAM-006: Zoom with Bounds
// ============================================================================

#[test]
fn cam_006_zoom_at_maximum() {
    // Given: a viewport at zoom 4.0 (at maximum)
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_zoom(MAX_ZOOM);

    // When: try to zoom in
    let result = viewport.zoom_in();

    // Then: no change
    assert!(!result);
    assert!((viewport.zoom() - MAX_ZOOM).abs() < TOLERANCE);
}

#[test]
fn cam_006_zoom_at_minimum() {
    // Given: a viewport at zoom 0.1 (at minimum)
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_zoom(MIN_ZOOM);

    // When: try to zoom out
    let result = viewport.zoom_out();

    // Then: no change
    assert!(!result);
    assert!((viewport.zoom() - MIN_ZOOM).abs() < TOLERANCE);
}

#[test]
fn cam_006_zoom_clamped_high() {
    // Given: any viewport
    let mut viewport = ViewportState::new(800.0, 600.0);

    // When: try to set zoom beyond max
    viewport.set_zoom(100.0);

    // Then: clamped to max
    assert!((viewport.zoom() - MAX_ZOOM).abs() < TOLERANCE);
}

#[test]
fn cam_006_zoom_clamped_low() {
    // Given: any viewport
    let mut viewport = ViewportState::new(800.0, 600.0);

    // When: try to set zoom below min
    viewport.set_zoom(0.001);

    // Then: clamped to min
    assert!((viewport.zoom() - MIN_ZOOM).abs() < TOLERANCE);
}

// ============================================================================
// CAM-007: Screen to World Transform
// ============================================================================

#[test]
fn cam_007_screen_to_world_transform() {
    // Given: a viewport with camera (100, 200) and zoom 2.0
    let viewport = ViewportState::with_camera_and_zoom(800.0, 600.0, 100.0, 200.0, 2.0);

    // When: converting screen point (400, 300)
    let world = viewport.screen_to_world(400.0, 300.0);

    // Then: world point is (300, 350)
    // world_x = camera_x + screen_x / zoom = 100 + 400/2 = 300
    // world_y = camera_y + screen_y / zoom = 200 + 300/2 = 350
    assert!((world.x - 300.0).abs() < TOLERANCE);
    assert!((world.y - 350.0).abs() < TOLERANCE);
}

#[test]
fn cam_007_screen_to_world_origin() {
    // Given: a viewport at origin with zoom 1.0
    let viewport = ViewportState::new(800.0, 600.0);

    // When: converting screen origin
    let world = viewport.screen_to_world(0.0, 0.0);

    // Then: world is at camera position
    assert!((world.x - viewport.camera_x()).abs() < TOLERANCE);
    assert!((world.y - viewport.camera_y()).abs() < TOLERANCE);
}

#[test]
fn cam_007_screen_to_world_with_zoom() {
    // Given: a viewport at origin with zoom 2.0
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_zoom(2.0);

    // When: converting screen (200, 200)
    let world = viewport.screen_to_world(200.0, 200.0);

    // Then: world is (100, 100) because zoom doubles apparent size
    assert!((world.x - 100.0).abs() < TOLERANCE);
    assert!((world.y - 100.0).abs() < TOLERANCE);
}

// ============================================================================
// CAM-008: World to Screen Transform
// ============================================================================

#[test]
fn cam_008_world_to_screen_transform() {
    // Given: a viewport with camera (100, 200) and zoom 2.0
    let viewport = ViewportState::with_camera_and_zoom(800.0, 600.0, 100.0, 200.0, 2.0);

    // When: converting world point (300, 350)
    let screen = viewport.world_to_screen(300.0, 350.0);

    // Then: screen point is (400, 300)
    // screen_x = (world_x - camera_x) * zoom = (300 - 100) * 2 = 400
    // screen_y = (world_y - camera_y) * zoom = (350 - 200) * 2 = 300
    assert!((screen.x - 400.0).abs() < TOLERANCE);
    assert!((screen.y - 300.0).abs() < TOLERANCE);
}

#[test]
fn cam_008_world_to_screen_camera_origin() {
    // Given: a viewport at origin with zoom 1.0
    let viewport = ViewportState::new(800.0, 600.0);

    // When: converting world point at camera position
    let screen = viewport.world_to_screen(0.0, 0.0);

    // Then: screen is origin
    assert!((screen.x).abs() < TOLERANCE);
    assert!((screen.y).abs() < TOLERANCE);
}

#[test]
fn cam_008_world_to_screen_with_zoom() {
    // Given: a viewport at origin with zoom 2.0
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_zoom(2.0);

    // When: converting world (100, 100)
    let screen = viewport.world_to_screen(100.0, 100.0);

    // Then: screen is (200, 200)
    assert!((screen.x - 200.0).abs() < TOLERANCE);
    assert!((screen.y - 200.0).abs() < TOLERANCE);
}

// ============================================================================
// CAM-009: Fit Content to Viewport
// ============================================================================

#[test]
fn cam_009_fit_content_to_viewport() {
    // Given: content bounds AABB(0, 0, 500, 400) and viewport (800, 600)
    let content = AABB::new(0.0, 0.0, 500.0, 400.0);
    let viewport = ViewportState::new(800.0, 600.0);

    // When: fitting content with padding 20
    let fit = viewport.fit_to_content(&content, 20.0);

    // Then: fit transform is calculated
    assert!(fit.is_ok());
    let fit = fit.unwrap();

    // Available: 760 x 560, Content: 500 x 400
    // Scale: min(760/500, 560/400) = min(1.52, 1.4) = 1.4
    assert!((fit.scale - 1.4).abs() < 0.01);
}

#[test]
fn cam_009_fit_content_empty() {
    // Given: empty content (zero size)
    let content = AABB::new(0.0, 0.0, 0.0, 0.0);
    let viewport = ViewportState::new(800.0, 600.0);

    // When: fitting
    let fit = viewport.fit_to_content(&content, 20.0);

    // Then: no valid fit
    assert!(fit.is_err());
}

#[test]
fn cam_009_fit_content_preserves_aspect_ratio() {
    // Given: content with 2:1 aspect ratio
    let content = AABB::new(0.0, 0.0, 200.0, 100.0);
    let viewport = ViewportState::new(100.0, 100.0);

    // When: fitting to square viewport
    let fit = viewport.fit_to_content(&content, 0.0);

    // Then: scale fits width (smaller dimension)
    assert!(fit.is_ok());
    let fit = fit.unwrap();
    // 100/200 = 0.5 fits width, 100/100 = 1.0 fits height
    // Use minimum to fit both
    assert!((fit.scale - 0.5).abs() < TOLERANCE);
}

// ============================================================================
// CAM-010: Center on Specific Point
// ============================================================================

#[test]
fn cam_010_center_on_specific_point() {
    // Given: a viewport at camera (0, 0) with zoom 1.0 and size (800, 600)
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_zoom(1.0);

    // When: centering on world point (250, 300)
    viewport.center_on(250.0, 300.0);

    // Then: camera moves to center that point
    // camera_x = point_x - viewport_width / 2 / zoom = 250 - 400 = -150
    // camera_y = point_y - viewport_height / 2 / zoom = 300 - 300 = 0
    assert!((viewport.camera_x() - (-150.0)).abs() < TOLERANCE);
    assert!((viewport.camera_y() - 0.0).abs() < TOLERANCE);
}

#[test]
fn cam_010_center_with_zoom() {
    // Given: a viewport with zoom 2.0
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_zoom(2.0);

    // When: centering on (250, 300)
    viewport.center_on(250.0, 300.0);

    // Then: camera accounts for zoom
    // camera_x = 250 - 400/2 = 250 - 200 = 50
    // camera_y = 300 - 300/2 = 300 - 150 = 150
    assert!((viewport.camera_x() - 50.0).abs() < TOLERANCE);
    assert!((viewport.camera_y() - 150.0).abs() < TOLERANCE);
}

// ============================================================================
// CAM-011: Zoom Around Point
// ============================================================================

#[test]
fn cam_011_zoom_around_point() {
    // Given: a viewport at zoom 1.0 with mouse at screen (400, 300)
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_camera(0.0, 0.0);
    viewport.set_zoom(1.0);

    let screen_point = (400.0, 300.0);
    let world_before = viewport.screen_to_world(screen_point.0, screen_point.1);

    // When: zooming to 2.0x around that point
    let result = viewport.zoom_around_point(2.0, screen_point.0, screen_point.1);

    // Then: world point under cursor is same
    assert!(result);
    let world_after = viewport.screen_to_world(screen_point.0, screen_point.1);
    assert!((world_before.x - world_after.x).abs() < TOLERANCE);
    assert!((world_before.y - world_after.y).abs() < TOLERANCE);
    assert!((viewport.zoom() - 2.0).abs() < TOLERANCE);
}

#[test]
fn cam_011_zoom_around_corner() {
    // Given: a viewport at origin
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_camera(0.0, 0.0);
    viewport.set_zoom(1.0);

    // When: zooming around top-left corner
    let screen_point = (0.0, 0.0);
    let world_before = viewport.screen_to_world(screen_point.0, screen_point.1);

    viewport.zoom_around_point(2.0, screen_point.0, screen_point.1);

    // Then: top-left world point stays under cursor
    let world_after = viewport.screen_to_world(screen_point.0, screen_point.1);
    assert!((world_before.x - world_after.x).abs() < TOLERANCE);
    assert!((world_before.y - world_after.y).abs() < TOLERANCE);
}

// ============================================================================
// CAM-012: Viewport State Persistence
// ============================================================================

#[test]
fn cam_012_viewport_state_persistence() {
    // Given: a viewport with camera (100, 200) and zoom 1.5
    let original = ViewportState::with_camera_and_zoom(800.0, 600.0, 100.0, 200.0, 1.5);

    // When: serializing and deserializing
    let json = serde_json::to_string(&original).unwrap();
    let restored: ViewportState = serde_json::from_str(&json).unwrap();

    // Then: state is preserved
    assert!((restored.camera_x() - 100.0).abs() < TOLERANCE);
    assert!((restored.camera_y() - 200.0).abs() < TOLERANCE);
    assert!((restored.zoom() - 1.5).abs() < TOLERANCE);
}

#[test]
fn cam_012_viewport_state_default_persistence() {
    // Given: a default viewport
    let original = ViewportState::default();

    // When: serializing and deserializing
    let json = serde_json::to_string(&original).unwrap();
    let restored: ViewportState = serde_json::from_str(&json).unwrap();

    // Then: state matches
    assert_eq!(original, restored);
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_coordinate_roundtrip(
            screen_x in 0.0_f64..1920.0,
            screen_y in 0.0_f64..1080.0,
            camera_x in -1000.0_f64..1000.0,
            camera_y in -1000.0_f64..1000.0,
            zoom in 0.1_f64..4.0
        ) {
            let viewport = ViewportState::with_camera_and_zoom(
                1920.0, 1080.0, camera_x, camera_y, zoom
            );

            let world = viewport.screen_to_world(screen_x, screen_y);
            let screen_back = viewport.world_to_screen(world.x, world.y);

            prop_assert!((screen_back.x - screen_x).abs() < 0.001);
            prop_assert!((screen_back.y - screen_y).abs() < 0.001);
        }

        #[test]
        fn prop_zoom_always_bounded(zoom_factor in 0.001_f64..1000.0) {
            let mut viewport = ViewportState::new(800.0, 600.0);
            viewport.set_zoom(1.0);

            let _ = viewport.zoom_by_factor(zoom_factor);

            prop_assert!(viewport.zoom() >= MIN_ZOOM);
            prop_assert!(viewport.zoom() <= MAX_ZOOM);
        }

        #[test]
        fn prop_pan_keeps_finite(dx in -10000.0_f64..10000.0, dy in -10000.0_f64..10000.0) {
            let mut viewport = ViewportState::new(800.0, 600.0);

            let _ = viewport.pan(dx, dy);

            prop_assert!(viewport.camera_x().is_finite());
            prop_assert!(viewport.camera_y().is_finite());
        }

        #[test]
        fn prop_visible_bounds_contains_origin_after_reset(
            camera_x in -1000.0_f64..1000.0,
            camera_y in -1000.0_f64..1000.0,
            zoom in 0.1_f64..4.0
        ) {
            let mut viewport = ViewportState::with_camera_and_zoom(
                800.0, 600.0, camera_x, camera_y, zoom
            );

            // After centering on origin, origin should be visible
            viewport.center_on(0.0, 0.0);
            let bounds = viewport.visible_world_bounds();

            prop_assert!(bounds.min_x <= 0.0);
            prop_assert!(bounds.max_x >= 0.0);
            prop_assert!(bounds.min_y <= 0.0);
            prop_assert!(bounds.max_y >= 0.0);
        }
    }
}

// ============================================================================
// Invariant Tests
// ============================================================================

#[test]
fn invariant_zoom_bounds() {
    let mut viewport = ViewportState::new(800.0, 600.0);

    // Try various invalid values
    for invalid in [0.0, -1.0, 0.01, 10.0, 100.0, f64::NAN, f64::INFINITY] {
        viewport.set_zoom(invalid);
        assert!(
            viewport.zoom() >= MIN_ZOOM && viewport.zoom() <= MAX_ZOOM,
            "Zoom {} should be clamped to [{}, {}]",
            viewport.zoom(),
            MIN_ZOOM,
            MAX_ZOOM
        );
    }
}

#[test]
fn invariant_camera_finite() {
    let mut viewport = ViewportState::new(800.0, 600.0);

    // Try various invalid values
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        viewport.set_camera(invalid, invalid);
        assert!(viewport.camera_x().is_finite(), "Camera X should be finite");
        assert!(viewport.camera_y().is_finite(), "Camera Y should be finite");
    }
}

#[test]
fn invariant_viewport_dimensions_positive() {
    // Try creating with invalid dimensions
    let viewport = ViewportState::new(0.0, 0.0);
    assert!(viewport.viewport_width() > 0.0);
    assert!(viewport.viewport_height() > 0.0);

    let viewport = ViewportState::new(-100.0, -100.0);
    assert!(viewport.viewport_width() > 0.0);
    assert!(viewport.viewport_height() > 0.0);
}
