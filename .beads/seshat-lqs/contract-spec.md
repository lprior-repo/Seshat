# Contract Specification: Fit-to-Content Camera with Padding and Huge Coordinate Handling

## Context
- **Feature**: Fit-to-content camera with padding and huge coordinate handling
- **Bead ID**: seshat-lqs
- **Domain terms**: Viewport, Camera, AABB (Axis-Aligned Bounding Box), FitTransform, Zoom bounds
- **Assumptions**:
  - Viewport dimensions are positive (> 0)
  - Content bounds may contain extreme values (1e15+)
  - Padding is non-negative
- **Open Questions**: None after code review

## Preconditions
- [P1] Padding must be non-negative (padding >= 0.0)
- [P2] Content bounds must represent a valid AABB (min_x <= max_x, min_y <= max_y)
- [P3] Content width and height must be positive (> 0) for scale calculation
- [P4] Viewport dimensions must be positive (> 0)

## Postconditions
- [Q1] Returned FitTransform has finite scale (is_finite() == true)
- [Q2] Returned FitTransform has finite offset values (is_finite() == true)
- [Q3] Scale is clamped to [MIN_ZOOM, MAX_ZOOM] range
- [Q4] FitTransform centers content in viewport when applied
- [Q5] Content fits within viewport with specified padding
- [Q6] Handles huge coordinates (1e15+) without overflow/NaN/Infinity
- [Q7] Handles negative coordinates including far negatives (offscreen elements)

## Invariants
- [I1] zoom in [0.1, 4.0] - enforced by clamp in fit_to_content
- [I2] camera_x always finite - enforced by Result or fallback
- [I3] camera_y always finite - enforced by Result or fallback
- [I4] Coordinate transforms are reversible (within floating-point tolerance)

## Error Taxonomy
- `Error::InvalidContentBounds` - when content width/height <= 0
- `Error::InvalidPadding` - when padding is negative
- `Error::InvalidViewport` - when viewport dimensions <= 0
- `Error::CoordinateOverflow` - when coordinate calculations overflow
- `Error::PreconditionViolation` - generic fallback

## Contract Signatures
```rust
/// Fit content bounds to viewport with padding, handling extreme coordinates
pub fn fit_to_content(&self, content: &AABB, padding: f64) -> Result<FitTransform, Error> {
    // Preconditions:
    // - padding >= 0.0 (P1)
    // - content.min_x <= content.max_x && content.min_y <= content.max_y (P2)
    // - content.width() > 0 && content.height() > 0 (P3)
    // - self.viewport_width > 0 && self.viewport_height > 0 (P4)
    
    // Postconditions:
    // - result.scale.is_finite() (Q1)
    // - result.offset_x.is_finite() && result.offset_y.is_finite() (Q2)
    // - MIN_ZOOM <= result.scale <= MAX_ZOOM (Q3)
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| padding >= 0.0 | Runtime-checked constructor | Result<T, Error::InvalidPadding> |
| content.width() > 0 | Runtime-checked | Result<T, Error::InvalidContentBounds> |
| viewport dimensions > 0 | Runtime-checked | Result<T, Error::InvalidViewport> |
| finite result | Runtime-checked | is_finite() checks, Error::CoordinateOverflow |

## Violation Examples (REQUIRED)
- VIOLATES P1: `fit_to_content(&AABB::new(0.0, 0.0, 100.0, 100.0), -10.0)` -- should produce `Err(Error::InvalidPadding)`
- VIOLATES P2: `fit_to_content(&AABB::new(100.0, 100.0, 50.0, 50.0), 10.0)` -- should produce `Err(Error::InvalidContentBounds)` (min > max)
- VIOLATES P3: `fit_to_content(&AABB::new(0.0, 0.0, 0.0, 100.0), 10.0)` -- should produce `Err(Error::InvalidContentBounds)` (zero width)
- VIOLATES Q1: `fit_to_content(&AABB::new(1e308, 1e308, 1e308+100, 1e308+100), 0.0)` -- should produce `Err(Error::CoordinateOverflow)` or safe finite result
- VIOLATIONS Q6: `fit_to_content(&AABB::new(-1e15, -1e15, -1e15+100, -1e15+100), 10.0)` -- must handle extreme negatives

## Ownership Contracts (Rust-specific)
- `&self` - shared borrow, no mutation
- `&AABB` - read-only borrow of content bounds
- No ownership transfer, no cloning required
- Return value is Copy (FitTransform contains f64 values)

## Non-goals
- [ ] Real-time zoom animation (not part of this feature)
- [ ] Caching fit results (future optimization)
- [ ] Multiple content bounds (single AABB only)
