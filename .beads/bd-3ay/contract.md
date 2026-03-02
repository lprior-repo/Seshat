bead_id: bd-3ay
bead_title: edge-case-bdd-tests-geometry-boundaries
phase: p0
updated_at: 2026-03-02T04:45:30Z

# Contract: Edge-Case BDD Tests for Geometry Boundary Conditions

## Overview

Add comprehensive BDD (Behavior-Driven Development) tests for geometry boundary
conditions in the diagram tool. These tests verify robust handling of edge cases
that may cause numerical instability, overflow, or unexpected behavior.

## Scope

The tests shall cover the following geometry boundary conditions:

### 1. Zero Dimensions (GEO-EDGE-001)
- Rectangles with zero width
- Rectangles with zero height
- Rectangles with both zero width and height
- Images with zero dimensions
- AABB with zero area

**Expected Behavior**: Operations on zero-dimension shapes should produce
deterministic results without panicking. AABB calculations should return valid
degenerate bounds.

### 2. Maximum Rotation Values (GEO-EDGE-002)
- Rotation at 2*pi (full circle)
- Rotation beyond 2*pi (e.g., 3*pi, 10*pi)
- Negative rotation values
- Rotation at pi/2, pi, 3*pi/2 boundaries
- Very large rotation values (near f64 limits)

**Expected Behavior**: Rotation should be mathematically correct. AABB
calculations for rotated shapes should contain all corners regardless of
rotation angle.

### 3. Negative Dimensions (GEO-EDGE-003)
- Negative width values
- Negative height values
- Mixed positive/negative dimensions
- Scaling to negative dimensions

**Expected Behavior**: The system should handle negative dimensions gracefully
through either clamping, normalization, or explicit rejection.

### 4. Infinite Coordinates (GEO-EDGE-004)
- Points at positive/negative infinity
- AABB with infinite bounds
- Operations involving infinity values
- Transformations of infinite coordinates

**Expected Behavior**: `safe_bounds()` should reject infinite values. Operations
should detect and handle infinity without producing NaN unexpectedly.

### 5. Stroke Width Boundaries (GEO-EDGE-005)
- Zero stroke width
- Negative stroke width
- Very large stroke width (larger than shape)
- Stroke width with zero-dimension shapes

**Expected Behavior**: Stroke width expansion should handle edge cases without
producing invalid bounds.

## Test Format

All tests shall follow the Given-When-Then BDD pattern:

```rust
#[test]
fn test_<feature>_<scenario>() {
    // Given: <preconditions>
    // When: <action>
    // Then: <expected outcome>
}
```

## Acceptance Criteria

1. All tests in GEO-EDGE-001 through GEO-EDGE-005 must pass
2. Tests must use the existing `TOLERANCE` constant (1e-10) for floating-point comparisons
3. Tests must not use `unwrap()`, `expect()`, or `panic!()` (per module lints)
4. Property-based tests using proptest are encouraged for boundary exploration
5. Tests should cover both success and failure paths where applicable

## Dependencies

- Existing geometry primitives: `Point`, `AABB`, `Rectangle`, `StrokedShape`, `Image`, `Text`
- Existing functions: `safe_bounds()`, `rotate_around_center()`, `scale_around_anchor()`
- Standard library: `std::f64::consts::PI`, `f64::INFINITY`, `f64::NAN`

## Non-Goals

- UI-level testing (these are unit tests for the geometry module)
- Performance benchmarking
- Visual rendering verification
