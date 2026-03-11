# Martin Fowler Test Plan: Edge-Node Binding (EDG-011 to EDG-015)

## Happy Path Tests
- `edg_011_valid_edge_creation`
- `edg_014_edge_deletion_isolated`
- `edg_015_node_deletion_cascades_edges`
- `test_allows_self_loop_edge`
- `test_allows_multiple_edges_between_same_nodes`

## Error Path Tests
- `edg_012_invalid_edge_missing_source`
- `edg_013_invalid_edge_missing_target`
- `test_returns_error_when_creating_edge_with_duplicate_id`
- `test_returns_error_when_deleting_missing_edge`

## Edge Case Tests
- `test_cascading_deletion_handles_multiple_edges_on_same_node`
- `test_cascading_deletion_handles_self_loop`

## Contract Verification Tests
- `test_precondition_source_node_must_exist`
- `test_precondition_target_node_must_exist`
- `test_precondition_edge_id_must_be_unique`
- `test_postcondition_edge_exists_after_creation`
- `test_postcondition_cascading_delete_maintains_invariants`
- `test_invariant_all_edges_reference_existing_nodes`

## Contract Violation Tests
- `test_p1_violation_returns_node_not_found`
  Given: A document with `valid_target` but no `missing_source`
  When: `add_edge` is called with `source = missing_source`
  Then: returns `Err(Error::NodeNotFound(missing_source))`

- `test_p2_violation_returns_node_not_found`
  Given: A document with `valid_source` but no `missing_target`
  When: `add_edge` is called with `target = missing_target`
  Then: returns `Err(Error::NodeNotFound(missing_target))`

- `test_p3_violation_returns_edge_already_exists`
  Given: A document with an existing edge `existing_id`
  When: `add_edge` is called with `existing_id`
  Then: returns `Err(Error::EdgeAlreadyExists(existing_id))`

## Given-When-Then Scenarios

### Scenario 1: EDG-011 Valid Edge Creation
Given: A document containing nodes `N1` and `N2`
When: `add_edge` is called with a new edge `E1` from `N1` to `N2`
Then:
- The function returns `Ok(())`
- The document's edge collection contains `E1`

### Scenario 2: EDG-012 Invalid Edge Missing Source
Given: A document containing node `N2` but not `N1`
When: `add_edge` is called with a new edge `E1` from `N1` to `N2`
Then:
- The function returns `Err(Error::NodeNotFound(N1))`
- The document's edge collection does not contain `E1`

### Scenario 3: EDG-013 Invalid Edge Missing Target
Given: A document containing node `N1` but not `N2`
When: `add_edge` is called with a new edge `E1` from `N1` to `N2`
Then:
- The function returns `Err(Error::NodeNotFound(N2))`
- The document's edge collection does not contain `E1`

### Scenario 4: EDG-014 Edge Deletion Isolated
Given: A document containing nodes `N1` and `N2`, and an edge `E1` from `N1` to `N2`
When: `remove_edge` is called for `E1`
Then:
- The function returns `Ok(())`
- The document's edge collection no longer contains `E1`
- The document's node collection still contains `N1` and `N2`

### Scenario 5: EDG-015 Node Deletion Cascades Edges
Given: A document containing nodes `N1`, `N2`, `N3` and edges `E1` (N1->N2) and `E2` (N2->N3)
When: `remove_node` is called for `N2`
Then:
- The function returns `Ok(())`
- The document's node collection no longer contains `N2`
- The document's edge collection no longer contains `E1` or `E2`
- The document's node collection still contains `N1` and `N3`
