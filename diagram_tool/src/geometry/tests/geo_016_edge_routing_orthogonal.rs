use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-016: Edge Routing - Orthogonal ==============

/// Represents an orthogonal route as a series of points
#[derive(Debug, Clone, PartialEq)]
pub struct OrthogonalRoute {
    pub points: Vec<Point>,
}

/// Compute a simple orthogonal route between two points
/// Uses L-shaped routing: horizontal first, then vertical
/// EDG-031: Route is stable when endpoints swap (symmetric)
#[must_use]
pub fn orthogonal_route(from: Point, to: Point) -> OrthogonalRoute {
    if (from.x - to.x).abs() < TOLERANCE {
        // Vertical line only
        OrthogonalRoute {
            points: vec![from, to],
        }
    } else if (from.y - to.y).abs() < TOLERANCE {
        // Horizontal line only
        OrthogonalRoute {
            points: vec![from, to],
        }
    } else {
        // L-shaped: vertical then horizontal (symmetric corner)
        // EDG-031 FIX: Use symmetric corner - min x, max y
        // This ensures swapping source/target produces reversed path, not different geometry
        let mid = Point::new(from.x.min(to.x), from.y.max(to.y));
        OrthogonalRoute {
            points: vec![from, mid, to],
        }
    }
}

#[test]
fn test_edge_routing_orthogonal_l_shape() {
    // Given: source at (0, 0), target at (100, 50)
    let from = Point::new(0.0, 0.0);
    let to = Point::new(100.0, 50.0);

    // When: computing orthogonal route
    let route = orthogonal_route(from, to);

    // Then: route has 3 points forming L-shape (vertical-first)
    assert_eq!(route.points.len(), 3);
    assert!((route.points[1].x - 0.0).abs() < TOLERANCE); // vertical first (min x)
    assert!((route.points[1].y - 50.0).abs() < TOLERANCE); // max y
}

#[test]
fn test_edge_routing_orthogonal_vertical() {
    // Given: vertically aligned points
    let from = Point::new(50.0, 0.0);
    let to = Point::new(50.0, 100.0);

    // When: computing orthogonal route
    let route = orthogonal_route(from, to);

    // Then: direct vertical line
    assert_eq!(route.points.len(), 2);
}

#[test]
fn test_edge_routing_orthogonal_horizontal() {
    // Given: horizontally aligned points
    let from = Point::new(0.0, 50.0);
    let to = Point::new(100.0, 50.0);

    // When: computing orthogonal route
    let route = orthogonal_route(from, to);

    // Then: direct horizontal line
    assert_eq!(route.points.len(), 2);
}
