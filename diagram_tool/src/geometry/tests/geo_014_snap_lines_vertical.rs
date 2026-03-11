use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-014: Snap Lines Vertical ==============

/// Snap a vertical line X coordinate to nearest target within tolerance
#[must_use]
pub fn snap_vertical(line_x: f64, targets: &[f64], tolerance: f64) -> Option<f64> {
    snap_horizontal(line_x, targets, tolerance)
}

#[test]
fn test_snap_vertical_within_tolerance() {
    // Given: line at x=102 and snap targets
    let line_x = 102.0;
    let targets = vec![0.0, 100.0, 200.0];
    let tolerance = 5.0;

    // When: snapping
    let result = snap_vertical(line_x, &targets, tolerance);

    // Then: snaps to 100
    assert_eq!(result, Some(100.0));
}

#[test]
fn test_snap_vertical_prefers_closest() {
    // Given: line at x=48 (equidistant to 0 and 100 within tolerance)
    let line_x = 48.0;
    let targets = vec![0.0, 100.0];
    let tolerance = 50.0;

    // When: snapping
    let result = snap_vertical(line_x, &targets, tolerance);

    // Then: snaps to closest (50)
    // Actually 48 is closer to 0 (dist 48) than to 100 (dist 52)
    assert_eq!(result, Some(0.0));
}
