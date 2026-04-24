# QA-MANUAL Findings: canvas_domain/src/perf/wheel.rs

**Reviewer**: seshat/polecats/nuka
**Date**: 2026-04-24
**Scope**: Exploratory edge case discovery on wheel zoom/pan transform logic

## Files Reviewed

- `canvas_domain/src/perf/wheel.rs` (95 lines) - primary target
- `canvas_domain/src/perf/transforms.rs` (30 lines) - coord transforms
- `canvas_domain/src/perf/mod.rs` (21 lines) - module config
- `canvas_domain/src/perf/tests.rs` (675 lines) - test suite
- `canvas_domain/src/math.rs` (29 lines) - math primitives
- `canvas_math/src/lib.rs` (92 lines) - core math library

## Summary

The wheel module is well-structured and defensively coded. The public API path
through `wheel_update` has comprehensive NaN/Inf protection. However, several
edge cases exist in `wheel_transform` (the lower-level pub fn) and in the
epsilon comparison thresholds.

## Bugs Found

### BUG-1: `wheel_transform` produces NaN with NaN dx/dy in shift_pan mode

**Severity**: Medium (protected by `wheel_update` for public API, but unsafe as standalone call)
**Location**: `wheel.rs:25-35`

When `shift_pan=true` and `dx=0.0` and `dy=NaN`:
- Line 26: `input.dx.abs() > f64::EPSILON` → `0.0 > EPSILON` → false
- Falls to line 29: `wheel_delta = NaN`
- NaN propagates through factor computation
- `NaN.clamp(MIN_ZOOM, MAX_ZOOM)` returns NaN (clamp is unordered with NaN)
- Line 46: `(NaN - current_zoom).abs() <= f64::EPSILON` → false (NaN comparisons are false)
- Enters the camera recomputation branch with NaN zoom → all outputs NaN

**Impact**: `wheel_update` catches this via output finiteness check (line 79), but `wheel_transform` is `pub` and can be called directly from other code.

**Recommendation**: Add input validation guard in `wheel_transform` or make it `pub(crate)`.

### BUG-2: `f64::EPSILON` too strict for "no change" detection

**Severity**: Low (causes unnecessary recomputation, not incorrect results)
**Location**: `wheel.rs:46`, `wheel.rs:83-85`

`f64::EPSILON` ≈ 2.22e-16 is machine epsilon, not a meaningful threshold for
camera coordinates. At zoom boundaries where clamp snaps back to the limit:
- `current_zoom = 3.999999999999`, `next_zoom = 4.0` (clamped)
- Difference ≈ 1e-12 > EPSILON → triggers full camera recomputation
- This causes unnecessary tiny camera position changes at zoom limits

Similarly in `wheel_update` (lines 83-85), three EPSILON comparisons may
detect "change" for camera coordinates differing by ~1e-16, which is
sub-pixel noise.

**Recommendation**: Use a domain-appropriate threshold like `1e-10` for zoom
and `0.01` for camera coordinates, or use relative epsilon comparison.

### BUG-3: Missing zoom finiteness check in `wheel_update`

**Severity**: Low (safe_zoom defaults to 1.0, which is safe but surprising)
**Location**: `wheel.rs:67-75`

`wheel_update` validates client_x, client_y, dx, dy, camera_x, camera_y for
finiteness but does NOT validate `input.zoom`. When zoom is NaN or Inf:
- `safe_zoom(NaN)` returns None → defaults to 1.0
- The wheel event is processed as if zoom were 1.0, silently
- This is a semantic error: the user's zoom level is discarded

**Recommendation**: Add `!input.zoom.0.is_finite()` to the guard clause.

## Design Issues (Non-Bug)

### DESIGN-1: Discrete wheel zoom asymmetry

**Location**: `wheel.rs:38`

Zoom-in factor = 0.95, zoom-out factor = 1.05. These are NOT reciprocals
(1/0.95 ≈ 1.0526 ≠ 1.05). Over many discrete wheel clicks, zoom drifts
slightly toward zoom-out. This is typical for mouse wheel UX but worth noting.

### DESIGN-2: Stale "known issue" comment in tests

**Location**: `tests.rs:324-326`

Comment says "the implementation has a known issue where to_screen_coords uses
screen_to_canvas instead of canvas_to_screen". Inspecting `transforms.rs:13`,
`to_screen_coords` correctly calls `canvas_to_screen`. The roundtrip also
works correctly mathematically. This comment appears stale and should be removed.

### DESIGN-3: `to_canvas_coords` fallback ignores zoom

**Location**: `transforms.rs:7-10`

When `safe_zoom` fails (zoom <= EPSILON or not finite), the fallback is
`CanvasCoord(client.x() + camera.x(), client.y() + camera.y())`. This
ignores zoom entirely, producing incorrect coordinates for any zoom != 1.0.
In practice, this path is rarely reached because wheel.rs always clamps zoom
to valid range first.

## Positive Findings

1. **Zoom clamping is solid**: All paths clamp to `[MIN_ZOOM, MAX_ZOOM]` = `[0.1, 4.0]`
2. **NaN/Inf protection**: `wheel_update` has comprehensive guards on all 6 input floats + 3 output floats
3. **Numerically sound**: `mul_add` used for better precision on FMA-capable hardware
4. **Pure functions**: No state mutation, no side effects, easy to test and reason about
5. **`#[must_use]` annotations**: Prevents accidentally discarding return values
6. **Clippy lints**: Module denies unwrap/expect/panic in non-test builds
7. **Comprehensive tests**: 23 tests including 8 property-based tests, covering NaN, Inf, extremes, pinch gestures, and roundtrips

## Test Coverage Gaps

1. No test for `shift_pan=true` with `dx=0.0, dy=NaN` (BUG-1 scenario)
2. No test for zoom=NaN/Inf directly through `wheel_update` (BUG-3 scenario)
3. No test verifying exact roundtrip `to_canvas_coords` → `to_screen_coords` (test only checks finiteness)
4. No test for extremely large camera coordinates (e.g., camera_x = 1e15) where floating-point precision degrades

## Discovered Work

None beyond the findings above. No code changes required for this QA pass.
