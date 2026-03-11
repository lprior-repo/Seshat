# Martin Fowler Test Plan: Nested Graphs (SUB-013 to SUB-018)

## Happy Path Tests
- test_creates_empty_subgraph_container_with_minimum_dimensions_sub_015
- test_creates_subgraph_with_pre_selected_nodes_encapsulated_sub_016
- test_creates_nested_subgraph_structure_sub_017
- test_subgraph_inherits_viewport_transforms_sub_018
- test_container_padding_alignment_is_maintained_sub_014
- test_container_expands_when_child_overflows_sub_013

## Error Path Tests
- test_returns_error_when_creating_subgraph_with_non_existent_nodes
- test_returns_error_when_nested_subgraph_creates_cycle
- test_returns_error_when_applying_invalid_viewport_transform

## Edge Case Tests
- test_container_behavior_with_zero_padding
- test_container_overflow_handling_with_massive_child_node
- test_deeply_nested_subgraphs_render_correctly

## Contract Verification Tests
- test_precondition_padding_must_be_non_negative
- test_precondition_viewport_scale_must_be_positive
- test_postcondition_container_bounds_encapsulate_all_children
- test_postcondition_subgraph_creation_updates_child_parent_references
- test_invariant_node_has_at_most_one_parent
- test_invariant_hierarchy_is_acyclic

## Contract Violation Tests

- `test_p1_violation_returns_type_error`
  Given: `calculate_container_bounds(&nodes, Padding { top: -10, ... })`
  When: A negative padding value is provided
  Then: returns `Err(Error::InvalidPadding)` or fails to compile

- `test_p2_violation_returns_error_node_not_found`
  Given: `create_subgraph_from_nodes(new_id, &["non_existent_id"], &mut canvas)`
  When: `create_subgraph_from_nodes` is called with "non_existent_id"
  Then: returns `Err(Error::NodeNotFound)` -- NOT a panic, NOT an unwrap failure

- `test_p3_violation_returns_error_circular_dependency`
  Given: `set_node_parent(container_a, container_b)` when `container_b` is already a child of `container_a`
  When: `set_node_parent` is called creating a cycle
  Then: returns `Err(Error::CircularDependency)` -- NOT a panic, NOT an unwrap failure

- `test_p4_violation_returns_invalid_transform`
  Given: `apply_viewport_transform(subgraph, Scale(0.0))`
  When: `apply_viewport_transform` is called with scale 0.0
  Then: returns `Err(Error::InvalidTransform)` -- NOT a panic, NOT an unwrap failure

- `test_q1_violation_returns_invariant_error`
  Given: Container bounds after child insertion do not encompass child bounding box plus padding
  When: Child bounds are manipulated such that the container bounding box doesn't cover them
  Then: The system bounds validation returns an error `Err(Error::InvariantViolation)`

- `test_q2_violation_returns_invariant_error`
  Given: `create_empty_subgraph` returns a container with smaller than minimum bounds
  When: `create_empty_subgraph` attempts to create one with dimensions smaller than the minimum limit
  Then: Validation returns an error `Err(Error::InvariantViolation)`

- `test_q3_violation_returns_invariant_error`
  Given: After `create_subgraph_from_nodes(id, &[child_id])`, `child.parent != Some(id)`
  When: The reparenting logic fails to persist `child.parent = Some(container_id)`
  Then: The system returns an integrity validation error `Err(Error::InvariantViolation)`

- `test_q4_violation_returns_invariant_error`
  Given: Nested subgraph renders at offset diverging from true inherited transform
  When: A nested subgraph fails to multiply the viewport scale correctly
  Then: Transform calculation returns an error `Err(Error::InvariantViolation)`

## Given-When-Then Scenarios

### Scenario 1: Container Overflow Handling (SUB-013)
Given: A container node with padding `10` and one child node
When: The child node is moved partially outside the container's current boundaries
Then:
- The container's bounding box expands automatically
- The container maintains exactly `10` units of padding between the child's new position and the container boundary
- No `Error::InvalidPadding` is thrown

### Scenario 2: Container Padding Alignment (SUB-014)
Given: A container node with padding `{top: 10, right: 20, bottom: 10, left: 20}`
When: Multiple child nodes are positioned within the container
Then:
- The overall bounding box of the children is calculated
- The container's top boundary is exactly `10` units above the highest child
- The container's right boundary is exactly `20` units to the right of the rightmost child

### Scenario 3: Create Nested Subgraph (SUB-017)
Given: A canvas containing an existing subgraph container `A`
When: A new subgraph container `B` is created and assigned as a child of `A`
Then:
- `B.parent` is set to `A.id`
- `A` expands its bounds to encapsulate `B` (including `B`'s minimum dimensions)
- No `Error::CircularDependency` is returned

### Scenario 4: Subgraph Inherits Viewport Transforms (SUB-018)
Given: A nested subgraph `B` (child of `A`)
When: The canvas viewport is scaled by `2.0` and translated by `(100, 100)`
Then:
- The absolute rendering coordinates of `B` reflect both the viewport transform and `A`'s position
- `B` maintains its relative layout within `A`
- The scale factor is exactly `2.0`
