# Martin Fowler Test Plan: seshat-a7z (Marquee Selection / SEL-011 to SEL-015)

## Happy Path Tests
- test_returns_success_when_alt_click_selects_parent_container
- test_returns_success_when_right_click_unselected_node_selects_it
- test_returns_success_when_click_edge_selects_connector

## Error Path Tests
- test_returns_error_when_alt_clicking_node_without_parent
- test_returns_error_when_selecting_locked_element
- test_returns_error_when_selecting_hidden_element
- test_returns_error_when_selecting_non_existent_element

## Edge Case Tests
- test_handles_click_passing_through_hidden_node_to_node_underneath
- test_handles_right_click_already_selected_node_preserves_selection

## Contract Verification Tests
- test_precondition_element_must_be_unlocked
- test_precondition_element_must_be_visible
- test_postcondition_alt_click_replaces_child_with_parent
- test_postcondition_right_click_replaces_selection
- test_invariant_selection_never_contains_locked_elements
- test_invariant_selection_never_contains_hidden_elements

## Contract Violation Tests
- `test_p1_violation_returns_no_parent_container_error`
  Given: `select_element(root_node_id, SelectModifiers { alt: true, .. })`
  When: function is called with a node lacking a parent
  Then: returns `Err(SelectionError::NoParentContainer)` -- NOT a panic, NOT an unwrap failure

- `test_p2_violation_returns_element_locked_error`
  Given: `select_element(locked_node_id, SelectModifiers::default())`
  When: function is called to select a locked node
  Then: returns `Err(SelectionError::ElementLocked)` -- NOT a panic, NOT an unwrap failure

- `test_p3_violation_returns_element_hidden_error`
  Given: `select_element(hidden_node_id, SelectModifiers::default())`
  When: function is called to select a hidden node
  Then: returns `Err(SelectionError::ElementHidden)` -- NOT a panic, NOT an unwrap failure

- `test_p4_violation_returns_element_not_found_error`
  Given: `select_element(non_existent_edge_id, SelectModifiers::default())`
  When: function is called to select a non-existent edge
  Then: returns `Err(SelectionError::ElementNotFound)` -- NOT a panic, NOT an unwrap failure

- `test_q1_violation_returns_precondition_violated`
  Given: `selected_items.contains(child_id)` after Alt-click
  When: parent replacement postcondition fails
  Then: returns `Err(SelectionError::PreconditionViolated)` -- NOT a panic, NOT an unwrap failure

- `test_q2_violation_returns_element_locked`
  Given: `selected_items.contains(locked_id)` after clicking locked node
  When: locked mutation postcondition fails
  Then: returns `Err(SelectionError::ElementLocked)` -- NOT a panic, NOT an unwrap failure

- `test_q3_violation_returns_element_hidden`
  Given: `selected_items.contains(hidden_id)` after clicking hidden node
  When: hidden mutation postcondition fails
  Then: returns `Err(SelectionError::ElementHidden)` -- NOT a panic, NOT an unwrap failure

- `test_q4_violation_returns_precondition_violated`
  Given: `selected_items` is empty after right-clicking an unselected node
  When: right-click selection postcondition fails
  Then: returns `Err(SelectionError::PreconditionViolated)` -- NOT a panic, NOT an unwrap failure

- `test_q5_violation_returns_precondition_violated`
  Given: `selected_items` does not contain edge ID after clicking edge
  When: edge selection postcondition fails
  Then: returns `Err(SelectionError::PreconditionViolated)` -- NOT a panic, NOT an unwrap failure

## Given-When-Then Scenarios

### Scenario 1: Alt-click Parent Selection (SEL-011)
Given: A document with a group container and a child node inside it
When: The user Alt-clicks the child node
Then: 
- The child node is NOT in `selected_items`
- The parent container IS in `selected_items`

### Scenario 2: Interaction with Locked Elements (SEL-012)
Given: A document with an unlocked node A and a locked node B
When: The user clicks node B
Then:
- Node B is not selected
- An error or feedback is returned indicating the element is locked
- `selected_items` remains unchanged

### Scenario 3: Right-Click Selection Replacement (SEL-014)
Given: A document with node A currently selected, and an unselected node B
When: The user right-clicks on node B to open a context menu
Then:
- Node A is removed from `selected_items`
- Node B is added to `selected_items`
- The context menu logic receives the updated selection containing node B

### Scenario 4: Click Passthrough for Hidden Elements (SEL-013)
Given: A hidden node A completely covering an unlocked, visible node B
When: The user clicks the canvas at the coordinates of both nodes
Then:
- The hit test ignores node A
- The hit test returns node B
- Node B is added to `selected_items`

### Scenario 5: Edge Selection (SEL-015)
Given: An edge connects two nodes
When: User clicks on the edge line
Then:
- The edge is selected
- The nodes themselves are not selected
