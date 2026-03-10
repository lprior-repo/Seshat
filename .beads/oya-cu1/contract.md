# Contract Specification

## Context
- **Feature**: Edge routing stability when endpoints swap order (EDG-031)
- **Domain terms**:
  - `orthogonal_route(from, to)` - computes an L-shaped orthogonal path between two points
  - `from` - source point of the route
  - `to` - target point of the route
  - `route.points` - the sequence of points forming the path
- **Assumptions**:
  - The function operates on 2D points with x and y coordinates
  - Route stability means reversing endpoints produces equivalent (reversed) path
- **Open questions**: None

## Preconditions
- [P1] Both `from` and `to` points must have finite (non-NaN, non-Inf) coordinates

## Postconditions
- [Q1] Route contains at least 2 points (start and end)
- [Q2] Route starts at `from` point (within tolerance)
- [Q3] Route ends at `to` point (within tolerance)
- [Q4] **STABILITY**: Swapping source and target produces route that is the reverse of original route (same path, reversed order)

## Invariants
- [I1] All points in route have finite coordinates
- [I2] Route uses horizontal-then-vertical L-shape pattern for non-aligned points
- [I3] Route length (number of points) is consistent for same input geometry regardless of direction

## Error Taxonomy
- **Error::InvalidInput** - when coordinates are NaN or Inf
- No other error conditions for basic routing (always succeeds with valid input)

## Contract Signatures
```rust
pub fn orthogonal_route(from: Point, to: Point) -> OrthogonalRoute
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: finite coordinates | Runtime-checked | Check `.is_finite()` on x/y values |

## Violation Examples (REQUIRED)
- VIOLATES Q4 (STABILITY): 
  ```rust
  // Given: from=(0,0), to=(100,50)
  let route_ab = orthogonal_route(Point::new(0.0, 0.0), Point::new(100.0, 50.0));
  // When: swapping source and target
  let route_ba = orthogonal_route(Point::new(100.0, 50.0), Point::new(0.0, 0.0));
  // Then: route_ba.points should equal route_ab.points.iter().rev().collect::<Vec<_>>()
  // Currently fails: route_ab has points [(0,0), (100,0), (100,50)]
  //                  route_ba has points [(100,50), (0,50), (0,0)] -- different geometry!
  ```

## Ownership Contracts
- All inputs are borrowed (Copy types), no ownership transfer
- Output is owned `OrthogonalRoute` with owned `Vec<Point>`

## Non-goals
- [ ] Obstacle avoidance routing (handled by separate `orthogonal_route_avoiding`)
- [ ] Curved/polyline edge routing with pre-defined bend points
- [ ] 3D or higher-dimensional routing
