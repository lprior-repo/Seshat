#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-008: Resize with Aspect Ratio Lock ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_resize_aspect_lock() {
    // Given: original dimensions 100x50 (2:1 aspect ratio)
    let original_width = 100.0;
    let original_height = 50.0;

    // When: resizing width to 200
    let new_height = resize_with_aspect_lock(original_width, original_height, 200.0);

    // Then: height maintains 2:1 aspect ratio (should be 100)
    assert!((new_height - 100.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_resize_aspect_lock_shrink() {
    // Given: original dimensions 100x50 (2:1 aspect ratio)
    let original_width = 100.0;
    let original_height = 50.0;

    // When: resizing width to 50
    let new_height = resize_with_aspect_lock(original_width, original_height, 50.0);

    // Then: height maintains aspect ratio (should be 25)
    assert!((new_height - 25.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_resize_aspect_lock_square() {
    // Given: square dimensions 100x100
    let original_width = 100.0;
    let original_height = 100.0;

    // When: resizing width to 200
    let new_height = resize_with_aspect_lock(original_width, original_height, 200.0);

    // Then: height equals new width (1:1 aspect ratio)
    assert!((new_height - 200.0).abs() < TOLERANCE);
}
