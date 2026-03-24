#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== Property-Based Tests ==============

proptest! {
    #[cfg(kani)]
#[kani::proof]
    fn prop_scale_around_anchor_idempotent_at_anchor(factor in -10.0_f64..10.0) {
        let anchor = Point::new(50.0, 50.0);
        let scaled = scale_around_anchor(anchor, anchor, factor);
        prop_assert!((scaled.x - anchor.x).abs() < TOLERANCE);
        prop_assert!((scaled.y - anchor.y).abs() < TOLERANCE);
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_rotate_around_center_idempotent_at_center(angle in -4.0_f64 * PI..4.0 * PI) {
        let center = Point::new(50.0, 50.0);
        let rotated = rotate_around_center(center, center, angle);
        prop_assert!((rotated.x - center.x).abs() < TOLERANCE);
        prop_assert!((rotated.y - center.y).abs() < TOLERANCE);
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_rotate_full_circle_returns_to_origin(angle in -4.0_f64 * PI..4.0 * PI) {
        let point = Point::new(100.0, 0.0);
        let center = Point::origin();
        let rotated_once = rotate_around_center(point, center, angle);
        let rotated_twice = rotate_around_center(rotated_once, center, 2.0 * PI - angle);
        prop_assert!((rotated_twice.x - point.x).abs() < 1e-9);
        prop_assert!((rotated_twice.y - point.y).abs() < 1e-9);
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_aabb_contains_all_corners(
        x in -1000.0_f64..1000.0,
        y in -1000.0_f64..1000.0,
        width in 1.0_f64..500.0,
        height in 1.0_f64..500.0,
        rotation in 0.0_f64..2.0 * PI
    ) {
        let rect = Rectangle::new(x, y, width, height).with_rotation(rotation);
        let aabb = rect.aabb();

        // All corners should be within or on the AABB
        let corners = rect.corners();
        for corner in corners {
            prop_assert!(corner.x >= aabb.min_x - TOLERANCE);
            prop_assert!(corner.x <= aabb.max_x + TOLERANCE);
            prop_assert!(corner.y >= aabb.min_y - TOLERANCE);
            prop_assert!(corner.y <= aabb.max_y + TOLERANCE);
        }
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_aspect_ratio_preserved(
        width in 1.0_f64..1000.0,
        height in 1.0_f64..1000.0,
        new_width in 1.0_f64..1000.0
    ) {
        let original_ratio = height / width;
        let new_height = resize_with_aspect_lock(width, height, new_width);
        let new_ratio = new_height / new_width;
        prop_assert!((original_ratio - new_ratio).abs() < TOLERANCE);
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_safe_bounds_finite_inputs_produce_valid_aabb(
        min_x in -1e6_f64..1e6,
        min_y in -1e6_f64..1e6,
        max_x in -1e6_f64..1e6,
        max_y in -1e6_f64..1e6
    ) {
        let result = safe_bounds(min_x, min_y, max_x, max_y);
        prop_assert!(result.is_ok());

        let aabb = result.unwrap();
        prop_assert!(aabb.min_x.is_finite());
        prop_assert!(aabb.min_y.is_finite());
        prop_assert!(aabb.max_x.is_finite());
        prop_assert!(aabb.max_y.is_finite());
        prop_assert!(aabb.min_x <= aabb.max_x);
        prop_assert!(aabb.min_y <= aabb.max_y);
    }
}
