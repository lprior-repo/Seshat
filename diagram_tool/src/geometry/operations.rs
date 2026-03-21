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
