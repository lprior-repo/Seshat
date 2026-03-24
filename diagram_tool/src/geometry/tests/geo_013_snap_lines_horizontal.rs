#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-013: Snap Lines Horizontal ==============

/// Snap a horizontal line Y coordinate to nearest target within tolerance
#[must_use]
pub fn snap_horizontal(line_y: f64, targets: &[f64], tolerance: f64) -> Option<f64> {
    targets
        .iter()
        .map(|&t| (t, (line_y - t).abs()))
        .filter(|(_, dist)| *dist <= tolerance)
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(t, _)| t)
}

#[cfg(kani)]
#[kani::proof]
fn test_snap_horizontal_within_tolerance() {
    // Given: line at y=52 and snap targets
    let line_y = 52.0;
    let targets = vec![0.0, 50.0, 100.0];
    let tolerance = 5.0;

    // When: snapping
    let result = snap_horizontal(line_y, &targets, tolerance);

    // Then: snaps to 50 (within tolerance of 5)
    assert_eq!(result, Some(50.0));
}

#[cfg(kani)]
#[kani::proof]
fn test_snap_horizontal_outside_tolerance() {
    // Given: line at y=60 (too far from targets)
    let line_y = 60.0;
    let targets = vec![0.0, 50.0, 100.0];
    let tolerance = 5.0;

    // When: snapping
    let result = snap_horizontal(line_y, &targets, tolerance);

    // Then: no snap
    assert!(result.is_none());
}

#[cfg(kani)]
#[kani::proof]
fn test_snap_horizontal_exact_match() {
    // Given: line exactly on target
    let line_y = 50.0;
    let targets = vec![0.0, 50.0, 100.0];
    let tolerance = 5.0;

    // When: snapping
    let result = snap_horizontal(line_y, &targets, tolerance);

    // Then: snaps to exact position
    assert_eq!(result, Some(50.0));
}
