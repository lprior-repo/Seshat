# Contract Specification

## Metadata
- bead_id: seshat-85y
- bead_title: CAM-001 to CAM-004: Zoom limits
- phase: contract
- updated_at: 2026-03-14T12:30:00Z

## Context
- **Feature**: Zoom limit enforcement - Clamp zoom between 0.1x and 4.0x scale
- **Domain terms**:
  - `zoom`: Scale factor where 1.0 = 100% (actual size), 0.1 = 10% (zoomed out), 4.0 = 400% (zoomed in)
  - `MIN_ZOOM`: Minimum allowed zoom value (0.1x)
  - `MAX_ZOOM`: Maximum allowed zoom value (4.0x)
  - `ViewportState`: Contains zoom, camera_x, camera_y for viewport state
- **Assumptions**:
  - Zoom must always be finite or recoverable (invalid values fallback to 1.0)
  - Zoom operations are multiplicative (zoom_in multiplies by 1.25, zoom_out by 0.8)
  - Zoom reset returns to 1.0 (100%)
- **Open questions**: None - existing codebase provides clear context

## Preconditions
- [P1] Zoom value passed to set_zoom must be finite or handled gracefully (NaN/Infinity fallback)
- [P2] Target zoom passed to set_zoom must be a valid f64 (not NaN/Infinity in target)
- [P3] Viewport dimensions must be positive (> 0.0)
- [P4] Camera coordinates must be finite (or recoverable)

## Postconditions
- [Q1] After any zoom operation: result in [MIN_ZOOM=0.1, MAX_ZOOM=4.0]
- [Q2] After any zoom operation: zoom always finite and within bounds
- [Q3] After zoom operations at boundaries: idempotent (no change when already at limit)
- [Q4] After zoom reset: zoom equals 1.0 exactly
- [Q5] Camera coordinates remain finite after zoom operations

## Invariants
- [I1] 0.1 <= zoom <= 4.0 (always enforced after any operation)
- [I2] camera_x is always finite (or defaults to 0.0)
- [I3] camera_y is always finite (or defaults to 0.0)
- [I4] Coordinate transforms are reversible (world_to_screen and screen_to_world are inverses)

## Error Taxonomy
All zoom operations return `bool` (changed: true/false) with state mutation. No Result type needed as invalid inputs are handled gracefully with clamping/fallback:

- `ZoomError::InvalidValue`: When zoom is NaN, Infinity, or negative (handled by clamping to 1.0)
- `ZoomError::OutOfBounds`: When caller requires explicit error for out-of-bounds (not used internally)

## Contract Signatures

### ViewportState Methods (from viewport/mod.rs)
```rust
impl ViewportState {
    /// Set zoom level directly, clamped to [MIN_ZOOM, MAX_ZOOM]
    /// Returns true if zoom changed, false otherwise
    pub fn set_zoom(&mut self, zoom: f64) -> bool

    /// Zoom in by ZOOM_IN_FACTOR (1.25), clamped to MAX_ZOOM
    /// Returns true if zoom changed, false otherwise
    pub fn zoom_in(&mut self) -> bool

    /// Zoom out by ZOOM_OUT_FACTOR (0.8), clamped to MIN_ZOOM
    /// Returns true if zoom changed, false otherwise
    pub fn zoom_out(&mut self) -> bool

    /// Zoom around a specific screen point
    pub fn zoom_around_point(&mut self, zoom: f64, screen_x: f64, screen_y: f64) -> bool
}
```

### Constants (from viewport/mod.rs)
```rust
pub const MIN_ZOOM: f64 = 0.1;
pub const MAX_ZOOM: f64 = 4.0;
pub const ZOOM_IN_FACTOR: f64 = 1.25;
pub const ZOOM_OUT_FACTOR: f64 = 0.8;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| zoom.is_finite() | Runtime-checked | `set_zoom()` returns false for NaN/Inf, keeps current |
| zoom > 0.0 | Runtime-checked | `set_zoom()` returns false for non-positive, keeps current |
| zoom in [0.1, 4.0] | Runtime-checked | `zoom.clamp(MIN_ZOOM, MAX_ZOOM)` |
| viewport dimensions > 0 | Runtime-checked | Minimum 1.0 enforced in constructor |
| camera coordinates finite | Runtime-checked | Fallback to 0.0 |

**Rationale**: Compile-time enforcement not possible for f64 zoom values. Runtime clamping is the appropriate pattern.

## Violation Examples (REQUIRED -- one per precondition and postcondition)

### Precondition Violations
- **VIOLATES P1**: `viewport.set_zoom(f64::NAN)` -- returns `false` (no change), zoom stays finite
- **VIOLATES P1**: `viewport.set_zoom(f64::INFINITY)` -- returns `false` (no change), zoom stays finite
- **VIOLATES P2**: `viewport.set_zoom(-5.0)` -- returns `false` (no change), zoom stays finite

### Postcondition Violations
- **VIOLATES Q1**: `viewport.set_zoom(100.0)` -- zoom becomes 4.0 (clamped to MAX_ZOOM), not 100.0
- **VIOLATES Q1**: `viewport.set_zoom(0.01)` -- zoom becomes 0.1 (clamped to MIN_ZOOM), not 0.01
- **VIOLATES Q3**: After `viewport.zoom_in()` at MAX_ZOOM -- zoom stays at 4.0, not changes
- **VIOLATES Q5**: With NaN camera, after zoom -- camera should become finite (0.0)

### Invariant Violations
- **VIOLATES I1**: After any zoom operation, if zoom > 4.0 -- violates invariant

## Ownership Contracts (Rust-specific)
- `fn viewport.set_zoom(&mut self, zoom: f64)`
  - Mutates: `self.zoom`
  - Postconditions: zoom in [0.1, 4.0], returns false if no change made

- `fn viewport.zoom_in(&mut self)`
  - Mutates: `self.zoom` (multiplied by 1.25, clamped)
  - Postconditions: zoom in [0.1, 4.0], returns false if at MAX_ZOOM

- `fn viewport.zoom_out(&mut self)`
  - Mutates: `self.zoom` (multiplied by 0.8, clamped)
  - Postconditions: zoom in [0.1, 4.0], returns false if at MIN_ZOOM

## Non-goals
- [ ] Adding zoom limits to serialization/deserialization (already validated on load)
- [ ] Supporting zoom values outside float range (not a realistic use case)
- [ ] Adding Result types to zoom functions (graceful handling preferred for UX)
- [ ] UI command layer tests (Signal/History dependencies make unit testing impractical)
