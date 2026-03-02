bead_id: bd-due
bead_title: tests: Implement HIS undo/redo tests 2/2
phase: p2
updated_at: 2026-03-02T02:36:00Z

# Verification: bd-due - HIS Undo/Redo Tests 2/2

## Test Results

### Unit Tests (diagram_tool)

```
running 42 tests
test history::tests::given_105_elements_when_truncate_then_exactly_100_preserved ... ok
test history::tests::given_cap_boundary_when_undo_and_redo_then_round_trip_is_sane ... ok
test history::tests::given_capped_history_when_undo_all_then_exactly_100_undos_succeed ... ok
test history::tests::given_document_with_high_revision_when_undo_then_state_and_revision_restored ... ok
test history::tests::given_drop_first_on_empty_then_empty ... ok
test history::tests::given_empty_history_when_undo_then_returns_none ... ok
test history::tests::given_empty_stack_when_drop_first_then_empty ... ok
test history::tests::given_empty_stack_when_truncate_then_empty ... ok
test history::tests::given_exactly_100_elements_when_truncate_then_all_preserved ... ok
test history::tests::given_fresh_history_when_redo_then_returns_none ... ok
test history::tests::given_history_after_multiple_undos_when_redo_then_walks_forward_correctly ... ok
test history::tests::given_history_with_four_states_when_undo_three_times_then_redo_chain_preserved ... ok
test history::tests::given_history_with_multiple_states_when_undo_multiple_times_then_walks_back_correctly ... ok
test history::tests::given_history_with_one_state_when_undo_then_returns_that_document ... ok
test history::tests::given_history_with_redo_entries_when_new_action_pushed_then_redo_stack_empty ... ok
test history::tests::given_history_with_redo_entries_when_push_then_redo_stack_cleared ... ok
test history::tests::given_history_with_redo_state_when_redo_then_returns_that_document ... ok
test history::tests::given_history_with_states_when_undo_then_new_history_has_dropped_first ... ok
test history::tests::given_more_than_cap_when_pushing_then_undo_stack_is_capped_at_100 ... ok
test history::tests::given_multiple_entries_when_undo_then_it_walks_back_in_order ... ok
test history::tests::given_node_at_original_position_when_moved_and_undo_then_exact_position_restored ... ok
test history::tests::given_node_at_position_when_moved_and_undo_then_position_restored ... ok
test history::tests::given_node_when_drag_completed_and_pushed_then_single_history_entry ... ok
test history::tests::given_node_with_dimensions_when_resized_and_undo_then_dimensions_restored ... ok
test history::tests::given_node_with_original_dimensions_when_resized_and_undo_then_exact_dimensions_restored ... ok
test history::tests::given_node_with_label_when_label_changed_and_pushed_then_single_history_entry ... ok
test history::tests::given_node_with_parent_when_reparented_and_undo_then_original_parent_restored ... ok
test history::tests::given_node_with_rotation_metadata_when_rotated_and_undo_then_rotation_restored ... ok
test history::tests::given_nodes_when_grouped_and_undo_then_group_removed_and_parents_restored ... ok
test history::tests::given_node_with_style_when_style_changed_and_undo_then_original_style_restored ... ok
test history::tests::given_single_element_when_drop_first_then_empty ... ok
test history::tests::given_single_push_when_undo_then_redo_then_returns_to_current ... ok
test history::tests::given_small_stack_when_truncate_then_same_elements ... ok
test history::tests::given_three_elements_when_drop_first_then_two_remain_in_order ... ok
test history::tests::given_three_pushes_when_undo_once_then_returns_most_recent_push ... ok
test history::tests::given_three_pushes_when_undo_twice_then_returns_second_push ... ok
test history::tests::given_two_nodes_when_edge_created_and_undo_then_edge_removed ... ok
test history::tests::given_undone_state_when_push_then_redo_stack_is_cleared ... ok
test history::tests::test_can_redo_returns_false_for_fresh_history ... ok
test history::tests::test_can_redo_returns_true_after_undo ... ok
test history::tests::test_can_undo_returns_false_for_fresh_history ... ok
test history::tests::test_can_undo_returns_true_after_push ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 988 filtered out
```

### Full Test Suite

```
Running unittests src/main.rs (target/debug/deps/diagram_tool-*)
running 1044 tests
test result: ok. 1044 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out

Running tests/cli_e2e.rs (target/debug/deps/cli_e2e-*)
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Running tests/golden_scenes.rs (target/debug/deps/golden_scenes-*)
running 27 tests
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Acceptance Criteria Verification

- [x] All 5 new HIS tests written and passing
  - HIS-014: `given_history_with_four_states_when_undo_three_times_then_redo_chain_preserved`
  - HIS-015: `given_history_with_redo_entries_when_new_action_pushed_then_redo_stack_empty`
  - HIS-016: `given_document_with_high_revision_when_undo_then_state_and_revision_restored`
  - HIS-017: `given_node_at_original_position_when_moved_and_undo_then_exact_position_restored`
  - HIS-018: `given_node_with_original_dimensions_when_resized_and_undo_then_exact_dimensions_restored`

- [x] Tests follow existing naming conventions (`given_X_when_Y_then_Z`)
- [x] No use of unwrap/expect in test bodies (pattern matching used)
- [x] Tests are in the existing `tests` module of history.rs
- [x] All 1044 diagram_tool tests pass
- [x] All 13 cli_e2e tests pass
- [x] All 27 golden_scenes tests pass

## Contract Compliance

| Requirement | Status | Evidence |
|------------|--------|----------|
| Redo chain preserved after multiple undos | PASS | HIS-014 test |
| New action clears redo | PASS | HIS-015 test |
| Undo across autosave boundary | PASS | HIS-016 test |
| Inverse property validation (move) | PASS | HIS-017 test |
| Inverse property validation (resize) | PASS | HIS-018 test |
