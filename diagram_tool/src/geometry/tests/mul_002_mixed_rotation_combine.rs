use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

    // ============== MUL-002: Mixed Rotation Combine ==============

    #[test]
    fn test_mul_mixed_rotation_combine() {
        // Given: a point and multiple rotation angles
        let original = Point::new(100.0, 0.0);
        let center = Point::origin();
        let angle_a = PI / 6.0; // 30 degrees
        let angle_b = PI / 3.0; // 60 degrees

        // When: rotating by A then by B (sequential)
        let after_a = rotate_around_center(original, center, angle_a);
        let after_a_then_b = rotate_around_center(after_a, center, angle_b);

        // And: rotating by (A + B) in one step
        let combined_angle = angle_a + angle_b;
        let after_combined = rotate_around_center(original, center, combined_angle);

        // Then: both approaches yield the same result
        assert!((after_a_then_b.x - after_combined.x).abs() < TOLERANCE);
        assert!((after_a_then_b.y - after_combined.y).abs() < TOLERANCE);
    }

    #[test]
    fn test_mul_mixed_rotation_combine_multiple() {
        // Given: a point and three rotation angles
        let original = Point::new(50.0, 50.0);
        let center = Point::new(25.0, 25.0);
        let angles = [PI / 12.0, PI / 8.0, PI / 6.0]; // 15, 22.5, 30 degrees

        // When: applying rotations sequentially
        let mut sequential = original;
        for &angle in &angles {
            sequential = rotate_around_center(sequential, center, angle);
        }

        // And: applying combined rotation
        let total_angle: f64 = angles.iter().sum();
        let combined = rotate_around_center(original, center, total_angle);

        // Then: both approaches yield the same result
        assert!((sequential.x - combined.x).abs() < TOLERANCE);
        assert!((sequential.y - combined.y).abs() < TOLERANCE);
    }

