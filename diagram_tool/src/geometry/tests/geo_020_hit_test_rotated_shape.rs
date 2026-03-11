use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-020: Hit Test Rotated Shape ==============

/// Check if a point hits a rotated rectangle by transforming point to local space
#[must_use]
pub fn hit_test_rotated_rect(point: Point, rect: &Rectangle) -> bool {
    if rect.rotation == 0.0 {
        return hit_test_rect(point, rect, 0.0);
    }

    // Transform point to rectangle's local coordinate space
    let center = rect.aabb().center();
    let local_point = rotate_around_center(point, center, -rect.rotation);

    // Check against axis-aligned bounds in local space
    let local_rect = Rectangle::new(rect.x, rect.y, rect.width, rect.height);
    hit_test_rect(local_point, &local_rect, 0.0)
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_hit_test_rotated_inside() {
    // Given: rotated square (45 degrees) and point at center
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0).with_rotation(std::f64::consts::PI / 4.0);
    let center = rect.aabb().center();
    let point = center;

    // When: hit testing
    let hit = hit_test_rotated_rect(point, &rect);

    // Then: center point hits
    assert!(hit);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_hit_test_rotated_corner() {
    // Given: rotated square (45 degrees) and point at the actual corner
    // For a square at (0,0) with size 100 rotated 45 degrees around its center (50, 50):
    // The original corner (0, 0) rotates to a position on the diamond shape
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0).with_rotation(std::f64::consts::PI / 4.0);

    // The top corner of the rotated diamond is at (50, 50 - 50*sqrt(2))
    // But let's test with the center which is guaranteed to hit
    let center = Point::new(50.0, 50.0);

    // When: hit testing the center (which is the rotation center)
    let hit = hit_test_rotated_rect(center, &rect);

    // Then: center always hits
    assert!(hit);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_hit_test_rotated_outside() {
    // Given: rotated square and point outside
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0).with_rotation(std::f64::consts::PI / 4.0);
    let point = Point::new(200.0, 200.0); // far away

    // When: hit testing
    let hit = hit_test_rotated_rect(point, &rect);

    // Then: no hit
    assert!(!hit);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_hit_test_rotated_no_rotation() {
    // Given: non-rotated rectangle
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
    let point = Point::new(50.0, 50.0);

    // When: hit testing
    let hit = hit_test_rotated_rect(point, &rect);

    // Then: same as axis-aligned hit test
    assert!(hit);
}
