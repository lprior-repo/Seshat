use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== MUL-001: Rotate Around Center ==============

/// Calculate the center (centroid) of multiple points
fn selection_center(points: &[Point]) -> Point {
    if points.is_empty() {
        return Point::origin();
    }
    let sum_x: f64 = points.iter().map(|p| p.x).sum();
    let sum_y: f64 = points.iter().map(|p| p.y).sum();
    let count = points.len() as f64;
    Point::new(sum_x / count, sum_y / count)
}

#[test]
fn test_mul_rotate_around_center() {
    // Given: multiple selected items at different positions
    let items = [
        Point::new(0.0, 0.0),
        Point::new(100.0, 0.0),
        Point::new(100.0, 100.0),
        Point::new(0.0, 100.0),
    ];

    // Calculate selection center (centroid)
    let center = selection_center(&items);
    assert!((center.x - 50.0).abs() < TOLERANCE);
    assert!((center.y - 50.0).abs() < TOLERANCE);

    // When: rotating all items 90 degrees around the selection center
    let angle = PI / 2.0;
    let rotated: Vec<Point> = items
        .iter()
        .map(|&p| rotate_around_center(p, center, angle))
        .collect();

    // Then: all items maintain relative positions (rotated as a group)
    // Original (0,0) relative to center (50,50) is (-50,-50)
    // After 90deg rotation: (-50,-50) -> (50,-50) relative -> (100,0) absolute
    assert!((rotated[0].x - 100.0).abs() < TOLERANCE);
    assert!((rotated[0].y - 0.0).abs() < TOLERANCE);

    // Verify the new selection center is unchanged
    let new_center = selection_center(&rotated);
    assert!((new_center.x - center.x).abs() < TOLERANCE);
    assert!((new_center.y - center.y).abs() < TOLERANCE);
}
