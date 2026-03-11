# Martin Fowler Test Plan

## Test Organization
- Test naming: `test_<scenario>_<expected_behavior>`
- Format: Given-When-Then
- Each test is an executable specification

## Happy Path Tests

### test_double_click_empty_canvas_creates_node_and_dispatches_to_wal
**Given**: Canvas with no nodes, Select tool active, WAL connected (db_tx = Some)
**When**: User double-clicks on empty canvas at (100, 100)
**Then**:
- New node created in document.nodes with valid UUID
- Node positioned at snapped coordinates
- EventEnvelope sent to db_tx containing DomainOp::NodeAdd
- Node ID added to editor_state.selected_items
- Document revision incremented by 1
- editing_node = None, editing_edge = None, edit_value = ""

### test_double_click_with_snap_to_grid_applies_snap
**Given**: Canvas with snap_to_grid=true, grid_size=16, WAL connected
**When**: User double-clicks at (105, 107)
**Then**:
- Node x = 96 (snapped to 16-grid: floor(105/16)*16)
- Node y = 96 (snapped to 16-grid: floor(107/16)*16)
- Dispatched coordinates match snapped values

### test_double_click_sets_correct_node_properties
**Given**: Canvas empty, Select tool, WAL connected
**When**: User double-clicks at (200, 200)
**Then**:
- Node has default width = 64.0
- Node has default height = 64.0
- Node has label = "Node"
- Node has kind = NodeKind::Node
- Node is not locked (locked = false)
- Node has empty tags and metadata

## Error Path Tests

### test_double_click_wal_disconnected_still_creates_node_locally
**Given**: Canvas empty, Select tool, WAL disconnected (db_tx = None)
**When**: User double-clicks at (100, 100)
**Then**:
- Node still created in document.nodes (best-effort UI responsiveness)
- No error returned to user
- No EventEnvelope sent (graceful degradation)
- Node ID added to selection
- Note: This is U1 (ubiquitous requirement) - UI must work offline

### test_double_click_with_invalid_coordinates_returns_error
**Given**: Canvas empty, Select tool, WAL connected
**When**: User double-clicks at (f64::NAN, 100)
**Then**:
- Err(DispatchError::InvalidCoordinates) returned
- No node created in document
- No EventEnvelope sent

### test_double_click_with_infinity_coordinates_returns_error
**Given**: Canvas empty, Select tool, WAL connected
**When**: User double-clicks at (f64::INFINITY, 100)
**Then**:
- Err(DispatchError::InvalidCoordinates) returned
- No node created

### test_double_click_send_failure_is_handled
**Given**: Canvas empty, Select tool, db_tx channel full/closed
**When**: User double-clicks at (100, 100) and send() fails
**Then**:
- Err(DispatchError::SendFailed) propagated
- Node may or may not be created locally (depends on implementation decision)

## Edge Case Tests

### test_double_click_on_existing_node_does_not_create_new
**Given**: Canvas with node at (100, 100), Select tool, WAL connected
**When**: User double-clicks on existing node
**Then**:
- No new node created
- No EventEnvelope sent
- Node enters edit mode (existing behavior preserved)

### test_double_click_on_existing_edge_does_not_create_new
**Given**: Canvas with edge, Select tool, WAL connected
**When**: User double-clicks on edge
**Then**:
- No new node created
- Edge enters edit mode (existing behavior preserved)

### test_double_click_with_different_tool_does_not_create
**Given**: Canvas empty, tool = ToolMode::Pan (not Select)
**When**: User double-clicks on empty canvas
**Then**:
- No node created
- No EventEnvelope sent

### test_double_click_near_grid_boundary
**Given**: Canvas with snap_to_grid=true, grid_size=16
**When**: User double-clicks at (0, 0)
**Then**:
- Node created at (0, 0) (corner case handled)
- Coordinates valid (not negative after snap)

