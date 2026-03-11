use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-010: Bounds Edge Cases ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_bounds_edge_cases_zero_size() {
    // Given: zero-sized bounds
    let result = safe_bounds(0.0, 0.0, 0.0, 0.0);

    // Then: valid AABB with zero dimensions
    assert!(result.is_ok());
    let aabb = result.unwrap();
    assert!((aabb.width() - 0.0).abs() < TOLERANCE);
    assert!((aabb.height() - 0.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_bounds_edge_cases_negative_coords() {
    // Given: negative coordinates
    let result = safe_bounds(-100.0, -50.0, -10.0, -5.0);

    // Then: valid AABB
    assert!(result.is_ok());
    let aabb = result.unwrap();
    assert!((aabb.min_x - (-100.0)).abs() < TOLERANCE);
    assert!((aabb.min_y - (-50.0)).abs() < TOLERANCE);
    assert!((aabb.max_x - (-10.0)).abs() < TOLERANCE);
    assert!((aabb.max_y - (-5.0)).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_bounds_edge_cases_large_coords() {
    // Given: very large coordinates
    let result = safe_bounds(1e10, 1e10, 1e10 + 100.0, 1e10 + 100.0);

    // Then: valid AABB (within f64 range)
    assert!(result.is_ok());
    let aabb = result.unwrap();
    assert!((aabb.width() - 100.0).abs() < TOLERANCE);
    assert!((aabb.height() - 100.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_bounds_edge_cases_nan() {
    // Given: NaN values
    let result = safe_bounds(f64::NAN, 0.0, 100.0, 100.0);

    // Then: None (invalid)
    assert!(result.is_err());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_bounds_edge_cases_infinity() {
    // Given: infinity values
    let result = safe_bounds(f64::INFINITY, 0.0, 100.0, 100.0);

    // Then: None (invalid)
    assert!(result.is_err());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_bounds_edge_cases_swapped_min_max() {
    // Given: min > max (swapped)
    let result = safe_bounds(100.0, 100.0, 0.0, 0.0);

    // Then: valid AABB with corrected order
    assert!(result.is_ok());
    let aabb = result.unwrap();
    assert!((aabb.min_x - 0.0).abs() < TOLERANCE);
    assert!((aabb.min_y - 0.0).abs() < TOLERANCE);
    assert!((aabb.max_x - 100.0).abs() < TOLERANCE);
    assert!((aabb.max_y - 100.0).abs() < TOLERANCE);
}
