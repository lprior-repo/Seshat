#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-006: Scale Around Anchor Point ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_scale_around_anchor() {
    // Given: a point and anchor
    let point = Point::new(100.0, 100.0);
    let anchor = Point::new(50.0, 50.0);

    // When: scaling by factor 2
    let scaled = scale_around_anchor(point, anchor, 2.0);

    // Then: point moves away from anchor by factor
    // new_x = 50 + (100 - 50) * 2 = 150
    // new_y = 50 + (100 - 50) * 2 = 150
    assert!((scaled.x - 150.0).abs() < TOLERANCE);
    assert!((scaled.y - 150.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_scale_around_anchor_keeps_anchor_fixed() {
    // Given: anchor point as the point to scale
    let anchor = Point::new(50.0, 50.0);

    // When: scaling anchor around itself
    let scaled = scale_around_anchor(anchor, anchor, 2.0);

    // Then: anchor stays fixed
    assert!((scaled.x - anchor.x).abs() < TOLERANCE);
    assert!((scaled.y - anchor.y).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_scale_around_anchor_shrink() {
    // Given: a point and anchor
    let point = Point::new(100.0, 100.0);
    let anchor = Point::new(50.0, 50.0);

    // When: scaling by factor 0.5
    let scaled = scale_around_anchor(point, anchor, 0.5);

    // Then: point moves toward anchor
    // new_x = 50 + (100 - 50) * 0.5 = 75
    // new_y = 50 + (100 - 50) * 0.5 = 75
    assert!((scaled.x - 75.0).abs() < TOLERANCE);
    assert!((scaled.y - 75.0).abs() < TOLERANCE);
}
