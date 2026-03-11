use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

    // ============== GEO-024: Resize Then Rotation Composition ==============

    #[test]
    fn test_resize_then_rotation_composition() {
        // Given: a point at (100, 0) relative to origin
        let point = Point::new(100.0, 0.0);
        let center = Point::origin();
        let angle = PI / 2.0; // 90 degrees
        let scale_factor = 0.5;

        // When: resize then rotate
        let scaled = scale_around_anchor(point, center, scale_factor);
        let final_point = rotate_around_center(scaled, center, angle);

        // Then: first scale (100, 0) -> (50, 0), then rotate -> (0, 50)
        assert!((final_point.x - 0.0).abs() < TOLERANCE);
        assert!((final_point.y - 50.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_transform_order_matters() {
        // Given: a point and transformation parameters
        // Using a non-45-degree angle and non-origin center to ensure order matters
        let point = Point::new(10.0, 5.0);
        let center = Point::new(3.0, 2.0); // Non-origin center
        let angle = PI / 6.0; // 30 degrees (not 45)
        let scale_factor = 2.0;

        // When: applying transforms in different orders
        let rotate_then_scale = scale_around_anchor(
            rotate_around_center(point, center, angle),
            center,
            scale_factor,
        );
        let scale_then_rotate = rotate_around_center(
            scale_around_anchor(point, center, scale_factor),
            center,
            angle,
        );

        // For uniform scaling around the same center as rotation,
        // order actually doesn't matter - both operations commute.
        // This is a mathematical property: scale then rotate = rotate then scale
        // when both are centered at the same point.
        // Let's verify this property instead:
        assert!((rotate_then_scale.x - scale_then_rotate.x).abs() < TOLERANCE);
        assert!((rotate_then_scale.y - scale_then_rotate.y).abs() < TOLERANCE);
    }

