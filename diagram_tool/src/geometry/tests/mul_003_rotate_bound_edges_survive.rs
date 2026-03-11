use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

    // ============== MUL-003: Rotate Bound Edges Survive ==============

    #[test]
    fn test_mul_rotate_bound_edges_survive() {
        // Given: multiple rectangles representing selected items
        let rects = [
            Rectangle::new(0.0, 0.0, 50.0, 50.0),
            Rectangle::new(100.0, 0.0, 50.0, 50.0),
            Rectangle::new(100.0, 100.0, 50.0, 50.0),
            Rectangle::new(0.0, 100.0, 50.0, 50.0),
        ];

        // Calculate selection bounds (AABB encompassing all items)
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for rect in &rects {
            let aabb = rect.aabb();
            min_x = min_x.min(aabb.min_x);
            min_y = min_y.min(aabb.min_y);
            max_x = max_x.max(aabb.max_x);
            max_y = max_y.max(aabb.max_y);
        }

        // Selection center
        let center = Point::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);

        // When: rotating all items by 45 degrees
        let angle = PI / 4.0;
        let rotated_rects: Vec<Rectangle> = rects
            .iter()
            .map(|r| {
                // Rotate the rectangle's position around the selection center
                let rotated_pos = rotate_around_center(Point::new(r.x, r.y), center, angle);
                Rectangle::new(rotated_pos.x, rotated_pos.y, r.width, r.height)
                    .with_rotation(r.rotation + angle)
            })
            .collect();

        // Calculate new selection bounds
        let mut new_min_x = f64::INFINITY;
        let mut new_min_y = f64::INFINITY;
        let mut new_max_x = f64::NEG_INFINITY;
        let mut new_max_y = f64::NEG_INFINITY;
        for rect in &rotated_rects {
            let aabb = rect.aabb();
            new_min_x = new_min_x.min(aabb.min_x);
            new_min_y = new_min_y.min(aabb.min_y);
            new_max_x = new_max_x.max(aabb.max_x);
            new_max_y = new_max_y.max(aabb.max_y);
        }

        // Then: all rotated item corners are within the new selection bounds
        for rect in &rotated_rects {
            let corners = rect.corners();
            for corner in corners {
                assert!(
                    corner.x >= new_min_x - TOLERANCE,
                    "Corner x {} < min_x {}",
                    corner.x,
                    new_min_x
                );
                assert!(
                    corner.x <= new_max_x + TOLERANCE,
                    "Corner x {} > max_x {}",
                    corner.x,
                    new_max_x
                );
                assert!(
                    corner.y >= new_min_y - TOLERANCE,
                    "Corner y {} < min_y {}",
                    corner.y,
                    new_min_y
                );
                assert!(
                    corner.y <= new_max_y + TOLERANCE,
                    "Corner y {} > max_y {}",
                    corner.y,
                    new_max_y
                );
            }
        }
    }

