# Martin Fowler Test Plan: Edge Label Inline Editing

## Test Structure
- Uses Given-When-Then pattern
- Expressive test names describing behavior
- Tests organized by category: Happy Path, Error Path, Edge Cases, Contract Verification

## Happy Path Tests

### Scenario: Double-click on edge enters editing mode
**test_double_click_on_edge_starts_editing**
- Given: Document with an edge (edge_id: "e1", label: "existing label")
- When: User double-clicks on the edge
- Then:
  - `editing_edge` signal contains `Some("e1")`
  - `edit_value` signal contains "existing label"
  - Input overlay appears at edge midpoint

### Scenario: Enter key commits label change
**test_enter_key_commits_label_change**
- Given: Document with edge (label: "old"), editing_edge = Some("e1"), edit_value = "new label"
- When: User presses Enter key
- Then:
  - Document edge label is updated to "new label"
  - History records the change for undo
  - `editing_edge` is set to None
  - `edit_value` reflects the new label (for next edit)

### Scenario: Escape key cancels editing
**test_escape_key_cancels_editing**
- Given: Document with edge (label: "original"), editing_edge = Some("e1"), edit_value = "modified"
- When: User presses Escape key
- Then:
  - `editing_edge` is set to None
  - Document edge label remains "original" (unchanged)
  - History is not modified

### Scenario: Blur event commits label change
**test_blur_commits_label_change**
- Given: Document with edge (label: "old"), editing_edge = Some("e1"), edit_value = "new"
- When: User clicks outside the input (blur event)
- Then:
  - Document edge label is updated to "new"
  - History records the change
  - Editing mode is exited

### Scenario: Empty label shows placeholder when selected
**test_empty_label_shows_placeholder_when_selected**
- Given: Document with edge (label: ""), edge is selected
- When: Edge is rendered
- Then:
  - Placeholder text "label" is displayed at edge midpoint

## Error Path Tests

### Scenario: Double-click on empty canvas does not enter editing
**test_double_click_on_background_does_not_edit**
- Given: Document with edges, no edge at click position
- When: User double-clicks on empty canvas area
- Then:
  - `editing_edge` remains None
  - No input overlay appears

### Scenario: Double-click on node does not edit edge
**test_double_click_on_node_does_not_edit_edge**
- Given: Document with node and edge
- When: User double-clicks on node (not edge)
- Then:
  - `editing_edge` remains None
  - `editing_node` is set to the node (node editing takes precedence)

### Scenario: Label change with deleted edge is handled gracefully
**test_edit_deleted_edge_handled_gracefully**
- Given: Document, editing_edge = Some("e1"), but edge "e1" no longer exists
- When: User presses Enter to commit
- Then:
  - No panic occurs
  - Error is logged (function uses `let _ =` pattern)
  - Editing mode is exited safely

## Edge Case Tests

### Scenario: Very long label text
**test_handles_very_long_label**
- Given: Document with edge, user enters very long text (1000+ chars)
- When: User commits the edit
- Then:
  - Label is updated to the long text
  - No truncation occurs (or truncation is explicit)

### Scenario: Label with special characters
**test_label_with_special_characters**
- Given: Document with edge
- When: User enters label with <, >, &, quotes, newlines
- Then:
  - Label is stored as-is (document model accepts any String)
  - Rendering handles escaped characters properly

### Scenario: Rapid double-click attempts
**test_rapid_double_click_handled**
- Given: Document with edge
- When: User double-clicks multiple times quickly
- Then:
  - Only one editing session is started
  - No duplicate signals or state corruption

### Scenario: Edit value cleared after cancel then new edit
**test_edit_value_reset_after_cancel_new_edit**
- Given: Document with edge (label: "first"), user starts edit, cancels with Escape
- When: User double-clicks same edge again
- Then:
  - `edit_value` shows "first" (current label), not previous edit value

### Scenario: Zoom below threshold hides labels
**test_zoom_below_threshold_hides_labels**
- Given: Document with edge (label: "test"), zoom = 0.2
- When: Edge is rendered
- Then:
  - Label text is not rendered (threshold is 0.3)

## Contract Verification Tests

### test_precondition_p1_edge_exists_on_double_click
- Given: Canvas position that hits an edge
- When: Double-click handler processes the event
- Then: find_edge_at returns Some(EdgeId)

### test_precondition_p2_edge_in_document
- Given: EdgeId from hit detection
- When: Editing is started
- Then: doc.document.edges.get(&eid) returns Some(Edge)

### test_postcondition_q1_editing_edge_set
- Given: Valid edge at click position
- When: Double-click processed
- Then: editing_edge.read() == Some(eid)

### test_postcondition_q3_label_updated_on_enter
- Given: Editing mode with changed value
- When: Enter key pressed
- Then: doc.document.edges.get(&eid).label == new_value

### test_postcondition_q4_history_updated
- Given: Editing mode with changed value
- When: Enter key pressed
- Then: history.undo_stack has new entry

### test_postcondition_q5_editing_cleared_on_escape
- Given: Editing mode active
- When: Escape key pressed
- Then: editing_edge.read() == None

### test_invariant_i1_mutual_exclusion
- Given: editing_node = Some(node_id)
- When: Edge editing is started
- Then: editing_node is set to None first (mutual exclusion)

## Contract Violation Tests

### test_violates_p1_double_click_no_edge_returns_none
- Given: Canvas coordinates with no edge
- When: find_edge_at(x, y) is called
- Then: Returns None (NOT a panic)

### test_violates_q3_label_unchanged_after_enter_bug
- Given: Edge with label "original", edit_value = "new"
- When: commit_inline_edit is called but has bug (no mutation)
- Then: Document label remains "original" (detectable via test)

### test_violates_q5_editing_not_cleared_on_escape_bug
- Given: editing_edge = Some(eid)
- When: Escape key handler has bug (doesn't clear)
- Then: editing_edge remains Some (detectable via test)

## Given-When-Then Scenarios

### Scenario: Complete edit workflow
**Given**: Diagram document with edge "e1" having label "flow"
**When**: 
1. User double-clicks on edge "e1"
2. Input appears with "flow"
3. User types "data flow"
4. User presses Enter
**Then**:
- Edge label is "data flow"
- History has entry for undo
- User can press Ctrl+Z to revert

### Scenario: Cancel edit workflow
**Given**: Diagram document with edge "e1" having label "old"
**When**:
1. User double-clicks on edge
2. User changes text to "new"
3. User presses Escape
**Then**:
- Edge label remains "old"
- No history entry created
- Editing mode exited cleanly

### Scenario: Edit empty label
**Given**: Diagram document with edge "e1" having empty label
**When**:
1. User double-clicks on edge
2. User types "process"
3. User presses Enter
**Then**:
- Edge label becomes "process"
- History records change
- Placeholder no longer shown (has actual label)
