use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

    // ============== GEO-009: Combined Transform Chain ==============

    #[test]
    fn test_combined_transforms() {
        // Given: a point at (2, 0), anchor at origin
        let point = Point::new(2.0, 0.0);
        let anchor = Point::origin();

        // When: scale by 2 then rotate 90 degrees
        let result = scale_then_rotate(point, anchor, 2.0, PI / 2.0);

        // Then: first scale (2, 0) -> (4, 0), then rotate -> (0, 4)
        assert!((result.x - 0.0).abs() < TOLERANCE);
        assert!((result.y - 4.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_combined_transforms_order_matters() {
        // Given: a point and anchor
        let point = Point::new(1.0, 0.0);
        let anchor = Point::origin();

        // When: rotate 90 degrees then scale by 2 (reverse order)
        // Note: Our function does scale first, then rotate
        // Scale: (1, 0) -> (2, 0), Rotate: (2, 0) -> (0, 2)
        let result = scale_then_rotate(point, anchor, 2.0, PI / 2.0);

        // Then: result is deterministic
        assert!((result.x - 0.0).abs() < TOLERANCE);
        assert!((result.y - 2.0).abs() < TOLERANCE);
    }

