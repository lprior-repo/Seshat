use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-017: Edge Routing - Avoid Obstacle ==============

/// Compute orthogonal route avoiding a rectangular obstacle
/// Uses simple detour: go around the obstacle on the shortest side
#[must_use]
pub fn orthogonal_route_avoiding(from: Point, to: Point, obstacle: &AABB) -> OrthogonalRoute {
    let direct = orthogonal_route(from, to);

    // Check if direct route intersects obstacle (simplified check)
    // For this test, we check if any segment crosses the obstacle
    let needs_detour = direct
        .points
        .windows(2)
        .any(|seg| segment_intersects_aabb(seg[0], seg[1], obstacle));

    if !needs_detour {
        return direct;
    }

    // Simple detour: go around top or bottom of obstacle
    let go_above = from.y < obstacle.max_y && to.y < obstacle.max_y;

    if go_above {
        let detour_y = obstacle.min_y - 10.0; // 10 unit margin
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
    } else {
        let detour_y = obstacle.max_y + 10.0;
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
}

/// Check if a line segment intersects an AABB
fn segment_intersects_aabb(p1: Point, p2: Point, aabb: &AABB) -> bool {
    // Simplified: check horizontal and vertical segments
    if (p1.y - p2.y).abs() < TOLERANCE {
        // Horizontal segment
        let min_x = p1.x.min(p2.x);
        let max_x = p1.x.max(p2.x);
        let y = p1.y;
        y >= aabb.min_y && y <= aabb.max_y && max_x >= aabb.min_x && min_x <= aabb.max_x
    } else if (p1.x - p2.x).abs() < TOLERANCE {
        // Vertical segment
        let x = p1.x;
        let min_y = p1.y.min(p2.y);
        let max_y = p1.y.max(p2.y);
        x >= aabb.min_x && x <= aabb.max_x && max_y >= aabb.min_y && min_y <= aabb.max_y
    } else {
        false
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_edge_routing_avoid_obstacle_no_intersection() {
    // Given: route that doesn't cross obstacle
    let from = Point::new(0.0, 0.0);
    let to = Point::new(200.0, 0.0);
    let obstacle = AABB::new(50.0, 50.0, 100.0, 100.0);

    // When: computing route
    let route = orthogonal_route_avoiding(from, to, &obstacle);

    // Then: direct route (no detour needed)
    assert_eq!(route.points.len(), 2);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_edge_routing_avoid_obstacle_with_intersection() {
    // Given: route that crosses obstacle
    let from = Point::new(0.0, 75.0);
    let to = Point::new(200.0, 75.0);
    let obstacle = AABB::new(50.0, 50.0, 100.0, 100.0);

    // When: computing route
    let route = orthogonal_route_avoiding(from, to, &obstacle);

    // Then: route has detour points
    assert!(route.points.len() > 2);
}
