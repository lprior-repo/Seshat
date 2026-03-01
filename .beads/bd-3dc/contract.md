bead_id: bd-3dc
bead_title: tests: Implement GEO geometry tests (GEO-011 to GEO-020)
phase: p0
updated_at: 2026-03-01T22:03:53Z

# Contract: GEO Geometry Tests (GEO-011 to GEO-020)

## Scope

Add 10 new geometry tests extending the existing GEO-001 to GEO-010 test suite in
`diagram_tool/src/geometry/mod.rs`.

## Test Specifications

### GEO-011: Rotation + Resize Composition
Test that rotation and resize transforms can be composed and the result is deterministic.
- Given a point, anchor, scale factor, and rotation angle
- When applying resize then rotation in sequence
- Then the result is mathematically equivalent to applying transforms individually

### GEO-012: Zoom at Pointer
Test zoom transformation centered at a specific pointer position.
- Given a view rectangle and a pointer position
- When zooming by a factor centered at the pointer
- Then the pointer stays at the same relative position

### GEO-013: Snap Lines Horizontal
Test horizontal line snapping to grid or other lines.
- Given a line and snap targets
- When computing snap position
- Then the line snaps to the nearest horizontal target within tolerance

### GEO-014: Snap Lines Vertical
Test vertical line snapping to grid or other lines.
- Given a line and snap targets
- When computing snap position
- Then the line snaps to the nearest vertical target within tolerance

### GEO-015: Grid Step
Test grid stepping for snapping points to grid intersections.
- Given a point and grid size
- When snapping to grid
- Then the point moves to the nearest grid intersection

### GEO-016: Edge Routing - Orthogonal
Test orthogonal edge routing between two rectangles.
- Given source and target rectangles
- When computing orthogonal route
- Then the path consists of horizontal and vertical segments only

### GEO-017: Edge Routing - Avoid Obstacle
Test edge routing that avoids an obstacle rectangle.
- Given source, target, and obstacle rectangles
- When computing route
- Then the path avoids intersecting the obstacle

### GEO-018: Fit to Content
Test content fitting within a viewport.
- Given content bounds and viewport dimensions
- When computing fit transform
- Then the scale and offset center the content with appropriate zoom

### GEO-019: Hit Test with Margin
Test hit testing with a margin around shapes.
- Given a point, shape, and hit margin
- When testing if point hits the shape
- Then hits within the margin are considered positive

### GEO-020: Hit Test Rotated Shape
Test hit testing on rotated shapes.
- Given a point and rotated rectangle
- When testing for hit
- Then the hit is computed in local coordinate space

## Acceptance Criteria

1. All 10 tests (GEO-011 to GEO-020) must pass
2. Tests must follow existing test patterns in the module
3. Use `TOLERANCE = 1e-10` for floating-point comparisons
4. Include edge case tests (zero values, boundary conditions)
5. Tests must compile without warnings
6. Existing tests must remain passing

## Preconditions

- GEO-001 to GEO-010 tests exist and pass
- Geometry module at `diagram_tool/src/geometry/mod.rs`

## Postconditions

- 10 new tests added to the geometry test module
- All tests (old and new) pass
- Code coverage increased for geometry functions

## Out of Scope

- New geometry functions (implement only tests for existing or minimal helpers)
- Property-based tests (optional, not required)
- Benchmark tests
