# Martin Fowler Test Plan: seshat-zrx

## Test Naming Convention
- Use expressive Given-When-Then names
- Format: `test_<scenario>_<expected_result>`
- Focus on behavior, not implementation

## Happy Path Tests

### test_single_node_selected_shows_shape_selector
**Given**: A document with exactly one node selected
**When**: PropertiesPanel renders
**Then**:
- Shape selector dropdown is visible
- Current node style is displayed as selected value

### test_shape_selector_changes_node_style
**Given**: A document with one node selected, current style is "box"
**When**: User selects "cloud" from shape dropdown
**Then**:
- Node.style is updated to NodeStyle::Cloud
- Document revision increments
- History contains previous state for undo

### test_style_change_dispatches_event_to_db_tx
**Given**: db_tx coroutine is available and listening
**When**: User changes node shape
**Then**:
- EventEnvelope is sent to db_tx
- Operation is DomainOp::NodeStyleUpdate { id, style }
- Author is "local-user"
- Timestamp is current Unix time

### test_style_change_only_dispatches_on_actual_change
**Given**: A node with style "box" selected
**When**: User selects "box" again (no change)
**Then**:
- History is NOT pushed (no redundant undo state)
- No event is dispatched to db_tx

## Error Path Tests

### test_no_shape_selector_when_zero_nodes_selected
**Given**: No nodes selected (selected_node_count = 0)
**When**: PropertiesPanel renders
**Then**:
- Shape selector is NOT rendered
- Panel shows "0 node(s), 0 edge(s) selected"

### test_no_shape_selector_when_multiple_nodes_selected
**Given**: Multiple nodes selected (selected_node_count > 1)
**When**: PropertiesPanel renders
**Then**:
- Shape selector is NOT rendered
- Multi-select hint is displayed

### test_shape_selector_hidden_when_no_single_node
**Given**: Exactly one edge selected (selected_edge_count = 1)
**When**: PropertiesPanel renders
**Then**:
- Shape selector is NOT rendered
- Edge properties panel is shown instead

### test_handles_invalid_style_string_gracefully
**Given**: User enters invalid style value in any scenario
**When**: parse_node_style is called with unknown string
**Then**:
- Returns NodeStyle::Box (default)
- No panic or error

## Edge Case Tests

### test_all_four_node_styles_are_available
**Given**: Single node selected
**When**: Shape dropdown is opened
**Then**:
- All 4 options present: Box, Cloud, Cylinder, Dashed

### test_style_persists_across_panel_renders
**Given**: Node style changed to "cloud"
**When**: PropertiesPanel re-renders (e.g., after other edit)
**Then**:
- "cloud" is displayed as selected value

### test_style_change_with_locked_node
**Given**: A locked node is selected
**When**: User attempts to change shape
**Then**:
- Shape selector may or may not be disabled (implementation decision)
- If enabled, change should still work

### test_history_contains_previous_state_after_change
**Given**: Document with node in initial state
**When**: User changes node shape
**Then**:
- history.push() was called with pre-change document
- Undo would restore previous style

## Contract Verification Tests

### test_precondition_single_node_selected
**Given**: selected_node_count = 1
**When**: Shape selector is rendered
**Then**: Guard passes, selector visible

### test_precondition_db_tx_available
**Given**: db_tx = Some(coroutine)
**When**: Event dispatch is attempted
**Then**: tx.send() succeeds without panic

### test_postcondition_envelope_sent
**Given**: Valid style change
**When**: onchange handler fires
**Then**: db_tx receives EventEnvelope

### test_postcondition_revision_incremented
**Given**: Any document mutation
**When**: After mutation completes
**Then**: doc.revision > original_revision

### test_invariant_style_valid
**Given**: Any NodeStyle value
**When**: Stored in document
**Then**: Is one of Box, Cloud, Cylinder, Dashed

## Contract Violation Tests

### test_violation_p1_zero_selected_returns_no_selector
**Given**: selected_node_count = 0
**When**: PropertiesPanel renders
**Then**: Shape selector is NOT in the output (verify via HTML inspection)

### test_violation_p1_multiple_selected_returns_no_selector
**Given**: selected_node_count = 3
**When**: PropertiesPanel renders
**Then**: Shape selector is NOT in the output

### test_violation_p3_db_tx_none_silent_failure
**Given**: db_tx = None
**When**: User triggers style change
**Then**: Local state updates (if any), no panic, db_tx not accessed

### test_violation_q1_no_envelope_without_db_tx
**Given**: db_tx = None
**When**: Style change is attempted
**Then**: No attempt to call tx.send()

## Given-When-Then Scenarios

### Scenario 1: User Changes Node Shape
**Given**: A diagram with one rectangle node (style: Box) is open
**And**: User has selected that node
**When**: User clicks the Shape dropdown and selects "Cloud"
**Then**:
- The node visually changes to cloud shape
- The PropertiesPanel shows "cloud" as selected
- The change appears in the document model as NodeStyle::Cloud
- An event is dispatched to db_tx for persistence

### Scenario 2: User Undoes Shape Change
**Given**: User previously changed node from Box to Cloud
**And**: History contains pre-change state
**When**: User triggers undo (Ctrl+Z)
**Then**:
- Node style reverts to Box
- Document revision increments
- Reverse event may be dispatched

### Scenario 3: Shape Change With No Persistence Backend
**Given**: db_tx context is not available (None)
**When**: User changes node shape
**Then**:
- UI still updates the local document
- No crash occurs
- User can continue editing

### Scenario 4: Rapid Shape Changes
**Given**: A node is selected
**When**: User quickly cycles through all 4 shape options
**Then**:
- Each change is dispatched as separate event
- History contains multiple entries
- Final state reflects last selection

## Integration Test (End-to-End)

### test_full_style_change_flow
**Given**: 
- Empty document with one node (id: "n1", style: Box)
- db_tx coroutine is initialized and capturing events

**When**:
1. User selects node "n1"
2. PropertiesPanel shows with Box selected
3. User selects "Cylinder"
4. Event is dispatched

**Then**:
- Node in document has style: NodeStyle::Cylinder
- Revision incremented from 1 to 2
- db_tx received EventEnvelope with:
  - op_id: valid UUID string
  - operation: DomainOp::NodeStyleUpdate { id: "n1", style: Cylinder }
  - author.id: "local-user"
  - timestamp: within last 1000ms
