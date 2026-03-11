# Martin Fowler Test Plan: SEL-021 to SEL-025

## Happy Path Tests
- `test_sel_021_bounding_box_covers_rotated_nodes`
- `test_sel_022_long_press_adds_node_to_selection_without_drag`
- `test_sel_023_double_click_enters_edit_mode_on_shape`
- `test_sel_024_selection_persists_across_camera_zoom_and_pan`
- `test_sel_025_marquee_selects_nodes_inside_subgraphs`

## Error Path Tests
- `test_returns_error_when_computing_bounds_for_missing_nodes`
- `test_long_press_fails_when_movement_exceeds_threshold`
- `test_double_click_fails_on_uneditable_nodes`

## Edge Case Tests
- `test_sel_021_bounding_box_with_mixed_rotated_and_unrotated_nodes`
- `test_sel_022_long_press_on_touch_shows_handles`
- `test_sel_023_double_click_on_empty_canvas_ignores_edit_mode`
- `test_sel_023_rapid_multi_clicks_do_not_create_accidental_text`
- `test_sel_024_selection_persists_during_react_rerender_cycle`
- `test_sel_025_marquee_partially_overlapping_parent_selects_fully_enclosed_child`

## Contract Verification Tests
- `test_precondition_node_must_exist_for_bounds`
- `test_precondition_movement_must_be_under_drag_threshold`
- `test_precondition_marquee_dimensions_must_be_non_negative`
- `test_postcondition_marquee_ignores_parent_hierarchy`
- `test_invariant_selection_set_contains_only_existing_nodes`

## Contract Violation Tests
- `test_p1_violation_returns_node_not_found`
  Given: `compute_selection_bounds(&doc_with_deleted_node_in_selection)`
  When: `compute_selection_bounds` is called
  Then: returns `Err(SelectionError::NodeNotFound)` -- NOT a panic, NOT an unwrap failure

- `test_p2_violation_returns_movement_exceeded_drag_threshold`
  Given: `handle_long_press(&mut doc, id, 15.0)` (where threshold is 5.0)
  When: `handle_long_press` is called with movement exceeding threshold
  Then: returns `Err(SelectionError::MovementExceededDragThreshold)` -- NOT a panic, NOT an unwrap failure

- `test_p3_violation_returns_node_not_editable`
  Given: `handle_double_click(&mut doc, locked_node_id)`
  When: `handle_double_click` is called on a locked/uneditable node
  Then: returns `Err(SelectionError::NodeNotEditable)` -- NOT a panic, NOT an unwrap failure

- `test_p5_violation_returns_marquee_invalid`
  Given: `compute_marquee_selection(&doc, Rect { width: -10.0, ... })`
  When: `compute_marquee_selection` is called with negative width
  Then: returns `Err(SelectionError::InvalidMarqueeBounds)` -- NOT a panic, NOT an unwrap failure

## Given-When-Then Scenarios

### Scenario 1: Box-Select Through Parent Boundaries (SEL-025)
Given: A document with a parent node `Group A` and a nested child node `Child B`
And: A standalone node `Node C` outside of `Group A`
When: A user drags a marquee selection box that intersects `Child B` and `Node C`, but only partially covers `Group A`
Then:
- `Child B` is selected
- `Node C` is selected
- `Group A` is not selected (because it is not fully enclosed by the marquee)
- The selection algorithm successfully traverses the hierarchy to find intersecting children regardless of parent bounds

### Scenario 2: Double-Click Edit Mode (SEL-023)
Given: A document with a standard shape node and an empty canvas area
When: A user double-clicks the shape node
Then:
- The shape enters text edit mode (`editor_state.edit_mode_target` is set)
When: The user double-clicks the empty canvas area
Then:
- No new text nodes are created (unless explicitly configured)
- Edit mode is not entered
- The system prevents accidental text creation on fast clicks
