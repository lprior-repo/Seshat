# Martin Fowler Test Plan

## Test ID Prefix: EDG-031

## Happy Path Tests
- **test_orthogonal_route_returns_two_points_for_vertically_aligned**
  - Given: source at (50, 0), target at (50, 100)
  - When: computing orthogonal route
  - Then: returns route with 2 points (direct vertical line)

- **test_orthogonal_route_returns_two_points_for_horizontally_aligned**
  - Given: source at (0, 50), target at (100, 50)
  - When: computing orthogonal route
  - Then: returns route with 2 points (direct horizontal line)

- **test_orthogonal_route_returns_l_shape_for_diagonal_points**
  - Given: source at (0, 0), target at (100, 50)
  - When: computing orthogonal route
  - Then: returns route with 3 points forming L-shape: [(0,0), (100,0), (100,50)]

## Error Path Tests
- **test_orthogonal_route_returns_error_for_nan_coordinates**
  - Given: source with NaN x-coordinate
  - When: computing orthogonal route
  - Then: returns error or handles gracefully (current: may produce invalid route)

## Edge Case Tests
- **test_orthogonal_route_handles_zero_distance**
  - Given: source and target at same point (0, 0)
  - When: computing orthogonal route
  - Then: returns route with single point or two identical points

- **test_orthogonal_route_symmetric_horizontal_then_vertical**
  - Given: from (0, 0) to (100, 50)
  - When: computing orthogonal route
  - Then: mid point uses (to.x, from.y) = (100, 0) - horizontal first

## Contract Verification Tests (EDG-031)

### Test: test_edge_routing_stable_when_endpoints_swap_order
**Requirement**: Q4 (STABILITY) - Route must be symmetric when endpoints swap

**Scenario 1: Basic diagonal swap**
- Given: from=(0,0), to=(100,50)
- When: 
  - route_ab = orthogonal_route(Point::new(0.0, 0.0), Point::new(100.0, 50.0))
  - route_ba = orthogonal_route(Point::new(100.0, 50.0), Point::new(0.0, 0.0))
- Then:
  - route_ab.points.len() == route_ba.points.len()
  - route_ba.points == route_ab.points.iter().rev().cloned().collect::<Vec<_>>()

**Scenario 2: Different start point same geometry**
- Given: from=(0,100), to=(100,50)
- When:
  - route_ab = orthogonal_route(Point::new(0.0, 100.0), Point::new(100.0, 50.0))
  - route_ba = orthogonal_route(Point::new(100.0, 50.0), Point::new(0.0, 100.0))
- Then: route_ba.points == route_ab.points.iter().rev().cloned().collect::<Vec<_>>()

**Scenario 3: Large coordinate values**
- Given: from=(1000,2000), to=(3000,1500)
- When: computing both directions
- Then: routes are reverses of each other

## Contract Violation Tests

### EDG-031-V1: STABILITY Violation - Current Implementation
```
Given: from=(0,0), to=(100,50)
When: route_ab = orthogonal_route(Point::new(0.0, 0.0), Point::new(100.0, 50.0))
Then: route_ab.points = [(0,0), (100,0), (100,50)]

Given: from=(100,50), to=(0,0)
When: route_ba = orthogonal_route(Point::new(100.0, 50.0), Point::new(0.0, 0.0))
Then: route_ba.points = [(100,50), (0,50), (0,0)]

Violation: route_ba.points != route_ab.points.iter().rev().cloned().collect()
           Expected: [(100,50), (100,0), (0,0)]
           Actual:   [(100,50), (0,50), (0,0)]
```

## Given-When-Then Scenarios

### Scenario: EDG-031 - Edge Routing Stability
**Story**: As a diagram editor user, I want edge routes to be stable when I swap the source and target nodes, so that the visual representation doesn't change unexpectedly.

**Scenario 1: Swap endpoints on simple diagonal edge**
- **Given** an edge from node A at position (0, 0) to node B at position (100, 50)
- **When** I swap the source and target (now from B to A)
- **Then** the route should be the same path, just reversed
- **And** the visual edge should appear identical (just arrow direction changes)

**Scenario 2: Swap endpoints on horizontal edge**
- **Given** an edge from (0, 50) to (100, 50)
- **When** I swap source and target
- **Then** the route should still be a 2-point horizontal line (unchanged)

**Scenario 3: Swap endpoints on vertical edge**
- **Given** an edge from (50, 0) to (50, 100)
- **When** I swap source and target
- **Then** the route should still be a 2-point vertical line (unchanged)
