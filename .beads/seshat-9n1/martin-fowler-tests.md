# Martin Fowler Test Plan: SEL-001 to SEL-005

## Happy Path Tests
- `test_sel_001_click_replaces_selection`
- `test_sel_002_shift_click_adds_to_selection`
- `test_sel_002_shift_click_removes_from_selection`
- `test_sel_003_left_to_right_marquee_selects_contained_nodes`
- `test_sel_004_click_empty_canvas_clears_selection`
- `test_sel_005_right_to_left_marquee_selects_intersected_nodes`

## Error Path Tests
- `test_returns_error_when_selecting_non_existent_node`
- `test_returns_error_when_marquee_starts_on_node`

## Edge Case Tests
- `test_marquee_selection_with_no_contained_or_intersected_nodes`
- `test_marquee_selection_exactly_matching_node_bounds`

## Contract Verification Tests
- `test_invariant_selection_set_contains_no_duplicates`
- `test_invariant_selection_set_only_contains_existing_nodes`

## Contract Violation Tests
- `test_p1_violation_returns_item_not_found_error`
  Given: `select_item(state, NodeId("non-existent"), SelectionMode::Replace)`
  When: function is called with a node ID that does not exist in the document
  Then: returns `Err(Error::ItemNotFound("non-existent"))` -- NOT a panic, NOT an unwrap failure

- `test_p2_violation_returns_invalid_interaction_state_error`
  Given: `marquee_select(state, Point::on_node(), end)`
  When: marquee selection is explicitly triggered starting on a known node's bounds
  Then: returns `Err(Error::InvalidInteractionState)` -- NOT a panic, NOT an unwrap failure

## Given-When-Then Scenarios

### Scenario 1: SEL-001 Click Selects Node (Replace)
Given: A diagram with Node A and Node B, where Node A is currently selected
When: `select_item` is called for Node B with `SelectionMode::Replace`
Then: 
- `selected_items` contains Node B
- `selected_items` does not contain Node A
- `selected_items.len()` == 1

### Scenario 2: SEL-002 Shift-Click Toggles Selection (Add)
Given: A diagram with Node A and Node B, where Node A is currently selected
When: `select_item` is called for Node B with `SelectionMode::Toggle`
Then:
- `selected_items` contains Node A and Node B
- `selected_items.len()` == 2

### Scenario 3: SEL-002 Shift-Click Toggles Selection (Remove)
Given: A diagram with Node A and Node B, where both are currently selected
When: `select_item` is called for Node B with `SelectionMode::Toggle`
Then:
- `selected_items` contains Node A
- `selected_items` does not contain Node B
- `selected_items.len()` == 1

### Scenario 4: SEL-003/005 Marquee Direction
Given: A diagram with a node at (10, 10) with width 50, height 50
When: `marquee_select` is called from (0, 0) to (30, 30) (Left-to-Right)
Then: Node is NOT selected (it is only partially contained, not fully)

When: `marquee_select` is called from (30, 30) to (0, 0) (Right-to-Left)
Then: Node IS selected (it intersects the marquee box)

### Scenario 5: SEL-004 Click Empty Clears Selection
Given: A diagram with multiple selected nodes
When: `clear_selection` is called
Then: `selected_items` is empty
