#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-023: Rotation Then Resize Composition ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_rotation_then_resize_composition() {
    // Given: a point at (100, 0) relative to origin
    let point = Point::new(100.0, 0.0);
    let center = Point::origin();
    let angle = PI / 2.0; // 90 degrees
    let scale_factor = 0.5;

    // When: rotate then resize
    let rotated = rotate_around_center(point, center, angle);
    let final_point = scale_around_anchor(rotated, center, scale_factor);

    // Then: first rotate (100, 0) -> (0, 100), then scale -> (0, 50)
    assert!((final_point.x - 0.0).abs() < TOLERANCE);
    assert!((final_point.y - 50.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_rotation_then_resize_45_degrees() {
    // Given: a point at (1, 0)
    let point = Point::new(1.0, 0.0);
    let center = Point::origin();
    let angle = PI / 4.0;
    let scale_factor = 2.0;

    // When: rotate 45 degrees then scale by 2
    let rotated = rotate_around_center(point, center, angle);
    let final_point = scale_around_anchor(rotated, center, scale_factor);

    // Then: result is 2 * (sqrt(2)/2, sqrt(2)/2) = (sqrt(2), sqrt(2))
    assert!((final_point.x - SQRT_2).abs() < TOLERANCE);
    assert!((final_point.y - SQRT_2).abs() < TOLERANCE);
}
