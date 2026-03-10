# Martin Fowler Tests: Freehand Drawing with Path Simplification

## Test Strategy

These tests follow the Martin Fowler Given-When-Then pattern for BDD-style test specifications. Tests are organized by feature area and test type.

---

## GEO-027: Path Simplification Tests

### Unit Tests: PathSimplification Algorithm

#### GEO-027-001: Basic Simplification
```
GIVEN: A path with 5 points in a rough line: [(0,0), (1,1), (2,1), (3,2), (4,4)]
WHEN:  Simplify is called with epsilon = 1.0
THEN:  The output should have 2 points: [(0,0), (4,4)]
WHY:   All intermediate points are within epsilon of the line from start to end
```

#### GEO-027-002: Endpoint Preservation - Start
```
GIVEN: A path: [(0,0), (10,10), (20,20), (30,30)]
WHEN:  Simplify is called with epsilon = 0.5
THEN:  The first point MUST be (0,0)
WHY:   Start point is always preserved
```

#### GEO-027-003: Endpoint Preservation - End
```
GIVEN: A path: [(0,0), (10,10), (20,20), (30,30)]
WHEN:  Simplify is called with epsilon = 0.5
THEN:  The last point MUST be (30,30)
WHY:   End point is always preserved
```

#### GEO-027-004: No Self-Intersection Spikes - Case 1
```
GIVEN: A path that would create a spike if simplified: [(0,0), (5,10), (10,0), (15,10), (20,0)]
WHEN:  Simplify is called with epsilon = 3.0
THEN:  The result MUST NOT have self-intersections
      AND The output should preserve the zig-zag nature or be rejected
WHY:   GEO-027 requirement: simplification must not create self-intersection spikes
```

#### GEO-027-005: No Self-Intersection Spikes - Case 2
```
GIVEN: A path with sharp turns: [(0,0), (1,5), (2,0), (3,5), (4,0), (5,5), (6,0)]
WHEN:  Simplify is called with epsilon = 2.0
THEN:  The simplified path MUST NOT have segments that cross other segments
WHY:   Self-intersection would create visual artifacts
```

#### GEO-027-006: Too Short Path Rejected - Zero Points
```
GIVEN: An empty path: []
WHEN:  Simplify is called with any epsilon
THEN:  The function SHOULD return Err(PathError::InsufficientPoints)
WHY:   Cannot create a shape with no points
```

#### GEO-027-007: Too Short Path Rejected - One Point
```
GIVEN: A path with one point: [(5,5)]
WHEN:  Simplify is called with any epsilon
THEN:  The function SHOULD return Err(PathError::InsufficientPoints)
WHY:   Need at least 2 points to define a line
```

#### GEO-027-008: Too Short Path Rejected - Two Points
```
GIVEN: A path with two points: [(0,0), (10,10)]
WHEN:  Simplify is called with epsilon = 1.0
THEN:  The output MUST have exactly 2 points: [(0,0), (10,10)]
WHY:   Two points define a line, cannot be simplified further
```

#### GEO-027-009: Invalid Points - NaN
```
GIVEN: A path with NaN: [(0,0), (NaN, 5), (10,10)]
WHEN:  Simplify is called
THEN:  The function MUST return Err(PathError::InvalidPoint)
WHY:   NaN indicates invalid input data
```

#### GEO-027-010: Invalid Points - Infinity
```
GIVEN: A path with Infinity: [(0,0), (Inf, 5), (10,10)]
WHEN:  Simplify is called
THEN:  The function MUST return Err(PathError::InvalidPoint)
WHY:   Infinity is not a valid coordinate
```

#### GEO-027-011: Epsilon Boundary - Exactly On Line
```
GIVEN: A path: [(0,0), (5,0), (10,0)]
WHEN:  Simplify is called with epsilon = 0.0
THEN:  The output should have 2 points: [(0,0), (10,0)]
WHY:   All points are exactly on the line (distance = 0)
```

#### GEO-027-012: Epsilon Boundary - Just Over Line
```
GIVEN: A path: [(0,0), (5,1), (10,0)]
WHEN:  Simplify is called with epsilon = 0.5
THEN:  The output should have 2 points: [(0,0), (10,0)]
WHY:   Point (5,1) distance from line = ~0.5, which is at epsilon boundary
```

#### GEO-027-013: Epsilon Boundary - Just Over Epsilon
```
GIVEN: A path: [(0,0), (5,1), (10,0)]
WHEN:  Simplify is called with epsilon = 0.4
THEN:  The output should have 3 points (intermediate point kept)
WHY:   Point (5,1) distance ~0.5 > epsilon 0.4, so kept
```

#### GEO-027-014: Curved Path Simplification
```
GIVEN: A curved path: [(0,0), (2,2), (4,4), (6,6), (8,8), (10,10)]
WHEN:  Simplify is called with epsilon = 1.0
THEN:  The output should have 2 points: [(0,0), (10,10)]
WHY:   All intermediate points are close to the diagonal line
```

#### GEO-027-015: Complex Path - Multiple Segments
```
GIVEN: A complex path with multiple direction changes:
       [(0,0), (1,10), (2,-10), (3,10), (4,-10), (5,10), (6,-10), (7,10), (8,0)]
WHEN:  Simplify is called with epsilon = 5.0
THEN:  The output MUST preserve the oscillatory nature OR be rejected for self-intersection
WHY:   High-frequency oscillations should be reduced but not create artifacts
```

