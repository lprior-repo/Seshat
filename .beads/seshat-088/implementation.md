# Implementation: seshat-088 (UI Dispatch: Edge Connect)

## Summary
Implemented edge drawing completion dispatch to backend via `DomainOp::EdgeConnect`.

## Contract Adherence

### Preconditions (P1-P6)
- ✅ **P1**: `db_tx` channel is Some - handled via `if let Some(tx) = &db_tx` pattern
- ✅ **P2**: Source node exists - enforced by `from_node` being from `InteractionMode::DrawingEdge`
- ✅ **P3**: Target node exists - enforced by `find_node_at` returning `Some`
- ✅ **P4**: Target != source - checked via `if &target_id != from_node`
- ✅ **P5**: DAG validation - handled by `edge_preserves_dag` check
- ✅ **P6**: DrawingEdge mode - enforced by match arm `InteractionMode::DrawingEdge { .. }`

### Postconditions (Q1-Q4)
- ✅ **Q1**: EventEnvelope with EdgeConnect sent to db_tx after edge creation
- ✅ **Q2**: Local document state updated (already existing code)
- ✅ **Q3**: UI transitions from DrawingEdge to Select mode (already existing code)
- ✅ **Q4**: No operation on precondition failure (handled via early returns)

## Files Changed
1. **diagram_tool/src/ui/dispatch.rs** - Added `create_edge_connect_envelope()` and `dispatch_edge_connect()`
2. **diagram_tool/src/ui/canvas.rs** - Added dispatch call in edge drawing completion handler (2 locations)

## Constraint Compliance
- Zero `unwrap`/`expect`/`panic` in source code
- Zero `mut` - uses persistent state patterns
- Expression-based logic throughout
- No panics - graceful degradation when `db_tx` is None
