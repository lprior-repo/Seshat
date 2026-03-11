use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

    // ============== GEO-TRN-003: Rotate Around Custom Pivot ==============
    //
    // Tests rotation operations using a user-defined pivot point.

    #[test]
    fn test_rotate_around_custom_pivot_origin() {
        // Given: a point and custom pivot at origin
        let point = Point::new(100.0, 0.0);
        let pivot = Point::origin();

        // When: rotating 90 degrees around the pivot
        let rotated = rotate_around_center(point, pivot, PI / 2.0);

        // Then: point rotates correctly
        assert!((rotated.x - 0.0).abs() < TOLERANCE);
        assert!((rotated.y - 100.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotate_around_custom_pivot_offset() {
        // Given: a point and custom pivot at offset position
        let point = Point::new(150.0, 50.0);
        let pivot = Point::new(100.0, 100.0);

        // When: rotating 180 degrees around the pivot
        let rotated = rotate_around_center(point, pivot, PI);

        // Then: point rotates to opposite side
        // Relative position: (50, -50)
        // After 180 degree rotation: (-50, 50)
        // Absolute position: (100-50, 100+50) = (50, 150)
        assert!((rotated.x - 50.0).abs() < TOLERANCE);
        assert!((rotated.y - 150.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotate_around_custom_pivot_270_degrees() {
        // Given: a point and custom pivot
        let point = Point::new(50.0, 0.0);
        let pivot = Point::new(0.0, 0.0);

        // When: rotating 270 degrees (3*PI/2) counter-clockwise
        let rotated = rotate_around_center(point, pivot, 3.0 * PI / 2.0);

        // Then: equivalent to 90 degrees clockwise
        // (50, 0) -> (0, -50)
        assert!((rotated.x - 0.0).abs() < TOLERANCE);
        assert!((rotated.y - (-50.0)).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotate_around_custom_pivot_preserves_distance() {
        // Given: a point at distance d from pivot
        let point = Point::new(30.0, 40.0);
        let pivot = Point::new(10.0, 10.0);
        let distance = ((point.x - pivot.x).powi(2) + (point.y - pivot.y).powi(2)).sqrt();

        // When: rotating by various angles
        let angles = [PI / 6.0, PI / 4.0, PI / 3.0, PI / 2.0, PI];
        for &angle in &angles {
            let rotated = rotate_around_center(point, pivot, angle);
            let rotated_distance =
                ((rotated.x - pivot.x).powi(2) + (rotated.y - pivot.y).powi(2)).sqrt();

            // Then: distance is preserved
            assert!((distance - rotated_distance).abs() < TOLERANCE);
        }
    }

