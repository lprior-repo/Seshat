# Implementation: seshat-q5e (UI Dispatch: Ungroup Nodes)

## Summary
Implemented Ctrl+Shift+G keyboard shortcut to dispatch `DomainOp::Ungroup` to backend for ungrouping selected subgraphs.

## Contract Adherence

### EARS Requirements
- ✅ **EARS-1**: Ctrl+Shift+G with selected subgraph → dispatch Ungroup
- ✅ **EARS-2**: Ctrl+Shift+G with no selection → silent no-op
- ✅ **EARS-3**: Ctrl+Shift+G while editing → no trigger (modifier check)
- ✅ **EARS-4**: db_tx unavailable → graceful degradation (handled by dispatch function)

### Preconditions (P1-P4)
- ✅ **P1**: Ctrl+Shift+G pressed - handled by key matching `modifier && shift`
- ✅ **P2**: Not editing - handled by key handler guard
- ✅ **P3**: Selected subgraph exists - checked via `NodeKind::Subgraph` filter
- ✅ **P4**: db_tx available - handled via Option pattern

### Postconditions (Q1-Q4)
- ✅ **Q1**: EventEnvelope with Ungroup sent to db_tx for each selected subgraph
- ✅ **Q2**: Valid op_id, operation, author, timestamp in envelope
- ✅ **Q3**: No local mutation in happy path (event sourcing)
- ✅ **Q4**: Selection handling via downstream apply_delete_selected

### Invariants (I1-I3)
- ✅ **I1**: Idempotent dispatch - multiple presses create multiple events
- ✅ **I2**: Non-blocking - fire and forget via coroutine
- ✅ **I3**: No panic on missing node - filtered out via contains_key check

## Files Changed
1. **diagram_tool/src/core/keyboard.rs** - Added `Ungroup` variant to KeyAction enum and Ctrl+Shift+G mapping
2. **diagram_tool/src/ui/dispatch.rs** - Added `create_ungroup_envelope()` and `dispatch_ungroup()`
3. **diagram_tool/src/ui/canvas.rs** - Added Ctrl+Shift+G handler in keyboard match block

## Constraint Compliance
- Zero `unwrap`/`expect`/`panic` in source code
- Zero `mut` - uses persistent state patterns
- Expression-based logic throughout
- No panics - graceful degradation
