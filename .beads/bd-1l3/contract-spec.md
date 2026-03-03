# Contract Specification: Viewport Operations (bd-1l3)

## Overview

This bead implements the Viewport/Camera test category (CAM-001 to CAM-012) for the Seshat Diagram Tool. Viewport operations manage the camera transformation between screen coordinates and world coordinates, enabling pan, zoom, and coordinate transformations.

## Design by Contract

### Preconditions (P1-P7)

| ID | Precondition | Enforcement |
|----|-------------|-------------|
| P1 | Zoom value must be finite and positive | Runtime check with default fallback |
| P2 | Camera coordinates must be finite | Runtime check with clamping |
| P3 | Viewport dimensions must be positive | Runtime check with minimum 1.0 |
| P4 | Coordinate transforms require valid zoom/pan state | Runtime check |
| P5 | Zoom bounds: 0.1 <= zoom <= 4.0 | Clamped at boundaries |
| P6 | Fit-to-viewport requires valid content bounds | Returns default if invalid |
| P7 | Pan delta must be finite | Runtime check |

### Postconditions (Q1-Q7)

| ID | Postcondition | Verification |
|----|--------------|--------------|
| Q1 | After zoom: new zoom within [0.1, 4.0] | Assert in tests |
| Q2 | After pan: camera coordinates are finite | Assert in tests |
| Q3 | Screen-to-world is inverse of world-to-screen | Property test |
| Q4 | Fit-to-viewport preserves aspect ratio | Assert in tests |
| Q5 | Zoom around point keeps point under cursor | Assert in tests |
| Q6 | State changes increment revision | Assert in tests |
| Q7 | Operations are idempotent at boundaries | Property test |

### Invariants (I1-I5)

| ID | Invariant | Description |
|----|-----------|-------------|
| I1 | `0.1 <= zoom <= 4.0` | Zoom is always clamped |
| I2 | `camera_x.is_finite()` | Camera X is always finite |
| I3 | `camera_y.is_finite()` | Camera Y is always finite |
| I4 | Coordinate transforms are reversible | screen_to_world(world_to_screen(p)) ~= p |
| I5 | Viewport dimensions are positive | width > 0, height > 0 |

## Test Cases

### CAM-001: Pan Viewport (Basic)

**Given**: A viewport at origin (0, 0) with zoom 1.0
**When**: User pans by delta (100, 50)
**Then**: Camera position updates to (-100, -50)

```rust
// World moves opposite to pan direction
// Pan right 100px => world appears to move left => camera_x decreases
```

### CAM-002: Pan with Bounds Checking

**Given**: A viewport at position near boundary
**When**: User pans beyond reasonable bounds
**Then**: Camera clamps to maximum pan distance

```rust
// Maximum pan: +/- 10000 world units from origin
```

### CAM-003: Zoom In Operation

**Given**: A viewport at zoom 1.0
**When**: User zooms in (factor 1.25x)
**Then**: Zoom becomes 1.25, centered on viewport

### CAM-004: Zoom Out Operation

**Given**: A viewport at zoom 1.0
**When**: User zooms out (factor 0.8x)
**Then**: Zoom becomes 0.8, centered on viewport

### CAM-005: Zoom to Specific Level

**Given**: A viewport at any zoom level
**When**: User sets zoom to 2.0
**Then**: Zoom becomes 2.0, centered on viewport

### CAM-006: Zoom with Bounds

**Given**: A viewport at zoom 4.0 (maximum)
**When**: User tries to zoom in further
**Then**: Zoom stays at 4.0 (no change, returns false)

### CAM-007: Screen to World Transform

**Given**: A viewport with camera (100, 200) and zoom 2.0
**When**: Converting screen point (400, 300)
**Then**: World point is calculated correctly

```rust
// world_x = camera_x + screen_x / zoom
// world_y = camera_y + screen_y / zoom
```

### CAM-008: World to Screen Transform

**Given**: A viewport with camera (100, 200) and zoom 2.0
**When**: Converting world point (200, 350)
**Then**: Screen point is calculated correctly

```rust
// screen_x = (world_x - camera_x) * zoom
// screen_y = (world_y - camera_y) * zoom
```

### CAM-009: Fit Content to Viewport

**Given**: Content with bounds AABB(0, 0, 500, 400)
**When**: Fitting to viewport (800, 600) with padding 20
**Then**: Scale and offset calculated to center content

### CAM-010: Center on Specific Point

**Given**: A viewport at camera (0, 0)
**When**: Centering on world point (250, 300) with viewport (800, 600)
**Then**: Camera moves to center that point

```rust
// camera_x = point_x - viewport_width / 2 / zoom
// camera_y = point_y - viewport_height / 2 / zoom
```

### CAM-011: Zoom Around Point

**Given**: A viewport at zoom 1.0 with mouse at screen (400, 300)
**When**: Zooming to 2.0x
**Then**: Point under mouse stays at same world position

```rust
// Zoom should keep the point under cursor stationary
```

### CAM-012: Viewport State Persistence

**Given**: A viewport with camera (100, 200) and zoom 1.5
**When**: Serializing and deserializing state
**Then**: State is preserved exactly

## Module Structure

```
diagram_tool/src/viewport/
  mod.rs           - Module exports and core ViewportState
  transform.rs     - Coordinate transformation functions
  operations.rs    - Pan, zoom, fit operations
  tests.rs         - CAM-001 to CAM-012 test implementations
```

## Dependencies

- `geometry` module for AABB and Point
- `models::document` for EditorState
- No external dependencies beyond std

## Error Handling

All functions return `Result` or `Option` types. No panics or unwraps allowed.

- Invalid zoom values are clamped to valid range
- Invalid coordinates default to 0.0
- Fit operations return None for empty/invalid content
