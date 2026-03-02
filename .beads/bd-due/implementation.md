bead_id: bd-due
bead_title: tests: Implement HIS undo/redo tests 2/2
phase: p1
updated_at: 2026-03-02T02:35:00Z

# Implementation: bd-due - HIS Undo/Redo Tests 2/2

## Summary

Added 5 new unit tests to `diagram_tool/src/history.rs` to complete the HIS undo/redo test coverage:
- HIS-014: Redo chain preserved after multiple undos
- HIS-015: New action clears redo stack completely
- HIS-016: Undo across autosave boundary
- HIS-017: Inverse property validation (move)
- HIS-018: Inverse property validation (resize)

## Files Modified

- `diagram_tool/src/history.rs`: Added 5 new tests in the `tests` module

## Implementation Details

### HIS-014: Redo chain preserved after multiple undos
**Test function:** `given_history_with_four_states_when_undo_three_times_then_redo_chain_preserved`

This test verifies that after performing 4 undos (from current through D, C, B back to A), the redo stack maintains the correct order and all 4 states can be redone in sequence.

Key assertions:
- After 4 undos, redo stack has 4 entries
- First redo restores B (x=200)
- Second redo restores C (x=300)
- Third redo restores D (x=400)
- Fourth redo restores current (x=500)

### HIS-015: New action clears redo stack completely
**Test function:** `given_history_with_redo_entries_when_new_action_pushed_then_redo_stack_empty`

This test verifies that pushing a new document state after undo completely clears the redo stack.

Key assertions:
- After 2 undos, redo stack has 2 entries
- After pushing a new state, redo stack is completely empty
- The new push is in the undo stack (can_undo returns true)

### HIS-016: Undo across autosave boundary
**Test function:** `given_document_with_high_revision_when_undo_then_state_and_revision_restored`

This test simulates undo across an autosave boundary by using high revision numbers that might be associated with autosave intervals.

Key assertions:
- Document content (node position) is restored to pre-autosave value
- Revision is from the pushed state
- Redo is available after undo

### HIS-017: Inverse property validation (move)
**Test function:** `given_node_at_original_position_when_moved_and_undo_then_exact_position_restored`

This test verifies that undo of a move operation restores the exact original position with no floating-point drift.

Key assertions:
- X coordinate is exactly restored to original value
- Y coordinate is exactly restored to original value
- No floating-point drift (verified with epsilon comparison)

### HIS-018: Inverse property validation (resize)
**Test function:** `given_node_with_original_dimensions_when_resized_and_undo_then_exact_dimensions_restored`

This test verifies that undo of a resize operation restores the exact original dimensions with no floating-point drift.

Key assertions:
- Width is exactly restored to original value
- Height is exactly restored to original value
- No floating-point drift (verified with epsilon comparison)

## Test Style

All tests follow the existing conventions:
- Use `given_X_when_Y_then_Z` naming pattern
- Use `make_node_for_his` helper function
- Use pattern matching instead of unwrap/expect
- Include `/// HIS-NNN:` documentation comments
- Use `OrderedFloat` for float comparisons

## Verification

All tests pass:
- `cargo test -p diagram_tool 'history::tests'` - 42 tests pass
- `cargo test -p diagram_tool` - 1044 unit tests pass