#### GEO-027-016: Single Segment Preservation
```
GIVEN: A simple straight line: [(0,0), (100,0)]
WHEN:  Simplify is called with epsilon = 10.0
THEN:  The output MUST be exactly [(0,0), (100,0)]
WHY:   Straight line cannot be simplified
```

#### GEO-027-017: Degenerate Case - Same Start and End
```
GIVEN: A path where start equals end: [(5,5), (10,10), (5,5)]
WHEN:  Simplify is called with epsilon = 1.0
THEN:  The function SHOULD handle gracefully (degenerate path)
WHY:   Must not panic on edge case
```

#### GEO-027-018: Large Number of Points Performance
```
GIVEN: A path with 10,000 points in a rough line
WHEN:  Simplify is called with epsilon = 1.0
THEN:  The function MUST complete within 100ms
WHY:   Performance requirement for real-time drawing
```

#### GEO-027-019: Self-Intersection Detection - Simple Cross
```
GIVEN: A path: [(0,0), (10,10), (0,10), (10,0)]
WHEN:  Simplify is called with epsilon = 1.0
THEN:  The result MUST either:
       - Not create the crossing (simplified to [(0,0), (10,10), (0,10), (10,0)])
       - OR return Err(PathError::SelfIntersection)
WHY:   Self-intersection is not allowed per GEO-027
```

#### GEO-027-020: Self-Intersection Detection - Touch at Non-Endpoint
```
GIVEN: A path: [(0,0), (5,5), (10,0), (5,5), (0,0)]
WHEN:  Simplify is called with epsilon = 0.5
THEN:  The function MUST detect the touch at (5,5) and handle appropriately
WHY:   Touch at non-endpoint is a self-intersection
```

---

## Integration Tests: Draw Tool

#### GEO-027-INT-001: Draw Tool Creates Path Node
```
GIVEN: The editor is in Draw tool mode
WHEN:  User clicks and drags to draw, then releases
THEN:  A new node of kind "path" should exist in the document
WHY:   Core functionality requirement
```

#### GEO-027-INT-002: Draw Tool Live Preview
```
GIVEN: The editor is in Draw tool mode and user has started drawing
WHEN:  User moves the pointer while holding button
THEN:  A preview path should be visible on the canvas
WHY:   User feedback requirement
```

#### GEO-027-INT-003: Tool Switch Cancels Drawing
```
GIVEN: The user is in the middle of drawing (capturing points)
WHEN:  User switches to Select tool
THEN:  The in-progress path should be discarded
WHY:   Clean state transition requirement
```

#### GEO-027-INT-004: ESC Cancels Drawing
```
GIVEN: The user is in the middle of drawing (capturing points)
WHEN:  User presses ESC key
THEN:  The in-progress path should be discarded
WHY:   Standard cancel behavior
```

#### GEO-027-INT-005: Very Short Path Not Created
```
GIVEN: The editor is in Draw tool mode
WHEN:  User clicks and immediately releases without moving
THEN:  No path node should be created
WHY:   GEO-027 requirement: < 3 points = click, not draw
```

---

## Edge Case Tests

#### GEO-027-EDGE-001: Path at Canvas Edge
```
GIVEN: Drawing near the canvas boundary
WHEN:  Path extends beyond visible canvas
THEN:  The path should still be created correctly
WHY:   Coordinate system handles off-canvas points
```

#### GEO-027-EDGE-002: Path with Very Close Points
```
GIVEN: User draws very slowly, generating points at sub-pixel distances
WHEN:  Simplify is applied
THEN:  Redundant points should be removed
WHY:   Simplification should handle dense sampling
```

#### GEO-027-EDGE-003: Path with Very Far Points
```
GIVEN: User makes large jump movements
WHEN:  Simplify is applied
THEN:  Points should be preserved appropriately
WHY:   Must handle sparse sampling
```

#### GEO-027-EDGE-004: Maximum Points Limit
```
GIVEN: User draws continuously for very long time (> 100,000 points)
WHEN:  Limit is reached during capture
THEN:  The capture should either:
       - Continue with sampling/dropping oldest points
       - OR auto-complete the path
WHY:   Must prevent unbounded memory growth
```

---

## Regression Tests

#### GEO-027-REG-001: Existing Tools Still Work
```
GIVEN: Before this feature, Select, Pan, Edge, Subgraph, Text tools existed
WHEN:  Each tool is selected and used
THEN:  All existing tools should work exactly as before
WHY:   No regression allowed
```

#### GEO-027-REG-002: Undo Works for Path Creation
```
GIVEN: User creates a path using Draw tool
WHEN:  User performs undo (Ctrl+Z)
THEN:  The path should be removed
WHY:   All document operations must support undo
```

#### GEO-027-REG-003: Document Persistence Includes Paths
```
GIVEN: A document with a path node is saved
WHEN:  Document is reloaded
THEN:  The path node should be restored exactly
WHY:   Persistence requirement
```

---

## Test Execution Order

1. Run all unit tests (GEO-027-001 through GEO-027-020)
2. Run integration tests (GEO-027-INT-001 through GEO-027-INT-005)
3. Run edge case tests (GEO-027-EDGE-001 through GEO-027-EDGE-004)
4. Run regression tests (GEO-027-REG-001 through GEO-027-REG-003)

All tests must pass before the feature is complete.
