# Contract Specification: seshat-cj1 (UI Dispatch: Inline Text Enter)

## Context
- **Feature**: Wire the inline text editor Enter key event to construct DomainOp::UpdateLabel and dispatch to db_tx
- **Domain terms**:
  - `DomainOp::UpdateLabel { target_id: String, label: String, target_type: TargetType }` - Operation to update a node or edge label
  - `TargetType` - Enum variant distinguishing node vs edge targets
  - `EventEnvelope` - Wrapper containing DomainOp, Author, timestamp, and op_id
  - `db_tx` - Dioxus coroutine for async dispatch of EventEnvelope to store bridge
  - `DiagramDocument` - Document signal containing nodes, edges, and editor state
  - `History` - Undo/redo stack
  - `commit_inline_edit` - Existing function that currently directly mutates document (to be refactored)
- **Assumptions**:
  - DomainOp::UpdateLabel variant exists (parent bead seshat-1aw)
  - db_tx is available via `use_context::<Option<Coroutine<EventEnvelope>>>()`
  - The store bridge is initialized in app.rs with async-db feature
  - Fallback to direct document manipulation when db_tx is None (WASM builds)
- **Open questions**:
  - Should both Enter key and onBlur trigger the same dispatch? Yes, both should dispatch.
  - Should we skip dispatch if the label hasn't actually changed? Yes, no-op if label unchanged.
  - Should the direct mutation path be completely replaced or kept as fallback? Keep as fallback for WASM.

## EARS (Requirements)
- **E1**: The system SHALL dispatch `DomainOp::UpdateLabel` to the db_tx channel when the Enter key is pressed in inline text edit mode.
- **E2**: The system SHALL dispatch `DomainOp::UpdateLabel` to the db_tx channel when focus leaves the inline text editor (onBlur).
- **E3**: The system SHALL NOT dispatch if the new label equals the current label (no-op).
- **E4**: The system SHALL handle db_tx unavailability gracefully (skip dispatch without crash, fallback to direct mutation).

## Preconditions
- [P1] **Editing active**: Either `editing_node` or `editing_edge` signal must be Some
- [P2] **Valid target**: If editing_node is Some, the node ID must exist in document.nodes; if editing_edge is Some, edge ID must exist in document.edges
- [P3] **db_tx available**: When `cfg!(feature = "async-db")`, db_tx context should be used; otherwise fallback path is used (not a hard requirement)
- [P4] **Label non-empty**: The new label value should be allowed (empty labels are valid for clearing)

## Postconditions
- [Q1] **Envelope dispatched**: If db_tx is Some and label changed, an EventEnvelope with DomainOp::UpdateLabel is sent via `db_tx.send(envelope)`
- [Q2] **Correct payload**: EventEnvelope.operation must be DomainOp::UpdateLabel with correct target_id, label, and target_type
- [Q3] **Author populated**: EventEnvelope.author must have id="local-user", name="Local User"
- [Q4] **Timestamp valid**: EventEnvelope.timestamp must be current Unix timestamp in milliseconds
- [Q5] **Unique op_id**: EventEnvelope.op_id must be a fresh UUID v4
- [Q6] **Editing cleared**: After dispatch, both editing_node and editing_edge signals must be set to None
- [Q7] **Fallback behavior**: If db_tx is None, the original direct document mutation path is executed (preserves existing behavior)

## Invariants
- [I1] Document revision increments exactly once per successful label update operation
- [I2] Editing state (editing_node/editing_edge) is cleared after successful commit
- [I3] Label changes are reflected in document only through store event replay path (not directly)

## Error Taxonomy
- **NoEditActive**: Returned when neither editing_node nor editing_edge is set (precondition P1 violation)
- **TargetNotFound**: Returned when target node/edge ID doesn't exist (precondition P2 violation)
- **DbTxUnavailable**: Returned when async-db feature enabled but db_tx is None (falls back to direct mutation, not an error)
- **DispatchFailed**: Returned when db_tx.send() fails (channel closed)
- **LabelUnchanged**: Not an error - returns Ok(false) to indicate no-op

## Contract Signatures
```rust
// Primary dispatch function (refactored from commit_inline_edit)
pub fn commit_inline_edit(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    editing_node: Signal<Option<NodeId>>,
    editing_edge: Signal<Option<EdgeId>>,
    edit_value: Signal<String>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> Result<bool, UpdateLabelError> {
    // If db_tx available and label changed: construct EventEnvelope, send via db_tx
    // If label unchanged: return Ok(false) (no-op)
    // If db_tx None: fallback to direct mutate_doc_with_history
}

// Helper: construct envelope from edit context
fn construct_update_label_envelope(
    target_id: &str,
    target_type: TargetType,
    new_label: &str,
    doc: &DiagramDocument,
) -> EventEnvelope;

// Error enum
#[derive(Debug, Error)]
pub enum UpdateLabelError {
    #[error("no edit is active")]
    NoEditActive,
    #[error("target not found: {0}")]
    TargetNotFound(String),
    #[error("db_tx not available")]
    DbTxUnavailable,
    #[error("dispatch failed: {0}")]
    DispatchFailed(String),
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Editing active | Runtime check + early return | `if node.is_none() && edge.is_none() { return Err(NoEditActive) }` |
| P2: Target exists | Runtime check | `doc.document.nodes.get(id).ok_or(TargetNotFound(id))` |
| P3: db_tx available | Compile-time conditional | `#[cfg(feature = "async-db")]` with fallback |
| P4: Label valid | N/A (empty allowed) | No enforcement needed |

## Violation Examples (REQUIRED)
- VIOLATES P1: Calling `commit_inline_edit` with both editing_node=None and editing_edge=None -- should produce `Err(UpdateLabelError::NoEditActive)`
- VIOLATES P2: Calling with editing_node=Some("nonexistent-id") where node doesn't exist -- should produce `Err(UpdateLabelError::TargetNotFound("nonexistent-id"))`
- VIOLATES Q1: Calling with db_tx=None (when async-db enabled) -- should fallback to direct manipulation (not an error)
- VIOLATES Q2: Dispatched envelope has wrong target_type or label -- should be caught by test
- VIOLATES Q6: After commit, editing_node still has value -- should be cleared to None

## Ownership Contracts
- `doc_signal: Signal<DiagramDocument>` - Read-only access to current state; clone taken for history push and envelope construction
- `history_signal: Signal<History>` - Exclusive write to push current document state (fallback path)
- `editing_node: Signal<Option<NodeId>>` - Exclusive write to clear editing state after commit
- `editing_edge: Signal<Option<EdgeId>>` - Exclusive write to clear editing state after commit
- `edit_value: Signal<String>` - Read-only access to get the new label value
- `db_tx: Option<Coroutine<EventEnvelope>>` - Owned sender; send() transfers ownership of envelope

## Implementation Phases
1. **Phase 1**: Add `DomainOp::UpdateLabel` variant to envelope.rs (if not done in seshat-1aw)
2. **Phase 2**: Modify `commit_inline_edit` to accept db_tx parameter and construct EventEnvelope
3. **Phase 3**: Implement dispatch logic: if db_tx available, send envelope; else fallback to direct mutation
4. **Phase 4**: Update canvas.rs call sites to pass db_tx context
5. **Phase 5**: Test that Enter key and onBlur both trigger the dispatch

## Non-goals
- [ ] Implement label validation (length limits, character restrictions) - future work
- [ ] Add undo/redo for label-only operations (already handled by history_signal)
- [ ] Batch multiple rapid label changes - future optimization
- [ ] Rich text editing - plain text only for now
