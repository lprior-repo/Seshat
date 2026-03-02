bead_id: bd-2b4
bead_title: commands: Add unit tests for copy/paste operations
phase: p1
updated_at: 2026-03-01T17:05:00Z

# Implementation: Copy/Paste Operations Unit Tests

## Approach

Adding missing tests to the existing `#[cfg(test)]` module in `diagram_tool/src/ui/commands.rs`.

## Tests Added

### Copy Operation Tests

1. `given_selection_with_nonexistent_ids_when_copy_then_returns_false`
   - Tests that selecting IDs not in document nodes returns false

2. `given_three_nodes_selected_when_copy_then_copies_all`
   - Tests copying multiple (3+) nodes

3. `given_partial_edge_selection_when_copy_then_excludes_edge`
   - Tests that edges are excluded when only one endpoint is selected

4. `given_nested_nodes_selected_when_copy_then_preserves_parent_reference`
   - Tests that parent references are preserved during copy (remapped at paste)

### Paste Operation Tests

5. `given_clipboard_with_empty_nodes_when_paste_then_returns_false`
   - Tests that empty nodes vector returns false

6. `given_second_paste_when_paste_then_applies_double_offset`
   - Tests offset accumulation across multiple pastes

7. `given_multiple_nodes_when_paste_then_all_ids_unique`
   - Tests that all pasted node IDs are unique

8. `given_edge_in_clipboard_when_paste_then_remapped_to_new_ids`
   - Tests edge source/target remapping

9. `given_parent_also_pasted_when_paste_then_remapped`
   - Tests parent remapping when parent is also in clipboard

10. `given_parent_not_pasted_when_paste_then_preserved`
    - Tests parent preservation when parent not in clipboard

11. `given_paste_successful_when_paste_then_selection_updated`
    - Tests selection contains only new pasted node IDs

12. `given_paste_successful_when_paste_then_revision_incremented`
    - Tests revision increment on successful paste

## Implementation Details

- Uses existing helper functions (`clear_clipboard`, `make_node`, `make_doc_with_node`, `make_doc_with_two_nodes_and_edge`)
- Added new helper `make_doc_with_parent_child` for parent-child tests
- All tests follow zero-unwrap policy using `if let` and `assert!` patterns
- Clipboard cleared at start of each test for isolation

## Files Modified

- `/home/lewis/src/seshat/diagram_tool/src/ui/commands.rs`
  - Added 12 new test functions to existing `tests` module
  - Added 1 new helper function for parent-child test setup
