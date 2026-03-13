---
bead_id: seshat-dfs
bead_title: UI Dispatch: Node Creation
phase: contract-verification
updated_at: 2026-03-12T00:00:00Z
---

# Martin Fowler Test Plan

## Happy Path Tests

### test_double_click_empty_canvas_creates_node_and_dispatches_to_wal
Given: User double-clicks on empty canvas (no nodes/edges at coordinates) with Select tool active, and db_tx is Some(coroutine)
When: handle_canvas_double_click_node_creation is invoked (or canvas.rs ondoubleclick handler runs)
Then:
- Node is inserted into document.nodes with generated UUID (Uuid::new_v4())
- Node appears at position (x-32, y-32) due to centering in canvas.rs line 1764-1765
- New node ID is inserted into editor_state.selected_items
- Document revision is incremented by 1
- editing_node and editing_edge are set to None
- edit_value is cleared
- db_tx.send() is called once with EventEnvelope containing DomainOp::NodeAdd
- DispatchResult with nodes_affected=1, dispatches_sent=1 is returned

### test_toolbar_add_node_button_creates_node_and_dispatches_to_wal
Given: User clicks Add Node button in toolbar (to be added in toolbar.rs), and db_tx is Some(coroutine)
When: handle_toolbar_add_node is invoked
Then:
- Node is inserted into document.nodes with generated UUID at viewport center
- New node ID is inserted into editor_state.selected_items
- Document revision is incremented by 1
- editing_node and editing_edge are set to None
- edit_value is cleared
- db_tx.send() is called once with EventEnvelope containing DomainOp::NodeAdd

### test_dispatch_envelope_data_matches_local_node
Given: User double-clicks on empty canvas with valid coordinates (100, 200), db_tx is Some
When: Node creation completes
Then:
- Local node has x=100, y=200, width=64, height=64, label="Node"
- Dispatched envelope has DomainOp::NodeAdd with matching id, x, y, width, height, label

## Error Path Tests

### test_double_click_with_invalid_coordinates_returns_error
Given: User double-clicks on canvas with coordinates (NaN, 100.0)
When: handle_canvas_double_click_node_creation is invoked
Then:
- Returns Err(DispatchError::InvalidCoordinates)
- No node is created in document.nodes
- No dispatch is sent to db_tx

### test_double_click_with_invalid_dimensions_returns_error
Given: User double-clicks on canvas with valid coordinates but width=0
When: create_node_add_envelope is invoked internally
Then:
- Returns Err(DispatchError::InvalidCoordinates) from create_node_add_envelope
- No node is created locally
- No dispatch is sent

### test_toolbar_add_node_without_select_tool_creates_node
Given: User clicks Add Node button in toolbar (toolbar ignores current tool mode)
When: handle_toolbar_add_node is invoked
Then:
- Node is created and dispatched regardless of current tool

### test_double_click_with_non_select_tool_does_not_create_node
Given: User double-clicks on empty canvas with tool = ToolMode::Pan (not Select)
When: handle_canvas_double_click_node_creation is invoked
Then:
- No node is created
- No dispatch is sent

## Edge Case Tests

### test_double_click_with_wal_disconnected_creates_node_locally
Given: User double-clicks on empty canvas, db_tx is None (WAL disconnected)
When: handle_canvas_double_click_node_creation is invoked
Then:
- Node is created locally in document.nodes
- editor_state.selected_items updated
- revision incremented
- Warning is logged about WAL disconnection
- No panic or error propagated to user

### test_toolbar_with_wal_disconnected_creates_node_locally
Given: User clicks Add Node button, db_tx is None
When: handle_toolbar_add_node is invoked
Then:
- Node is created locally
- Warning is logged

### test_double_click_hitting_node_does_not_create_new_node
Given: User double-clicks on existing node (hit test returns Some node)
When: handle_canvas_double_click_node_creation is invoked
Then:
- No new node is created
- No dispatch is sent
- Existing node enters edit mode instead

### test_double_click_hitting_edge_does_not_create_new_node
Given: User double-clicks on existing edge (hit test returns Some edge, no node)
When: handle_canvas_double_click_node_creation is invoked
Then:
- No new node is created
- No dispatch is sent
- Existing edge enters edit mode instead

### test_multiple_nodes_have_unique_ids
Given: User creates multiple nodes via double-clicks
When: handle_canvas_double_click_node_creation is called multiple times
Then:
- Each node has a unique UUID
- No ID collisions occur

