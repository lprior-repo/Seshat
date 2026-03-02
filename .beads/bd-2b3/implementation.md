# Implementation: bd-2b3 - edge-case-bdd-tests-projection-replay

## Summary

Added 25 comprehensive BDD-style tests for projection replay edge cases in `diagram_tool/src/models/projection.rs`.

## Files Modified

- `diagram_tool/src/models/projection.rs`: Added BDD test section (lines 3658-4350)

## Test Categories

### 1. Empty Event Stream Edge Cases (2 tests)
- `bdd_given_empty_event_stream_when_replaying_then_returns_empty_projection`
- `bdd_given_empty_stream_when_replaying_from_nonempty_state_then_state_unchanged`

### 2. Non-Sequential Revision Edge Cases (3 tests)
- `bdd_given_revision_gap_when_replaying_then_returns_invariant_violation`
- `bdd_given_non_monotonic_revision_when_replaying_then_returns_invariant_violation`
- `bdd_given_wrong_start_revision_when_replaying_then_returns_invariant_violation`

### 3. Duplicate Operation ID Edge Cases (2 tests)
- `bdd_given_duplicate_op_id_when_replaying_then_returns_invariant_violation`
- `bdd_given_preexisting_op_id_when_applying_event_then_returns_invariant_violation`

### 4. CycleViolation Edge Cases (4 tests)
- `bdd_given_cycle_creating_edge_with_deny_policy_when_applying_then_returns_cycle_violation`
- `bdd_given_self_loop_with_deny_policy_when_applying_then_returns_cycle_violation`
- `bdd_given_complex_cycle_in_larger_graph_when_enforcing_then_returns_cycle_violation`
- `bdd_given_cycle_with_allow_policy_when_enforcing_then_succeeds`

### 5. InvariantViolation Edge Cases (6 tests)
- `bdd_given_duplicate_node_id_when_replaying_then_returns_invariant_violation`
- `bdd_given_edge_to_nonexistent_source_when_replaying_then_returns_invariant_violation`
- `bdd_given_node_move_on_nonexistent_node_when_replaying_then_returns_invariant_violation`
- `bdd_given_edge_disconnect_nonexistent_when_replaying_then_returns_invariant_violation`
- `bdd_given_node_delete_nonexistent_when_replaying_then_returns_invariant_violation`
- `bdd_given_duplicate_edge_id_when_replaying_then_returns_invariant_violation`

### 6. Revision Increment Edge Cases (3 tests)
- `bdd_given_successful_operation_when_applying_then_revision_increments_by_one`
- `bdd_given_multiple_operations_when_replaying_then_revision_increments_sequentially`
- `bdd_given_failed_operation_when_applying_then_state_unchanged`

### 7. Determinism Edge Cases (2 tests)
- `bdd_given_same_events_multiple_times_when_replaying_then_produces_identical_projections`
- `bdd_given_same_events_when_hashing_multiple_times_then_produces_identical_hashes`

### 8. Author Priority Edge Cases (2 tests)
- `bdd_given_human_and_ai_operations_when_replaying_then_priority_correctly_tracks`
- `bdd_given_large_event_stream_when_replaying_then_all_priorities_tracked`

### 9. Error Message Quality Tests (1 test)
- `bdd_given_error_condition_when_returning_error_then_message_is_descriptive`

## Verification Results

```
running 25 tests
test models::projection::tests::bdd_given_edge_to_nonexistent_source_when_replaying_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_edge_disconnect_nonexistent_when_replaying_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_duplicate_node_id_when_replaying_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_duplicate_op_id_when_replaying_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_empty_stream_when_replaying_from_nonempty_state_then_state_unchanged ... ok
test models::projection::tests::bdd_given_empty_event_stream_when_replaying_then_returns_empty_projection ... ok
test models::projection::tests::bdd_given_duplicate_edge_id_when_replaying_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_cycle_with_allow_policy_when_enforcing_then_succeeds ... ok
test models::projection::tests::bdd_given_cycle_creating_edge_with_deny_policy_when_applying_then_returns_cycle_violation ... ok
test models::projection::tests::bdd_given_complex_cycle_in_larger_graph_when_enforcing_then_returns_cycle_violation ... ok
test models::projection::tests::bdd_given_human_and_ai_operations_when_replaying_then_priority_correctly_tracks ... ok
test models::projection::tests::bdd_given_error_condition_when_returning_error_then_message_is_descriptive ... ok
test models::projection::tests::bdd_given_failed_operation_when_applying_then_state_unchanged ... ok
test models::projection::tests::bdd_given_node_delete_nonexistent_when_replaying_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_multiple_operations_when_replaying_then_revision_increments_sequentially ... ok
test models::projection::tests::bdd_given_node_move_on_nonexistent_node_when_replaying_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_non_monotonic_revision_when_replaying_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_preexisting_op_id_when_applying_event_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_revision_gap_when_replaying_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_same_events_when_hashing_multiple_times_then_produces_identical_hashes ... ok
test models::projection::tests::bdd_given_same_events_multiple_times_when_replaying_then_produces_identical_projections ... ok
test models::projection::tests::bdd_given_successful_operation_when_applying_then_revision_increments_by_one ... ok
test models::projection::tests::bdd_given_self_loop_with_deny_policy_when_applying_then_returns_cycle_violation ... ok
test models::projection::tests::bdd_given_wrong_start_revision_when_replaying_then_returns_invariant_violation ... ok
test models::projection::tests::bdd_given_large_event_stream_when_replaying_then_all_priorities_tracked ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1146 filtered out; finished in 0.00s
```

## Naming Convention

All tests follow BDD naming pattern:
```
bdd_given_<precondition>_when_<action>_then_<expected_result>
```

This makes the test purpose immediately clear from the name.
