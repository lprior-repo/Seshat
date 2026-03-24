use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-015: Grid Step ==============

/// Snap a point to the nearest grid intersection
#[must_use]
pub fn snap_to_grid(point: Point, grid_size: f64) -> Point {
    Point::new(
        (point.x / grid_size).round() * grid_size,
        (point.y / grid_size).round() * grid_size,
    )
}

#[cfg(kani)]
#[kani::proof]
fn test_grid_step_snap() {
    // Given: point at (47, 53) with grid size 10
    let point = Point::new(47.0, 53.0);
    let grid_size = 10.0;

    // When: snapping to grid
    let snapped = snap_to_grid(point, grid_size);

    // Then: snaps to (50, 50)
    assert!((snapped.x - 50.0).abs() < TOLERANCE);
    assert!((snapped.y - 50.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_grid_step_already_on_grid() {
    // Given: point already on grid
    let point = Point::new(50.0, 100.0);
    let grid_size = 10.0;

    // When: snapping to grid
    let snapped = snap_to_grid(point, grid_size);

    // Then: stays at same position
    assert!((snapped.x - 50.0).abs() < TOLERANCE);
    assert!((snapped.y - 100.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_grid_step_negative_coords() {
    // Given: point at negative coordinates
    let point = Point::new(-47.0, -53.0);
    let grid_size = 10.0;

    // When: snapping to grid
    let snapped = snap_to_grid(point, grid_size);

    // Then: snaps correctly in negative space
    assert!((snapped.x - (-50.0)).abs() < TOLERANCE);
    assert!((snapped.y - (-50.0)).abs() < TOLERANCE);
}
