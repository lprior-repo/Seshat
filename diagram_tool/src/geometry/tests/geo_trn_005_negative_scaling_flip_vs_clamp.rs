#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-TRN-005: Negative Scaling Flip vs Clamp ==============
//
// Tests behavior when scale factors become negative.
// Two strategies: flip (mirror) or clamp to zero/minimum.

/// Scale result with flip behavior - negative scale mirrors the geometry
fn scale_with_flip(width: f64, height: f64, scale_x: f64, scale_y: f64) -> (f64, f64) {
    // Negative scaling causes a flip - the dimension becomes positive but mirrored
    let new_width = (width * scale_x).abs();
    let new_height = (height * scale_y).abs();
    (new_width, new_height)
}

/// Scale result with clamp behavior - negative scale is clamped to minimum
fn scale_with_clamp(
    width: f64,
    height: f64,
    scale_x: f64,
    scale_y: f64,
    min_size: f64,
) -> (f64, f64) {
    let new_width = if scale_x < 0.0 {
        min_size
    } else {
        (width * scale_x).max(min_size)
    };
    let new_height = if scale_y < 0.0 {
        min_size
    } else {
        (height * scale_y).max(min_size)
    };
    (new_width, new_height)
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_negative_scaling_flip_x() {
    // Given: a rectangle with positive dimensions
    let width = 100.0;
    let height = 50.0;
    let scale_x = -1.0; // Flip horizontally
    let scale_y = 1.0;

    // When: using flip behavior
    let (new_width, new_height) = scale_with_flip(width, height, scale_x, scale_y);

    // Then: width is preserved (mirrored), height unchanged
    assert!((new_width - 100.0).abs() < TOLERANCE);
    assert!((new_height - 50.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_negative_scaling_flip_y() {
    // Given: a rectangle with positive dimensions
    let width = 100.0;
    let height = 50.0;
    let scale_x = 1.0;
    let scale_y = -2.0; // Flip and scale vertically

    // When: using flip behavior
    let (new_width, new_height) = scale_with_flip(width, height, scale_x, scale_y);

    // Then: width unchanged, height doubled (mirrored)
    assert!((new_width - 100.0).abs() < TOLERANCE);
    assert!((new_height - 100.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_negative_scaling_flip_both() {
    // Given: a rectangle
    let width = 100.0;
    let height = 50.0;
    let scale_x = -0.5;
    let scale_y = -2.0;

    // When: using flip behavior (both negative)
    let (new_width, new_height) = scale_with_flip(width, height, scale_x, scale_y);

    // Then: both dimensions use absolute values
    assert!((new_width - 50.0).abs() < TOLERANCE);
    assert!((new_height - 100.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_negative_scaling_clamp_x() {
    // Given: a rectangle
    let width = 100.0;
    let height = 50.0;
    let scale_x = -1.0; // Negative scale
    let scale_y = 1.0;

    // When: using clamp behavior
    let (new_width, new_height) = scale_with_clamp(width, height, scale_x, scale_y, MIN_SIZE);

    // Then: negative scale is clamped to minimum
    assert!((new_width - MIN_SIZE).abs() < TOLERANCE);
    assert!((new_height - 50.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_negative_scaling_clamp_y() {
    // Given: a rectangle
    let width = 100.0;
    let height = 50.0;
    let scale_x = 1.0;
    let scale_y = -0.5; // Negative scale

    // When: using clamp behavior
    let (new_width, new_height) = scale_with_clamp(width, height, scale_x, scale_y, MIN_SIZE);

    // Then: negative scale is clamped to minimum
    assert!((new_width - 100.0).abs() < TOLERANCE);
    assert!((new_height - MIN_SIZE).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_negative_scaling_clamp_both() {
    // Given: a rectangle
    let width = 100.0;
    let height = 50.0;
    let scale_x = -2.0;
    let scale_y = -3.0;

    // When: using clamp behavior (both negative)
    let (new_width, new_height) = scale_with_clamp(width, height, scale_x, scale_y, MIN_SIZE);

    // Then: both dimensions are clamped to minimum
    assert!((new_width - MIN_SIZE).abs() < TOLERANCE);
    assert!((new_height - MIN_SIZE).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_negative_scaling_zero_transition() {
    // Given: scale factor approaching zero from positive side
    let width = 100.0;
    let height = 50.0;

    // When: scaling with very small positive factor then negative
    let tiny_positive = 0.001;
    let tiny_negative = -0.001;

    let (flip_pos_w, _) = scale_with_flip(width, height, tiny_positive, 1.0);
    let (flip_neg_w, _) = scale_with_flip(width, height, tiny_negative, 1.0);

    // Then: flip behavior treats both the same (absolute value)
    assert!((flip_pos_w - flip_neg_w).abs() < TOLERANCE);

    // Clamp behavior gives different results
    let (clamp_pos_w, _) = scale_with_clamp(width, height, tiny_positive, 1.0, MIN_SIZE);
    let (clamp_neg_w, _) = scale_with_clamp(width, height, tiny_negative, 1.0, MIN_SIZE);

    // Positive tiny scale clamps to min, negative also clamps to min
    assert!((clamp_pos_w - MIN_SIZE).abs() < TOLERANCE);
    assert!((clamp_neg_w - MIN_SIZE).abs() < TOLERANCE);
}
