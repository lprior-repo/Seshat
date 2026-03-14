bead_id: seshat-i0u
bead_title: Implement copy/paste for single node (CLP-001)
phase: test-plan
updated_at: 2026-03-14T16:04:00Z

# Martin Fowler Test Plan

## Happy Path Tests
- test_clp001_copy_paste_single_node_creates_new_node_with_new_id
  Given: Document with one node at (0,0)
  When: Copy node, then paste with serial 1
  Then: New node created at (20,20) with new UUID, original node unchanged

## Error Path Tests
- test_copy_returns_error_when_selection_is_empty
  Given: Empty selection
  When: copy() is called
  Then: Returns Err(Error::EmptySelection)

- test_cut_returns_error_when_selection_is_empty
  Given: Empty selection
  When: cut() is called  
  Then: Returns Err(Error::EmptySelection)

- test_paste_returns_error_when_clipboard_is_empty
  Given: Empty clipboard
  When: paste() is called
  Then: Returns Err(Error::EmptyClipboard)

## Edge Case Tests
- test_clp005_paste_operation_applies_incremental_offset_based_on_serial
  Given: One node in document
  When: Paste 3 times with serial 1, 2, 3
  Then: Each paste creates node at 20*x, 40*x, 60*x offset

## Contract Verification Tests
- test_p1_violation_returns_empty_selection_error
  Verifies P1: Empty selection returns EmptySelection error

- test_p3_violation_returns_empty_selection_error
  Verifies P2: Empty selection for cut returns EmptySelection error

- test_p4_violation_returns_empty_clipboard_error
  Verifies P3: Empty clipboard for paste returns EmptyClipboard error

- test_q6_violation_returns_invalid_edge_reference_error
  Verifies edge validation in paste

- test_q7_violation_returns_invalid_parent_reference_error  
  Verifies parent reference validation in paste

## Given-When-Then Scenarios

### Scenario 1: Single Node Copy/Paste
Given: A document with a single node at position (0, 0)
When: User selects the node and copies, then pastes
Then: A new node appears at (20, 20) with a new unique ID
And: The original node remains at (0, 0) unchanged
