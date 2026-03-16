#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-007: Rotate Around Center ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_rotate_around_center_90_degrees() {
    // Given: a point at (100, 0) and center at origin
    let point = Point::new(100.0, 0.0);
    let center = Point::origin();

    // When: rotating 90 degrees counter-clockwise
    let rotated = rotate_around_center(point, center, PI / 2.0);

    // Then: point is at (0, 100)
    assert!((rotated.x - 0.0).abs() < TOLERANCE);
    assert!((rotated.y - 100.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_rotate_around_center_180_degrees() {
    // Given: a point at (100, 0) and center at origin
    let point = Point::new(100.0, 0.0);
    let center = Point::origin();

    // When: rotating 180 degrees
    let rotated = rotate_around_center(point, center, PI);

    // Then: point is at (-100, 0)
    assert!((rotated.x - (-100.0)).abs() < TOLERANCE);
    assert!((rotated.y - 0.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_rotate_around_center_keeps_center_fixed() {
    // Given: center as the point to rotate
    let center = Point::new(50.0, 50.0);

    // When: rotating center around itself
    let rotated = rotate_around_center(center, center, PI / 4.0);

    // Then: center stays fixed
    assert!((rotated.x - center.x).abs() < TOLERANCE);
    assert!((rotated.y - center.y).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_rotate_around_center_45_degrees() {
    // Given: a point at (1, 0) and center at origin
    let point = Point::new(1.0, 0.0);
    let center = Point::origin();

    // When: rotating 45 degrees
    let rotated = rotate_around_center(point, center, PI / 4.0);

    // Then: point is at (sqrt(2)/2, sqrt(2)/2)
    assert!((rotated.x - FRAC_1_SQRT_2).abs() < TOLERANCE);
    assert!((rotated.y - FRAC_1_SQRT_2).abs() < TOLERANCE);
}
