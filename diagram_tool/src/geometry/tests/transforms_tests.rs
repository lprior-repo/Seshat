#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]
#![allow(clippy::float_cmp)]

use crate::geometry::primitives::{Point, Rectangle, AABB};
use crate::geometry::transforms::{
    clamp_to_min_size, fit_to_viewport, get_corner_point, resize_with_aspect_lock,
    rotate_around_center, scale_around_anchor, scale_rect_around_corner, scale_then_rotate,
    scale_with_clamp, scale_with_flip, Corner,
};
use proptest::prelude::*;

// -----------------------------------------------------------------------------
// Proptest Generators
// -----------------------------------------------------------------------------

prop_compose! {
    fn arb_f64()(f in any::<f64>()) -> f64 {
        f
    }
}

prop_compose! {
    fn arb_finite_f64()(f in any::<f64>()) -> f64 {
        if f.is_finite() { f } else { 0.0 }
    }
}

prop_compose! {
    fn arb_point()(x in arb_f64(), y in arb_f64()) -> Point {
        Point::new(x, y)
    }
}

prop_compose! {
    fn arb_finite_point()(x in arb_finite_f64(), y in arb_finite_f64()) -> Point {
        Point::new(x, y)
    }
}

prop_compose! {
    fn arb_rect()(x in arb_f64(), y in arb_f64(), w in arb_f64(), h in arb_f64()) -> Rectangle {
        Rectangle::new(x, y, w, h)
    }
}

prop_compose! {
    fn arb_aabb()(min_x in arb_f64(), min_y in arb_f64(), max_x in arb_f64(), max_y in arb_f64()) -> AABB {
        AABB::new(min_x, min_y, max_x, max_y)
    }
}

fn arb_corner() -> impl Strategy<Value = Corner> {
    prop_oneof![
        Just(Corner::NorthWest),
        Just(Corner::NorthEast),
        Just(Corner::SouthEast),
        Just(Corner::SouthWest),
    ]
}

// -----------------------------------------------------------------------------
// Proptest Suites
// -----------------------------------------------------------------------------



// -----------------------------------------------------------------------------
// Explicit boundary tests
// -----------------------------------------------------------------------------

#[test]
fn test_resize_with_aspect_lock_zero_original_width() {
    let result = resize_with_aspect_lock(0.0, 10.0, 5.0);
    assert_eq!(result, 5.0); // Should return new_width directly as per implementation
}

#[test]
fn test_fit_to_viewport_zero_size_content() {
    let empty_aabb = AABB::new(0.0, 0.0, 0.0, 0.0);
    let result = fit_to_viewport(&empty_aabb, 100.0, 100.0, 10.0);
    assert_eq!(result.scale, 1.0);
    assert_eq!(result.offset_x, 0.0);
    assert_eq!(result.offset_y, 0.0);
}

#[test]
fn test_scale_with_clamp_negative_scale() {
    let (w, h) = scale_with_clamp(10.0, 10.0, -2.0, -3.0, 5.0);
    assert_eq!(w, 5.0);
    assert_eq!(h, 5.0);
}

#[test]
fn test_subnormal_rotation_no_panic() {
    let pt = Point::new(1e-310, 1e-310);
    let center = Point::new(0.0, 0.0);
    let result = rotate_around_center(pt, center, std::f64::consts::PI);
    assert!(result.x.is_finite());
    assert!(result.y.is_finite());
}
#[test]
fn test_scale_around_anchor_concrete() {
    let point = Point::new(10.0, 10.0);
    let anchor = Point::new(0.0, 0.0);
    assert_eq!(scale_around_anchor(point, anchor, 2.0), Point::new(20.0, 20.0));
}
#[test] fn test_rotate_around_center_concrete() { assert!(rotate_around_center(Point::new(10.0, 0.0), Point::new(0.0, 0.0), std::f64::consts::PI/2.0).y > 9.0); }
#[test] fn test_resize_with_aspect_lock_concrete() { assert_eq!(resize_with_aspect_lock(100.0, 50.0, 200.0), 100.0); }
#[test] fn test_scale_then_rotate_concrete() { assert!(scale_then_rotate(Point::new(10.0, 0.0), Point::new(0.0, 0.0), 2.0, std::f64::consts::PI/2.0).y > 19.0); }
#[test] fn test_fit_to_viewport_concrete() { assert_eq!(fit_to_viewport(&AABB::new(0.0, 0.0, 100.0, 100.0), 200.0, 200.0, 0.0).scale, 2.0); }
#[test] fn test_clamp_to_min_size_concrete() { assert_eq!(clamp_to_min_size(5.0, 10.0, 20.0), (20.0, 20.0)); }
#[test] fn test_scale_with_flip_concrete() { assert_eq!(scale_with_flip(10.0, 20.0, -1.0, 2.0), (10.0, 40.0)); }
#[test] fn test_scale_with_clamp_concrete() { assert_eq!(scale_with_clamp(10.0, 10.0, 0.5, 0.5, 20.0), (20.0, 20.0)); }
#[test] fn test_get_corner_point_concrete() { assert_eq!(get_corner_point(&Rectangle::new(10.0, 20.0, 100.0, 50.0), Corner::SouthEast), Point::new(110.0, 70.0)); }
#[test] fn test_scale_rect_around_corner_concrete() { assert_eq!(scale_rect_around_corner(&Rectangle::new(10.0, 10.0, 100.0, 100.0), Corner::NorthWest, 2.0).width, 200.0); }
