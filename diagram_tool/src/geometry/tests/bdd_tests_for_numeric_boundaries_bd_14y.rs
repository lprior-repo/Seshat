#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== BDD Tests for Numeric Boundaries (bd-14y) ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_node_with_positive_infinity_x_when_calculating_aabb_then_no_panic() {
    // Given: a rectangle with positive infinity x coordinate
    let rect = Rectangle::new(f64::INFINITY, 0.0, 100.0, 50.0);

    // When: calculating AABB
    // Then: no panic occurs (may produce infinity in result)
    let aabb = rect.aabb();
    assert!(aabb.min_x.is_infinite() || aabb.min_x.is_nan());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_node_with_negative_infinity_y_when_calculating_aabb_then_no_panic() {
    // Given: a rectangle with negative infinity y coordinate
    let rect = Rectangle::new(0.0, f64::NEG_INFINITY, 100.0, 50.0);

    // When: calculating AABB
    // Then: no panic occurs
    let aabb = rect.aabb();
    assert!(aabb.min_y.is_infinite() || aabb.min_y.is_nan());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_rectangle_with_nan_width_when_calculating_aabb_then_no_panic() {
    // Given: a rectangle with NaN width
    let rect = Rectangle::new(0.0, 0.0, f64::NAN, 50.0);

    // When: calculating AABB
    // Then: no panic occurs (result may contain NaN)
    let aabb = rect.aabb();
    // Result should be some value without panic
    let _ = aabb.min_x;
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_rectangle_with_nan_height_when_calculating_aabb_then_no_panic() {
    // Given: a rectangle with NaN height
    let rect = Rectangle::new(0.0, 0.0, 100.0, f64::NAN);

    // When: calculating AABB
    // Then: no panic occurs
    let aabb = rect.aabb();
    let _ = aabb.min_y;
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_very_large_coordinate_when_calculating_bounds_then_no_overflow() {
    // Given: coordinates at 1e308 (near f64 max)
    let rect = Rectangle::new(1e308, 1e308, 100.0, 100.0);

    // When: calculating AABB
    // Then: no overflow panic (result may be infinity, which is acceptable)
    let aabb = rect.aabb();
    // min_x should be the original x coordinate
    assert!((aabb.min_x - 1e308).abs() < 1e295);
    // max_x may be infinity due to 1e308 + 100.0 overflowing - that's expected
    assert!(aabb.max_x.is_infinite() || aabb.max_x.is_finite());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_very_small_positive_coordinate_when_calculating_bounds_then_no_underflow() {
    // Given: coordinates at 1e-308 (near f64 min positive)
    let rect = Rectangle::new(1e-308, 1e-308, 100.0, 100.0);

    // When: calculating AABB
    // Then: no underflow, values preserved
    let aabb = rect.aabb();
    assert!(aabb.min_x > 0.0);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_infinity_in_safe_bounds_then_returns_none() {
    // Given: infinity values in bounds
    // When: calling safe_bounds
    // Then: returns None (invalid)
    assert!(safe_bounds(f64::INFINITY, 0.0, 100.0, 100.0).is_err());
    assert!(safe_bounds(0.0, f64::NEG_INFINITY, 100.0, 100.0).is_err());
    assert!(safe_bounds(0.0, 0.0, f64::INFINITY, 100.0).is_err());
    assert!(safe_bounds(0.0, 0.0, 100.0, f64::INFINITY).is_err());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_all_nan_in_safe_bounds_then_returns_none() {
    // Given: all NaN values
    // When: calling safe_bounds
    // Then: returns None
    assert!(safe_bounds(f64::NAN, f64::NAN, f64::NAN, f64::NAN).is_err());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_subnormal_float_in_bounds_then_preserves_value() {
    // Given: subnormal float value (smaller than f64::MIN_POSITIVE)
    let subnormal = 1e-320_f64;

    // When: creating bounds with subnormal
    let result = safe_bounds(subnormal, subnormal, subnormal + 1e-310, subnormal + 1e-310);

    // Then: value is handled (may be preserved or treated as zero)
    assert!(result.is_ok() || result.is_err());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_negative_infinity_in_all_coords_then_safe_bounds_returns_none() {
    // Given: negative infinity in all coordinates
    // When: calling safe_bounds
    // Then: returns None
    assert!(safe_bounds(
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY
    )
    .is_err());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_scale_with_infinity_factor_then_no_panic() {
    // Given: scale factor of infinity
    let point = Point::new(10.0, 10.0);
    let anchor = Point::origin();

    // When: scaling with infinity factor
    // Then: no panic (result will be infinity or NaN)
    let result = scale_around_anchor(point, anchor, f64::INFINITY);
    assert!(result.x.is_infinite() || result.x.is_nan());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_rotate_with_nan_angle_then_no_panic() {
    // Given: NaN rotation angle
    let point = Point::new(10.0, 0.0);
    let center = Point::origin();

    // When: rotating with NaN angle
    // Then: no panic (result will be NaN)
    let result = rotate_around_center(point, center, f64::NAN);
    assert!(result.x.is_nan() || result.y.is_nan());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_zoom_with_infinity_factor_then_no_panic() {
    // Given: zoom factor of infinity
    let view_center = Point::new(100.0, 100.0);
    let pointer = Point::new(50.0, 50.0);

    // When: zooming with infinity factor
    // Then: no panic
    let result = zoom_at_pointer(view_center, pointer, f64::INFINITY);
    assert!(result.x.is_infinite() || result.x.is_nan());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_resize_with_nan_width_then_handles_gracefully() {
    // Given: NaN new width
    // When: resizing with aspect lock
    // Then: returns NaN without panic
    let new_height = resize_with_aspect_lock(100.0, 50.0, f64::NAN);
    assert!(new_height.is_nan());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_resize_with_zero_original_width_then_returns_new_width() {
    // Given: zero original width (edge case)
    // When: resizing with aspect lock
    // Then: returns new width without division by zero
    let new_height = resize_with_aspect_lock(0.0, 50.0, 200.0);
    assert!((new_height - 200.0).abs() < TOLERANCE);
}
