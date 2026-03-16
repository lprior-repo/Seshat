use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-003: Stroke Width Inclusion in Bounds ==============

#[test]
fn test_stroke_width_inclusion() {
    // Given: a rectangle with stroke
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
    let stroked = StrokedShape::new(rect, 4.0);

    // When: calculating bounds with stroke
    let bounds = stroked.bounds_with_stroke();

    // Then: bounds are expanded by stroke_width/2 on each side
    assert!((bounds.min_x - (-2.0)).abs() < TOLERANCE);
    assert!((bounds.min_y - (-2.0)).abs() < TOLERANCE);
    assert!((bounds.max_x - 102.0).abs() < TOLERANCE);
    assert!((bounds.max_y - 52.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_stroke_width_inclusion_kani() {
    // Given: a rectangle with stroke
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
    let stroked = StrokedShape::new(rect, 4.0);

    // When: calculating bounds with stroke
    let bounds = stroked.bounds_with_stroke();

    // Then: bounds are expanded by stroke_width/2 on each side
    assert!((bounds.min_x - (-2.0)).abs() < TOLERANCE);
    assert!((bounds.min_y - (-2.0)).abs() < TOLERANCE);
    assert!((bounds.max_x - 102.0).abs() < TOLERANCE);
    assert!((bounds.max_y - 52.0).abs() < TOLERANCE);
}

#[test]
fn test_stroke_width_zero() {
    // Given: a rectangle with zero stroke
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
    let stroked = StrokedShape::new(rect, 0.0);

    // When: calculating bounds
    let bounds = stroked.bounds_with_stroke();

    // Then: bounds equal the shape bounds
    let expected = rect.aabb();
    assert!((bounds.min_x - expected.min_x).abs() < TOLERANCE);
    assert!((bounds.min_y - expected.min_y).abs() < TOLERANCE);
    assert!((bounds.max_x - expected.max_x).abs() < TOLERANCE);
    assert!((bounds.max_y - expected.max_y).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_stroke_width_zero_kani() {
    // Given: a rectangle with zero stroke
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
    let stroked = StrokedShape::new(rect, 0.0);

    // When: calculating bounds
    let bounds = stroked.bounds_with_stroke();

    // Then: bounds equal the shape bounds
    let expected = rect.aabb();
    assert!((bounds.min_x - expected.min_x).abs() < TOLERANCE);
    assert!((bounds.min_y - expected.min_y).abs() < TOLERANCE);
    assert!((bounds.max_x - expected.max_x).abs() < TOLERANCE);
    assert!((bounds.max_y - expected.max_y).abs() < TOLERANCE);
}
