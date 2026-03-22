use crate::geometry::primitives::{Point, Rectangle, AABB};

pub mod container_bounds;
pub mod edge_bounds;

pub use container_bounds::{compute_subgraph_bounds, recompute_container_bounds, SUBGRAPH_PADDING};
pub use edge_bounds::{edge_bounds, EdgeArrowType, EdgeBoundsError};

// Re-export zoom safety functions from canvas_math
pub use canvas_math::safe_zoom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BoundsError {
    #[error("Invalid coordinate: NaN or Infinity")]
    InvalidCoordinate,
}

/// Creates a bounding box safely.
///
/// # Errors
/// Returns an error if any coordinate is NaN or infinite.
pub const fn safe_bounds(
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> Result<AABB, BoundsError> {
    if min_x.is_nan()
        || min_y.is_nan()
        || max_x.is_nan()
        || max_y.is_nan()
        || min_x.is_infinite()
        || min_y.is_infinite()
        || max_x.is_infinite()
        || max_y.is_infinite()
    {
        return Err(BoundsError::InvalidCoordinate);
    }

    let final_min_x = min_x.min(max_x);
    let final_max_x = min_x.max(max_x);
    let final_min_y = min_y.min(max_y);
    let final_max_y = min_y.max(max_y);

    Ok(AABB::new(
        final_min_x,
        final_min_y,
        final_max_x,
        final_max_y,
    ))
}

#[must_use]
pub fn zoom_at_pointer(view_center: Point, pointer: Point, factor: f64) -> Point {
    Point::new(
        (view_center.x - pointer.x).mul_add(factor, pointer.x),
        (view_center.y - pointer.y).mul_add(factor, pointer.y),
    )
}

#[must_use]
pub fn snap_horizontal(line_y: f64, targets: &[f64], tolerance: f64) -> Option<f64> {
    targets
        .iter()
        .map(|&t| (t, (line_y - t).abs()))
        .filter(|(_, dist)| *dist <= tolerance)
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(t, _)| t)
}

#[must_use]
pub fn snap_vertical(line_x: f64, targets: &[f64], tolerance: f64) -> Option<f64> {
    snap_horizontal(line_x, targets, tolerance)
}

#[must_use]
pub fn hit_test_rect(point: Point, rect: &Rectangle, margin: f64) -> bool {
    let aabb = rect.aabb();
    point.x >= aabb.min_x - margin
        && point.x <= aabb.max_x + margin
        && point.y >= aabb.min_y - margin
        && point.y <= aabb.max_y + margin
}

#[must_use]
pub fn hit_test_rotated_rect(point: Point, rect: &Rectangle) -> bool {
    if rect.rotation == 0.0 {
        return hit_test_rect(point, rect, 0.0);
    }
    let center = rect.aabb().center();
    let local_point =
        crate::geometry::transforms::rotate_around_center(point, center, -rect.rotation);
    let local_rect = Rectangle::new(rect.x, rect.y, rect.width, rect.height);
    hit_test_rect(local_point, &local_rect, 0.0)
}

#[must_use]
pub fn world_to_screen(world: Point, camera: Point, zoom: f64) -> Point {
    canvas_math::canvas_to_screen(world.x, world.y, camera.x, camera.y, zoom)
        .map_or(Point::origin(), |(x, y)| Point::new(x, y))
}

#[must_use]
pub fn screen_to_world(screen: Point, camera: Point, zoom: f64) -> Point {
    canvas_math::screen_to_canvas(screen.x, screen.y, camera.x, camera.y, zoom)
        .map_or(Point::origin(), |(x, y)| Point::new(x, y))
}

