# Martin Fowler Test Plan

## Happy Path Tests
- test_repeated_tiny_rotations_no_drift: 1000 tiny rotations should equal single total rotation within epsilon
- test_repeated_tiny_scales_no_drift: 1000 tiny scales should equal single composed scale within relative error
- test_full_circle_rotation_returns_to_origin: 1000 rotations of 2π/1000 should return to starting position
- test_scale_up_then_down_inverse_returns_to_original: Scale by 1.001 then 1/1.001 repeatedly returns to original

## Edge Case Tests
- test_tiny_rotation_preserves_distance_from_center: Distance from rotation center preserved
- test_tiny_scale_preserves_direction_from_anchor: Direction from scale anchor preserved
- test_zero_rotation_returns_identical_point: Rotation by 0 returns same point
- test_unit_scale_returns_identical_point: Scale by 1.0 returns same point

## Contract Verification Tests
- test_drift_bounded_by_epsilon_rotation: After 1000 iterations, drift < 1e-6
- test_drift_bounded_by_epsilon_scale: After 1000 iterations, relative error < 1e-6
- test_drift_bounded_by_epsilon_full_circle: Full circle rotation drift < 1e-6
- test_drift_bounded_by_epsilon_scale_inverse: Scale up/down inverse drift < 1e-6

## Given-When-Then Scenarios

### Scenario 1: Repeated Tiny Rotations No Drift
**ID**: GEO-016-Rot1
Given: A point at (100, 0) and rotation center at origin
When: Applying 1000 rotations of 0.001 radians each (total ~57.3 degrees)
Then: Final position equals applying single rotation of 1.0 rad within drift < 1e-6

### Scenario 2: Repeated Tiny Scales No Drift  
**ID**: GEO-016-Scale1
Given: A point at (100, 0) and scale anchor at origin
When: Applying 1000 scales of 1.001 each (total factor = 1.001^1000)
Then: Final position equals applying single scale by total factor within relative error < 1e-6

### Scenario 3: Full Circle Rotation Returns to Origin
**ID**: GEO-016-Rot2
Given: A point at (100, 0) and rotation center at origin
When: Rotating 1000 times by 2π/1000 (full circle)
Then: Final position equals original position within drift < 1e-6

### Scenario 4: Scale Up Then Down Inverse
**ID**: GEO-016-Scale2
Given: A point at (100, 50) and scale anchor at origin
When: Applying 500 iterations of scale by 1.001 then scale by 1/1.001
Then: Final position equals original position within relative error < 1e-6

### Scenario 5: Combined Scale Then Rotate Composition
**ID**: GEO-016-Combined
Given: A point at (100, 0), scale anchor at origin, rotation center at origin
When: Applying scale by 2.0 then rotate by π/2 (180 degrees)
Then: Result equals expected composed transform within epsilon

## Test Implementation Reference
Location: `diagram_tool/src/geometry/mod.rs` (existing tests at lines ~1428-1520)
- `test_repeated_tiny_transforms_no_drift` - GEO-025 pattern
- `test_repeated_tiny_rotations_full_circle` - GEO-025 pattern  
- `test_repeated_tiny_scales_no_drift` - GEO-026 pattern
- `test_repeated_tiny_scales_inverse` - GEO-026 pattern
