#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-031: AABB for Rotated Rectangle at Cardinal Angles ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_aabb_rotated_0_degrees() {
    // Given: a rectangle rotated 0 degrees (no rotation)
    let rect = Rectangle::new(10.0, 20.0, 100.0, 50.0).with_rotation(0.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB equals the rectangle bounds
    assert!((aabb.min_x - 10.0).abs() < TOLERANCE);
    assert!((aabb.min_y - 20.0).abs() < TOLERANCE);
    assert!((aabb.max_x - 110.0).abs() < TOLERANCE);
    assert!((aabb.max_y - 70.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_aabb_rotated_90_degrees_cardinal() {
    // Given: a rectangle rotated 90 degrees (PI/2)
    // Rectangle at (10, 20) with size 100x50, center at (60, 45)
    let rect = Rectangle::new(10.0, 20.0, 100.0, 50.0).with_rotation(PI / 2.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: width and height are swapped (centered at same point)
    // Original center: (60, 45), half-width: 50, half-height: 25
    // After 90 degree rotation: half-width becomes 25, half-height becomes 50
    let center_x = 60.0;
    let center_y = 45.0;
    assert!((aabb.min_x - (center_x - 25.0)).abs() < TOLERANCE);
    assert!((aabb.max_x - (center_x + 25.0)).abs() < TOLERANCE);
    assert!((aabb.min_y - (center_y - 50.0)).abs() < TOLERANCE);
    assert!((aabb.max_y - (center_y + 50.0)).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_aabb_rotated_180_degrees_cardinal() {
    // Given: a rectangle rotated 180 degrees (PI)
    let rect = Rectangle::new(10.0, 20.0, 100.0, 50.0).with_rotation(PI);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB is same as unrotated (180 degree rotation doesn't change AABB)
    assert!((aabb.min_x - 10.0).abs() < TOLERANCE);
    assert!((aabb.min_y - 20.0).abs() < TOLERANCE);
    assert!((aabb.max_x - 110.0).abs() < TOLERANCE);
    assert!((aabb.max_y - 70.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_aabb_rotated_270_degrees_cardinal() {
    // Given: a rectangle rotated 270 degrees (3*PI/2)
    let rect = Rectangle::new(10.0, 20.0, 100.0, 50.0).with_rotation(3.0 * PI / 2.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: same as 90 degree rotation (just opposite direction)
    let center_x = 60.0;
    let center_y = 45.0;
    assert!((aabb.min_x - (center_x - 25.0)).abs() < TOLERANCE);
    assert!((aabb.max_x - (center_x + 25.0)).abs() < TOLERANCE);
    assert!((aabb.min_y - (center_y - 50.0)).abs() < TOLERANCE);
    assert!((aabb.max_y - (center_y + 50.0)).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_aabb_rotated_360_degrees_cardinal() {
    // Given: a rectangle rotated 360 degrees (2*PI)
    let rect = Rectangle::new(10.0, 20.0, 100.0, 50.0).with_rotation(2.0 * PI);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB is same as unrotated
    assert!((aabb.min_x - 10.0).abs() < TOLERANCE);
    assert!((aabb.min_y - 20.0).abs() < TOLERANCE);
    assert!((aabb.max_x - 110.0).abs() < TOLERANCE);
    assert!((aabb.max_y - 70.0).abs() < TOLERANCE);
}
