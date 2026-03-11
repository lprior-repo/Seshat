use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

    // ============== GEO-025: Repeated Tiny Transforms - Rotation Drift ==============

    #[test]
    fn test_repeated_tiny_transforms_no_drift() {
        // Given: a point at (100, 0)
        let original = Point::new(100.0, 0.0);
        let center = Point::origin();
        let tiny_angle = 0.001; // ~0.057 degrees
        let iterations = 1000;

        // When: applying 1000 tiny rotations that sum to ~57.3 degrees
        let mut current = original;
        for _ in 0..iterations {
            current = rotate_around_center(current, center, tiny_angle);
        }

        // Then: compare with single rotation of total angle
        let total_angle = tiny_angle * f64::from(iterations);
        let expected = rotate_around_center(original, center, total_angle);

        // Drift should be bounded (accumulated floating-point error)
        let drift = ((current.x - expected.x).powi(2) + (current.y - expected.y).powi(2)).sqrt();
        assert!(drift < 1e-6, "Drift {} exceeds threshold", drift);
    }

    #[test]
    fn test_repeated_tiny_rotations_full_circle() {
        // Given: a point at (100, 0)
        let original = Point::new(100.0, 0.0);
        let center = Point::origin();
        let total_angle = 2.0 * PI;
        let iterations = 1000;
        let tiny_angle = total_angle / f64::from(iterations);

        // When: rotating in tiny steps for a full circle
        let mut current = original;
        for _ in 0..iterations {
            current = rotate_around_center(current, center, tiny_angle);
        }

        // Then: should return close to original
        let drift = ((current.x - original.x).powi(2) + (current.y - original.y).powi(2)).sqrt();
        assert!(
            drift < 1e-6,
            "Full circle drift {} exceeds threshold",
            drift
        );
    }

