# Martin Fowler Test Plan

## Test Naming Convention
Tests follow `test_<scenario>_<expected_behavior>` format with Given-When-Then structure.

## Happy Path Tests

### test_single_node_style_change_dispatches_update_to_db
**Given**: A document with one node "n1" selected in PropertiesPanel
**When**: User changes node style from "box" to "cloud"
**Then**:
- Node style in document is updated to Cloud
- Revision is incremented
- History contains previous state
- EventEnvelope with NodeStyleUpdate is sent to db_tx

### test_style_change_to_dashed_updates_document
**Given**: A node with NodeStyle::Box currently set
**When**: User selects "dashed" from the style dropdown
**Then**:
- doc.document.nodes["n1"].style == Some(NodeStyle::Dashed)
- doc.revision incremented by 1

### test_style_change_to_cylinder_updates_document
**Given**: A node with default style
**When**: User selects "cylinder" style
**Then**:
- Node style is updated to Cylinder
- EventEnvelope sent with style: NodeStyle::Cylinder

## Error Path Tests

### test_no_node_selected_hides_style_selector
**Given**: PropertiesPanel with no nodes selected (selected_node_count = 0)
**When**: User views PropertiesPanel
**Then**:
- Style selector is not rendered
- No error occurs

### test_multiple_nodes_selected_hides_style_selector
**Given**: PropertiesPanel with 2 nodes selected
**When**: User views PropertiesPanel
**Then**:
- Style selector is not rendered (only shown for single node)

### test_db_tx_none_does_not_panic
**Given**: db_tx is None (async-db feature disabled)
**When**: User attempts to change node style
**Then**:
- Document is still updated locally
- No panic occurs
- Warning is logged (if logging configured)

### test_invalid_style_value_rejected
**Given**: Parser receives invalid style string
**When**: parse_node_style("invalid") is called
**Then**:
- Returns Error::InvalidStyle
- Document unchanged

### test_node_not_found_returns_error
**Given**: Selected node ID does not exist in document
**When**: on_node_style_change is called with missing node ID
**Then**:
- Returns Err(Error::NodeNotFound)
- No mutation occurs

## Edge Case Tests

### test_same_style_no_dispatch
**Given**: Node already has style = Box
**When**: User selects "box" again (no actual change)
**Then**:
- History NOT pushed (no state change)
- db_tx NOT sent (idempotent)
- Revision NOT incremented

### test_style_change_from_none_to_some
**Given**: Node has style = None (not set)
**When**: User selects "box" style
**Then**:
- Node style becomes Some(Box)
- Event dispatched normally

### test_style_change_clears_to_none
**Given**: Node has style = Some(Cloud)
**When**: User selects "default" or "none" option (if available)
**Then**:
- Node style becomes None
- Event dispatched with null/None

### test_rapid_style_changes_dispatches_all
**Given**: User quickly changes style box -> cloud -> dashed
**When**: All three changes processed
**Then**:
- Three EventEnvelopes sent to db_tx
- Each with correct style value

### test_style_change_preserves_other_node_fields
**Given**: Node with various fields (label, x, y, width, height, icon, etc.)
**When**: Only style is changed
**Then**:
- All other node fields remain unchanged
- Only style field and revision modified

## Contract Verification Tests

### test_precondition_single_node_selected
**Given**: selected_node_count = 1
**When**: on_node_style_change is called
**Then**: Precondition P1 satisfied - function proceeds

### test_precondition_valid_style
**Given**: new_style = NodeStyle::Box
**When**: on_node_style_change is called
**Then**: Precondition P2 satisfied - valid enum value

### test_precondition_node_exists
**Given**: node_id exists in document
**When**: on_node_style_change is called
**Then**: Precondition P3 satisfied - node found

### test_postcondition_document_updated
**Given**: Node with old_style, new_style = Cloud
**When**: on_node_style_change completes
**Then**: Postcondition Q1 - node.style == Some(Cloud)

### test_postcondition_revision_incremented
**Given**: Initial revision = 5
**When**: Style change succeeds
**Then**: Postcondition Q2 - revision == 6

### test_postcondition_history_pushed
**Given**: History with 3 entries
**When**: Style change succeeds
**Then**: Postcondition Q3 - History has 4 entries, last is pre-change state

### test_postcondition_event_dispatched
**Given**: db_tx is Some(coroutine)
**When**: Style change succeeds
**Then**: Postcondition Q4 - db_tx received EventEnvelope with NodeStyleUpdate

## Contract Violation Tests

### test_violation_p1_no_selection_does_nothing
**Given**: selected_node_count = 0
**When**: User tries to change style (selector should be hidden anyway)
**Then**: Returns early, no error, no mutation

### test_violation_p2_invalid_style_rejected
**Given**: parse function receives "invalid_style"
**When**: Function executes
**Then**: Returns Err(Error::InvalidStyle)

### test_violation_p3_missing_node_rejected
**Given**: node_id = "nonexistent"
**When**: on_node_style_change("nonexistent", Box)
**Then**: Returns Err(Error::NodeNotFound)

### test_violation_q1_document_not_updated_on_error
**Given**: Valid style change but db error
**When**: After error, document state checked
**Then**: Q1 - node.style should be unchanged (transaction rollback)

### test_violation_q2_revision_not_incremented_on_failure
**Given**: Operation fails midway
**When**: After failure, revision checked
**Then**: Q2 - revision unchanged (failed operations don't increment)

### test_violation_q4_event_not_sent_on_failure
**Given**: db_tx send fails
**When**: After failure, db_tx state checked
**Then**: Q4 - No event sent (or sent but failed, depending on design)

## Integration / E2E Tests

### test_properties_panel_complete_style_update_flow
**Given**: Running app with canvas, PropertiesPanel visible, one node selected
**When**:
1. User clicks style dropdown in PropertiesPanel
2. User selects "Cloud"
3. Event processes

**Then**:
- UI reflects new style immediately
- db_tx receives NodeStyleUpdate envelope
- Store persists the change
- Undo (Ctrl+Z) reverts to previous style

### test_style_change_survives_refresh
**Given**: Node style changed and persisted
**When**: App refreshes/reloads document
**Then**:
- Node loads with correct style from store
- PropertiesPanel shows current style as selected

## Given-When-Then Scenarios

### Scenario 1: User Changes Node Style
**Given**: A diagram with a single node "API" selected
**And**: Current style is "box"
**When**: User selects "cloud" from PropertiesPanel style dropdown
**Then**:
- The node shape changes to cloud shape on canvas
- PropertiesPanel shows "cloud" as selected
- History stack now includes pre-change state
- db_tx received EventEnvelope: `{op_type: "node_style_update", id: "n1", style: "cloud"}`

### Scenario 2: User Tries to Change Style Without Selection
**Given**: PropertiesPanel open, no nodes selected
**When**: User looks for style dropdown
**Then**:
- Style dropdown is not visible
- No error messages displayed

### Scenario 3: User Changes Style Then Undoes
**Given**: Node style changed from box to cloud
**When**: User presses Ctrl+Z (undo)
**Then**:
- Node reverts to box style
- History pops the state
- db_tx does NOT receive another event (undo is local)

---

## Test Implementation Notes

1. **Mock db_tx**: Use mock or spy pattern to verify EventEnvelope sent
2. **Test data**: Create test DiagramDocument with known node IDs and styles
3. **Isolation**: Each test should create fresh doc_signal and history
4. **Assertions**: Use exact equality for revision, style enum matching
