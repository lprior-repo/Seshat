# Contract Specification

## Context
- **Feature**: Intersection algorithms for connector logic (GEO-021 to GEO-025)
- **Domain terms**: Line segment, intersection, rectangle, connector routing
- **Assumptions**: Use f64 for all coordinates, standard mathematical precision
- **Open questions**: Should rotation be handled for rectangles?

## Preconditions

- **P1**: Line segment endpoints must have finite coordinates (not NaN, not Infinity)
- **P2**: Line segment must have non-zero length (start != end)
- **P3**: Rectangle must have positive width and height

## Postconditions

- **Q1**: line_line_intersects returns true iff the two line segments share at least one point
- **Q2**: line_line_intersection_point returns Some(p) iff lines intersect, where p lies on both segments
- **Q3**: line_rect_intersects returns true iff the line segment intersects any edge of the rectangle
- **Q4**: line_rect_intersection_points returns all intersection points with rectangle edges

## Invariants

- **I1**: All intersection points returned must lie within epsilon tolerance of both line segments
- **I2**: line_line_intersection_point is consistent with line_line_intersects

## Error Taxonomy
- `Error::InvalidEndpoint` - when line endpoint has NaN or Infinity coordinates
- `Error::DegenerateLine` - when line segment has zero length

## Contract Signatures

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment {
    pub start: Point,
    pub end: Point,
}

/// Check if two line segments intersect (boolean)
/// Returns true if segments share at least one point
pub fn line_line_intersects(a: LineSegment, b: LineSegment) -> bool;

/// Find intersection point of two line segments
/// Returns None if lines are parallel or don't intersect
pub fn line_line_intersection(a: LineSegment, b: LineSegment) -> Option<Point>;

/// Check if line segment intersects axis-aligned rectangle
pub fn line_rect_intersects(line: LineSegment, rect: &AABB) -> bool;

/// Find all intersection points between line segment and rectangle edges
pub fn line_rect_intersections(line: LineSegment, rect: &AABB) -> Vec<Point>;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Finite coordinates | Runtime-checked constructor | `LineSegment::new() -> Result` |
| Non-zero length | Runtime-checked constructor | `LineSegment::new() -> Result` |
| Positive rect dimensions | Runtime assertion | AABB invariants |

## Violation Examples

- **VIOLATES P1**: `LineSegment::new(Point::new(f64::NAN, 0.0), Point::new(1.0, 1.0))` → should produce `Err(Error::InvalidEndpoint)`
- **VIOLATES P1**: `LineSegment::new(Point::new(0.0, f64::INFINITY), Point::new(1.0, 1.0))` → should produce `Err(Error::InvalidEndpoint)`
- **VIOLATES P2**: `LineSegment::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0))` → should produce `Err(Error::DegenerateLine)`
- **VIOLATES Q1**: Two parallel lines `line_line_intersects` should return false

## Ownership Contracts
- All functions take values or references with no mutation
- No clone operations required in the public API

## Non-goals
- Rotated rectangle intersection (axis-aligned only)
- Ray intersection (segment only)
- Self-intersection handling
