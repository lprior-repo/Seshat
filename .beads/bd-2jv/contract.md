bead_id: bd-2jv
bead_title: tests: Implement GEO geometry tests (GEO-021 to GEO-030)
phase: p0
updated_at: 2026-03-01T22:05:00Z

# Contract: GEO-021 to GEO-030 Geometry Tests

## Scope

Add 10 geometry tests to `/home/lewis/src/seshat/diagram_tool/src/geometry/mod.rs`:

### Test Specifications

| ID | Test Name | Description |
|----|-----------|-------------|
| GEO-021 | `test_world_to_screen_round_trip` | Transform world coordinates to screen and back; verify round-trip preserves original within tolerance |
| GEO-022 | `test_aabb_at_various_angles` | AABB calculation for rectangles at 15, 30, 45, 60, 75 degrees; verify bounds contain all corners |
| GEO-023 | `test_rotation_then_resize_composition` | Apply rotation then resize; verify final corner positions are mathematically correct |
| GEO-024 | `test_resize_then_rotation_composition` | Apply resize then rotation; verify order matters and results differ from GEO-023 |
| GEO-025 | `test_repeated_tiny_transforms_no_drift` | Apply 1000 tiny rotations (0.001 rad each); verify cumulative error is bounded |
| GEO-026 | `test_repeated_tiny_scales_no_drift` | Apply 1000 tiny scales (1.001x each); verify cumulative error is bounded |
| GEO-027 | `test_camera_constraints_min_zoom` | Camera zoom clamped to minimum value (e.g., 0.1); verify zoom cannot go below |
| GEO-028 | `test_camera_constraints_max_zoom` | Camera zoom clamped to maximum value (e.g., 10.0); verify zoom cannot exceed |
| GEO-029 | `test_camera_pan_with_zoom` | Camera pan speed scales inversely with zoom; verify consistent screen-space movement |
| GEO-030 | `test_camera_world_to_screen_at_extremes` | World-to-screen transform at extreme coordinates (1e6, -1e6); verify finite results |

## Preconditions

- `diagram_tool/src/geometry/mod.rs` exists with existing GEO-001 to GEO-010 tests
- `Point`, `AABB`, `Rectangle` structs are available
- `scale_around_anchor`, `rotate_around_center` functions are available
- `#![deny(clippy::unwrap_used)]` is in effect

## Postconditions

- All 10 tests pass: `cargo test --package diagram_tool --lib geometry::tests::test_geo_02`
- No `unwrap()` or `expect()` usage
- All tests follow Given/When/Then pattern with comments
- Property-based tests use proptest where appropriate

## Invariants

- TOLERANCE constant (1e-10) used for floating-point comparisons
- All test functions return unit type `()`
- Tests are deterministic (no random without seeding)

## Acceptance Criteria

1. `moon run :test` passes
2. `moon run :ci` passes
3. Test coverage includes edge cases (zero, negative, extreme values)
4. Tests follow existing naming conventions in the module
