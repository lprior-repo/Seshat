use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== MUL-016: Rotate Asymmetric Selection ==============

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
fn test_mul_016_rotate_asymmetric_selection() {
    let positions = [
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(5.0, 10.0),
        Point::new(200.0, 200.0),
    ];

    let center = selection_center(&positions);
    let expected_center_x = (0.0 + 10.0 + 5.0 + 200.0) / 4.0;
    let expected_center_y = (0.0 + 0.0 + 10.0 + 200.0) / 4.0;
    assert!((center.x - expected_center_x).abs() < TOLERANCE);
    assert!((center.y - expected_center_y).abs() < TOLERANCE);

    let angle = PI / 2.0;
    let rotated: Vec<Point> = positions
        .iter()
        .map(|&p| rotate_around_center(p, center, angle))
        .collect();

    for (original, rotated_p) in positions.iter().zip(rotated.iter()) {
        let rel_before = Point::new(original.x - center.x, original.y - center.y);
        let rel_after = Point::new(rotated_p.x - center.x, rotated_p.y - center.y);
        assert!((rel_after.x - (-rel_before.y)).abs() < TOLERANCE);
        assert!((rel_after.y - rel_before.x).abs() < TOLERANCE);
    }

    let new_center = selection_center(&rotated);
    assert!((new_center.x - center.x).abs() < TOLERANCE);
    assert!((new_center.y - center.y).abs() < TOLERANCE);
}
