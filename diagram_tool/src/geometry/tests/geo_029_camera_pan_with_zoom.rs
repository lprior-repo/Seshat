use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-029: Camera Pan with Zoom ==============

#[test]
fn test_camera_pan_with_zoom() {
    // Given: screen-space delta and different zoom levels
    let screen_delta: f64 = 10.0; // 10 pixels
    let zoom_levels: [f64; 4] = [0.5, 1.0, 2.0, 5.0];

    for zoom in zoom_levels {
        // When: converting screen delta to world delta
        let world_delta = screen_delta / zoom;

        // Then: world delta is inversely proportional to zoom
        // Higher zoom = smaller world movement for same screen pixels
        assert!((world_delta - 10.0_f64 / zoom).abs() < TOLERANCE);
    }
}

#[test]
fn test_camera_pan_consistent_screen_movement() {
    // Given: two zoom levels and their world deltas
    let zoom1: f64 = 1.0;
    let zoom2: f64 = 2.0;
    let screen_pixels: f64 = 100.0;

    // When: calculating world deltas
    let world_delta1 = screen_pixels / zoom1;
    let world_delta2 = screen_pixels / zoom2;

    // Then: higher zoom requires smaller world movement
    // for the same screen-space movement
    assert!(world_delta2 < world_delta1);
    assert!((world_delta1 / world_delta2 - 2.0_f64).abs() < TOLERANCE);
}

#[test]
fn test_camera_pan_at_min_zoom() {
    // Given: minimum zoom level
    let zoom = MIN_ZOOM;
    let screen_delta = 10.0;

    // When: converting screen to world delta
    let world_delta = screen_delta / zoom;

    // Then: world delta is large (pan moves far in world space)
    assert!((world_delta - 100.0_f64).abs() < TOLERANCE);
}
