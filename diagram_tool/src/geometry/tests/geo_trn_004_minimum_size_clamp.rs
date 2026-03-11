use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-TRN-004: Minimum Size Clamp ==============
//
// Tests that geometry cannot be scaled below minimum bounds.

const MIN_SIZE: f64 = 1.0;

/// Clamp dimensions to minimum size
fn clamp_to_min_size(width: f64, height: f64, min_size: f64) -> (f64, f64) {
    let clamped_width = width.max(min_size);
    let clamped_height = height.max(min_size);
    (clamped_width, clamped_height)
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_min_size_clamp_below_minimum() {
    // Given: dimensions below minimum
    let width = 0.5;
    let height = 0.3;

    // When: clamping to minimum size
    let (clamped_w, clamped_h) = clamp_to_min_size(width, height, MIN_SIZE);

    // Then: both dimensions are clamped to minimum
    assert!((clamped_w - MIN_SIZE).abs() < TOLERANCE);
    assert!((clamped_h - MIN_SIZE).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_min_size_clamp_one_below_minimum() {
    // Given: one dimension below minimum
    let width = 50.0;
    let height = 0.5;

    // When: clamping to minimum size
    let (clamped_w, clamped_h) = clamp_to_min_size(width, height, MIN_SIZE);

    // Then: only the small dimension is clamped
    assert!((clamped_w - 50.0).abs() < TOLERANCE);
    assert!((clamped_h - MIN_SIZE).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_min_size_clamp_at_minimum() {
    // Given: dimensions at exactly minimum
    let width = MIN_SIZE;
    let height = MIN_SIZE;

    // When: clamping to minimum size
    let (clamped_w, clamped_h) = clamp_to_min_size(width, height, MIN_SIZE);

    // Then: dimensions remain unchanged
    assert!((clamped_w - MIN_SIZE).abs() < TOLERANCE);
    assert!((clamped_h - MIN_SIZE).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_min_size_clamp_above_minimum() {
    // Given: dimensions above minimum
    let width = 100.0;
    let height = 50.0;

    // When: clamping to minimum size
    let (clamped_w, clamped_h) = clamp_to_min_size(width, height, MIN_SIZE);

    // Then: dimensions remain unchanged
    assert!((clamped_w - 100.0).abs() < TOLERANCE);
    assert!((clamped_h - 50.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_min_size_clamp_with_scaling() {
    // Given: a rectangle being scaled down
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
    let scale_factor = 0.005; // Would result in 0.5 x 0.5

    // When: scaling and clamping
    let scaled_width = rect.width * scale_factor;
    let scaled_height = rect.height * scale_factor;
    let (clamped_w, clamped_h) = clamp_to_min_size(scaled_width, scaled_height, MIN_SIZE);

    // Then: result is clamped to minimum
    assert!((clamped_w - MIN_SIZE).abs() < TOLERANCE);
    assert!((clamped_h - MIN_SIZE).abs() < TOLERANCE);
}
