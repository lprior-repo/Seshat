# Contract Specification

## Context
- **Feature**: GEO-020 Hit test margin respects zoom level
- **Domain terms**:
  - `SCREEN_HIT_MARGIN`: Constant screen-space hit margin in pixels (5.0)
  - `zoom`: Current zoom level (f64), range MIN_ZOOM (0.1) to MAX_ZOOM (4.0)
  - `hit_margin_world`: The computed margin in world coordinates used for hit testing
  - `screen-space behavior`: Hit margin remains constant in screen pixels regardless of zoom
- **Assumptions**:
  - MIN_ZOOM = 0.1, MAX_ZOOM = 4.0 (from viewport/mod.rs)
- **Open questions**: RESOLVED - Screen-space behavior is the CORRECT UX pattern
  - The margin should appear constant in screen pixels regardless of zoom level
  - At higher zoom, the world-space margin gets smaller (user sees same-sized hit area)

## Preconditions
- [P1] `zoom` must be within valid range [MIN_ZOOM, MAX_ZOOM]
- [P2] `SCREEN_HIT_MARGIN` must be a positive value (> 0)
- [P3] `point` coordinates must be finite f64 values

## Postconditions
- [Q1] When zoom = MIN_ZOOM (0.1): hit_margin_world must equal SCREEN_HIT_MARGIN / MIN_ZOOM = 50.0
- [Q2] When zoom = MAX_ZOOM (4.0): hit_margin_world must equal SCREEN_HIT_MARGIN / MAX_ZOOM = 1.25
- [Q3] When zoom = 1.0: hit_margin_world must equal SCREEN_HIT_MARGIN / 1.0 = 5.0
- [Q4] hit_margin_world must be monotonically decreasing as zoom increases

## Invariants
- [I1] Hit test at a fixed screen distance from a node edge must return consistent results when zoom changes (screen-space behavior)
- [I2] World-space margin decreases proportionally as zoom increases to maintain constant screen-space hit area

## Error Taxonomy
- **HitTestError::InvalidZoom** - when zoom is outside valid range [MIN_ZOOM, MAX_ZOOM]
- **HitTestError::InvalidMargin** - when SCREEN_HIT_MARGIN is not positive
- **HitTestError::InvalidPoint** - when point coordinates are not finite

## Contract Signatures

### Core Function (Screen-Space Behavior)
```rust
/// Computes hit margin in world coordinates from screen-space margin.
/// Screen-space behavior: margin appears constant in screen pixels regardless of zoom.
/// At higher zoom, world-space margin gets smaller.
///
/// # Preconditions:
/// - zoom must be in range [MIN_ZOOM, MAX_ZOOM]
/// - screen_margin must be > 0
///
/// # Postconditions:
/// - Returns screen_margin / zoom
/// - At MIN_ZOOM returns largest world margin
/// - At MAX_ZOOM returns smallest world margin
fn screen_to_world_margin(screen_margin: f64, zoom: f64) -> Result<f64, HitTestError>
```

### Hit Test Function
```rust
/// Determines if a point hits a rectangle with margin adjusted for zoom.
/// Uses screen-space behavior: same screen distance always triggers hit.
///
/// # Preconditions:
/// - point.x and point.y must be finite
/// - rect must be valid (width > 0, height > 0)
/// - zoom must be in range [MIN_ZOOM, MAX_ZOOM]
///
/// # Postconditions:
/// - Returns true if point is within rect expanded by hit_margin_world
/// - hit_margin_world = SCREEN_HIT_MARGIN / zoom
fn hit_test_with_margin(
    point: Point,
    rect: &Rectangle,
    zoom: f64,
    screen_margin: f64,
) -> Result<bool, HitTestError>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| zoom in [MIN_ZOOM, MAX_ZOOM] | Runtime-checked | `zoom.is_finite() && (MIN_ZOOM..=MAX_ZOOM).contains(&zoom)` |
| screen_margin > 0 | Runtime-checked | `screen_margin > 0.0` |
| point coords finite | Runtime-checked | `point.x.is_finite() && point.y.is_finite()` |
| rect valid | Compile-time (newtype) | `NonZeroRectangle` or validated constructor |

## Violation Examples (REQUIRED -- one per precondition and postcondition)
- VIOLATES P1: `screen_to_world_margin(5.0, 0.05)` with zoom=0.05 (below MIN_ZOOM) -- should produce `Err(HitTestError::InvalidZoom)`
- VIOLATES P1: `screen_to_world_margin(5.0, 5.0)` with zoom=5.0 (above MAX_ZOOM) -- should produce `Err(HitTestError::InvalidZoom)`
- VIOLATES P2: `screen_to_world_margin(0.0, 1.0)` with margin=0.0 -- should produce `Err(HitTestError::InvalidMargin)`
- VIOLATES P2: `screen_to_world_margin(-5.0, 1.0)` with margin=-5.0 -- should produce `Err(HitTestError::InvalidMargin)`
- VIOLATES P3: `hit_test_with_margin(Point::new(f64::NAN, 50.0), &rect, 1.0, 5.0)` -- should produce `Err(HitTestError::InvalidPoint)`
- VIOLATES Q1: `screen_to_world_margin(5.0, 0.1)` returns value != 50.0 -- should produce `Err(HitTestError::PostconditionViolation)`
- VIOLATES Q2: `screen_to_world_margin(5.0, 4.0)` returns value != 1.25 -- should produce `Err(HitTestError::PostconditionViolation)`

## Ownership Contracts (Rust-specific)
- `rect: &Rectangle` - shared borrow, read-only, no mutation
- `point: Point` - copy type (f64, f64), no ownership concerns
- `zoom: f64` - copy type, no ownership concerns

## Integration Test References
- This feature is exercised by integration tests in `diagram_tool/src/geometry/hit_test_tests.rs`
- The `diagram_tool/src/models/io_tests.rs` file contains tests for serialization of hit test parameters
- See GEO-020 integration coverage in the project's CI test suite

## Non-goals
- [ ] Handle rotated rectangles in hit test margin calculation (covered by GEO-020 separately)
- [ ] Handle multiple selection hit testing (out of scope for this contract)
- [ ] Handle touch vs mouse input differentiation (covered by existing handle hit tests)
- [ ] World-space constant behavior (rejected - screen-space is the correct UX pattern)

(End of file - total 147 lines)
