// ============== GEO-021 to GEO-025: Intersection Algorithms ==============

use super::*;

#[test]
fn test_line_segment_new_rejects_nan() {
    let result = LineSegment::new(Point::new(f64::NAN, 0.0), Point::new(1.0, 1.0));
    assert!(result.is_err());
}

#[test]
fn test_line_segment_new_rejects_infinity() {
    let result = LineSegment::new(Point::new(0.0, f64::INFINITY), Point::new(1.0, 1.0));
    assert!(result.is_err());
}

#[test]
fn test_line_segment_new_rejects_zero_length() {
    let result = LineSegment::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0));
    assert!(result.is_err());
}

#[test]
fn test_line_line_intersects_crossing() {
    let a = LineSegment::new_unchecked(Point::new(0.0, 0.0), Point::new(10.0, 10.0));
    let b = LineSegment::new_unchecked(Point::new(0.0, 10.0), Point::new(10.0, 0.0));
    assert!(line_line_intersects(a, b));
}

#[test]
fn test_line_line_intersects_parallel() {
    let a = LineSegment::new_unchecked(Point::new(0.0, 0.0), Point::new(10.0, 0.0));
    let b = LineSegment::new_unchecked(Point::new(0.0, 5.0), Point::new(10.0, 5.0));
    assert!(!line_line_intersects(a, b));
}

#[test]
fn test_line_line_intersection_crossing() {
    let a = LineSegment::new_unchecked(Point::new(0.0, 0.0), Point::new(10.0, 10.0));
    let b = LineSegment::new_unchecked(Point::new(0.0, 10.0), Point::new(10.0, 0.0));
    let result = line_line_intersection(a, b);
    assert!(result.is_some());
    let p = result.unwrap();
    assert!((p.x - 5.0).abs() < 1e-10);
    assert!((p.y - 5.0).abs() < 1e-10);
}

#[test]
fn test_line_line_intersection_parallel() {
    let a = LineSegment::new_unchecked(Point::new(0.0, 0.0), Point::new(10.0, 0.0));
    let b = LineSegment::new_unchecked(Point::new(0.0, 5.0), Point::new(10.0, 5.0));
    assert!(line_line_intersection(a, b).is_none());
}

#[test]
fn test_line_rect_intersects_crossing() {
    let line = LineSegment::new_unchecked(Point::new(0.0, 50.0), Point::new(100.0, 50.0));
    let rect = AABB::new(30.0, 30.0, 70.0, 70.0);
    assert!(line_rect_intersects(line, &rect));
}

#[test]
fn test_line_rect_intersects_outside() {
    let line = LineSegment::new_unchecked(Point::new(0.0, 0.0), Point::new(10.0, 10.0));
    let rect = AABB::new(20.0, 20.0, 30.0, 30.0);
    assert!(!line_rect_intersects(line, &rect));
}

#[test]
fn test_line_rect_intersections_two_points() {
    let line = LineSegment::new_unchecked(Point::new(0.0, 50.0), Point::new(100.0, 50.0));
    let rect = AABB::new(30.0, 30.0, 70.0, 70.0);
    let points = line_rect_intersections(line, &rect);
    assert_eq!(points.len(), 2);
}
