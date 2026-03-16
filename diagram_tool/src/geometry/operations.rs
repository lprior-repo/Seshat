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

// ============== GEO-005: Edge Bounds for Curved Connectors ==============

/// Arrow type for edge rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeArrowType {
    #[default]
    Default,
    Sharp,
    Curved,
    Step,
    Straight,
}

/// Error type for edge bounds calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EdgeBoundsError {
    #[error("Invalid node position: coordinates must be finite")]
    InvalidNodePosition,
    #[error("Invalid thickness: must be positive and finite")]
    InvalidThickness,
}

/// Validates that a point has finite coordinates
const fn validate_point(point: &Point) -> Result<(), EdgeBoundsError> {
    if point.x.is_nan() || point.x.is_infinite() || point.y.is_nan() || point.y.is_infinite() {
        return Err(EdgeBoundsError::InvalidNodePosition);
    }
    Ok(())
}

/// Validates that thickness is positive and finite
fn validate_thickness(thickness: f64) -> Result<(), EdgeBoundsError> {
    if thickness.is_nan() || thickness <= 0.0 || thickness.is_infinite() {
        return Err(EdgeBoundsError::InvalidThickness);
    }
    Ok(())
}

/// Calculate bounds for a straight line segment with stroke width
fn line_bounds(start: Point, end: Point, stroke_width: f64) -> AABB {
    let half_stroke = stroke_width / 2.0;
    AABB::new(
        start.x.min(end.x) - half_stroke,
        start.y.min(end.y) - half_stroke,
        start.x.max(end.x) + half_stroke,
        start.y.max(end.y) + half_stroke,
    )
}

/// Calculate the control point for a curved (Bezier) edge
fn bezier_control_point(source: Point, target: Point) -> Point {
    let mid_x = f64::midpoint(source.x, target.x);
    let mid_y = f64::midpoint(source.y, target.y);
    // Offset perpendicular to the line
    let dx = target.x - source.x;
    let dy = target.y - source.y;
    let dist = dx.hypot(dy);
    if dist < 1e-10 {
        // Points are coincident, use offset
        return Point::new(mid_x + 30.0, mid_y + 30.0);
    }
    // Perpendicular offset (rotate 90 degrees)
    let offset = 0.3 * dist;
    Point::new(
        (dy / dist).mul_add(-offset, mid_x),
        (dx / dist).mul_add(offset, mid_y),
    )
}

/// Calculate tight bounds for a quadratic Bezier curve
/// Uses derivative analysis to find exact extrema
fn quadratic_bezier_tight_bounds(
    start: Point,
    control: Point,
    end: Point,
    stroke_width: f64,
) -> AABB {
    let tolerance = 1e-10;

    // Start with endpoints
    let mut min_x = start.x.min(end.x);
    let mut max_x = start.x.max(end.x);
    let mut min_y = start.y.min(end.y);
    let mut max_y = start.y.max(end.y);

    // Check x extrema using derivative
    let denom_x = 2.0f64.mul_add(-control.x, start.x) + end.x;
    if denom_x.abs() > tolerance {
        let t = (start.x - control.x) / denom_x;
        if (0.0..=1.0).contains(&t) {
            let t2 = t * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let px = t2.mul_add(end.x, mt2 * start.x + 2.0 * mt * t * control.x);
            min_x = min_x.min(px);
            max_x = max_x.max(px);
        }
    }

    // Check y extrema using derivative
    let denom_y = 2.0f64.mul_add(-control.y, start.y) + end.y;
    if denom_y.abs() > tolerance {
        let t = (start.y - control.y) / denom_y;
        if (0.0..=1.0).contains(&t) {
            let t2 = t * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let py = t2.mul_add(end.y, mt2 * start.y + 2.0 * mt * t * control.y);
            min_y = min_y.min(py);
            max_y = max_y.max(py);
        }
    }

    let half_stroke = stroke_width / 2.0;
    AABB::new(
        min_x - half_stroke,
        min_y - half_stroke,
        max_x + half_stroke,
        max_y + half_stroke,
    )
}

/// Calculate bounds for an edge including Bezier curve extents
///
/// # Errors
/// Returns an error if source/target coordinates are NaN/infinite, or thickness is invalid.
pub fn edge_bounds(
    source: Point,
    target: Point,
    arrow_type: EdgeArrowType,
    thickness: f64,
    bend_points: &[Point],
) -> Result<AABB, EdgeBoundsError> {
    // Validate inputs
    validate_point(&source)?;
    validate_point(&target)?;
    validate_thickness(thickness)?;

    // Build the path points: source -> bend_points -> target
    let mut all_points = vec![source];
    all_points.extend(bend_points.iter().copied());
    all_points.push(target);

    // Calculate bounds for each segment
    let mut bounds = AABB::new(
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );

    for window in all_points.windows(2) {
        let segment_bounds = match arrow_type {
            EdgeArrowType::Curved => {
                // For curved edges, use quadratic Bezier
                let control = bezier_control_point(window[0], window[1]);
                quadratic_bezier_tight_bounds(window[0], control, window[1], thickness)
            }
            EdgeArrowType::Step
            | EdgeArrowType::Default
            | EdgeArrowType::Sharp
            | EdgeArrowType::Straight => {
                // For non-curved edges, use simple line bounds
                line_bounds(window[0], window[1], thickness)
            }
        };
        bounds = bounds.union(&segment_bounds);
    }

    // Add arrowhead extension for directed edges
    // Arrowhead extends backward from target
    let arrowhead_size = thickness * 4.0;
    bounds = AABB::new(
        bounds.min_x,
        bounds.min_y,
        bounds.max_x + arrowhead_size,
        bounds.max_y,
    );

    Ok(bounds)
}
