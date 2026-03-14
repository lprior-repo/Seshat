use crate::geometry::primitives::{Point, AABB};
use crate::geometry::routing::*;

// ============== Martin Fowler Test Plan: EDG-006 to EDG-010 ==============

// --------------------------------------------------------------------------
// Happy Path Tests
// --------------------------------------------------------------------------

#[test]
fn test_returns_success_when_points_are_vertically_aligned() {
    let from = Point::new(50.0, 10.0);
    let to = Point::new(50.0, 100.0);

    let route = compute_orthogonal_route(from, to).expect("Should compute route");

    assert_eq!(route.points.len(), 2);
    assert!(is_orthogonal(&route));
}

#[test]
fn test_returns_success_when_points_are_horizontally_aligned() {
    let from = Point::new(10.0, 50.0);
    let to = Point::new(100.0, 50.0);

    let route = compute_orthogonal_route(from, to).expect("Should compute route");

    assert_eq!(route.points.len(), 2);
    assert!(is_orthogonal(&route));
}

#[test]
fn test_returns_success_l_shape_when_points_are_diagonal() {
    let from = Point::new(0.0, 0.0);
    let to = Point::new(100.0, 50.0);

    let route = compute_orthogonal_route(from, to).expect("Should compute route");

    assert_eq!(route.points.len(), 3);
    assert!(is_orthogonal(&route));
}

#[test]
fn test_returns_success_detour_when_avoiding_obstacle() {
    let from = Point::new(0.0, 25.0);
    let to = Point::new(150.0, 25.0);
    let obstacle = AABB::new(50.0, 0.0, 100.0, 50.0);

    let route =
        compute_orthogonal_route_avoiding(from, to, &obstacle).expect("Should compute route");

    assert!(route.points.len() > 2);
    assert!(is_orthogonal(&route));
    assert!(!route_intersects(&route, &obstacle));
}

#[test]
fn test_returns_symmetric_route_when_endpoints_are_swapped() {
    let from = Point::new(0.0, 0.0);
    let to = Point::new(100.0, 50.0);

    let route_ab = compute_orthogonal_route(from, to).expect("Should compute route");
    let route_ba = compute_orthogonal_route(to, from).expect("Should compute route");

    let reversed_ab: Vec<Point> = route_ab.points.into_iter().rev().collect();
    assert_eq!(reversed_ab, route_ba.points);
}

// --------------------------------------------------------------------------
// Error Path Tests
// --------------------------------------------------------------------------

#[test]
fn test_returns_error_when_start_point_is_nan() {
    let from = Point::new(f64::NAN, 0.0);
    let to = Point::new(10.0, 10.0);

    let result = compute_orthogonal_route(from, to);
    assert_eq!(result, Err(RoutingError::InvalidEndpoint));
}

#[test]
fn test_returns_error_when_end_point_is_infinity() {
    let from = Point::new(0.0, 0.0);
    let to = Point::new(f64::INFINITY, 10.0);

    let result = compute_orthogonal_route(from, to);
    assert_eq!(result, Err(RoutingError::InvalidEndpoint));
}

#[test]
fn test_returns_error_when_nan_end_coordinate() {
    let from = Point::new(0.0, 0.0);
    let to = Point::new(10.0, f64::NAN);

    let result = compute_orthogonal_route(from, to);
    assert_eq!(result, Err(RoutingError::InvalidEndpoint));
}

#[test]
fn test_returns_error_when_infinity_start_coordinate() {
    let from = Point::new(f64::INFINITY, 10.0);
    let to = Point::new(10.0, 10.0);

    let result = compute_orthogonal_route(from, to);
    assert_eq!(result, Err(RoutingError::InvalidEndpoint));
}

#[test]
fn test_returns_error_when_both_coordinates_invalid() {
    let from = Point::new(f64::NAN, f64::NAN);
    let to = Point::new(f64::INFINITY, f64::INFINITY);

    let result = compute_orthogonal_route(from, to);
    assert_eq!(result, Err(RoutingError::InvalidEndpoint));
}

#[test]
fn test_returns_error_when_start_and_end_are_identical() {
    let from = Point::new(5.0, 5.0);
    let to = Point::new(5.0, 5.0);

    let result = compute_orthogonal_route(from, to);
    assert_eq!(result, Err(RoutingError::DegenerateRoute));
}

#[test]
fn test_returns_error_when_start_point_inside_obstacle() {
    let from = Point::new(50.0, 50.0);
    let to = Point::new(200.0, 200.0);
    let obstacle = AABB::new(0.0, 0.0, 100.0, 100.0);

    let result = compute_orthogonal_route_avoiding(from, to, &obstacle);
    assert_eq!(result, Err(RoutingError::EndpointInsideObstacle));
}

#[test]
fn test_returns_error_when_end_point_inside_obstacle() {
    let from = Point::new(200.0, 200.0);
    let to = Point::new(50.0, 50.0);
    let obstacle = AABB::new(0.0, 0.0, 100.0, 100.0);

    let result = compute_orthogonal_route_avoiding(from, to, &obstacle);
    assert_eq!(result, Err(RoutingError::EndpointInsideObstacle));
}

#[test]
fn test_returns_error_when_both_points_inside_obstacle() {
    let from = Point::new(25.0, 25.0);
    let to = Point::new(75.0, 75.0);
    let obstacle = AABB::new(0.0, 0.0, 100.0, 100.0);

    let result = compute_orthogonal_route_avoiding(from, to, &obstacle);
    assert_eq!(result, Err(RoutingError::EndpointInsideObstacle));
}

