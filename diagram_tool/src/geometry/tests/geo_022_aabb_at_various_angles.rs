use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-022: AABB at Various Angles ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_aabb_at_various_angles() {
    // Given: a rectangle at various rotation angles
    let angles = [PI / 12.0, PI / 6.0, PI / 4.0, PI / 3.0, 5.0 * PI / 12.0]; // 15, 30, 45, 60, 75 degrees

    for angle in angles {
        // When: calculating AABB for rotated rectangle
        let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(angle);
        let aabb = rect.aabb();

        // Then: AABB contains all corners
        let corners = rect.corners();
        for corner in corners {
            assert!(corner.x >= aabb.min_x - TOLERANCE);
            assert!(corner.x <= aabb.max_x + TOLERANCE);
            assert!(corner.y >= aabb.min_y - TOLERANCE);
            assert!(corner.y <= aabb.max_y + TOLERANCE);
        }
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_aabb_at_15_degrees() {
    // Given: rectangle rotated 15 degrees
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI / 12.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB is larger than axis-aligned but smaller than 45-degree case
    let axis_aligned_area = 100.0 * 50.0;
    let aabb_area = aabb.width() * aabb.height();
    assert!(aabb_area > axis_aligned_area);
    // At 45 degrees, area would be maximum for square, so 15 degrees should be smaller
    let rect_45 = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI / 4.0);
    assert!(aabb_area < rect_45.aabb().width() * rect_45.aabb().height() + TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_aabb_at_60_degrees() {
    // Given: rectangle rotated 60 degrees
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI / 3.0);

    // When: calculating AABB
    let aabb = rect.aabb();

    // Then: AABB contains all corners
    let corners = rect.corners();
    for corner in corners {
        assert!(corner.x >= aabb.min_x - TOLERANCE);
        assert!(corner.x <= aabb.max_x + TOLERANCE);
    }
}
