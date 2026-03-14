# Martin Fowler Test Plan: seshat-x68 - MUL-016 to MUL-020 Multi-select rotation

## Happy Path Tests
- test_rotate_single_node_90_degrees
- test_rotate_two_nodes_180_degrees
- test_rotate_asymmetric_selection_preserves_shape
- test_rotate_selection_around_centroid_unchanged
- test_rotate_full_360_degrees_returns_to_original

## Error Path Tests
- test_returns_error_when_node_not_found
- test_returns_error_when_item_locked
- test_returns_error_when_invalid_hierarchy
- test_returns_error_when_angle_is_nan
- test_returns_error_when_angle_is_infinity

## Edge Case Tests
- test_handles_single_node_selection
- test_handles_two_nodes
- test_handles_collinear_nodes
- test_handles_empty_selection_wrapper (via NonEmptyVec compile-time enforcement)
- test_handles_subpixel_precision_coordinates

## Contract Verification Tests
- test_precondition_selection_not_empty_compile_time
- test_precondition_nodes_exist_runtime
- test_precondition_nodes_not_locked_runtime
- test_precondition_angle_finite_runtime
- test_postcondition_centroid_unchanged_after_rotation
- test_postcondition_relative_distances_preserved
- test_postcondition_node_count_unchanged
- test_invariant_rotation_then_reverse_returns_original

## Contract Violation Tests
- test_p3_violation_locked_node_returns_item_locked_error
  Given: Document with one locked node in selection
  When: rotate_selection is called with that selection
  Then: returns Err(Error::ItemLocked)

- test_p5_violation_nan_angle_returns_invalid_rotation_error
  Given: Valid selection with finite nodes
  When: rotate_selection is called with angle = f64::NAN
  Then: returns Err(Error::InvalidRotation)

- test_p5_violation_infinity_angle_returns_invalid_rotation_error
  Given: Valid selection with finite nodes
  When: rotate_selection is called with angle = f64::INFINITY
  Then: returns Err(Error::InvalidRotation)

- test_q2_violation_centroid_must_remain_unchanged
  Given: Multi-selection with asymmetric distribution
  When: rotate_selection is called
  Then: centroid of all rotated node positions equals original centroid (within floating-point tolerance)

- test_q3_violation_relative_distances_must_preserve
  Given: Multi-selection with multiple nodes
  When: rotate_selection is called
  Then: pairwise distances between all nodes equal original distances (within floating-point tolerance)

## MUL-016: Rotate Asymmetric Selection (Specific Tests)
- test_mul_016_irregular_distribution_around_selection_center
- test_mul_016_triangular_selection_rotates_correctly
- test_mul_016_four_corner_selection_rotates_correctly

## MUL-017: Rotate Preserves Relative Distances (Specific Tests)
- test_mul_017_all_pairwise_distances_preserved
- test_mul_017_distance_matrix_unchanged_after_rotation
- test_mul_017_relative_positions_maintained

## MUL-018: Rotate Snaps to 90-Degree Increments (Specific Tests)
- test_mul_018_snap_0_degrees_near_zero
- test_mul_018_snap_90_degrees_near_pi_over_2
- test_mul_018_snap_180_degrees_near_pi
- test_mul_018_snap_270_degrees_near_3pi_over_2
- test_mul_018_snap_within_tolerance_boundary

## MUL-019: Rotate with Subpixel Precision (Specific Tests)
- test_mul_019_fractional_coordinates_rotate_correctly
- test_mul_019_subpixel_precision_maintained
- test_mul_019_very_small_rotations_preserved

## MUL-020: Rotate Edge Cases (Specific Tests)
- test_mul_020_single_node_rotates_around_own_center
- test_mul_020_two_nodes_opposite_sides
- test_mul_020_collinear_nodes_horizontal
- test_mul_020_collinear_nodes_vertical
- test_mul_020_empty_selection_rejected_at_compile_time

## Given-When-Then Scenarios

### Scenario 1: Rotate asymmetric selection
Given: Three nodes at positions (0,0), (10,0), (5,10) forming a triangle
When: rotate_selection is called with 90-degree rotation around centroid
Then:
- Each node is rotated 90 degrees around the centroid
- The centroid remains at the same position
- The shape (triangle) is preserved but rotated

### Scenario 2: Rotate preserves all relative distances
Given: Four nodes at corners of a rectangle (0,0), (100,0), (100,50), (0,50)
When: rotate_selection is called with any angle
Then:
- Distance between every pair of nodes equals the original distance
- Rectangle becomes rotated rectangle but dimensions unchanged

### Scenario 3: Snap to cardinal directions
Given: Rotation angle of 89 degrees
When: snap_angle_to_cardinal is called with 5-degree tolerance
Then:
- Returns 90 degrees (snaps to nearest cardinal)

### Scenario 4: Locked node prevents rotation
Given: Selection containing one locked node
When: rotate_selection is attempted
Then:
- Returns Error::ItemLocked
- No nodes in document are modified

### Scenario 5: Invalid rotation angle rejected
Given: Selection with valid nodes
When: rotate_selection is called with angle = NaN
Then:
- Returns Error::InvalidRotation
- Document state unchanged
