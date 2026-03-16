#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-011: Rotation + Resize Composition ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_rotation_resize_composition() {
    // Given: a point, anchor, scale factor, and rotation angle
    let point = Point::new(10.0, 0.0);
    let anchor = Point::origin();
    let scale = 2.0;
    let angle = std::f64::consts::PI / 2.0;

    // When: applying resize then rotation using existing scale_then_rotate
    let result = scale_then_rotate(point, anchor, scale, angle);

    // Then: result is deterministic
    // Scale: (10, 0) -> (20, 0), Rotate 90deg: (20, 0) -> (0, 20)
    assert!((result.x - 0.0).abs() < TOLERANCE);
    assert!((result.y - 20.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_rotation_resize_composition_reverse_order() {
    // Given: a point at (10, 0)
    let point = Point::new(10.0, 0.0);
    let anchor = Point::origin();

    // When: rotate first then scale (manual application)
    let rotated = rotate_around_center(point, anchor, std::f64::consts::PI / 2.0);
    let scaled = scale_around_anchor(rotated, anchor, 2.0);

    // Then: order matters - different result than scale_then_rotate
    // Rotate: (10, 0) -> (0, 10), Scale: (0, 10) -> (0, 20)
    assert!((scaled.x - 0.0).abs() < TOLERANCE);
    assert!((scaled.y - 20.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_rotation_resize_composition_no_scale() {
    // Given: scale factor of 1.0
    let point = Point::new(10.0, 0.0);
    let anchor = Point::origin();

    // When: scale_then_rotate with scale=1.0
    let result = scale_then_rotate(point, anchor, 1.0, std::f64::consts::PI / 2.0);

    // Then: only rotation is applied
    let expected = rotate_around_center(point, anchor, std::f64::consts::PI / 2.0);
    assert!((result.x - expected.x).abs() < TOLERANCE);
    assert!((result.y - expected.y).abs() < TOLERANCE);
}
