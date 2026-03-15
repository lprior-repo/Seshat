# Martin Fowler Test Plan: LockState Enum Migration (seshat-sr3b)

## Overview

This test plan covers the migration from `Node.locked: bool` to `Node.lock_state: LockState` enum with backwards-compatible JSON transformation. Tests verify:
1. The new enum works correctly internally
2. JSON serialization/deserialization maintains backwards compatibility
3. The Subgraph exception is encapsulated
4. Real integration scenarios work end-to-end
5. User-facing behaviors are correctly implemented

## Happy Path Tests (Internal API)

### test_lock_state_unlocked_default
**Given**: A new Node created with Default trait  
**When**: Node is constructed without explicit lock_state  
**Then**: `node.lock_state` equals `LockState::Unlocked`

### test_lock_state_locked_variant
**Given**: A Node with lock_state set to LockState::Locked  
**When**: `node.lock_state.is_locked()` is called  
**Then**: Returns `true`

### test_lock_state_unlocked_is_locked_returns_false
**Given**: A Node with lock_state set to LockState::Unlocked  
**When**: `node.lock_state.is_locked()` is called  
**Then**: Returns `false`

### test_subgraph_always_movable_when_locked
**Given**: A Node with kind=NodeKind::Subgraph and lock_state=LockState::Locked  
**When**: `node.lock_state.is_movable(&node.kind)` is called  
**Then**: Returns `true` (Subgraphs are always movable)

### test_regular_node_not_movable_when_locked
**Given**: A Node with kind=NodeKind::Node and lock_state=LockState::Locked  
**When**: `node.lock_state.is_movable(&node.kind)` is called  
**Then**: Returns `false`

### test_regular_node_movable_when_unlocked
**Given**: A Node with kind=NodeKind::Node and lock_state=LockState::Unlocked  
**When**: `node.lock_state.is_movable(&node.kind)` is called  
**Then**: Returns `true`

### test_text_node_movable_when_locked
**Given**: A Node with kind=NodeKind::Text and lock_state=LockState::Locked  
**When**: `node.lock_state.is_movable(&node.kind)` is called  
**Then**: Returns `false` (Text nodes behave like regular nodes)

### test_text_node_movable_when_unlocked
**Given**: A Node with kind=NodeKind::Text and lock_state=LockState::Unlocked  
**When**: `node.lock_state.is_movable(&node.kind)` is called  
**Then**: Returns `true` (Text nodes behave like regular nodes)

## Serialization Tests (Backwards Compatible JSON)

### test_serialization_outputs_locked_bool
**Given**: A Node with lock_state=LockState::Locked  
**When**: Node is serialized to JSON  
**Then**: JSON contains `"locked": true` (not `"lock_state"`)

### test_serialization_outputs_unlocked_bool
**Given**: A Node with lock_state=LockState::Unlocked  
**When**: Node is serialized to JSON  
**Then**: JSON contains `"locked": false` (not `"lock_state"`)

### test_deserialization_accepts_legacy_locked_true
**Given**: JSON with `"locked": true` (legacy format)  
**When**: Node is deserialized  
**Then**: `node.lock_state` equals `LockState::Locked`

### test_deserialization_accepts_legacy_locked_false
**Given**: JSON with `"locked": false` (legacy format)  
**When**: Node is deserialized  
**Then**: `node.lock_state` equals `LockState::Unlocked`

### test_deserialization_missing_locked_field_defaults_to_unlocked
**Given**: JSON without locked or lock_state field  
**When**: Node is deserialized  
**Then**: `node.lock_state` defaults to `LockState::Unlocked`

### test_json_roundtrip_preserves_lock_state
**Given**: A Node with lock_state=LockState::Locked  
**When**: Node is serialized then deserialized  
**Then**: `node.lock_state` still equals `LockState::Locked`

### test_json_roundtrip_preserves_unlocked_state
**Given**: A Node with lock_state=LockState::Unlocked  
**When**: Node is serialized then deserialized  
**Then**: `node.lock_state` still equals `LockState::Unlocked`

### test_lock_state_hash_consistency
**Given**: Two Nodes with same lock_state  
**When**: Hash is computed for both  
**Then**: Hash values are equal

### test_lock_state_clone_preserves_value
**Given**: A Node with lock_state=LockState::Locked  
**When**: Node is cloned  
**Then**: Cloned node.lock_state equals `LockState::Locked`

## Error Path Tests (Runtime)

### test_deserialization_invalid_locked_value_handled
**Given**: JSON with `"locked": "invalid_string"` (not a boolean)  
**When**: Node is deserialized  
**Then**: Defaults to `LockState::Unlocked` (graceful degradation)

### test_deserialization_null_locked_field_handled
**Given**: JSON with `"locked": null`  
**When**: Node is deserialized  
**Then**: Defaults to `LockState::Unlocked`

