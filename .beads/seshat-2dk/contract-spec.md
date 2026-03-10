# Contract Specification: GEO-020 Hit Test Margin Respects Zoom Level

## Context
- **Feature**: Hit test margin should be constant screen-space (same pixel size on screen regardless of zoom level)
- **Bead ID**: seshat-2dk
- **Domain terms**:
  - `screen_hit_radius`: Hit radius measured in screen pixels (constant)
  - `hit_radius_world`: Hit radius in world coordinates (varies with zoom)
  - `zoom`: Current viewport zoom level (zoom > 0)
- **Assumptions**:
  - Screen-space hit testing is the desired behavior (consistent UX regardless of zoom)
  - The implementation already exists in `canvas_view.rs:find_edge_at()`
- **Open questions**: None - behavior is already implemented and tested

## Preconditions
- [P1] Zoom level must be positive (> 0)
- [P2] Screen hit radius must be non-negative (>= 0)
- [P3] Point coordinates (x, y) must be finite f64 values

## Postconditions
- [Q1] Hit radius in world coordinates = screen_hit_radius / zoom
- [Q2] At lower zoom (e.g., 0.5x), world hit radius is LARGER (easier to hit)
- [Q3] At higher zoom (e.g., 2.0x), world hit radius is SMALLER (harder to hit)
- [Q4] Same screen position always hits the same element regardless of zoom level

## Invariants
- [I1] World hit radius is always positive when zoom > 0 and screen hit radius > 0
- [I2] Hit test behavior is deterministic for given (point, element, zoom) tuple

## Error Taxonomy
- No error variants - this is a pure function with defined behavior for all inputs
- Precondition violations result in division by zero or invalid results (handled by debug_assert in debug builds)

## Contract Signatures
```rust
/// Convert screen-space hit radius to world-space based on zoom level
/// screen_hit_radius: Hit radius in screen pixels (constant)
/// zoom: Current viewport zoom level (must be > 0)
/// Returns: Hit radius in world coordinates
fn screen_to_world_hit_radius(screen_hit_radius: f64, zoom: f64) -> f64 {
    debug_assert!(zoom > 0.0, "Zoom must be positive");
    debug_assert!(screen_hit_radius >= 0.0, "Screen hit radius must be non-negative");
    screen_hit_radius / zoom
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| zoom > 0 | Debug-only | `debug_assert!(zoom > 0.0)` |
| screen_hit_radius >= 0 | Debug-only | `debug_assert!(screen_hit_radius >= 0.0)` |
| coordinates finite | Debug-only | `debug_assert!(x.is_finite() && y.is_finite())` |

## Violation Examples
- VIOLATES P1: `screen_to_world_hit_radius(17.0, 0.0)` -- division by zero, returns inf
- VIOLATES P2: `screen_to_world_hit_radius(-5.0, 1.0)` -- returns negative hit radius (invalid)
- VIOLATES Q1: If implementation uses constant world radius instead of dividing by zoom, the behavior would be wrong

## Ownership Contracts
- This is a pure function with no ownership transfer
- All parameters are copied (f64), no borrows

## Non-goals
- [ ] World-space hit testing (different UX behavior)
- [ ] Dynamic hit radius based on element size
