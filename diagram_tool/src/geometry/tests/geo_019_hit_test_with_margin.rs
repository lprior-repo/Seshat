use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-019: Hit Test with Margin ==============

/// Check if a point hits a rectangle with optional margin
#[must_use]
pub fn hit_test_rect(point: Point, rect: &Rectangle, margin: f64) -> bool {
    let aabb = rect.aabb();
    point.x >= aabb.min_x - margin
        && point.x <= aabb.max_x + margin
        && point.y >= aabb.min_y - margin
        && point.y <= aabb.max_y + margin
}

#[cfg(kani)]
#[kani::proof]
fn test_hit_test_margin_inside() {
    // Given: point inside rectangle
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
    let point = Point::new(50.0, 50.0);

    // When: hit testing with margin
    let hit = hit_test_rect(point, &rect, 5.0);

    // Then: hit is true
    assert!(hit);
}

#[cfg(kani)]
#[kani::proof]
fn test_hit_test_margin_within_margin() {
    // Given: point just outside rectangle but within margin
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
    let point = Point::new(-3.0, 50.0); // 3 pixels left of rect

    // When: hit testing with margin of 5
    let hit = hit_test_rect(point, &rect, 5.0);

    // Then: hit is true (within margin)
    assert!(hit);
}

#[cfg(kani)]
#[kani::proof]
fn test_hit_test_margin_outside() {
    // Given: point outside margin
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
    let point = Point::new(-10.0, 50.0); // 10 pixels left of rect

    // When: hit testing with margin of 5
    let hit = hit_test_rect(point, &rect, 5.0);

    // Then: hit is false
    assert!(!hit);
}

#[cfg(kani)]
#[kani::proof]
fn test_hit_test_margin_zero() {
    // Given: point on exact edge
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
    let point = Point::new(0.0, 50.0);

    // When: hit testing with zero margin
    let hit = hit_test_rect(point, &rect, 0.0);

    // Then: hit is true (on edge counts as hit)
    assert!(hit);
}