### test_deserialization_malformed_json_handled
**Given**: JSON with `"locked":` (malformed)  
**When**: Deserialization attempted  
**Then**: Returns deserialization error (proper error handling)

## Edge Case Tests (Complete Combinatorial Coverage)

### test_all_node_kinds_with_locked_state
**Given**: Nodes of each NodeKind (Node, Subgraph, Text) with LockState::Locked  
**When**: `is_movable()` is called for each  
**Then**: 
- NodeKind::Node returns false
- NodeKind::Subgraph returns true (exception)
- NodeKind::Text returns false

### test_all_node_kinds_with_unlocked_state
**Given**: Nodes of each NodeKind (Node, Subgraph, Text) with LockState::Unlocked  
**When**: `is_movable()` is called for each  
**Then**: All return true

### test_lock_state_enum_exhaustiveness
**Given**: LockState enum  
**When**: Match statement handles all variants  
**Then**: All variants are covered (compile-time verification)

### test_document_with_mixed_lock_states
**Given**: A Document containing multiple nodes with various lock states  
**When**: Nodes are queried for movement eligibility  
**Then**: Only unlocked nodes (and Subgraphs) are eligible for movement

## Contract Verification Tests

### test_invariant_subgraph_always_movable
**Given**: Any NodeKind::Subgraph with any LockState variant  
**When**: `is_movable()` is called with each LockState  
**Then**: Always returns true for both Locked and Unlocked states

### test_invariant_regular_nodes_follow_lock_state
**Given**: NodeKind::Node and NodeKind::Text nodes  
**When**: lock_state is LockState::Unlocked  
**Then**: `is_movable()` returns true  
**And**: When lock_state is LockState::Locked  
**Then**: `is_movable()` returns false

### test_invariant_json_roundtrip_preserves_state
**Given**: Any Node with any LockState  
**When**: Serialize → Deserialize → Serialize is performed  
**Then**: Output JSON is identical to input JSON

### test_postcondition_locked_field_removed_from_rust
**Given**: The Node struct definition  
**When**: Inspecting Rust fields  
**Then**: No field named `locked` exists in Rust code  
**And**: Field `lock_state` of type LockState exists

### test_postcondition_lock_state_has_required_methods
**Given**: LockState enum  
**When**: Calling methods  
**Then**: `is_locked()` method exists and returns bool  
**And**: `is_movable(&NodeKind)` method exists and returns bool

### test_postcondition_json_backwards_compatible
**Given**: Serialized output  
**When**: Inspecting JSON keys  
**Then**: Contains `locked` key (not `lock_state`)

## Contract Violation Tests

### test_violation_p1_old_locked_field_access
**Given**: Legacy Rust code accessing `node.locked`  
**When**: Compiled  
**Then**: Compile fails - field does not exist (enforces P1)

### test_violation_p2_old_locked_assignment
**Given**: Legacy Rust code with `node.locked = true`  
**When**: Compiled  
**Then**: Compile fails - field does not exist (enforces P2)

### test_violation_p5_old_movable_pattern
**Given**: Code using `!node.locked || node.kind == NodeKind::Subgraph`  
**When**: Compiled  
**Then**: Compile fails - field does not exist (enforces P5)

### test_violation_q7_json_uses_lock_state_key
**Given**: Serialization output  
**When**: JSON is inspected  
**Then**: Does NOT contain `"lock_state"` key (must use `"locked"`)

### test_violation_i4_roundtrip_mismatch
**Given**: A locked Node  
**When**: Serialize → Deserialize → Serialize is performed  
**Then**: Output equals input (invariant I4)

## REAL INTEGRATION TESTS (Testing Trophy)

These tests execute the actual diagram tool, not mocks or grep commands.

### test_integration_load_legacy_document_with_locked_nodes
**Given**: A legacy JSON file saved before migration with `"locked": true`  
**When**: Document is loaded using DiagramTool::load()  
**Then**: Nodes have correct lock_state set internally  
**And**: Nodes are not movable via canvas interactions  
**And**: Subgraph nodes remain movable despite lock state

### test_integration_save_document_preserves_lock_states
**Given**: A document with mixed lock states created in the tool  
**When**: Document is saved to JSON  
**Then**: JSON contains correct `"locked": true/false` values  
**And**: Saved file can be reloaded with same lock states

### test_integration_user_toggles_lock_via_properties_panel
**Given**: A node in the canvas with lock_state=LockState::Unlocked  
**When**: User clicks lock toggle in properties panel  
**Then**: Node's lock_state changes to LockState::Locked  
**And**: Node becomes immovable via drag interaction  
**And**: Serialized output reflects new lock state

### test_integration_user_cannot_move_locked_node
**Given**: A regular node with lock_state=LockState::Locked on canvas  
**When**: User attempts to drag the node  
**Then**: Movement is blocked  
**And**: Node remains at its original position

