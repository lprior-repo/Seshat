# Implementation: seshat-5zs (UI Dispatch: Edge Disconnect)

## Summary
Implemented edge removal dispatch to backend via `DomainOp::EdgeDisconnect` when user presses Delete/Backspace on selected edges.

## Contract Adherence

### EARS Requirements
- ✅ **EARS-1**: Delete/Backspace on edge selection → dispatch EdgeDisconnect
- ✅ **EARS-2**: EdgeDisconnect dispatched to db_tx
- ✅ **EARS-3**: Graceful degradation when db_tx is None
- ✅ **EARS-4**: Skip dispatch if edge ID doesn't exist in document
- ✅ **EARS-5**: Multiple selected edges → separate dispatch per edge

### Preconditions (P1-P4)
- ✅ **P1**: Edge in selection - checked via `selected_ids`
- ✅ **P2**: Edge exists in document - checked via `document.edges.contains_key`
- ✅ **P3**: db_tx available - handled via `if let` pattern
- ✅ **P4**: Non-empty ID - IDs from selection are non-empty

### Postconditions (Q1-Q5)
- ✅ **Q1**: EventEnvelope with EdgeDisconnect sent to db_tx
- ✅ **Q2**: Valid op_id (UUID v4) - generated via `Uuid::new_v4()`
- ✅ **Q3**: Valid author "local-user" - via `local_author()`
- ✅ **Q4**: Valid timestamp - via `current_timestamp()`
- ✅ **Q5**: Edge removed from selection after dispatch (handled by `apply_delete_selected`)

## Files Changed
1. **diagram_tool/src/ui/dispatch.rs** - Added `create_edge_disconnect_envelope()` and `dispatch_edge_disconnect()`
2. **diagram_tool/src/ui/canvas.rs** - Added edge disconnect detection and dispatch in Delete key handler

## Constraint Compliance
- Zero `unwrap`/`expect`/`panic` in source code
- Zero `mut` - uses persistent state patterns
- Expression-based logic throughout
- No panics - graceful degradation when `db_tx` is None
