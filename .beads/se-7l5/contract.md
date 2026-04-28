# GEO-001 to GEO-030: Geometry and Transform Math

## Invariant
- All transforms must be invertible (round-trip identity)
- All output coordinates must be finite (no NaN/Infinity)
- OrderedFloat semantics for all diagram coordinates
- Zero panics/unwrap in production code

## Test Specifications

### Coordinate Transforms
| ID   | Description | Functions | Key Property |
|------|-------------|-----------|--------------|
| GEO-001 | AABB for axis-aligned rectangles | `Rectangle::aabb()` | AABB == rect bounds |
| GEO-002 | AABB for rotated rectangles | `Rectangle::aabb()` with rotation | AABB contains all corners |
| GEO-003 | Stroke width inclusion in bounds | `AABB::expand()` | expanded by stroke/2 |
| GEO-004 | Text bounds calculation | Text metrics | width/height finite |
| GEO-005 | Image bounds calculation | Image metrics | width/height finite |
| GEO-006 | Scale around anchor point | `scale_around_anchor()` | anchor invariant |
| GEO-007 | Rotate around center | `rotate_around_center()` | center invariant |
| GEO-008 | Resize with aspect ratio lock | `resize_with_aspect_lock()` | ratio preserved |
| GEO-009 | Combined transform chain | `scale_then_rotate()` | sequential compose |
| GEO-010 | Bounds edge cases | `safe_bounds()` | rejects NaN/Inf |
| GEO-011 | Rotation+resize composition | compose both | finite outputs |
| GEO-012 | Zoom at pointer | `zoom_at_pointer()` | pointer stays fixed |
| GEO-013 | Snap lines horizontal | `snap_horizontal()` | nearest target |
| GEO-014 | Snap lines vertical | `snap_vertical()` | nearest target |
| GEO-015 | Grid step | `GridSize` validation | valid range |
| GEO-016 | Edge routing orthogonal | `compute_orthogonal_route()` | is_orthogonal |
| GEO-017 | Edge routing avoid obstacle | `compute_orthogonal_route_avoiding()` | no intersection |
| GEO-018 | Fit to content | `fit_to_viewport()` | content fits |
| GEO-019 | Hit test with margin | `hit_test_rect()` / `hit_test_with_margin()` | zoom-aware |
| GEO-020 | Hit test rotated shape | `hit_test_rotated_rect()` | inverse rotation |
| GEO-021 | Line intersection | `line_line_intersection()` | cross/parallel/collinear |
| GEO-022 | AABB at various angles | `Rectangle::aabb()` | monotonically grows |
| GEO-023 | Rotation then resize | compose order | order matters |
| GEO-024 | Resize then rotation | compose order | order matters |
| GEO-025 | Repeated tiny transforms rotation drift | N * small rotation | bounded drift |
| GEO-026 | Repeated tiny scales scale drift | N * small scale | bounded drift |
| GEO-027 | Camera constraints min zoom | `MIN_ZOOM`, `safe_zoom()` | clamped |
| GEO-028 | Camera constraints max zoom | `MAX_ZOOM`, `sanitize_zoom()` | clamped |
| GEO-029 | Camera pan with zoom | screen_to_canvas | inverse proportional |
| GEO-030 | Camera world-to-screen at extremes | world_to_screen | finite at extremes |

### Transform Function Contracts

```
scale_around_anchor(point, anchor, factor) -> Point
  PRE: factor > 0 (or any finite f64)
  POST: anchor unchanged, result finite
  PROP: scale(anchor, anchor, f) == anchor

rotate_around_center(point, center, angle) -> Point
  PRE: angle finite
  POST: center unchanged, result finite
  PROP: rotate(rotate(p, c, a), c, -a) == p (within tolerance)

world_to_screen(world, camera, zoom) -> Point
  PRE: zoom > 0
  POST: result finite
  PROP: screen_to_world(world_to_screen(w,c,z),c,z) == w

screen_to_world(screen, camera, zoom) -> Point
  PRE: zoom > 0
  POST: result finite
  PROP: world_to_screen(screen_to_world(s,c,z),c,z) == s
```

### Proptest Properties
1. Transform inverse: round-trip world->screen->world identity
2. Coordinate finiteness: all outputs finite for finite inputs
3. Scale anchor invariance: anchor point unchanged after scale
4. Rotation center invariance: center point unchanged after rotation
5. Aspect ratio preservation after resize_with_aspect_lock
6. safe_bounds rejects NaN/Infinity, accepts finite