### test_integration_subgraph_movement_regardless_of_lock
**Given**: A Subgraph with lock_state=LockState::Locked on canvas  
**When**: User drags the Subgraph  
**Then**: Movement is allowed  
**And**: Subgraph moves to new position

### test_integration_selection_respects_lock_state
**Given**: A diagram with locked and unlocked nodes  
**When**: User selects all nodes  
**Then**: Unlocked nodes are selected  
**And**: Locked nodes (except Subgraphs) are excluded from selection

### test_integration_batch_operations_on_mixed_states
**Given**: A selection containing both locked and unlocked nodes  
**When**: User performs alignment operation  
**Then**: Only unlocked nodes are affected  
**And**: Locked nodes remain in place

### test_integration_nudge_blocked_for_locked_nodes
**Given**: A locked regular node  
**When**: User presses arrow key to nudge  
**Then**: Node does not move  
**And**: Unlocked nodes in selection do move

## DSL-Style Acceptance Tests (Dave Farley ATDD)

These tests describe WHAT the system does, not HOW it implements it. They use DSL-like language.

### DSL_ACCEPTANCE_USER_CANNOT_MOVE_LOCKED_NODE
```
Scenario: User cannot move a locked node
  Given a diagram with a regular node marked as locked
  When the user attempts to drag the node on the canvas
  Then the node does not move
  And the node remains at its original position
  And the lock indicator is visible on the node
```

### DSL_ACCEPTANCE_USER_CAN_MOVE_UNLOCKED_NODE
```
Scenario: User can move an unlocked node
  Given a diagram with a regular node marked as unlocked
  When the user drags the node on the canvas
  Then the node follows the cursor
  And the node is placed at the drop position
```

### DSL_ACCEPTANCE_SUBGRAPH_ALWAYS_MOVABLE
```
Scenario: Subgraphs can always be moved
  Given a diagram with a subgraph marked as locked
  When the user drags the subgraph
  Then the subgraph moves to the new position
  And the lock state does not affect movement
```

### DSL_ACCEPTANCE_LEGACY_FILE_LOADING
```
Scenario: Loading legacy file preserves lock states
  Given a JSON file saved with the old "locked" boolean format
  When the file is loaded into the diagram tool
  Then all nodes display their correct lock states
  And locked nodes cannot be moved
  And unlocked nodes can be moved
```

### DSL_ACCEPTANCE_NEW_FILE_SAVING
```
Scenario: Saving new file uses backwards-compatible format
  Given a diagram with nodes in various lock states
  When the document is saved to JSON
  Then the JSON uses "locked": true/false format
  And the file can be loaded by older versions of the tool
```

### DSL_ACCEPTANCE_USER_TOGGLES_LOCK_VIA_UI
```
Scenario: User toggles node lock state via properties panel
  Given a node selected on the canvas
  And the node is currently unlocked
  When the user clicks the lock toggle in the properties panel
  Then the node becomes locked
  And the lock indicator appears on the node
  And the node cannot be moved
  When the user clicks the lock toggle again
  Then the node becomes unlocked
  And the node can be moved again
```

### DSL_ACCEPTANCE_TEXT_NODE_BEHAVIOR
```
Scenario: Text nodes follow same lock behavior as regular nodes
  Given a diagram with a text node
  When the text node is unlocked
  Then it can be moved on the canvas
  When the text node is locked
  Then it cannot be moved on the canvas
```

## BDD Scenarios (Missing from Original)

### Scenario: User exports diagram to JSON
**Given**: A diagram with nodes in various lock states  
**When**: User selects File > Export > JSON  
**Then**: Exported JSON contains `"locked": true/false` for each node  
**And**: Format is compatible with older tool versions

### Scenario: User imports legacy JSON file
**Given**: A legacy JSON file with `"locked": true/false` format  
**When**: User selects File > Import  
**Then**: File imports successfully  
**And**: All nodes have correct lock states  
**And**: No data is lost during import

### Scenario: User groups locked and unlocked nodes
**Given**: A selection containing both locked and unlocked nodes  
**When**: User creates a group  
**Then**: Group is created  
**And**: Individual node lock states are preserved within group

---

## Test Execution Notes

1. **Compile-time verification**: Primary "test" is successful compilation - any remaining `.locked` references cause compile failures
2. **Reference count**: After migration, `grep -r "\.locked" diagram_tool/src/` returns zero matches on Node Rust fields
3. **Method coverage**: All public methods on LockState have corresponding tests
4. **Integration tests**: Use actual DiagramTool instance, not mocks
5. **JSON format**: Must use `"locked": bool` for backwards compatibility, NOT `"lock_state"`
6. **Round-trip**: JSON → Deserialize → Serialize → JSON must produce equivalent output
