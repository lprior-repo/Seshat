# Contract Specification

## Context
- Feature: GEO-016 Floating Point Drift Prevention
- Domain terms: 
  - `Point` - 2D point with f64 x, y coordinates
  - `rotate_around_center(point, center, angle)` - rotate point around center
  - `scale_around_anchor(point, anchor, factor)` - scale point around anchor
  - `scale_then_rotate(point, anchor, scale, angle)` - compose scale then rotate
  - `drift` - accumulated floating point error after many operations
- Assumptions:
  - f64 precision (~15-16 decimal digits)
  - Standard IEEE 754 floating point arithmetic
  - Existing component-wise transforms already implemented
- Open questions: None - specification based on existing implementation pattern

## Preconditions
- P1: `rotate_around_center` requires valid f64 values (no NaN, no Inf)
- P2: `scale_around_anchor` requires valid f64 values (no NaN, no Inf)
- P3: `scale_then_rotate` requires valid f64 values (no NaN, no Inf)

## Postconditions
- Q1: `rotate_around_center` returns Point where distance from expected < 1e-6 after 1000 iterations
- Q2: `scale_around_anchor` returns Point where relative error < 1e-6 after 1000 iterations
- Q3: Full circle rotation (2π) returns to within 1e-6 of original position
- Q4: Scale up then scale down by inverse returns to within relative error < 1e-6

## Invariants
- I1: Transform functions are pure - same input always produces same output
- I2: Transform functions preserve dimension (2D stays 2D)
- I3: Drift is bounded by epsilon comparison - never grows unbounded

## Error Taxonomy
- No error types needed - all operations are pure math with bounded drift
- This is a property verification, not an fallible operation

## Contract Signatures
```rust
// Pure functions - no error returns, drift is bounded property
fn rotate_around_center(point: Point, center: Point, angle_radians: f64) -> Point
fn scale_around_anchor(point: Point, anchor: Point, factor: f64) -> Point
fn scale_then_rotate(point: Point, anchor: Point, scale_factor: f64, angle_radians: f64) -> Point
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Valid f64 values | Runtime check | `f64::is_finite()` debug_assert |
| N/A - pure functions | N/A | N/A |

## Violation Examples
- VIOLATES Q1: 1000 rotations of 0.001 rad should equal single 1.0 rad rotation within 1e-6
- VIOLATES Q2: 1000 scales of 1.001 should equal single scale of 1.001^1000 within relative error 1e-6
- VIOLATES Q3: Point rotated 1000 times by 2π/1000 should return to original within 1e-6
- VIOLATES Q4: Scale 1.001 then scale (1/1.001) 1000 times should return to original

## Ownership Contracts
- All functions take `Point` by value and return new `Point`
- No mutation - all operations are immutable
- Clone semantics: Point implements Copy (f64 fields)

## Non-goals
- Error handling for invalid inputs (not applicable for pure math)
- Matrix-based transformations (using component-wise approach)
- Numeric stability for extreme values (focus on common use case: repeated small transforms)
