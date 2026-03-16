use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-002: AABB for Rotated Rectangles ==============

#[test]
fn test_aabb_rotated_rectangle_45_degrees() {
    // Given: a square rotated 45 degrees
    let size = 100.0;
    let rect = Rectangle::new(0.0, 0.0, size, size).with_rotation(PI / 4.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB should be expanded by sqrt(2)/2 factor (diagonal)
    // For a square centered at (50, 50), rotated 45 degrees:
    // The corners extend from center by (size/2) * sqrt(2)
    let expected_half_extent = (size / 2.0) * SQRT_2;
    let center = 50.0;

    assert!((aabb.min_x - (center - expected_half_extent)).abs() < TOLERANCE);
    assert!((aabb.max_x - (center + expected_half_extent)).abs() < TOLERANCE);
    assert!((aabb.min_y - (center - expected_half_extent)).abs() < TOLERANCE);
    assert!((aabb.max_y - (center + expected_half_extent)).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_aabb_rotated_rectangle_45_degrees_kani() {
    // Given: a square rotated 45 degrees
    let size = 100.0;
    let rect = Rectangle::new(0.0, 0.0, size, size).with_rotation(PI / 4.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB should be expanded by sqrt(2)/2 factor (diagonal)
    // For a square centered at (50, 50), rotated 45 degrees:
    // The corners extend from center by (size/2) * sqrt(2)
    let expected_half_extent = (size / 2.0) * SQRT_2;
    let center = 50.0;

    assert!((aabb.min_x - (center - expected_half_extent)).abs() < TOLERANCE);
    assert!((aabb.max_x - (center + expected_half_extent)).abs() < TOLERANCE);
    assert!((aabb.min_y - (center - expected_half_extent)).abs() < TOLERANCE);
    assert!((aabb.max_y - (center + expected_half_extent)).abs() < TOLERANCE);
}

#[test]
fn test_aabb_rotated_rectangle_90_degrees() {
    // Given: a rectangle rotated 90 degrees
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI / 2.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB dimensions are swapped (centered)
    // Original center: (50, 25), after 90 degree rotation
    // width becomes height and vice versa
    let center_x = 50.0;
    let center_y = 25.0;
    let expected_half_w = 25.0; // original height/2
    let expected_half_h = 50.0; // original width/2

    assert!((aabb.min_x - (center_x - expected_half_w)).abs() < TOLERANCE);
    assert!((aabb.max_x - (center_x + expected_half_w)).abs() < TOLERANCE);
    assert!((aabb.min_y - (center_y - expected_half_h)).abs() < TOLERANCE);
    assert!((aabb.max_y - (center_y + expected_half_h)).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_aabb_rotated_rectangle_90_degrees_kani() {
    // Given: a rectangle rotated 90 degrees
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI / 2.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB dimensions are swapped (centered)
    // Original center: (50, 25), after 90 degree rotation
    // width becomes height and vice versa
    let center_x = 50.0;
    let center_y = 25.0;
    let expected_half_w = 25.0; // original height/2
    let expected_half_h = 50.0; // original width/2

    assert!((aabb.min_x - (center_x - expected_half_w)).abs() < TOLERANCE);
    assert!((aabb.max_x - (center_x + expected_half_w)).abs() < TOLERANCE);
    assert!((aabb.min_y - (center_y - expected_half_h)).abs() < TOLERANCE);
    assert!((aabb.max_y - (center_y + expected_half_h)).abs() < TOLERANCE);
}

#[test]
fn test_aabb_rotated_rectangle_180_degrees() {
    // Given: a rectangle rotated 180 degrees
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB is same as unrotated (180 degree rotation doesn't change AABB)
    assert!((aabb.min_x - 0.0).abs() < TOLERANCE);
    assert!((aabb.min_y - 0.0).abs() < TOLERANCE);
    assert!((aabb.max_x - 100.0).abs() < TOLERANCE);
    assert!((aabb.max_y - 50.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_aabb_rotated_rectangle_180_degrees_kani() {
    // Given: a rectangle rotated 180 degrees
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB is same as unrotated (180 degree rotation doesn't change AABB)
    assert!((aabb.min_x - 0.0).abs() < TOLERANCE);
    assert!((aabb.min_y - 0.0).abs() < TOLERANCE);
    assert!((aabb.max_x - 100.0).abs() < TOLERANCE);
    assert!((aabb.max_y - 50.0).abs() < TOLERANCE);
}
