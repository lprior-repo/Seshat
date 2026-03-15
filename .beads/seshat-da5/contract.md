# Contract Specification: AABB includes stroke width (GEO-003)

## Context
- **Feature**: AABB includes stroke width / hit margin
- **Bead ID**: seshat-da5
- **Domain terms**:
  - AABB (Axis-Aligned Bounding Box): A rectangle defined by min_x, min_y, max_x, max_y
  - Stroke width: The thickness of a shape's outline
  - Hit margin: Additional padding for hit testing to make selection more forgiving
- **Assumptions**:
  - The existing `StrokedShape<T>` wrapper exists but may need generalization
  - Hit margin is a separate concept from stroke width, but both affect the effective bounds
- **Open questions**: None - the requirement is clear from the bead description

## Preconditions

- [P1] `AABB::new(min_x, min_y, max_x, max_y)` requires max_x >= min_x AND max_y >= min_y
- [P2] `expand(amount)` requires amount >= 0.0
- [P3] `bounds_with_stroke()` requires stroke_width >= 0.0

## Postconditions

- [Q1] `expand(amount)` returns an AABB that is larger by `amount` on all four sides
- [Q2] `bounds_with_stroke()` returns an AABB that includes both the shape bounds AND the stroke width (stroke_width/2 on each side)
- [Q3] The center of the expanded AABB should remain unchanged (center is preserved)
- [Q4] Hit margin can be optionally added to AABB for hit testing purposes

## Invariants

- [I1] AABB always has min_x <= max_x and min_y <= max_y
- [I2] width() = max_x - min_x always >= 0
- [I3] height() = max_y - min_y always >= 0

## Error Taxonomy

- `BoundsError::InvalidBounds` - when min > max (invalid AABB)
- `BoundsError::NegativeExpansion` - when expand amount is negative

## Contract Signatures

```rust
/// Creates a new AABB, returns Err if invalid bounds
fn new_aabb(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<AABB, BoundsError>;

/// Expands AABB by amount on all sides, amount must be >= 0
fn expand(&self, amount: f64) -> AABB;

/// Returns bounds including stroke width
fn bounds_with_stroke(&self, stroke_width: f64) -> AABB;

/// Returns bounds including hit margin for hit testing
fn bounds_with_hit_margin(&self, margin: f64) -> AABB;
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| max_x >= min_x | Runtime-checked constructor | `Result<AABB, BoundsError>` |
| max_y >= min_y | Runtime-checked constructor | `Result<AABB, BoundsError>` |
| amount >= 0.0 | Runtime-checked | `debug_assert!(amount >= 0.0)` or Result |
| stroke_width >= 0.0 | Runtime-checked | `debug_assert!(stroke_width >= 0.0)` |
| margin >= 0.0 | Runtime-checked | `debug_assert!(margin >= 0.0)` |

## Violation Examples

- VIOLATES P1: `AABB::new(100.0, 0.0, 50.0, 100.0)` -- should produce `Err(BoundsError::InvalidBounds)` because max_x < min_x
- VIOLATES P1: `AABB::new(0.0, 100.0, 100.0, 50.0)` -- should produce `Err(BoundsError::InvalidBounds)` because max_y < min_y
- VIOLATES P2: `expand(-5.0)` -- should produce error or be prevented
- VIOLATES Q1: `AABB::new(0, 0, 100, 100).expand(10)` should have min_x=-10, min_y=-10, max_x=110, max_y=110

## Ownership Contracts

- `expand()` takes `&self` (immutable borrow) - no mutation
- `bounds_with_stroke()` takes `&self` - no mutation, returns new AABB
- All AABB methods are stateless and return new values

## Non-goals
- Rotation handling (covered by GEO-002)
- Collision detection between multiple AABBs
