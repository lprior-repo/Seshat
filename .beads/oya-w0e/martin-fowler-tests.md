# Martin Fowler Test Plan

## Test Categories

### Happy Path Tests
- test_single_pointer_down_sets_captured_pointer
- test_pointer_move_processed_for_captured_pointer_only
- test_pointer_up_clears_captured_pointer_and_resets_mode

### Error Path / Edge Cases
- test_second_pointer_down_while_dragging_is_ignored
- test_pointer_move_ignored_when_not_captured
- test_rapid_pointer_sequence_no_state_corruption
- test_pointer_up_for_non_captured_pointer_removes_from_active

## Given-When-Then Scenarios

### Scenario 1: Single pointer drag selection works
**Given**: Canvas is idle, no active pointers
**When**: User presses pointer 1 on canvas and drags
**Then**:
- captured_pointer = Some(1)
- active_pointers = {1}
- Drag selection proceeds normally
- On release: captured_pointer = None, mode = Select

### Scenario 2: Second pointer down while dragging is ignored (MUL-009)
**Given**: captured_pointer = Some(1), active_pointers = {1}, user is dragging
**When**: Second pointer (id=2) touches down on canvas
**Then**:
- captured_pointer remains Some(1)
- active_pointers remains {1} (2 is NOT added for drag purposes)
- Pointer 2 does NOT initiate any action
- No state corruption occurs

### Scenario 3: Captured pointer releases while another is down
**Given**: captured_pointer = Some(1), active_pointers = {1, 2} (both down)
**When**: Pointer 1 releases (pointerup with id=1)
**Then**:
- captured_pointer = None
- active_pointers = {2}
- interaction_mode = Select (resets from DraggingSelection)
- Pointer 2 remains but is ignored (no capture)

### Scenario 4: Non-captured pointer releases
**Given**: captured_pointer = Some(1), active_pointers = {1, 2}
**When**: Pointer 2 releases (pointerup with id=2)
**Then**:
- captured_pointer remains Some(1)
- active_pointers = {1}
- Drag continues for pointer 1 uninterrupted

### Scenario 5: Move ignored for non-captured pointer
**Given**: captured_pointer = Some(1), user is dragging
**When**: Pointer 2 moves on canvas
**Then**:
- Pointer 2's movement is ignored
- Selection box does not update
- No state corruption

## Contract Verification Tests

### Precondition Tests
- test_precondition_p1_pointer_id_valid
- test_precondition_p2_active_pointers_initialized
- test_precondition_p3_single_captured_pointer

### Postcondition Tests
- test_postcondition_q1_pointer_added_to_active_on_down
- test_postcondition_q2_pointer_removed_from_active_on_up
- test_postcondition_q3_new_pointer_ignored_when_captured_exists
- test_postcondition_q4_mode_resets_on_captured_release

### Invariant Tests
- test_invariant_i1_captured_pointer_single_or_none
- test_invariant_i2_active_pointers_count_matches_browser
- test_invariant_i3_only_captured_pointer_can_drag
