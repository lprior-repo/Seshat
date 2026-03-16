#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== MUL-004: Rotate 360 No Drift ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_mul_rotate_360_no_drift() {
    // Given: multiple points representing selected item centers
    let items = [
        Point::new(10.0, 20.0),
        Point::new(100.0, 50.0),
        Point::new(200.0, 150.0),
    ];
    let center = Point::new(100.0, 100.0);

    // When: rotating by 360 degrees (2 * PI)
    let full_rotation = 2.0 * PI;
    let after_rotation: Vec<Point> = items
        .iter()
        .map(|&p| rotate_around_center(p, center, full_rotation))
        .collect();

    // Then: all items return to original positions with minimal drift
    for (original, rotated) in items.iter().zip(after_rotation.iter()) {
        let drift = ((rotated.x - original.x).powi(2) + (rotated.y - original.y).powi(2)).sqrt();
        assert!(
            drift < 1e-9,
            "Drift {} exceeds threshold for point {:?}",
            drift,
            original
        );
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_mul_rotate_360_no_drift_incremental() {
    // Given: points and incremental rotation steps
    let items = [
        Point::new(50.0, 0.0),
        Point::new(-30.0, 40.0),
        Point::new(100.0, 100.0),
    ];
    let center = Point::origin();
    let steps = 360;
    let angle_per_step = 2.0 * PI / f64::from(steps);

    // When: rotating in 360 small steps (1 degree each)
    let mut current = items;
    for _ in 0..steps {
        current = current.map(|p| rotate_around_center(p, center, angle_per_step));
    }

    // Then: all items return to original positions with bounded drift
    for (original, final_pos) in items.iter().zip(current.iter()) {
        let drift =
            ((final_pos.x - original.x).powi(2) + (final_pos.y - original.y).powi(2)).sqrt();
        // Allow slightly more drift for incremental operations
        assert!(
            drift < 1e-6,
            "Incremental drift {} exceeds threshold",
            drift
        );
    }
}
