use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-032: AABB Includes Stroke Width (Extended) ==============

#[test]
fn test_aabb_stroke_width_thick_stroke() {
    // Given: a rectangle with thick stroke
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
    let stroked = StrokedShape::new(rect, 20.0);

    // When: calculating bounds with stroke
    let bounds = stroked.bounds_with_stroke();

    // Then: bounds are expanded by stroke_width/2 = 10 on each side
    assert!((bounds.min_x - (-10.0)).abs() < TOLERANCE);
    assert!((bounds.min_y - (-10.0)).abs() < TOLERANCE);
    assert!((bounds.max_x - 110.0).abs() < TOLERANCE);
    assert!((bounds.max_y - 60.0).abs() < TOLERANCE);
}

#[test]
fn test_aabb_stroke_width_rotated_shape() {
    // Given: a rotated rectangle with stroke
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0).with_rotation(PI / 4.0);
    let stroked = StrokedShape::new(rect, 10.0);

    // When: calculating bounds with stroke
    let bounds = stroked.bounds_with_stroke();

    // Then: stroke expansion applies to the rotated AABB
    let rect_aabb = rect.aabb();
    let expected = rect_aabb.expand(5.0); // stroke_width / 2
    assert!((bounds.min_x - expected.min_x).abs() < TOLERANCE);
    assert!((bounds.min_y - expected.min_y).abs() < TOLERANCE);
    assert!((bounds.max_x - expected.max_x).abs() < TOLERANCE);
    assert!((bounds.max_y - expected.max_y).abs() < TOLERANCE);
}

#[test]
fn test_aabb_stroke_width_fractional() {
    // Given: a rectangle with fractional stroke width
    let rect = Rectangle::new(50.0, 50.0, 100.0, 50.0);
    let stroked = StrokedShape::new(rect, 3.5);

    // When: calculating bounds with stroke
    let bounds = stroked.bounds_with_stroke();

    // Then: bounds are expanded by 1.75 on each side
    assert!((bounds.min_x - 48.25).abs() < TOLERANCE);
    assert!((bounds.min_y - 48.25).abs() < TOLERANCE);
    assert!((bounds.max_x - 151.75).abs() < TOLERANCE);
    assert!((bounds.max_y - 101.75).abs() < TOLERANCE);
}
