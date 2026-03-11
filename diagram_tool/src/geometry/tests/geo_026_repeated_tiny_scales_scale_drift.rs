use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-026: Repeated Tiny Scales - Scale Drift ==============

#[test]
fn test_repeated_tiny_scales_no_drift() {
    // Given: a point at (100, 0) with anchor at origin
    let original = Point::new(100.0, 0.0);
    let anchor = Point::origin();
    let tiny_factor = 1.001; // 0.1% growth
    let iterations = 1000;

    // When: applying 1000 tiny scales
    let mut current = original;
    for _ in 0..iterations {
        current = scale_around_anchor(current, anchor, tiny_factor);
    }

    // Then: compare with single scale of total factor
    let total_factor = tiny_factor.powi(iterations);
    let expected = scale_around_anchor(original, anchor, total_factor);

    // Relative error should be bounded
    let relative_error = ((current.x - expected.x).abs() / expected.x.abs().max(1.0))
        .max((current.y - expected.y).abs() / expected.y.abs().max(1.0));
    assert!(
        relative_error < 1e-6,
        "Relative error {} exceeds threshold",
        relative_error
    );
}

#[test]
fn test_repeated_tiny_scales_inverse() {
    // Given: a point and scale factors that should cancel
    let original = Point::new(100.0, 50.0);
    let anchor = Point::origin();
    let factor_up = 1.001;
    let factor_down = 1.0 / factor_up;
    let iterations = 500;

    // When: scaling up then down repeatedly
    let mut current = original;
    for _ in 0..iterations {
        current = scale_around_anchor(current, anchor, factor_up);
        current = scale_around_anchor(current, anchor, factor_down);
    }

    // Then: should return close to original
    let drift = ((current.x - original.x).powi(2) + (current.y - original.y).powi(2)).sqrt();
    assert!(
        drift < 1e-9,
        "Inverse scale drift {} exceeds threshold",
        drift
    );
}
