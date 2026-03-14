# Martin Fowler Test Plan: seshat-axc (SUB-013 to SUB-017)

## Happy Path Tests
- `test_returns_success_when_grouping_at_allowed_depth`
- `test_returns_success_when_grouping_at_zero_depth`
- `test_returns_success_when_grouping_at_depth_4`

## Error Path Tests
- `test_returns_error_when_grouping_at_max_depth_5`
- `test_returns_error_when_grouping_causes_any_child_to_exceed_max_depth`

## Edge Case Tests
- `test_handles_deeply_nested_subgraphs_gracefully`
- `test_handles_grouping_multiple_subgraphs_robustly`

## Contract Verification Tests
- `test_precondition_nesting_depth_limit`
- `test_postcondition_all_nodes_within_depth_limit`
- `test_invariant_nesting_depth_constant`

## Contract Violation Tests
- `test_group_operation_violation_returns_limit_exceeded`
  Given: A node `n1` at nesting depth 5
  When: `apply_group(state, &["n1", "n2"])` is called
  Then: Returns `Err(ReplayError::NestedSubgraphLimitExceeded(5))`

## Given-When-Then Scenarios

### Scenario 1: Grouping nodes at max depth
Given:
- A diagram with a chain of subgraphs: Root > S1 > S2 > S3 > S4 > S5
- A node `N` whose parent is `S5` (depth = 5)
When:
- User attempts to group `N` with another node `M` (also child of `S5`)
Then:
- System returns `Err(ReplayError::NestedSubgraphLimitExceeded(5))`
- New group is not created
- Nodes `N` and `M` still have `S5` as parent

### Scenario 2: Moving node into deep subgraph
Given:
- A subgraph `S` at nesting depth 5
- A node `N` at the root level
When:
- User attempts to move/reparent `N` into `S`
Then:
- System returns `Err(ReplayError::NestedSubgraphLimitExceeded(5))`
- `N` remains at the root level
