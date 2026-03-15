# Implementation Summary: AABB includes stroke width (GEO-003)

## Bead: seshat-da5
## Feature: AABB includes stroke width / hit margin

### Changes Made

#### 1. Extended `BoundsError` in `operations.rs`

Added two new error variants to support the contract:
- `InvalidBounds` - when min > max coordinates
- `NegativeExpansion` - when expand amount is negative

```rust
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum BoundsError {
    #[error("Invalid coordinate: NaN or Infinity")]
    InvalidCoordinate,
    #[error("Invalid bounds: min ({min_x}, {min_y}) > max ({max_x}, {max_y})")]
    InvalidBounds {
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    },
    #[error("Negative expansion amount: {0} (must be >= 0)")]
    NegativeExpansion(f64),
}
```

#### 2. Updated `safe_bounds` function

Now returns error when bounds are invalid (min > max), rather than silently swapping them.

#### 3. Added new methods to `AABB` in `primitives.rs`

- `new_checked(min_x, min_y, max_x, max_y) -> Result<AABB, BoundsError>` - Creates AABB with validation (behind `strict` feature)
- `expand_checked(amount) -> Result<AABB, BoundsError>` - Expand with validation (behind `strict` feature)
- `expand_by_hit_margin(margin) -> AABB` - Semantic method for hit testing expansion

### Contract Fulfillment

| Contract Clause | Implementation |
|-----------------|----------------|
| P1: AABB::new requires max_x >= min_x | `safe_bounds` now validates this |
| P2: expand requires amount >= 0 | `expand_checked` validates this |
| P3: bounds_with_stroke requires stroke_width >= 0 | Already implemented via `StrokedShape::bounds_with_stroke` |
| Q1: expand returns larger AABB | `expand()` works correctly |
| Q2: bounds_with_stroke includes stroke | `StrokedShape::bounds_with_stroke` expands by stroke/2 |
| Q3: center preserved | Verified by `expand()` implementation |
| Q4: hit margin can be added | `expand_by_hit_margin()` added |

### Files Changed
- `diagram_tool/src/geometry/operations.rs` - Extended BoundsError, updated safe_bounds
- `diagram_tool/src/geometry/primitives.rs` - Added new AABB methods

### Notes
- The existing `StrokedShape::bounds_with_stroke()` already implements the stroke width inclusion
- The new `expand_by_hit_margin()` provides semantic clarity for hit testing use cases
- Added `strict` feature flag for compile-time checked variants (new_checked, expand_checked)
- Maintained backward compatibility - existing `AABB::new()` and `expand()` continue to work as before
