use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-001: AABB for Axis-Aligned Rectangles ==============

#[test]
fn test_aabb_axis_aligned() {
    // Given: a rectangle at origin
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB equals the rectangle itself
    assert!((aabb.min_x - 0.0).abs() < TOLERANCE);
    assert!((aabb.min_y - 0.0).abs() < TOLERANCE);
    assert!((aabb.max_x - 100.0).abs() < TOLERANCE);
    assert!((aabb.max_y - 50.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_aabb_axis_aligned_kani() {
    // Given: a rectangle at origin
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB equals the rectangle itself
    assert!((aabb.min_x - 0.0).abs() < TOLERANCE);
    assert!((aabb.min_y - 0.0).abs() < TOLERANCE);
    assert!((aabb.max_x - 100.0).abs() < TOLERANCE);
    assert!((aabb.max_y - 50.0).abs() < TOLERANCE);
}

#[test]
fn test_aabb_axis_aligned_with_offset() {
    // Given: a rectangle at non-origin position
    let rect = Rectangle::new(50.0, 25.0, 100.0, 50.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB correctly reflects position
    assert!((aabb.min_x - 50.0).abs() < TOLERANCE);
    assert!((aabb.min_y - 25.0).abs() < TOLERANCE);
    assert!((aabb.max_x - 150.0).abs() < TOLERANCE);
    assert!((aabb.max_y - 75.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_aabb_axis_aligned_with_offset_kani() {
    // Given: a rectangle at non-origin position
    let rect = Rectangle::new(50.0, 25.0, 100.0, 50.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB correctly reflects position
    assert!((aabb.min_x - 50.0).abs() < TOLERANCE);
    assert!((aabb.min_y - 25.0).abs() < TOLERANCE);
    assert!((aabb.max_x - 150.0).abs() < TOLERANCE);
    assert!((aabb.max_y - 75.0).abs() < TOLERANCE);
}
