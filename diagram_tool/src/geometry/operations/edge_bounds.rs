use crate::geometry::primitives::{Point, AABB};

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
    for point in bend_points {
        validate_point(point)?;
    }

    // Build the path points: source -> bend_points -> target
    let mut all_points = vec![source];
    all_points.extend(bend_points.iter().copied());
    all_points.push(target);

    // Calculate bounds for each segment
    let mut bounds: Option<AABB> = None;

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

        if let Some(b) = bounds {
            bounds = Some(b.union(&segment_bounds));
        } else {
            bounds = Some(segment_bounds);
        }
    }

    let mut bounds = bounds.unwrap_or_else(|| AABB::new(0.0, 0.0, 0.0, 0.0));

    // Add arrowhead extension for directed edges
    // Arrowhead extends backward from target
    let arrowhead_size = thickness * 4.0;

    let new_max_x = bounds.max_x + arrowhead_size;
    let new_min_x = bounds.min_x;
    let new_min_y = bounds.min_y;
    let new_max_y = bounds.max_y;

    if !new_min_x.is_finite()
        || !new_min_y.is_finite()
        || !new_max_x.is_finite()
        || !new_max_y.is_finite()
    {
        return Err(EdgeBoundsError::InvalidNodePosition);
    }

    bounds = AABB::new(new_min_x, new_min_y, new_max_x, new_max_y);

    Ok(bounds)
}