#[must_use]
pub fn selection_center(points: &[Point]) -> Point {
    if points.is_empty() {
        return Point::origin();
    }
    let sum_x: f64 = points.iter().map(|p| p.x).sum();
    let sum_y: f64 = points.iter().map(|p| p.y).sum();
    let count = points.len() as f64;
    Point::new(sum_x / count, sum_y / count)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::geometry::primitives::{Point, Rectangle};
    use std::f64;

    #[test]
    fn test_safe_bounds_valid() {
        let bounds = safe_bounds(10.0, 20.0, 30.0, 40.0).unwrap();
        assert_eq!(bounds.min_x, 10.0);
        assert_eq!(bounds.min_y, 20.0);
        assert_eq!(bounds.max_x, 30.0);
        assert_eq!(bounds.max_y, 40.0);
    }

    #[test]
    fn test_safe_bounds_inverted() {
        let bounds = safe_bounds(30.0, 40.0, 10.0, 20.0).unwrap();
        assert_eq!(bounds.min_x, 10.0);
        assert_eq!(bounds.min_y, 20.0);
        assert_eq!(bounds.max_x, 30.0);
        assert_eq!(bounds.max_y, 40.0);
    }

    #[test]
    fn test_safe_bounds_invalid() {
        assert!(safe_bounds(f64::NAN, 0.0, 10.0, 10.0).is_err());
        assert!(safe_bounds(0.0, f64::INFINITY, 10.0, 10.0).is_err());
        assert!(safe_bounds(0.0, 0.0, f64::NEG_INFINITY, 10.0).is_err());
    }

    #[test]
    fn test_zoom_at_pointer() {
        let center = Point::new(100.0, 100.0);
        let pointer = Point::new(50.0, 50.0);
        let zoom = zoom_at_pointer(center, pointer, 2.0);
        assert_eq!(zoom.x, 150.0);
        assert_eq!(zoom.y, 150.0);
    }

    #[test]
    fn test_snap_horizontal() {
        let targets = vec![10.0, 20.0, 30.0];
        assert_eq!(snap_horizontal(14.0, &targets, 5.0), Some(10.0));
        assert_eq!(snap_horizontal(16.0, &targets, 5.0), Some(20.0));
        assert_eq!(snap_horizontal(25.0, &targets, 1.0), None);
    }

    #[test]
    fn test_snap_vertical() {
        let targets = vec![10.0, 20.0, 30.0];
        assert_eq!(snap_vertical(14.0, &targets, 5.0), Some(10.0));
    }

    #[test]
    fn test_hit_test_rect() {
        let rect = Rectangle::new(10.0, 10.0, 20.0, 20.0);
        let center = rect.aabb().center();
        assert!(hit_test_rect(center, &rect, 0.0));
        assert!(hit_test_rect(
            Point::new(rect.aabb().max_x + 1.0, center.y),
            &rect,
            2.0
        )); // within margin
        assert!(!hit_test_rect(
            Point::new(rect.aabb().max_x + 3.0, center.y),
            &rect,
            2.0
        )); // outside margin
    }

    #[test]
    fn test_hit_test_rotated_rect() {
        let mut rect = Rectangle::new(10.0, 10.0, 20.0, 20.0);
        let center = rect.aabb().center();
        assert!(hit_test_rotated_rect(center, &rect));

        rect.rotation = std::f64::consts::PI / 4.0; // 45 degrees
        assert!(hit_test_rotated_rect(center, &rect));
    }

    #[test]
    fn test_world_to_screen_and_back() {
        let world = Point::new(100.0, 100.0);
        let camera = Point::new(50.0, 50.0);
        let zoom = 2.0;

        let screen = world_to_screen(world, camera, zoom);
        let world_back = screen_to_world(screen, camera, zoom);

        // Allow some float imprecision
        assert!((world.x - world_back.x).abs() < 1e-6);
        assert!((world.y - world_back.y).abs() < 1e-6);
    }

    #[test]
    fn test_selection_center() {
        let points = vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0)];
        let center = selection_center(&points);
        assert_eq!(center.x, 5.0);
        assert_eq!(center.y, 5.0);

        let empty: Vec<Point> = vec![];
        let center_empty = selection_center(&empty);
        assert_eq!(center_empty.x, 0.0);
        assert_eq!(center_empty.y, 0.0);
    }
}
