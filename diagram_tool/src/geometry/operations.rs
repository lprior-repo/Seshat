use crate::geometry::primitives::{Point, Rectangle, AABB};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BoundsError {
    #[error("Invalid coordinate: NaN or Infinity")]
    InvalidCoordinate,
}

/// Creates a bounding box safely.
///
/// # Errors
/// Returns an error if any coordinate is NaN or infinite.
pub fn safe_bounds(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<AABB, BoundsError> {
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
    Point::new((world.x - camera.x) * zoom, (world.y - camera.y) * zoom)
}

#[must_use]
pub fn screen_to_world(screen: Point, camera: Point, zoom: f64) -> Point {
    Point::new(screen.x / zoom + camera.x, screen.y / zoom + camera.y)
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

/// Computes the bounding box of a container based on its children's bounds.
///
/// # Parameters
/// - `children`: An iterator of (x, y, width, height) tuples representing child nodes
///
/// # Returns
/// - `Some((x, y, width, height))`: The computed bounds if children exist and are valid
/// - `None`: If there are no children or all children have invalid (NaN/Infinity) coordinates
///
/// # Contract (KIRK-001)
/// - Returns bounds that encompass ALL children geometrically
/// - Returns None if children list is empty
/// - Bounds are minimal (tight fit to children)
/// - All coordinate values are finite (not NaN/Infinity)
#[must_use]
pub fn compute_subgraph_bounds(
    children: impl IntoIterator<Item = (f64, f64, f64, f64)>,
) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut has_valid_child = false;

    for (x, y, width, height) in children {
        // Skip invalid child bounds (NaN or Infinity)
        if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
            continue;
        }

        has_valid_child = true;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + width);
        max_y = max_y.max(y + height);
    }

    if !has_valid_child {
        return None;
    }

    // Verify final bounds are valid
    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }

    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}
