# Contract Specification

## Context
- **Feature**: Implement edge_bounds() function for curved connectors (GEO-005)
- **Domain terms**:
  - Edge: A connector between two nodes with source/target, bend_points, style, arrow_type, thickness
  - QuadraticBezier: A quadratic Bezier curve with start, control, end points
  - AABB: Axis-Aligned Bounding Box with min_x, min_y, max_x, max_y
- **Assumptions**:
  - Edge geometry is calculated from node positions (source/target coordinates)
  - Curved edges use QuadraticBezier based on ArrowType::Curved
  - bend_points are used for orthogonal/polyline edges
- **Open questions**: None - the task is well-defined

## Preconditions
- [P1] Edge must have valid source and target NodeIds (non-empty)
- [P2] Edge thickness must be a positive finite f64
- [P3] Source and target coordinates must be finite f64 values

## Postconditions
- [Q1] Returns an AABB that encloses all curve segments and endpoints
- [Q2] Returns AABB expanded by stroke_width/2 for hit-testing margin
- [Q3] For ArrowType::Curved, bounds must include Bezier curve extrema
- [Q4] For edges with bend_points, bounds must include all intermediate points

## Invariants
- [I1] Result AABB always has min_x <= max_x and min_y <= max_y
- [I2] Result AABB contains start point, end point, and all control points
- [I3] Arrowhead area is included in bounds (extends backward from target)

## Error Taxonomy
- Error::InvalidNodePosition - when source/target coordinates are NaN or infinite
- Error::InvalidThickness - when thickness is <= 0 or NaN

## Contract Signatures
```rust
/// Calculate bounds for an edge including Bezier curve extents
fn edge_bounds(
    source_pos: Point,
    target_pos: Point,
    edge: &Edge,
) -> Result<AABB, EdgeBoundsError>

/// Calculate tight bounds for a quadratic Bezier curve
/// Uses derivative analysis to find exact extrema
fn quadratic_bezier_tight_bounds(
    start: Point,
    control: Point,
    end: Point,
    stroke_width: f64,
) -> AABB
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| source/target valid | Runtime-checked | Result<AABB, Error> |
| thickness > 0 | Runtime-checked | Result<AABB, Error> |
| coordinates finite | Runtime-checked | Result<AABB, Error> |

## Violation Examples (REQUIRED)
- VIOLATES P1: edge_bounds(Point::new(0.0, 0.0), Point::new(100.0, 100.0), &edge_with_invalid_node) -- should produce `Err(EdgeBoundsError::InvalidNodePosition)`
- VIOLATES P2: edge_bounds(Point::new(0.0, 0.0), Point::new(100.0, 100.0), &edge_with_zero_thickness) -- should produce `Err(EdgeBoundsError::InvalidThickness)`
- VIOLATES P3: edge_bounds(Point::new(f64::NAN, 0.0), Point::new(100.0, 100.0), &valid_edge) -- should produce `Err(EdgeBoundsError::InvalidNodePosition)`

## Ownership Contracts
- edge_bounds takes `&Edge` (shared borrow) - read-only, no mutation
- source_pos and target_pos are passed by value (owned) - caller retains ownership
- Function returns owned AABB

## Non-goals
- [ ] Hit-testing implementation (separate concern)
- [ ] Arrowhead rendering geometry (separate concern)
- [ ] Edge label positioning (separate concern)
