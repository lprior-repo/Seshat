use crate::geometry::primitives::{Point, AABB};
use smallvec::SmallVec;

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

fn bezier_extrema(p0: f64, p1: f64, p2: f64) -> (f64, f64) {
    let mut min_val = p0.min(p2);
    let mut max_val = p0.max(p2);
    let denom = 2.0f64.mul_add(-p1, p0) + p2;

    if denom.abs() > 1e-10 {
        let t = (p0 - p1) / denom;
        if (0.0..=1.0).contains(&t) {
            let t2 = t * t;
            let mt = 1.0 - t;
            let px = t2.mul_add(p2, mt * mt * p0 + 2.0 * mt * t * p1);
            min_val = min_val.min(px);
            max_val = max_val.max(px);
        }
    }
    (min_val, max_val)
}

fn compute_bezier_extrema(start: &Point, control: &Point, end: &Point) -> (f64, f64, f64, f64) {
    let (min_x, max_x) = bezier_extrema(start.x, control.x, end.x);
    let (min_y, max_y) = bezier_extrema(start.y, control.y, end.y);
    (min_x, max_x, min_y, max_y)
}

/// Calculate tight bounds for a quadratic Bezier curve
/// Uses derivative analysis to find exact extrema
fn quadratic_bezier_tight_bounds(
    start: Point,
    control: Point,
    end: Point,
    stroke_width: f64,
) -> AABB {
    let (min_x, max_x, min_y, max_y) = compute_bezier_extrema(&start, &control, &end);

    let half_stroke = stroke_width / 2.0;
    AABB::new(
        min_x - half_stroke,
        min_y - half_stroke,
        max_x + half_stroke,
        max_y + half_stroke,
    )
}

fn validate_edge_inputs(
    source: &Point,
    target: &Point,
    thickness: f64,
    bend_points: &[Point],
) -> Result<(), EdgeBoundsError> {
    validate_point(source)?;
    validate_point(target)?;
    validate_thickness(thickness)?;
    for point in bend_points {
        validate_point(point)?;
    }
    Ok(())
}

fn calculate_segment_bounds(
    p1: Point,
    p2: Point,
    arrow_type: EdgeArrowType,
    thickness: f64,
) -> AABB {
    if arrow_type == EdgeArrowType::Curved {
        let control = bezier_control_point(p1, p2);
        quadratic_bezier_tight_bounds(p1, control, p2, thickness)
    } else {
        line_bounds(p1, p2, thickness)
    }
}

fn validate_edge_bounds(bounds: AABB, thickness: f64) -> Result<AABB, EdgeBoundsError> {
    let new_max_x = bounds.max_x + thickness * 4.0;
    if !bounds.min_x.is_finite()
        || !bounds.min_y.is_finite()
        || !new_max_x.is_finite()
        || !bounds.max_y.is_finite()
    {
        return Err(EdgeBoundsError::InvalidNodePosition);
    }
    Ok(AABB::new(
        bounds.min_x,
        bounds.min_y,
        new_max_x,
        bounds.max_y,
    ))
}

fn build_all_points(source: Point, target: Point, bend_points: &[Point]) -> SmallVec<[Point; 4]> {
    let mut all_points = SmallVec::new();
    all_points.push(source);
    all_points.extend(bend_points.iter().copied());
    all_points.push(target);
    all_points
}

fn reduce_segment_bounds(points: &[Point], arrow_type: EdgeArrowType, thickness: f64) -> AABB {
    points
        .windows(2)
        .map(|w| calculate_segment_bounds(w[0], w[1], arrow_type, thickness))
        .reduce(|a, b| a.union(&b))
        .unwrap_or_else(|| AABB::new(0.0, 0.0, 0.0, 0.0))
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
    validate_edge_inputs(&source, &target, thickness, bend_points)?;
    let all_points = build_all_points(source, target, bend_points);
    let raw_bounds = reduce_segment_bounds(&all_points, arrow_type, thickness);
    validate_edge_bounds(raw_bounds, thickness)
}
