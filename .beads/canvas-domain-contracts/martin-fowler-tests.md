# Martin Fowler Test Plan

## Happy Path Tests
- `test_translates_node_by_delta_successfully`
- `test_resizes_node_to_valid_dimensions_successfully`
- `test_reparents_node_to_new_parent_preserving_absolute_position`
- `test_snaps_coordinates_to_grid_correctly`
- `test_maps_viewport_coordinates_to_canvas_coordinates`

## Error Path Tests
- `test_translate_fails_when_node_not_found`
- `test_reparent_fails_when_target_parent_not_found`
- `test_reparent_fails_when_cycle_detected`
- `test_resize_creation_fails_when_dimensions_invalid`

## Edge Case Tests
- `test_translates_with_zero_delta_causes_no_change`
- `test_resizes_to_exact_minimum_allowed_dimensions`
- `test_reparents_node_to_root_level_from_deep_hierarchy`
- `test_snapping_exactly_on_grid_line_does_not_shift`

## Contract Verification Tests
- `test_precondition_node_must_exist`
- `test_precondition_no_cycles_in_hierarchy`
- `test_postcondition_translation_applied_correctly`
- `test_postcondition_absolute_position_preserved_on_reparent`
- `test_postcondition_mapping_is_reversible`
- `test_invariant_selection_contains_only_existing_nodes`

## Contract Violation Tests

- `test_p1_violation_returns_node_not_found`
  Given: `translate_node(NodeId(999), Vector(10.0, 10.0))` -- where NodeId(999) does not exist in the canvas
  When: function is called with non-existent node ID
  Then: returns `Err(CanvasError::NodeNotFound(NodeId(999)))` -- NOT a panic, NOT an unwrap failure

- `test_p2_violation_returns_cycle_detected`
  Given: `reparent_node(NodeId(1), Some(NodeId(1)))` -- where NodeId(1) tries to parent itself
  When: a node attempts to become its own parent (or an ancestor of itself)
  Then: returns `Err(CanvasError::CycleDetected(NodeId(1), NodeId(1)))` -- NOT a panic, NOT an unwrap failure

- `test_p3_violation_returns_invalid_grid_resolution`
  Given: `NonZeroPositiveF64::new(0.0)`
  When: grid resolution is zero or negative
  Then: returns `Err(CanvasError::InvalidGridResolution(0.0))` -- NOT a panic, NOT an unwrap failure

- `test_p4_violation_returns_invalid_bounding_box`
  Given: `ValidBoundingBox::new(0.0, 0.0, -10.0, -10.0)`
  When: bounding box initialization is attempted with negative width or height
  Then: returns `Err(CanvasError::InvalidBoundingBox { width: -10.0, height: -10.0 })` -- NOT a panic, NOT an unwrap failure

- `test_p5_violation_returns_invalid_zoom_scale`
  Given: `ValidTransform::new(-1.0, (0.0, 0.0))`
  When: negative zoom scale is provided for a transformation matrix
  Then: returns `Err(CanvasError::InvalidZoomScale(-1.0))` -- NOT a panic, NOT an unwrap failure

## Given-When-Then Scenarios

### Scenario 1: Dragging a node with grid snap enabled
Given: A node `NodeId(1)` at canvas position `(12.0, 15.0)` and grid resolution of `10.0` (`NonZeroPositiveF64(10.0)`)
When: The user drags the node by delta `(9.0, 4.0)` via `translate_node_snapped`
Then:
- The raw target position is `(21.0, 19.0)`
- The grid-snapped position becomes `(20.0, 20.0)`
- The node's internal top-left position is updated to `(20.0, 20.0)`
- The returned Result is `Ok(())`

### Scenario 2: Reparenting a nested node (Q3 Postcondition)
Given: 
- Node A `NodeId(1)` at absolute `(100.0, 100.0)`
- Node B `NodeId(2)` is a child of A at local `(50.0, 50.0)` [absolute `(150.0, 150.0)`]
- Node C `NodeId(3)` is at absolute `(200.0, 200.0)`
When: Node B is reparented to Node C via `reparent_node(NodeId(2), Some(NodeId(3)))`
Then:
- Node B's absolute position remains `(150.0, 150.0)`
- Node B's local position relative to C becomes `(-50.0, -50.0)`
- Node A's children list no longer contains Node B
- Node C's children list now contains Node B
- The returned Result is `Ok(())`

### Scenario 3: Attempting to Reparent causing a Cycle
Given:
- Node A `NodeId(1)` is the root
- Node B `NodeId(2)` is a child of Node A
- Node C `NodeId(3)` is a child of Node B
When: Node A is reparented to Node C via `reparent_node(NodeId(1), Some(NodeId(3)))`
Then:
- The function detects that Node C is a descendant of Node A
- Node A remains the root
- Node C remains a child of Node B
- The returned Result is `Err(CanvasError::CycleDetected(NodeId(1), NodeId(3)))`
