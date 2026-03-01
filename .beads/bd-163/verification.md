bead_id: bd-163
bead_title: tests: Implement SUB subgraph tests 1/4
phase: p2
updated_at: 2026-03-01T22:45:00Z

# Verification: SUB Subgraph Tests 1/4

## Test Execution Results

### Unit Tests - All Pass

```
running 869 tests
test result: ok. 869 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out
```

### New Tests Added (8 total)

#### commands.rs Tests (5 tests)

| Test Name | Status |
|-----------|--------|
| given_two_selected_nodes_when_group_selection_then_creates_subgraph_with_correct_bounds | PASS |
| given_selected_subgraph_with_children_when_ungroup_then_children_restored_to_root | PASS |
| given_nested_subgraphs_when_validated_then_parent_chain_correct | PASS |
| given_single_node_selected_when_group_selection_then_returns_false | PASS |
| given_subgraph_selected_when_group_selection_then_subgraph_excluded | PASS |

#### schema.rs Tests (3 tests)

| Test Name | Status |
|-----------|--------|
| given_circular_parent_chain_when_validated_then_schema_fails | PASS |
| given_self_referential_parent_when_validated_then_schema_fails | PASS |
| given_two_node_parent_cycle_when_validated_then_schema_fails | PASS |

### E2E Tests - All Pass

```
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Contract Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Group selection creates group with correct bounds | PASS | Test verifies 24px padding and bounds calculation |
| Ungroup restores children to canvas root | PASS | Test verifies parent=None and position preservation |
| Nested groups work correctly | PASS | Test verifies outer->inner->child hierarchy |
| Container/frame creation | N/A | Covered by existing InteractionMode::DrawingSubgraph tests |
| Prevent parent cycles | PASS | 3 tests for different cycle scenarios |

## Code Quality

- No clippy errors
- All tests use `#![deny(clippy::unwrap_used)]` pattern
- All tests follow `given_X_when_Y_then_Z` naming convention
- No file I/O or network dependencies in tests
