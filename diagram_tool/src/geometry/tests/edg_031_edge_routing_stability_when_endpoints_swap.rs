#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== EDG-031: Edge Routing Stability When Endpoints Swap ==============

#[cfg(kani)]
#[kani::proof]
fn test_edge_routing_stable_when_endpoints_swap_order() {
    // EDG-031: Route must be stable when endpoints swap (same path, reversed)
    // Given: from=(0,0), to=(100,50)
    let from = Point::new(0.0, 0.0);
    let to = Point::new(100.0, 50.0);

    // When: computing orthogonal route in both directions
    let route_ab = orthogonal_route(from, to);
    let route_ba = orthogonal_route(to, from);

    // Then: routes have same length
    assert_eq!(route_ab.points.len(), route_ba.points.len());

    // Then: reversed route_ab equals route_ba
    let reversed_ab: Vec<Point> = route_ab.points.iter().rev().cloned().collect();
    assert_eq!(
        reversed_ab, route_ba.points,
        "Swapped route should be reverse of original"
    );
}

#[cfg(kani)]
#[kani::proof]
fn test_edge_routing_stable_different_start_point() {
    // EDG-031: Test with different start point
    // Given: from=(0,100), to=(100,50)
    let from = Point::new(0.0, 100.0);
    let to = Point::new(100.0, 50.0);

    // When: computing orthogonal route in both directions
    let route_ab = orthogonal_route(from, to);
    let route_ba = orthogonal_route(to, from);

    // Then: reversed route_ab equals route_ba
    let reversed_ab: Vec<Point> = route_ab.points.iter().rev().cloned().collect();
    assert_eq!(
        reversed_ab, route_ba.points,
        "Swapped route should be reverse of original"
    );
}
