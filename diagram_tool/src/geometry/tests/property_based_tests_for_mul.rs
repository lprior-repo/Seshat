use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== Property-Based Tests for MUL ==============

proptest! {
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_mul_rotation_preserves_distances(
        x1 in -100.0_f64..100.0,
        y1 in -100.0_f64..100.0,
        x2 in -100.0_f64..100.0,
        y2 in -100.0_f64..100.0,
        cx in -50.0_f64..50.0,
        cy in -50.0_f64..50.0,
        angle in 0.0_f64..2.0 * PI
    ) {
        let p1 = Point::new(x1, y1);
        let p2 = Point::new(x2, y2);
        let center = Point::new(cx, cy);

        // Distance between points before rotation
        let dist_before = ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt();

        // Rotate both points around the same center
        let r1 = rotate_around_center(p1, center, angle);
        let r2 = rotate_around_center(p2, center, angle);

        // Distance after rotation
        let dist_after = ((r2.x - r1.x).powi(2) + (r2.y - r1.y).powi(2)).sqrt();

        // Rotation preserves distances between points
        prop_assert!((dist_before - dist_after).abs() < 1e-9);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_mul_full_rotation_returns_to_origin(
        x in -1000.0_f64..1000.0,
        y in -1000.0_f64..1000.0,
        cx in -500.0_f64..500.0,
        cy in -500.0_f64..500.0
    ) {
        let point = Point::new(x, y);
        let center = Point::new(cx, cy);

        let rotated = rotate_around_center(point, center, 2.0 * PI);

        let drift = ((rotated.x - point.x).powi(2) + (rotated.y - point.y).powi(2)).sqrt();
        prop_assert!(drift < 1e-9, "Drift {} exceeds threshold", drift);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_mul_selection_center_unchanged_by_rotation(
        n in 2usize..10,
        angle in 0.0_f64..2.0 * PI
    ) {
        // Generate n random points
        let points: Vec<Point> = (0..n)
            .map(|i| Point::new(i as f64 * 10.0, (i as f64 * 7.0) % 100.0))
            .collect();

        let center = selection_center(&points);

        // Rotate all points
        let rotated: Vec<Point> = points
            .iter()
            .map(|&p| rotate_around_center(p, center, angle))
            .collect();

        let new_center = selection_center(&rotated);

        // Selection center is invariant under rotation
        prop_assert!((new_center.x - center.x).abs() < 1e-10);
        prop_assert!((new_center.y - center.y).abs() < 1e-10);
    }
}
