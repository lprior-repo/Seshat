use crate::geometry::primitives::{Point, Rectangle, AABB};

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum BoundsError {
    #[error("Invalid coordinate: NaN or Infinity")]
    InvalidCoordinate,
    #[error("Invalid bounds: min ({min_x}, {min_y}) > max ({max_x}, {max_y})")]
    InvalidBounds {
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    },
    #[error("Negative expansion amount: {0} (must be >= 0)")]
    NegativeExpansion(f64),
}

/// Creates a bounding box safely.
///
/// # Errors
/// Returns an error if any coordinate is NaN or infinite, or if min > max.
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

    // Check for invalid bounds (min > max)
    if min_x > max_x {
        return Err(BoundsError::InvalidBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        });
    }
    if min_y > max_y {
        return Err(BoundsError::InvalidBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        });
    }

    Ok(AABB::new(min_x, min_y, max_x, max_y))
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

#[derive(Debug, Clone, PartialEq)]
pub struct OrthogonalRoute {
    pub points: Vec<Point>,
}

#[must_use]
pub fn orthogonal_route(from: Point, to: Point) -> OrthogonalRoute {
    let tolerance = 1e-10;
    if (from.x - to.x).abs() < tolerance || (from.y - to.y).abs() < tolerance {
        OrthogonalRoute {
            points: vec![from, to],
        }
    } else {
        let mid = Point::new(to.x, from.y);
        OrthogonalRoute {
            points: vec![from, mid, to],
        }
    }
}

#[must_use]
pub fn segment_intersects_aabb(p1: Point, p2: Point, aabb: &AABB) -> bool {
    let tolerance = 1e-10;
    if (p1.y - p2.y).abs() < tolerance {
        let min_x = p1.x.min(p2.x);
        let max_x = p1.x.max(p2.x);
        let y = p1.y;
        y >= aabb.min_y && y <= aabb.max_y && max_x >= aabb.min_x && min_x <= aabb.max_x
    } else if (p1.x - p2.x).abs() < tolerance {
        let x = p1.x;
        let min_y = p1.y.min(p2.y);
        let max_y = p1.y.max(p2.y);
        x >= aabb.min_x && x <= aabb.max_x && max_y >= aabb.min_y && min_y <= aabb.max_y
    } else {
        false
    }
}

#[must_use]
pub fn orthogonal_route_avoiding(from: Point, to: Point, obstacle: &AABB) -> OrthogonalRoute {
    let direct = orthogonal_route(from, to);

    if !direct
        .points
        .windows(2)
        .any(|seg| segment_intersects_aabb(seg[0], seg[1], obstacle))
    {
        return direct;
    }

    let detour_y = if from.y < obstacle.max_y && to.y < obstacle.max_y {
        obstacle.min_y - 10.0
    } else {
        obstacle.max_y + 10.0
    };

    OrthogonalRoute {
        points: vec![
            from,
            Point::new(obstacle.min_x - 10.0, from.y),
            Point::new(obstacle.min_x - 10.0, detour_y),
            Point::new(obstacle.max_x + 10.0, detour_y),
            Point::new(obstacle.max_x + 10.0, to.y),
            to,
        ],
    }
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

#[cfg(test)]
mod subgraph_bounds_tests {
    use super::*;

    // ============== GEO-025: Container Bounds Recomputation ==============

    #[test]
    fn test_single_child_container() {
        // Given: a subgraph container with one child
        let children = vec![(100.0, 100.0, 50.0, 30.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should be the child's bounds
        assert_eq!(result, Some((100.0, 100.0, 50.0, 30.0)));
    }

    #[test]
    fn test_multiple_children_horizontal_spread() {
        // Given: children spread horizontally
        let children = vec![
            (0.0, 0.0, 50.0, 50.0),
            (100.0, 0.0, 50.0, 50.0),
            (200.0, 0.0, 50.0, 50.0),
        ];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should encompass all children
        assert_eq!(result, Some((0.0, 0.0, 250.0, 50.0)));
    }

    #[test]
    fn test_multiple_children_vertical_spread() {
        // Given: children spread vertically
        let children = vec![
            (0.0, 0.0, 50.0, 50.0),
            (0.0, 100.0, 50.0, 50.0),
            (0.0, 200.0, 50.0, 50.0),
        ];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should encompass all children
        assert_eq!(result, Some((0.0, 0.0, 50.0, 250.0)));
    }

    #[test]
    fn test_empty_container() {
        // Given: no children
        let children: Vec<(f64, f64, f64, f64)> = vec![];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should be None
        assert_eq!(result, None);
    }

    #[test]
    fn test_child_with_negative_coordinates() {
        // Given: children with negative coordinates
        let children = vec![(-50.0, -50.0, 50.0, 50.0), (0.0, 0.0, 50.0, 50.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should handle negative coords
        assert_eq!(result, Some((-50.0, -50.0, 100.0, 100.0)));
    }

    #[test]
    fn test_child_overlap() {
        // Given: overlapping children
        let children = vec![(0.0, 0.0, 100.0, 100.0), (50.0, 50.0, 100.0, 100.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should be the union
        assert_eq!(result, Some((0.0, 0.0, 150.0, 150.0)));
    }

    #[test]
    fn test_invalid_child_nan() {
        // Given: one valid child and one with NaN coordinates
        let children = vec![(0.0, 0.0, 50.0, 50.0), (f64::NAN, 0.0, 50.0, 50.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should contain only valid child
        assert_eq!(result, Some((0.0, 0.0, 50.0, 50.0)));
    }

    #[test]
    fn test_invalid_child_infinity() {
        // Given: one valid child and one with Infinity coordinates
        let children = vec![(0.0, 0.0, 50.0, 50.0), (f64::INFINITY, 0.0, 50.0, 50.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should contain only valid child
        assert_eq!(result, Some((0.0, 0.0, 50.0, 50.0)));
    }

    #[test]
    fn test_single_point_child() {
        // Given: child with zero size
        let children = vec![(50.0, 50.0, 0.0, 0.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should be degenerate but valid
        assert_eq!(result, Some((50.0, 50.0, 0.0, 0.0)));
    }

    #[test]
    fn test_all_children_at_same_position() {
        // Given: multiple children at same position with different sizes
        let children = vec![
            (50.0, 50.0, 30.0, 30.0),
            (50.0, 50.0, 50.0, 50.0),
            (50.0, 50.0, 20.0, 40.0),
        ];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should be the union
        assert_eq!(result, Some((50.0, 50.0, 50.0, 50.0)));
    }
}
