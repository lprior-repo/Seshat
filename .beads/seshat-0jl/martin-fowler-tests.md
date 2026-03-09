# Martin Fowler Test Plan: Graph Cycle Policy Engine

## Happy Path Tests
- test_cycle_policy_default_is_allow
- test_enforce_cycle_policy_succeeds_when_allow
- test_apply_policy_op_succeeds_with_valid_operation
- test_apply_policy_op_succeeds_under_deny_policy_for_acyclic_graph
- test_cycle_policy_deny_allows_valid_dag_structure

## Error Path Tests
- test_enforce_cycle_policy_returns_error_when_deny_with_cycle
- test_apply_policy_op_returns_cycle_violation_when_creating_cycle
- test_apply_policy_op_returns_invalid_event_for_non_edge_op

## Edge Case Tests
- test_enforce_cycle_policy_handles_empty_graph
- test_enforce_cycle_policy_handles_single_node_no_edges
- test_apply_policy_op_handles_node_operations_correctly
- test_apply_policy_op_handles_disconnect_operations

## Contract Verification Tests
- test_precondition_state_valid_for_enforce
- test_precondition_operation_valid_for_apply
- test_postcondition_enforce_returns_ok_or_error
- test_postcondition_apply_returns_new_state_or_error

## Contract Violation Tests
- `test_cycle_policy_deny_violation_returns_cycle_violation`
  Given: DiagramProjection with cycle_policy = Deny and edges forming a cycle (a->b, b->a)
  When: enforce_cycle_policy is called
  Then: returns `Err(ReplayError::CycleViolation(...))` -- NOT a panic

- `test_apply_policy_op_cycle_violation_returns_error`
  Given: DiagramProjection with cycle_policy = Deny and edge a->b exists
  When: apply_policy_op is called with EdgeConnect b->a
  Then: returns `Err(ReplayError::CycleViolation(...))`

- `test_apply_policy_op_does_not_mutate_state_on_violation`
  Given: DiagramProjection with cycle_policy = Deny and edge a->b
  When: apply_policy_op is called with EdgeConnect b->a (creates cycle)
  Then: Original state is unchanged, error is returned

## Given-When-Then Scenarios

### Scenario 1: Allow Policy Permits Cycles
Given: A DiagramProjection with cycle_policy = Allow
And: Two nodes exist with edges forming a cycle (a->b, b->a)
When: enforce_cycle_policy is called
Then: Returns Ok(()) and permits the cycle

### Scenario 2: Deny Policy Rejects Cycles
Given: A DiagramProjection with cycle_policy = Deny
And: Two nodes exist with edges forming a cycle
When: enforce_cycle_policy is called
Then: Returns Err(ReplayError::CycleViolation(...))

### Scenario 3: Apply Edge Connect Under Deny Policy
Given: A DiagramProjection with cycle_policy = Deny
And: Node 'a' exists, Node 'b' exists
And: Edge 'a->b' already exists
When: apply_policy_op is called with EdgeConnect 'b->a' (creates cycle)
Then: Returns Err(ReplayError::CycleViolation(...))
And: The new edge is NOT added to the state

### Scenario 4: Apply Node Operation Under Deny Policy
Given: A DiagramProjection with cycle_policy = Deny
And: Node 'a' exists
When: apply_policy_op is called with NodeAdd 'b'
Then: Returns Ok(new_state) with the new node added
And: No cycle check is performed (node operations don't affect cycles)

### Scenario 5: Apply Edge Disconnect Under Deny Policy
Given: A DiagramProjection with cycle_policy = Deny
And: Edge 'a->b' exists
When: apply_policy_op is called with EdgeDisconnect 'a->b'
Then: Returns Ok(new_state) with the edge removed

## Integration Test
- test_full_policy_enforcement_workflow: Creates projection, applies operations, enforces policy end-to-end
