use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-027: Camera Constraints - Min Zoom ==============

const MIN_ZOOM: f64 = 0.1;
const MAX_ZOOM: f64 = 10.0;

fn clamp_zoom(zoom: f64) -> f64 {
    zoom.clamp(MIN_ZOOM, MAX_ZOOM)
}

#[test]
fn test_camera_constraints_min_zoom() {
    // Given: zoom values below minimum
    let below_min = [0.01, 0.05, 0.099, 0.0];

    for zoom in below_min {
        // When: clamping zoom
        let clamped = clamp_zoom(zoom);

        // Then: zoom is clamped to minimum
        assert!((clamped - MIN_ZOOM).abs() < TOLERANCE);
    }
}

#[test]
fn test_camera_constraints_min_zoom_exact() {
    // Given: zoom at exact minimum
    let zoom = MIN_ZOOM;

    // When: clamping zoom
    let clamped = clamp_zoom(zoom);

    // Then: zoom remains unchanged
    assert!((clamped - MIN_ZOOM).abs() < TOLERANCE);
}
