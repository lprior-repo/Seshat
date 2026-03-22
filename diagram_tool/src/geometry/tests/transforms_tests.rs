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

proptest! {
    #[test]
    fn test_scale_around_anchor_no_panic(
        point in arb_point(),
        anchor in arb_point(),
        factor in arb_f64()
    ) {
        let _ = scale_around_anchor(point, anchor, factor);
    }

    #[test]
    fn test_rotate_around_center_no_panic(
        point in arb_point(),
        center in arb_point(),
        angle in arb_f64()
    ) {
        let _ = rotate_around_center(point, center, angle);
    }

    #[test]
    fn test_resize_with_aspect_lock_no_panic(
        orig_w in arb_f64(),
        orig_h in arb_f64(),
        new_w in arb_f64()
    ) {
        let _ = resize_with_aspect_lock(orig_w, orig_h, new_w);
    }

    #[test]
    fn test_scale_then_rotate_no_panic(
        point in arb_point(),
        anchor in arb_point(),
        factor in arb_f64(),
        angle in arb_f64()
    ) {
        let _ = scale_then_rotate(point, anchor, factor, angle);
    }

    #[test]
    fn test_fit_to_viewport_no_panic(
        content in arb_aabb(),
        vw in arb_f64(),
        vh in arb_f64(),
        padding in arb_f64()
    ) {
        let _ = fit_to_viewport(&content, vw, vh, padding);
    }

    #[test]
    fn test_clamp_to_min_size_no_panic(
        w in arb_f64(),
        h in arb_f64(),
        min in arb_f64()
    ) {
        let _ = clamp_to_min_size(w, h, min);
    }

    #[test]
    fn test_scale_with_flip_no_panic(
        w in arb_f64(),
        h in arb_f64(),
        sx in arb_f64(),
        sy in arb_f64()
    ) {
        let _ = scale_with_flip(w, h, sx, sy);
    }

    #[test]
    fn test_scale_with_clamp_no_panic(
        w in arb_f64(),
        h in arb_f64(),
        sx in arb_f64(),
        sy in arb_f64(),
        min in arb_f64()
    ) {
        let _ = scale_with_clamp(w, h, sx, sy, min);
    }

    #[test]
    fn test_get_corner_point_no_panic(
        rect in arb_rect(),
        corner in arb_corner()
    ) {
        let _ = get_corner_point(&rect, corner);
    }

    #[test]
    fn test_scale_rect_around_corner_no_panic(
        rect in arb_rect(),
        corner in arb_corner(),
        factor in arb_f64()
    ) {
        let _ = scale_rect_around_corner(&rect, corner, factor);
    }
}

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
