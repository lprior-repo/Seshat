use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-012: Zoom at Pointer ==============

/// Zoom a view rectangle around a pointer position
#[must_use]
pub fn zoom_at_pointer(view_center: Point, pointer: Point, factor: f64) -> Point {
    // The pointer stays fixed; the view center moves relative to it
    // new_view_center = pointer + (view_center - pointer) * factor
    Point::new(
        pointer.x + (view_center.x - pointer.x) * factor,
        pointer.y + (view_center.y - pointer.y) * factor,
    )
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_zoom_at_pointer_center() {
    // Given: view centered at origin, pointer at origin
    let view_center = Point::origin();
    let pointer = Point::origin();

    // When: zooming by 2x
    let new_center = zoom_at_pointer(view_center, pointer, 2.0);

    // Then: center stays at pointer (which is at origin)
    assert!((new_center.x - 0.0).abs() < TOLERANCE);
    assert!((new_center.y - 0.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_zoom_at_pointer_offset() {
    // Given: view at (100, 100), pointer at (50, 50)
    let view_center = Point::new(100.0, 100.0);
    let pointer = Point::new(50.0, 50.0);

    // When: zooming in by 2x
    let new_center = zoom_at_pointer(view_center, pointer, 2.0);

    // Then: center moves away from pointer
    // new = 50 + (100 - 50) * 2 = 50 + 100 = 150
    assert!((new_center.x - 150.0).abs() < TOLERANCE);
    assert!((new_center.y - 150.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_zoom_at_pointer_zoom_out() {
    // Given: view at (100, 100), pointer at (50, 50)
    let view_center = Point::new(100.0, 100.0);
    let pointer = Point::new(50.0, 50.0);

    // When: zooming out by 0.5x
    let new_center = zoom_at_pointer(view_center, pointer, 0.5);

    // Then: center moves toward pointer
    // new = 50 + (100 - 50) * 0.5 = 50 + 25 = 75
    assert!((new_center.x - 75.0).abs() < TOLERANCE);
    assert!((new_center.y - 75.0).abs() < TOLERANCE);
}
