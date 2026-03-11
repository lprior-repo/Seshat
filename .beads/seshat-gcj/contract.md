# Contract Specification: EDG-006 to EDG-010 (Edge Routing Logic)

## Context
- Feature: Edge Routing Algorithms (Orthogonal and Obstacle Avoidance)
- Domain terms:
  - `Point`: 2D finite coordinates (x, y).
  - `OrthogonalRoute`: A sequence of points where every contiguous pair forms an axis-aligned line.
  - `AABB`: Axis-Aligned Bounding Box representing a rectangular obstacle.
- Assumptions:
  - Routing logic relies purely on math/geometry (Data -> Calc -> Actions pattern).
  - Default margin for obstacle avoidance detours is 10.0 units.
  - Routing will not pass through the interior of the provided AABB obstacle.
- Open questions:
  - How should we route if both start and end are inside an obstacle? (Assuming it is treated as an error per strict constraints).

## Preconditions
- [P1] Endpoints must be valid finite numbers (not `NaN` or `Infinity`).
- [P2] Start and end points must not be identical (distance > tolerance `1e-10`).
- [P3] For obstacle avoidance, the start and end points must not lie strictly inside the obstacle's `AABB`.

## Postconditions
- [Q1] The resulting `OrthogonalRoute` contains at least 2 points.
- [Q2] Every contiguous segment in the `OrthogonalRoute` is strictly orthogonal (either `p1.x == p2.x` or `p1.y == p2.y`).
- [Q3] The first point in the route exactly matches the provided start point.
- [Q4] The last point in the route exactly matches the provided end point.
- [Q5] For obstacle avoidance, no segment in the route geometrically intersects the interior of the given `AABB`.
- [Q6] The route generation is symmetric (swapping `from` and `to` yields the exact same geometry but reversed).

## Invariants
- [I1] All points in `OrthogonalRoute.points` are finite (no `NaN`/`Infinity`).

## Error Taxonomy
- `RoutingError::InvalidEndpoint` - when a coordinate is `NaN` or `Infinity`.
- `RoutingError::DegenerateRoute` - when start and end are the same point within tolerance.
- `RoutingError::EndpointInsideObstacle` - when start or end is inside the obstacle `AABB`, making non-colliding routing impossible.

## Contract Signatures
- `fn compute_orthogonal_route(from: Point, to: Point) -> Result<OrthogonalRoute, RoutingError>`
- `fn compute_orthogonal_route_avoiding(from: Point, to: Point, obstacle: &AABB) -> Result<OrthogonalRoute, RoutingError>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Endpoints are finite | Error variant | `Result<OrthogonalRoute, RoutingError::InvalidEndpoint>` |
| Start != End | Error variant | `Result<OrthogonalRoute, RoutingError::DegenerateRoute>` |
| Endpoints not inside obstacle | Error variant | `Result<OrthogonalRoute, RoutingError::EndpointInsideObstacle>` |
| Route segments orthogonal | Debug-only | `debug_assert!(is_orthogonal(route))` |
| Non-intersecting | Debug-only | `debug_assert!(!route_intersects(route, obstacle))` |

## Violation Examples
- VIOLATES P1: `compute_orthogonal_route(Point::new(f64::NAN, 0.0), Point::new(10.0, 10.0))` -- should produce `Err(RoutingError::InvalidEndpoint)`
- VIOLATES P2: `compute_orthogonal_route(Point::new(5.0, 5.0), Point::new(5.0, 5.0))` -- should produce `Err(RoutingError::DegenerateRoute)`
- VIOLATES P3: `compute_orthogonal_route_avoiding(Point::new(50.0, 50.0), Point::new(200.0, 200.0), &AABB::new(0.0, 0.0, 100.0, 100.0))` -- should produce `Err(RoutingError::EndpointInsideObstacle)`

## Ownership Contracts
- Ownership transfer: `Point` is `Copy` and passed by value.
- Shared borrow: `&AABB` is passed as an immutable reference since the obstacle is read-only.
- Exclusive borrow: None.
- Clone policy: `OrthogonalRoute` owns its `Vec<Point>`, allocating a new vector upon successful calculation.

## Non-goals
- Multi-obstacle pathfinding (like A* or Dijkstra) is excluded from this logic; only single `AABB` avoidance is handled.