// --------------------------------------------------------------------------
// Edge Case Tests
// --------------------------------------------------------------------------

#[test]
fn test_handles_points_with_sub_pixel_differences_as_identical() {
    let from = Point::new(5.0, 5.0);
    let to = Point::new(5.0 + 1e-11, 5.0 - 1e-11);

    let result = compute_orthogonal_route(from, to);
    assert_eq!(result, Err(RoutingError::DegenerateRoute));
}

#[test]
fn test_handles_very_large_coordinates() {
    let from = Point::new(1e150, 1e150);
    let to = Point::new(-1e150, -1e150);

    let result = compute_orthogonal_route(from, to);
    // Should either succeed or fail gracefully - both values are finite
    assert!(result.is_ok() || matches!(result, Err(RoutingError::DegenerateRoute)));
}

#[test]
fn test_handles_negative_coordinates() {
    let from = Point::new(-100.0, -50.0);
    let to = Point::new(-50.0, -100.0);

    let route = compute_orthogonal_route(from, to).expect("Should compute route");
    assert!(is_orthogonal(&route));
    assert_eq!(*route.points.first().unwrap(), from);
    assert_eq!(*route.points.last().unwrap(), to);
}

#[test]
fn test_detour_margin_calculation_near_boundaries() {
    // Tests that obstacle avoidance works even if the endpoints are near the margins
    let from = Point::new(0.0, 10.0);
    let to = Point::new(100.0, 10.0);
    let obstacle = AABB::new(40.0, 0.0, 60.0, 50.0);

    let route =
        compute_orthogonal_route_avoiding(from, to, &obstacle).expect("Should compute route");
    assert!(!route_intersects(&route, &obstacle));
}

#[test]
fn test_obstacle_avoidance_when_route_barely_touches_edge() {
    let from = Point::new(0.0, 50.0);
    let to = Point::new(100.0, 50.0);
    let obstacle = AABB::new(40.0, 50.0, 60.0, 100.0); // route is exactly on min_y

    let route =
        compute_orthogonal_route_avoiding(from, to, &obstacle).expect("Should compute route");
    // Should NOT intersect the strictly interior of the AABB.
    // The margin check in our implementation uses < and >, so the segment right on the boundary might pass as ok or detour, either is fine as long as no strict interior intersection.
    assert!(!route_intersects(&route, &obstacle));
}

#[test]
fn test_handles_zero_size_obstacle() {
    let from = Point::new(0.0, 25.0);
    let to = Point::new(100.0, 25.0);
    let obstacle = AABB::new(50.0, 25.0, 50.0, 25.0); // zero-size obstacle (point)

    // Should handle gracefully - either return direct route or error
    let result = compute_orthogonal_route_avoiding(from, to, &obstacle);
    // Either succeeds or returns EndpointInsideObstacle (if point is treated as containing endpoints)
    assert!(result.is_ok() || matches!(result, Err(RoutingError::EndpointInsideObstacle)));
}

// --------------------------------------------------------------------------
// Contract Verification Tests
// --------------------------------------------------------------------------

#[test]
fn test_postcondition_route_has_minimum_two_points() {
    let from = Point::new(0.0, 0.0);
    let to = Point::new(10.0, 0.0);
    let route = compute_orthogonal_route(from, to).unwrap();
    assert!(route.points.len() >= 2);
}

#[test]
fn test_postcondition_all_segments_are_strictly_orthogonal() {
    let from = Point::new(0.0, 0.0);
    let to = Point::new(10.0, 20.0);
    let route = compute_orthogonal_route(from, to).unwrap();
    assert!(is_orthogonal(&route));
}

#[test]
fn test_postcondition_start_and_end_points_match_input() {
    let from = Point::new(10.0, 20.0);
    let to = Point::new(30.0, 40.0);
    let route = compute_orthogonal_route(from, to).unwrap();
    assert_eq!(*route.points.first().unwrap(), from);
    assert_eq!(*route.points.last().unwrap(), to);
}

#[test]
fn test_postcondition_route_never_intersects_obstacle_interior() {
    let from = Point::new(0.0, 25.0);
    let to = Point::new(100.0, 25.0);
    let obstacle = AABB::new(25.0, 0.0, 75.0, 50.0);
    let route = compute_orthogonal_route_avoiding(from, to, &obstacle).unwrap();
    assert!(!route_intersects(&route, &obstacle));
}

// --------------------------------------------------------------------------
// Contract Violation Tests
// --------------------------------------------------------------------------

#[test]
fn test_p1_violation_returns_invalid_endpoint() {
    assert_eq!(
        compute_orthogonal_route(Point::new(f64::NAN, 0.0), Point::new(10.0, 10.0)),
        Err(RoutingError::InvalidEndpoint)
    );
}

#[test]
fn test_p2_violation_returns_degenerate_route() {
    assert_eq!(
        compute_orthogonal_route(Point::new(5.0, 5.0), Point::new(5.0, 5.0)),
        Err(RoutingError::DegenerateRoute)
    );
}

#[test]
fn test_p3_violation_returns_endpoint_inside_obstacle() {
    assert_eq!(
        compute_orthogonal_route_avoiding(
            Point::new(50.0, 50.0),
            Point::new(200.0, 200.0),
            &AABB::new(0.0, 0.0, 100.0, 100.0)
        ),
        Err(RoutingError::EndpointInsideObstacle)
    );
}