### test_double_click_generates_unique_node_ids
**Given**: Canvas empty, Select tool, WAL connected
**When**: User double-clicks twice at different locations (100,100) then (200,200)
**Then**:
- First node ID != second node ID
- Both nodes exist in document.nodes

## Contract Verification Tests

### test_precondition_p1_double_click_event_triggers
**Given**: Dioxus event system
**When**: ondoubleclick handler fires
**Then**: Handler receives MouseEvent with trigger = DoubleClick

### test_precondition_p2_select_tool_required
**Given**: Tool signal
**When**: tool != ToolMode::Select
**Then**: Node creation branch not entered

### test_precondition_p3_empty_canvas_required
**Given**: Hit test result
**When**: hit_node.is_some() || hit_edge.is_some()
**Then**: Node creation branch not entered

### test_precondition_p5_wal_connected_for_dispatch
**Given**: db_tx context
**When**: db_tx = None
**Then**: No send() attempted (graceful: still create locally per U1)

### test_postcondition_q1_event_dispatched
**Given**: WAL connected, valid double-click
**When**: Node created
**Then**: db_tx.send() called with EventEnvelope containing DomainOp::NodeAdd

### test_postcondition_q2_node_in_document
**Given**: Valid double-click
**When**: Handler completes
**Then**: doc_signal.read().document.nodes contains new node

### test_postcondition_q3_selection_updated
**Given**: Valid double-click
**When**: Handler completes
**Then**: editor_state.selected_items contains new node ID

### test_postcondition_q4_revision_incremented
**Given**: Initial revision = N
**When**: Handler completes
**Then**: document.revision = N + 1

### test_invariant_i1_envelope_validity
**Given**: EventEnvelope sent
**When**: Examining envelope
**Then**:
- op_id is non-empty valid UUID string
- operation is DomainOp::NodeAdd
- author.id = "local-user"
- author.name = "Local User"
- timestamp > 0

### test_invariant_i2_node_uniqueness
**Given**: New node created
**When**: Checking document.nodes
**Then**: node ID does not exist in original nodes map

## Contract Violation Tests

### test_violation_p5_wal_disconnected_returns_error_variant
**Given**: db_tx = None
**When**: handle_canvas_double_click(...) is called
**Then**: Returns `Err(DispatchError::WalDisconnected)` for dispatch path (but node still created locally per U1)

### test_violation_p4_invalid_coordinates_returns_error
**Given**: coords = (f64::NAN, 100.0)
**When**: handle_canvas_double_click(...) is called
**Then**: Returns `Err(DispatchError::InvalidCoordinates)`, no node created

### test_violation_q1_send_failure_propagates
**Given**: db_tx = Some(closed_channel)
**When**: handle_canvas_double_click(...) is called
**Then**: Returns `Err(DispatchError::SendFailed)`

## Given-When-Then Scenarios

### Scenario: User creates node via double-click (optimal path)
**Given**:
- Canvas is empty (no nodes or edges at click point)
- Current tool is Select mode
- WAL is connected (db_tx = Some)
- snap_to_grid is disabled

**When**:
- User double-clicks at canvas position (150, 200)

**Then**:
- A new node appears at (150, 200)
- Node has default dimensions (64x64) and label "Node"
- Node is selected (highlighted in UI)
- Backend receives NodeAdd event via db_tx
- Edit mode is not entered (focus remains on canvas)

### Scenario: User creates node while WAL is offline
**Given**:
- Canvas is empty
- Current tool is Select mode
- WAL is disconnected (db_tx = None)

**When**:
- User double-clicks at canvas position (150, 200)

**Then**:
- A new node appears locally (best-effort)
- No error shown to user
- Event will be dispatched when WAL reconnects (eventual consistency)

### Scenario: User accidentally double-clicks on existing node
**Given**:
- Canvas contains a node at (150, 200)
- Current tool is Select mode

**When**:
- User double-clicks on the existing node

**Then**:
- No new node created
- Existing node enters edit mode (label becomes editable)
- This is existing behavior, unchanged by this feature
