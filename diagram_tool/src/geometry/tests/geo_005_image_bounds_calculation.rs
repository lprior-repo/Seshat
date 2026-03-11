use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-005: Image Bounds Calculation ==============

#[test]
fn test_image_bounds() {
    // Given: an image with position and dimensions
    let image = Image::new(50.0, 100.0, 200.0, 150.0);

    // When: calculating bounds
    let bounds = image.bounds();

    // Then: bounds equal position + dimensions
    assert!((bounds.min_x - 50.0).abs() < TOLERANCE);
    assert!((bounds.min_y - 100.0).abs() < TOLERANCE);
    assert!((bounds.max_x - 250.0).abs() < TOLERANCE);
    assert!((bounds.max_y - 250.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_image_bounds_kani() {
    // Given: an image with position and dimensions
    let image = Image::new(50.0, 100.0, 200.0, 150.0);

    // When: calculating bounds
    let bounds = image.bounds();

    // Then: bounds equal position + dimensions
    assert!((bounds.min_x - 50.0).abs() < TOLERANCE);
    assert!((bounds.min_y - 100.0).abs() < TOLERANCE);
    assert!((bounds.max_x - 250.0).abs() < TOLERANCE);
    assert!((bounds.max_y - 250.0).abs() < TOLERANCE);
}

#[test]
fn test_image_bounds_at_origin() {
    // Given: an image at origin
    let image = Image::new(0.0, 0.0, 100.0, 100.0);

    // When: calculating bounds
    let bounds = image.bounds();

    // Then: bounds start at origin
    assert!((bounds.min_x - 0.0).abs() < TOLERANCE);
    assert!((bounds.min_y - 0.0).abs() < TOLERANCE);
    assert!((bounds.max_x - 100.0).abs() < TOLERANCE);
    assert!((bounds.max_y - 100.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_image_bounds_at_origin_kani() {
    // Given: an image at origin
    let image = Image::new(0.0, 0.0, 100.0, 100.0);

    // When: calculating bounds
    let bounds = image.bounds();

    // Then: bounds start at origin
    assert!((bounds.min_x - 0.0).abs() < TOLERANCE);
    assert!((bounds.min_y - 0.0).abs() < TOLERANCE);
    assert!((bounds.max_x - 100.0).abs() < TOLERANCE);
    assert!((bounds.max_y - 100.0).abs() < TOLERANCE);
}
