# Contract Specification: seshat-nwc

## Context
- **Feature**: UI Dispatch: Node Ungrouping
- **Bead ID**: seshat-nwc
- **Domain terms**:
  - `DomainOp::Ungroup` - Domain operation that ungroups a subgraph node
  - `db_tx` - Coroutine channel for dispatching to the WAL (Write-Ahead Log)
  - `EventEnvelope` - Wrapper containing operation metadata and DomainOp
  - `DiagramDocument` - The document state signal
  - `History` - Undo/redo history signal
  - `dispatch_ungroup` - Function that sends DomainOp::Ungroup to db_tx
  - `apply_ungroup_selection` - Function that applies ungroup to document

## Preconditions

- **[P1]**: `db_tx` must be available in context (not None) OR the function must handle the None case gracefully
  - **Enforcement**: Runtime check via `Option<Coroutine<EventEnvelope>>` context access

- **[P2]**: At least one subgraph must be selected for ungrouping
  - **Enforcement**: Runtime-filter via `selected_subgraphs_for_ungroup()` returning non-empty BTreeSet

- **[P3]**: The group_id passed to dispatch must be a valid NodeId string
  - **Enforcement**: Runtime via `create_ungroup_envelope` which accepts String

## Postconditions

- **[Q1]**: When ungroup preconditions are met and db_tx is available, exactly one `EventEnvelope` containing `DomainOp::Ungroup` is sent to db_tx
- **[Q2]**: After successful dispatch, the selection is cleared in editor_state
- **[Q3]**: When ungroup preconditions are NOT met (no subgraphs selected), no dispatch occurs and function returns `false`

## Invariants

- **[I1]**: The document revision must be incremented exactly once per successful ungroup operation
- **[I2]**: History must be pushed BEFORE any document mutation or dispatch
- **[I3]**: Either the envelope is dispatched to db_tx OR direct mutation occurs, not both

## Error Taxonomy

- `DispatchError::WalDisconnected` - db_tx is None (WAL not connected)
- `DispatchError::SendFailed` - Channel send operation failed
- `DispatchError::NoSelection` - No subgraphs selected for ungroup
