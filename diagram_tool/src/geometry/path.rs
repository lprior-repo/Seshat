use super::*;

// ============== GEO-027: Path Simplification (Ramer-Douglas-Peucker) ==============

/// Error types for path operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    /// Not enough points to form a valid path
    InsufficientPoints,
    /// Invalid point coordinate (NaN or Infinity)
    InvalidPoint,
    /// Self-intersection detected in simplified path
    SelfIntersection,
    /// Invalid epsilon value
    InvalidEpsilon,
}

impl core::fmt::Display for PathError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InsufficientPoints => write!(f, "Path has insufficient points"),
            Self::InvalidPoint => write!(f, "Path contains invalid point (NaN/Infinity)"),
            Self::SelfIntersection => write!(f, "Path has self-intersection"),
            Self::InvalidEpsilon => write!(f, "Epsilon must be non-negative"),
        }
    }
}

/// Configuration for path simplification
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathSimplificationConfig {
    /// Maximum distance from line for points to be considered on the line
    pub epsilon: f64,
    /// Minimum number of points required for a valid path
    pub min_points: usize,
}

impl PathSimplificationConfig {
    #[must_use]
    pub const fn new(epsilon: f64, min_points: usize) -> Option<Self> {
        if epsilon < 0.0 || min_points < 2 {
            return None;
        }
        Some(Self {
            epsilon,
            min_points,
        })
    }

    /// Default configuration with epsilon=1.0 and `min_points=2`
    #[must_use]
    pub const fn default_config() -> Self {
        Self {
            epsilon: 1.0,
            min_points: 2,
        }
    }
}

/// Calculate the perpendicular distance from a point to a line defined by two endpoints
fn point_to_line_distance(point: Point, line_start: Point, line_end: Point) -> f64 {
    // Handle degenerate case where line start equals end
    let dx = line_end.x - line_start.x;
    let dy = line_end.y - line_start.y;
    let line_length_sq = dx.mul_add(dx, dy * dy);

    if line_length_sq < f64::EPSILON {
        // Line is a point, return distance to that point
        let px = point.x - line_start.x;
        let py = point.y - line_start.y;
        return px.hypot(py);
    }

    // Calculate perpendicular distance using cross product formula
    // Distance = |(P - A) x (B - A)| |B - A|
    let px = point.x - line_start.x;
    let py = point.y - line_start.y;
    let cross = px.mul_add(dy, -(py * dx));
    cross.abs() / line_length_sq.sqrt()
}

/// Recursive Ramer-Douglas-Peucker simplification
#[allow(non_snake_case)]
fn rdp_simplifyRecursive(points: &[Point], epsilon: f64) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }

    // Find point with maximum distance from line between first and last
    let mut max_distance = 0.0;
    let mut max_index = 0;

    for i in 1..points.len() - 1 {
        let distance = point_to_line_distance(points[i], points[0], points[points.len() - 1]);
        if distance > max_distance {
            max_distance = distance;
            max_index = i;
        }
    }

    // If max distance is greater than epsilon, recursively simplify
    if max_distance > epsilon {
        // Recursively simplify left and right parts
        let left = rdp_simplifyRecursive(&points[..=max_index], epsilon);
        let right = rdp_simplifyRecursive(&points[max_index..], epsilon);

        // Combine results (avoid duplicate point at max_index)
        let mut result = left;
        result.pop();
        result.extend(right);
        result
    } else {
        // All intermediate points are within epsilon, keep only endpoints
        vec![points[0], points[points.len() - 1]]
    }
}

/// Validate that a path has no self-intersections
/// Returns true if path is simple (no self-intersections), false if it has intersections
fn is_path_simple(points: &[Point]) -> bool {
    if points.len() < 4 {
        return true;
    }

    // Check each pair of non-adjacent segments for intersection
    for i in 0..points.len() - 1 {
        for j in i + 2..points.len() - 1 {
            // Skip adjacent segments (they share an endpoint)
            if j == i + 1 {
                continue;
            }

            if segments_intersect(points[i], points[i + 1], points[j], points[j + 1]) {
                return false;
            }
        }
    }
    true
}

/// Check if two line segments intersect
fn segments_intersect(p1: Point, p2: Point, p3: Point, p4: Point) -> bool {
    // Use orientation test
    let d1 = orientation(p3, p4, p1);
    let d2 = orientation(p3, p4, p2);
    let d3 = orientation(p1, p2, p3);
    let d4 = orientation(p1, p2, p4);

    // General case: segments intersect if orientations have different signs
    if d1 != 0.0 && d2 != 0.0 && d3 != 0.0 && d4 != 0.0 {
        return (d1 > 0.0) != (d2 > 0.0) && (d3 > 0.0) != (d4 > 0.0);
    }

    // Special cases for collinear points
    false
}

/// Calculate orientation of three points
/// Returns positive if counter-clockwise, negative if clockwise, 0 if collinear
fn orientation(p1: Point, p2: Point, p3: Point) -> f64 {
    (p2.y - p1.y).mul_add(p3.x - p2.x, -((p2.x - p1.x) * (p3.y - p2.y)))
}

/// Validate a point is valid (not NaN or Infinity)
const fn is_valid_point(point: &Point) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

/// Simplify a path using the Ramer-Douglas-Peucker algorithm
///
/// # Errors
/// Returns `PathError::InsufficientPoints` if the path has fewer than `min_points`
/// Returns `PathError::InvalidPoint` if any point is NaN or Infinity
/// Returns `PathError::SelfIntersection` if the simplified path has self-intersections
/// Returns `PathError::InvalidEpsilon` if epsilon is negative
pub fn simplify_path(
    points: &[Point],
    config: PathSimplificationConfig,
) -> Result<Vec<Point>, PathError> {
    // Validate epsilon
    if config.epsilon < 0.0 {
        return Err(PathError::InvalidEpsilon);
    }

    // Validate minimum points
    if points.len() < config.min_points {
        return Err(PathError::InsufficientPoints);
    }

    // Validate all points are finite
    if !points.iter().all(is_valid_point) {
        return Err(PathError::InvalidPoint);
    }

    // If epsilon is 0 or we have exactly min_points, return as-is
    if config.epsilon == 0.0 || points.len() == config.min_points {
        return Ok(points.to_vec());
    }

    // Apply RDP algorithm
    let simplified = rdp_simplifyRecursive(points, config.epsilon);

    // Validate result has at least min_points
    if simplified.len() < config.min_points {
        return Err(PathError::InsufficientPoints);
    }

    // Check for self-intersection in simplified path
    if !is_path_simple(&simplified) {
        // Try with higher epsilon or return error
        // For now, return error as per GEO-027 requirement
        return Err(PathError::SelfIntersection);
    }

    Ok(simplified)
}

/// Simplified function with default configuration
#[must_use]
pub fn simplify_path_default(points: &[Point]) -> Option<Vec<Point>> {
    simplify_path(points, PathSimplificationConfig::default_config()).ok()
}

