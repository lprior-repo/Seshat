bead_id: bd-2jv
bead_title: tests: Implement GEO geometry tests (GEO-021 to GEO-030)
phase: p1
updated_at: 2026-03-01T22:10:00Z

# Implementation: GEO-021 to GEO-030 Geometry Tests

## Summary

Added 10 geometry test functions (GEO-021 to GEO-030) to `/home/lewis/src/seshat/diagram_tool/src/geometry/mod.rs`:

| ID | Test Function | Description |
|----|---------------|-------------|
| GEO-021 | `test_world_to_screen_round_trip` | World-to-screen coordinate transformation round-trip |
| GEO-021 | `test_world_to_screen_round_trip_at_origin` | Round-trip at origin |
| GEO-021 | `test_world_to_screen_round_trip_high_zoom` | Round-trip at high zoom (10x) |
| GEO-022 | `test_aabb_at_various_angles` | AABB at 15, 30, 45, 60, 75 degrees |
| GEO-022 | `test_aabb_at_15_degrees` | AABB area comparison at 15 degrees |
| GEO-022 | `test_aabb_at_60_degrees` | AABB corner containment at 60 degrees |
| GEO-023 | `test_rotation_then_resize_composition` | Rotate then scale transform composition |
| GEO-023 | `test_rotation_then_resize_45_degrees` | Rotate 45 degrees then scale |
| GEO-024 | `test_resize_then_rotation_composition` | Scale then rotate transform composition |
| GEO-024 | `test_transform_order_matters` | Verify transform order affects result |
| GEO-025 | `test_repeated_tiny_transforms_no_drift` | 1000 tiny rotations drift bounded |
| GEO-025 | `test_repeated_tiny_rotations_full_circle` | Full circle rotation drift |
| GEO-026 | `test_repeated_tiny_scales_no_drift` | 1000 tiny scales drift bounded |
| GEO-026 | `test_repeated_tiny_scales_inverse` | Inverse scale cancellation drift |
| GEO-027 | `test_camera_constraints_min_zoom` | Zoom clamped to 0.1 minimum |
| GEO-027 | `test_camera_constraints_min_zoom_exact` | Zoom at exact minimum unchanged |
| GEO-028 | `test_camera_constraints_max_zoom` | Zoom clamped to 10.0 maximum |
| GEO-028 | `test_camera_constraints_max_zoom_exact` | Zoom at exact maximum unchanged |
| GEO-028 | `test_camera_constraints_valid_range` | Valid zoom values unchanged |
| GEO-029 | `test_camera_pan_with_zoom` | Pan speed inversely proportional to zoom |
| GEO-029 | `test_camera_pan_consistent_screen_movement` | Screen movement consistency |
| GEO-029 | `test_camera_pan_at_min_zoom` | Pan at minimum zoom |
| GEO-030 | `test_camera_world_to_screen_at_extremes` | Extreme coords (1e6) finite results |
| GEO-030 | `test_camera_world_to_screen_at_extremes_with_zoom` | Extreme coords with 10x zoom |
| GEO-030 | `test_camera_round_trip_at_extremes` | Round-trip at extreme coordinates |

## Helper Functions Added

- `world_to_screen(world: Point, camera: Point, zoom: f64) -> Point`
- `screen_to_world(screen: Point, camera: Point, zoom: f64) -> Point`
- `clamp_zoom(zoom: f64) -> f64`

## Constants Added

- `MIN_ZOOM: f64 = 0.1`
- `MAX_ZOOM: f64 = 10.0`

## Compliance

- All tests follow Given/When/Then pattern with comments
- No `unwrap()` or `expect()` usage
- Uses existing `TOLERANCE` constant (1e-10)
- All test functions return unit type `()`
- Tests are deterministic
