# Contract Specification: Geometry Math Tests (GEO-001 to GEO-030)

**Bead ID**: bd-2qj
**Title**: geometry: Implement geometry math tests (GEO-001 to GEO-030)
**Priority**: P2
**Type**: feature

## Overview

This contract specifies the geometry math operations and test coverage for the diagram tool. All 30 geometry test categories (GEO-001 to GEO-030) are implemented with 225+ individual test cases.

## Core Types

### Point
```rust
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

**Invariants:**
- x and y must be finite for valid operations
- Origin point (0.0, 0.0) is the default

### AABB (Axis-Aligned Bounding Box)
```rust
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}
```

**Invariants:**
- min_x <= max_x (enforced by safe_bounds)
- min_y <= max_y (enforced by safe_bounds)
- All coordinates must be finite for valid AABB

### Rectangle
```rust
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64, // radians
}
```

**Invariants:**
- rotation is in radians
- width and height should be positive for meaningful bounds

## Function Contracts

### GEO-001, GEO-002: AABB Calculation

```rust
pub fn Rectangle::aabb(&self) -> AABB
```

**Preconditions:** None (handles all edge cases)
**Postconditions:**
- Returns AABB containing all four corners of the rectangle
- For axis-aligned rectangles, AABB equals the rectangle bounds
- For rotated rectangles, AABB is the minimal axis-aligned box containing all corners

### GEO-003: Stroke Width Inclusion

```rust
pub fn StrokedShape::bounds_with_stroke(&self) -> AABB
```

**Preconditions:** None
**Postconditions:**
- Bounds expanded by stroke_width / 2.0 on all sides

### GEO-006: Scale Around Anchor

```rust
pub fn scale_around_anchor(point: Point, anchor: Point, factor: f64) -> Point
```

**Preconditions:** None
**Postconditions:**
- Anchor point remains unchanged when scaled
- Distance from anchor scales by factor

### GEO-007: Rotate Around Center

```rust
pub fn rotate_around_center(point: Point, center: Point, angle_radians: f64) -> Point
```

**Preconditions:** None
**Postconditions:**
- Center point remains unchanged when rotated
- Distance from center is preserved

### GEO-008: Resize with Aspect Lock

```rust
pub fn resize_with_aspect_lock(original_width: f64, original_height: f64, new_width: f64) -> f64
```

**Preconditions:** original_width > 0
**Postconditions:**
- Returned height maintains aspect ratio
- Returns new_width if original_width <= 0

### GEO-010: Safe Bounds

```rust
pub fn safe_bounds(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Option<AABB>
```

**Preconditions:** None
**Postconditions:**
- Returns None if any input is NaN or infinity
- Returns Some(AABB) with corrected min/max ordering
- All coordinates in returned AABB are finite

### GEO-012: Zoom at Pointer

```rust
pub fn zoom_at_pointer(view_center: Point, pointer: Point, factor: f64) -> Point
```

**Preconditions:** None
**Postconditions:**
- Pointer position remains fixed in world space after zoom
- View center moves relative to pointer

### GEO-013, GEO-014: Snap Lines

```rust
pub fn snap_horizontal(line_y: f64, targets: &[f64], tolerance: f64) -> Option<f64>
pub fn snap_vertical(line_x: f64, targets: &[f64], tolerance: f64) -> Option<f64>
```

**Preconditions:** tolerance >= 0
**Postconditions:**
- Returns Some(nearest_target) if within tolerance
- Returns None if no target within tolerance

### GEO-015: Grid Snap

```rust
pub fn snap_to_grid(point: Point, grid_size: f64) -> Point
```

**Preconditions:** grid_size > 0
**Postconditions:**
- Returned point is on grid intersection
- Distance to original point is minimized

## Test Categories

| ID | Category | Test Count | Description |
|----|----------|------------|-------------|
| GEO-001 | AABB Axis-Aligned | 2 | Axis-aligned rectangle bounds |
| GEO-002 | AABB Rotated | 3 | Rotated rectangle bounds |
| GEO-003 | Stroke Width | 2 | Stroke inclusion in bounds |
| GEO-004 | Text Bounds | 8 | Text bounds with Unicode |
| GEO-005 | Image Bounds | 2 | Image dimension bounds |
| GEO-006 | Scale Anchor | 4 | Scaling around anchor point |
| GEO-007 | Rotate Center | 5 | Rotation around center |
| GEO-008 | Aspect Lock | 3 | Aspect ratio preservation |
| GEO-009 | Combined Transform | 2 | Scale then rotate |
| GEO-010 | Safe Bounds | 8 | Edge case handling |
| GEO-011 | Rotation+Resize | 3 | Composition tests |
| GEO-012 | Zoom Pointer | 3 | Zoom at pointer position |
| GEO-013 | Snap Horizontal | 3 | Horizontal line snapping |
| GEO-014 | Snap Vertical | 2 | Vertical line snapping |
| GEO-015 | Grid Step | 3 | Grid snapping |
| GEO-016 | Edge Routing | 3 | Orthogonal routing |
| GEO-017 | Avoid Obstacle | 2 | Route obstacle avoidance |
| GEO-018 | Fit Content | 4 | Fit-to-viewport calculations |
| GEO-019 | Hit Test Margin | 4 | Hit testing with margin |
| GEO-020 | Hit Test Rotated | 4 | Rotated shape hit testing |
| GEO-021 | World-Screen | 3 | Coordinate transforms |
| GEO-022 | AABB Angles | 3 | AABB at various angles |
| GEO-023 | Rotate+Resize | 2 | Transform composition |
| GEO-024 | Resize+Rotate | 2 | Transform order |
| GEO-025 | Rotation Drift | 2 | Float drift bounds |
| GEO-026 | Scale Drift | 2 | Scale drift bounds |
| GEO-027 | Min Zoom | 2 | Camera constraints |
| GEO-028 | Max Zoom | 3 | Camera constraints |
| GEO-029 | Pan Zoom | 3 | Pan with zoom |
| GEO-030 | Extremes | 3 | Extreme coordinate handling |

## Error Handling

All functions use `Option<T>` or direct returns (never panic):

1. **NaN/Infinity**: `safe_bounds` returns `None` for invalid inputs
2. **Zero dimensions**: Handled gracefully, returns valid bounds
3. **Negative dimensions**: Treated as valid coordinates
4. **Edge cases**: All math operations handle edge cases without panic

## Property-Based Tests

The following properties are verified using `proptest`:

1. `prop_scale_around_anchor_idempotent_at_anchor` - Anchor stays fixed
2. `prop_rotate_around_center_idempotent_at_center` - Center stays fixed
3. `prop_rotate_full_circle_returns_to_origin` - 360 degree rotation preserves position
4. `prop_aabb_contains_all_corners` - AABB always contains all corners
5. `prop_aspect_ratio_preserved` - Aspect ratio maintained during resize
6. `prop_safe_bounds_finite_always_succeeds` - Finite inputs produce valid AABB

## Quality Requirements

1. **Zero unwrap/panic**: All functions handle edge cases gracefully
2. **Pure functions**: All math operations are pure (no side effects)
3. **Finite outputs**: All outputs are finite for finite inputs
4. **Deterministic**: Same inputs always produce same outputs
