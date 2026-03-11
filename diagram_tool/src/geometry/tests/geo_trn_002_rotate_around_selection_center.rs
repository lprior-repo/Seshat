use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

    // ============== GEO-TRN-002: Rotate Around Selection Center ==============
    //
    // Tests rotation operations centered on the selection's centroid.
    // The selection center is computed as the average of all item positions.

    #[test]
    fn test_rotate_around_selection_center_single_item() {
        // Given: a single rectangle
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let items = [rect];
        let center = selection_center(&items.map(|r| Point::new(r.x, r.y)));

        // When: rotating 90 degrees around selection center
        let angle = PI / 2.0;
        let rotated_pos = rotate_around_center(Point::new(rect.x, rect.y), center, angle);

        // Then: the item rotates around the center
        // For a single item at (0,0), center is (0,0), so position stays
        assert!((rotated_pos.x - 0.0).abs() < TOLERANCE);
        assert!((rotated_pos.y - 0.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotate_around_selection_center_multiple_items() {
        // Given: multiple items forming a pattern
        let positions = [
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ];
        let center = selection_center(&positions);
        // Center should be at (50, 50)
        assert!((center.x - 50.0).abs() < TOLERANCE);
        assert!((center.y - 50.0).abs() < TOLERANCE);

        // When: rotating all items 90 degrees around selection center
        let angle = PI / 2.0;
        let rotated: Vec<Point> = positions
            .iter()
            .map(|&p| rotate_around_center(p, center, angle))
            .collect();

        // Then: items rotate as a group maintaining relative positions
        // Original (0, 0) relative to (50, 50) is (-50, -50)
        // After 90deg: (-50, -50) -> (50, -50) relative -> (100, 0) absolute
        assert!((rotated[0].x - 100.0).abs() < TOLERANCE);
        assert!((rotated[0].y - 0.0).abs() < TOLERANCE);

        // Verify selection center is unchanged
        let new_center = selection_center(&rotated);
        assert!((new_center.x - center.x).abs() < TOLERANCE);
        assert!((new_center.y - center.y).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotate_around_selection_center_45_degrees() {
        // Given: three items at different positions
        let positions = [
            Point::new(0.0, 0.0),
            Point::new(50.0, 0.0),
            Point::new(25.0, 50.0),
        ];
        let center = selection_center(&positions);
        // Center = ((0+50+25)/3, (0+0+50)/3) = (25, 16.67)
        let expected_center_x = 25.0;
        let expected_center_y = 50.0 / 3.0;
        assert!((center.x - expected_center_x).abs() < TOLERANCE);
        assert!((center.y - expected_center_y).abs() < TOLERANCE);

        // When: rotating 45 degrees
        let angle = PI / 4.0;
        let rotated: Vec<Point> = positions
            .iter()
            .map(|&p| rotate_around_center(p, center, angle))
            .collect();

        // Then: distances from center are preserved
        for (original, rotated_p) in positions.iter().zip(rotated.iter()) {
            let dist_before =
                ((original.x - center.x).powi(2) + (original.y - center.y).powi(2)).sqrt();
            let dist_after =
                ((rotated_p.x - center.x).powi(2) + (rotated_p.y - center.y).powi(2)).sqrt();
            assert!((dist_before - dist_after).abs() < TOLERANCE);
        }
    }

