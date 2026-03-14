use crate::geometry::primitives::{Point, AABB};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RoutingError {
    #[error("Invalid endpoint: NaN or Infinity")]
    InvalidEndpoint,
    #[error("Degenerate route: start and end points are identical")]
    DegenerateRoute,
    #[error("Endpoint inside obstacle")]
    EndpointInsideObstacle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrthogonalRoute {
    pub points: Vec<Point>,
}

const TOLERANCE: f64 = 1e-10;

#[must_use]
pub fn is_orthogonal(route: &OrthogonalRoute) -> bool {
    route
        .points
        .windows(2)
        .all(|w| (w[0].x - w[1].x).abs() < TOLERANCE || (w[0].y - w[1].y).abs() < TOLERANCE)
}

#[must_use]
pub fn route_intersects(route: &OrthogonalRoute, obstacle: &AABB) -> bool {
    route
        .points
        .windows(2)
        .any(|w| segment_intersects_aabb(w[0], w[1], obstacle))
}

fn segment_intersects_aabb(p1: Point, p2: Point, aabb: &AABB) -> bool {
    if (p1.y - p2.y).abs() < TOLERANCE {
        let min_x = p1.x.min(p2.x);
        let max_x = p1.x.max(p2.x);
        p1.y > aabb.min_y && p1.y < aabb.max_y && max_x > aabb.min_x && min_x < aabb.max_x
    } else if (p1.x - p2.x).abs() < TOLERANCE {
        let min_y = p1.y.min(p2.y);
        let max_y = p1.y.max(p2.y);
        p1.x > aabb.min_x && p1.x < aabb.max_x && max_y > aabb.min_y && min_y < aabb.max_y
    } else {
        false
    }
}

fn validate_endpoints(from: Point, to: Point) -> Result<(), RoutingError> {
    if !from.x.is_finite() || !from.y.is_finite() || !to.x.is_finite() || !to.y.is_finite() {
        return Err(RoutingError::InvalidEndpoint);
    }
    if (from.x - to.x).abs() < TOLERANCE && (from.y - to.y).abs() < TOLERANCE {
        return Err(RoutingError::DegenerateRoute);
    }
    Ok(())
}

/// Computes an orthogonal route between two points.
///
/// # Errors
/// Returns `RoutingError::InvalidEndpoint` if any coordinate is not finite.
/// Returns `RoutingError::DegenerateRoute` if start and end are identical.
pub fn compute_orthogonal_route(from: Point, to: Point) -> Result<OrthogonalRoute, RoutingError> {
    validate_endpoints(from, to)?;

    if (from.x - to.x).abs() < TOLERANCE || (from.y - to.y).abs() < TOLERANCE {
        let route = OrthogonalRoute {
            points: vec![from, to],
        };
        debug_assert!(is_orthogonal(&route));
        return Ok(route);
    }

    let mid = if from.x < to.x {
        Point::new(from.x, to.y)
    } else {
        Point::new(to.x, from.y)
    };
    let route = OrthogonalRoute {
        points: vec![from, mid, to],
    };
    debug_assert!(is_orthogonal(&route));
    Ok(route)
}

fn is_inside(p: Point, aabb: &AABB) -> bool {
    p.x > aabb.min_x && p.x < aabb.max_x && p.y > aabb.min_y && p.y < aabb.max_y
}

fn calc_detour_y(from_y: f64, to_y: f64, obstacle: &AABB) -> f64 {
    let mid_y = from_y.midpoint(to_y);
    let go_above = mid_y < obstacle.center().y;
    if go_above {
        obstacle.min_y - 10.0
    } else {
        obstacle.max_y + 10.0
    }
}

fn calc_detour_x_bounds(obstacle: &AABB, from_x: f64, to_x: f64) -> (f64, f64) {
    let mid_x1 = obstacle.min_x - 10.0;
    let mid_x2 = obstacle.max_x + 10.0;
    if from_x < to_x {
        (mid_x1, mid_x2)
    } else {
        (mid_x2, mid_x1)
    }
}

fn build_detour_route(from: Point, to: Point, obstacle: &AABB) -> OrthogonalRoute {
    let detour_y = calc_detour_y(from.y, to.y, obstacle);
    let (dx1, dx2) = calc_detour_x_bounds(obstacle, from.x, to.x);

    let route = OrthogonalRoute {
        points: vec![
            from,
            Point::new(dx1, from.y),
            Point::new(dx1, detour_y),
            Point::new(dx2, detour_y),
            Point::new(dx2, to.y),
            to,
        ],
    };
    debug_assert!(is_orthogonal(&route));
    debug_assert!(!route_intersects(&route, obstacle));
    route
}

/// Computes an orthogonal route avoiding an obstacle.
///
/// # Errors
/// Returns errors from `compute_orthogonal_route`.
/// Returns `RoutingError::EndpointInsideObstacle` if endpoints are strictly inside the obstacle.
pub fn compute_orthogonal_route_avoiding(
    from: Point,
    to: Point,
    obstacle: &AABB,
) -> Result<OrthogonalRoute, RoutingError> {
    if is_inside(from, obstacle) || is_inside(to, obstacle) {
        return Err(RoutingError::EndpointInsideObstacle);
    }

    let direct = compute_orthogonal_route(from, to)?;
    if !route_intersects(&direct, obstacle) {
        return Ok(direct);
    }

    Ok(build_detour_route(from, to, obstacle))
}

// Legacy wrapper for protected tests (GEO-017) and EDG-031
#[must_use]
pub fn orthogonal_route(from: Point, to: Point) -> OrthogonalRoute {
    compute_orthogonal_route(from, to)
        .map_or_else(|_| OrthogonalRoute { points: vec![] }, |route| route)
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    fn any_valid_f64() -> f64 {
        let v: f64 = kani::any();
        kani::assume(v.is_finite());
        // Constrain magnitude to prevent overflow when adding 10.0 or computing midpoints
        kani::assume(v > -1e150 && v < 1e150);
        v
    }

    fn any_point() -> Point {
        Point::new(any_valid_f64(), any_valid_f64())
    }

    fn any_aabb() -> AABB {
        let min_x = any_valid_f64();
        let max_x = any_valid_f64();
        let min_y = any_valid_f64();
        let max_y = any_valid_f64();
        kani::assume(min_x <= max_x);
        kani::assume(min_y <= max_y);
        AABB::new(min_x, min_y, max_x, max_y)
    }

    #[kani::proof]
    fn verify_compute_orthogonal_route_invariants() {
        let p1 = any_point();
        let p2 = any_point();

        if let Ok(route) = compute_orthogonal_route(p1, p2) {
            // Invariant 1: Route must be orthogonal
            assert!(is_orthogonal(&route));

            // Invariant 2: Endpoints match inputs exactly
            let first = route.points.first().unwrap();
            let last = route.points.last().unwrap();
            assert_eq!(first.x, p1.x);
            assert_eq!(first.y, p1.y);
            assert_eq!(last.x, p2.x);
            assert_eq!(last.y, p2.y);

            // Invariant 3: Routes have either 2 or 3 points
            assert!(route.points.len() == 2 || route.points.len() == 3);
        }
    }

    #[kani::proof]
    fn verify_compute_orthogonal_route_avoiding_invariants() {
        let p1 = any_point();
        let p2 = any_point();
        let obstacle = any_aabb();

        if let Ok(route) = compute_orthogonal_route_avoiding(p1, p2, &obstacle) {
            // Invariant 1: Route must be orthogonal
            assert!(is_orthogonal(&route));

            // Invariant 2: Endpoints match inputs exactly
            let first = route.points.first().unwrap();
            let last = route.points.last().unwrap();
            assert_eq!(first.x, p1.x);
            assert_eq!(first.y, p1.y);
            assert_eq!(last.x, p2.x);
            assert_eq!(last.y, p2.y);

            // Invariant 3: It does not intersect the obstacle strictly
            assert!(!route_intersects(&route, &obstacle));
        }
    }

    #[kani::proof]
    fn verify_orthogonal_route_wrapper() {
        let p1 = any_point();
        let p2 = any_point();
        let route = orthogonal_route(p1, p2);

        if route.points.is_empty() {
            // Error case handling
            assert!(compute_orthogonal_route(p1, p2).is_err());
        } else {
            assert!(is_orthogonal(&route));
        }
    }
}