## Contract Verification Tests

### test_precondition_double_click_event_triggers_handler
Given: Dioxus ondoubleclick event handler is bound
When: User double-clicks on canvas
Then:
- handle_canvas_double_click_node_creation is invoked

### test_precondition_tool_is_select_mode
Given: Current tool is ToolMode::Select
When: User double-clicks on empty canvas
Then:
- Node creation proceeds

### test_precondition_empty_canvas_hit_test
Given: hit_node is None and hit_edge is None
When: Double-click occurs
Then:
- Node creation proceeds

### test_precondition_valid_coordinates
Given: Coordinates x, y are finite (not NaN/Infinity)
When: create_node_add_envelope is called
Then:
- Envelope is created successfully

### test_postcondition_dispatch_sent
Given: db_tx is Some(coroutine) and all preconditions met
When: Node creation completes
Then:
- db_tx.send(envelope) was called exactly once

### test_postcondition_local_node_created
Given: All preconditions met
When: Node creation completes
Then:
- document.nodes contains new node with generated ID

### test_postcondition_selection_updated
Given: All preconditions met
When: Node creation completes
Then:
- editor_state.selected_items contains new node ID

### test_postcondition_revision_incremented
Given: All preconditions met
When: Node creation completes
Then:
- document.revision = old_revision + 1

### test_postcondition_edit_state_cleared
Given: All preconditions met
When: Node creation completes
Then:
- editing_node == None
- editing_edge == None
- edit_value == String::new()

### test_invariant_envelope_validity
Given: Envelope is dispatched
When: Envelope is processed
Then:
- op_id is non-empty UUID string
- operation is DomainOp::NodeAdd with all fields valid
- author is present
- timestamp is valid

### test_invariant_node_uniqueness
Given: New node is created
When: Checking document.nodes
Then:
- nodes.contains_key(new_node_id) == true
- No other node has same ID

### test_invariant_consistency_local_vs_dispatched
Given: Node is created and dispatched
When: Comparing local node with dispatched envelope
Then:
- local_node.id == envelope.operation.id
- local_node.x == envelope.operation.x
- local_node.y == envelope.operation.y
- local_node.width == envelope.operation.width
- local_node.height == envelope.operation.height
- local_node.label == envelope.operation.label

## Contract Violation Tests

### test_violation_p4_invalid_coordinates_returns_invalid_coordinates_error
Given: coords = (f64::NAN, 100.0)
When: handle_canvas_double_click_node_creation(coords)
Then: returns Err(DispatchError::InvalidCoordinates)

### test_violation_p5_invalid_dimensions_returns_invalid_coordinates_error
Given: width = 0.0, height = 64.0
When: create_node_add_envelope(id, x, y, width, height, label)
Then: returns Err(DispatchError::InvalidCoordinates)

### test_violation_q6_mismatch_local_vs_envelope_fails_consistency_check
Given: Local node and envelope are created but x differs
When: Invariant I3 consistency check runs
Then: Test fails - local and dispatched data must match

## Given-When-Then Scenarios

### Scenario 1: User creates node via double-click
Given: Canvas has no nodes, Select tool is active, user double-clicks at (150, 250), db_tx is Some
When: handle_canvas_double_click_node_creation is invoked
Then:
- New node appears at position (150, 250) in document
- Node is selected in UI
- Backend receives DomainOp::NodeAdd envelope
- User can immediately edit the new node's label

### Scenario 2: User creates node via toolbar button
Given: Canvas has existing nodes, user clicks "Add Node" in toolbar, db_tx is Some
When: handle_toolbar_add_node is invoked
Then:
- New node appears at viewport center
- Node is selected in UI
- Backend receives DomainOp::NodeAdd envelope

### Scenario 3: Network disconnected during node creation
Given: User double-clicks to create node, but WAL is disconnected (db_tx = None)
When: Node creation completes
Then:
- Node appears locally in document
- UI shows no error (fire-and-forget dispatch)
- When WAL reconnects, subsequent operations work normally

### Scenario 4: User tries to create node on top of existing node
Given: Canvas has node at position (100, 100), user double-clicks at (100, 100)
When: hit test finds existing node
Then:
- No new node is created
- Existing node enters edit mode (editing_node set to existing ID)
