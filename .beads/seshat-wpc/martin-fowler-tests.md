# Martin Fowler Test Plan: Node Grouping (SUB-001 to SUB-006)

## Happy Path Tests
- `test_sub001_click_inside_container_selects_child_vs_container_with_modifier`
- `test_sub002_box_select_across_container_boundary_includes_children`
- `test_sub003_collapse_and_expand_container_toggles_child_visibility`
- `test_sub004_locked_container_allows_unlocked_children_interactions`
- `test_sub005_parent_child_relationship_preservation_during_deep_selection`
- `test_sub006_delete_container_reparents_children_to_grandparent`

## Error Path Tests
- `test_returns_error_when_grouping_with_empty_selection`
- `test_returns_error_when_grouping_nonexistent_nodes`
- `test_returns_error_when_reparenting_creates_cycle`
- `test_returns_error_when_ungrouping_non_container_node`

## Edge Case Tests
- `test_handles_deeply_nested_subgraphs_during_reparenting`
- `test_handles_boundary_values_for_container_bounds_calculation`
- `test_handles_ungrouping_when_container_is_at_root_level`

## Contract Verification Tests
- `test_precondition_non_empty_children`
- `test_postcondition_group_nodes_encapsulates_children`
- `test_postcondition_children_reparented_to_group`
- `test_postcondition_ungroup_preserves_children`
- `test_invariant_child_bounds_strictly_within_parent_bounds`

## Contract Violation Tests
- `test_p1_violation_returns_empty_selection`
  Given: `group_nodes(canvas, id, &[])`
  When: function is called with empty child list
  Then: returns `Err(Error::EmptySelection)` -- NOT a panic, NOT an unwrap failure

- `test_p2_violation_returns_node_not_found`
  Given: `group_nodes(canvas, id, &["missing".into()])`
  When: function is called with non-existent child id
  Then: returns `Err(Error::NodeNotFound("missing"))` -- NOT a panic, NOT an unwrap failure

- `test_p3_violation_returns_circular_dependency`
  Given: `set_parent(canvas, "A".into(), "B".into())` where B is ancestor of A
  When: function is called with cyclic relationship
  Then: returns `Err(Error::CircularDependency)` -- NOT a panic, NOT an unwrap failure

- `test_p4_violation_returns_node_locked`
  Given: `group_nodes(canvas, id, &["locked_id".into()])`
  When: function is called with locked child id
  Then: returns `Err(Error::NodeLocked("locked_id"))` -- NOT a panic, NOT an unwrap failure

- `test_p5_violation_returns_invalid_node_type`
  Given: `ungroup_nodes(canvas, "text_node".into())`
  When: function is called on non-container node
  Then: returns `Err(Error::InvalidNodeType)` -- NOT a panic, NOT an unwrap failure

- `test_q1_violation_returns_invariant_violation_for_bounds`
  Given: `group_nodes` completes but container bounds are smaller than a child's bounds after call
  When: container creation bounds are maliciously altered or calculated incorrectly
  Then: returns `Err(Error::InvariantViolation)` -- NOT a panic, NOT an unwrap failure

- `test_q2_violation_returns_invariant_violation_for_parent`
  Given: `group_nodes` completes but a child's parent is not `group_id` after call
  When: post-operation parent validation fails
  Then: returns `Err(Error::InvariantViolation)` -- NOT a panic, NOT an unwrap failure

- `test_q3_violation_returns_invariant_violation_for_reparenting`
  Given: `ungroup_nodes` completes but children are missing from canvas state after call
  When: container deletion erroneously removes child nodes
  Then: returns `Err(Error::InvariantViolation)` -- NOT a panic, NOT an unwrap failure

## Given-When-Then Scenarios

### Scenario 1: SUB-006 Delete Container Reparents Children
Given: A canvas with a root container "Group A" and children "Node 1", "Node 2"
When: `ungroup_nodes` is called targeting "Group A"
Then:
- "Group A" is removed from `canvas.nodes`
- "Node 1" and "Node 2" remain present in `canvas.nodes`
- "Node 1".parent and "Node 2".parent are updated to `None`
- The operation returns `Ok(vec!["Node 1", "Node 2"])`

### Scenario 2: SUB-001 Modifier Selection
Given: A canvas with a container "Group A" and a child "Node 1", and the user holds the `Ctrl` (or `Cmd`) modifier key
When: The user clicks the absolute coordinates corresponding to "Node 1"
Then:
- The selection algorithm bypasses "Group A"
- "Node 1" is directly added to the active selection
- The operation returns `Ok(SelectionResult::NodeSelected("Node 1"))`

### Scenario 3: SUB-004 Locked Container Interaction
Given: A canvas with a locked container "Group A" (locked = true) and an unlocked child "Node 1" (locked = false)
When: The user attempts to move "Node 1" via translation
Then:
- The translation is applied to "Node 1"
- "Group A" expands or recalculates bounds if necessary, without failing due to being locked (as only the container itself cannot be user-translated)
- The operation completes successfully without `Error::NodeLocked`