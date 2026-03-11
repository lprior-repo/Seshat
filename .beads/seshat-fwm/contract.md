# Contract Specification

## Context

- **Bead ID**: seshat-fwm
- **Feature**: UI Dispatch - Inline Text Blur
- **Description**: Wire the inline text editor onBlur event to construct `DomainOp::UpdateLabel` and dispatch to `db_tx`
- **Domain terms**:
  - `DomainOp` - Enum of domain operations that can be dispatched to the backend
  - `EventEnvelope` - Wrapper containing operation, author, timestamp, and op_id
  - `db_tx` - Dioxus coroutine for sending events to the backend store
  - `commit_inline_edit` - Function that commits inline text edits (node/edge labels)
  - `onBlur` - DOM event fired when an element loses focus
  - `onEnter` - Key event (Enter key) that also triggers commit
- **Assumptions**:
  - `DomainOp::UpdateLabel` does not yet exist and must be added
  - The `commit_inline_edit` function must be modified to accept `db_tx` and dispatch the event
  - Both node and edge label editing must trigger the dispatch
- **Open questions**:
  - Should `DomainOp::UpdateLabel` handle both nodes and edges in one variant, or have separate variants?
  - What author metadata should be used for the event?

---

## EARS Specification

### Ubiquitous Language
- **UI shall save text edits to backend**: All label changes from inline text editors must be persisted via the event dispatch system, not just local state mutation

### Event-Driven Specification
- **text input loses focus triggers UpdateLabel**: When the inline text input field loses focus (onBlur), a `DomainOp::UpdateLabel` event shall be constructed and dispatched to `db_tx`

### Unwanted Behavior
- **no dispatch if text unchanged**: If the user focuses the text input and then blurs without changing the text, NO `DomainOp::UpdateLabel` shall be dispatched to `db_tx`

---

## Preconditions

- [P1] `commit_inline_edit` receives a valid `db_tx` handle (may be `None` if no backend connection)
- [P2] The target node or edge exists in the document at the time of commit
- [P3] The edit value is a valid UTF-8 string
- [P4] The label change has been validated (current_label != new_label)

---

## Postconditions

- [Q1] If `new_label != current_label`, an `EventEnvelope` containing `DomainOp::UpdateLabel` is sent to `db_tx`
- [Q2] If `new_label == current_label`, NO event is dispatched (idempotent)
- [Q3] The document state is updated locally (revision incremented) BEFORE or AFTER dispatch (consistent ordering)
- [Q4] The `editing_node` and `editing_edge` signals are set to `None` after commit

---

## Invariants

- [I1] At most one of `editing_node` or `editing_edge` may be `Some` at any time (mutual exclusion)
- [I2] The revision number increments monotonically with each successful label update
- [I3] No panic may occur if `db_tx` is `None` - the function must handle missing backend gracefully

---

## Error Taxonomy

- `Error::DispatchFailed` - when `db_tx.send()` fails (e.g., channel closed)
- `Error::TargetNotFound` - when the target node/edge does not exist
- `Error::PreconditionViolation` - when P1-P4 are not met (programming error, should debug_assert)

---

## Contract Signatures

```rust
/// Domain operation for updating node or edge labels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op_type", rename_all = "snake_case")]
pub enum DomainOp {
    // ... existing variants ...
    
    /// Update label for a node or edge
    UpdateLabel {
        target_id: String,
        target_type: LabelTargetType,
        new_label: String,
        old_label: String,
    },
}

/// Type of label target
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LabelTargetType {
    Node,
    Edge,
}

/// Commit inline text edit and dispatch to backend
pub fn commit_inline_edit(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut editing_node: Signal<Option<NodeId>>,
    mut editing_edge: Signal<Option<EdgeId>>,
    edit_value: Signal<String>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> Result<(), CommitError>;
```

---

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| db_tx handle valid | Runtime (graceful None) | `Option<Coroutine<..>>` with None-handling |
| target exists | Runtime | `doc.document.nodes.get(&target).is_some()` |
| valid UTF-8 | Compile-time | Rust strings are always UTF-8 |
| label changed | Compile-time | `if current_label != new_label` guard |

---

## Violation Examples

- **VIOLATES P1**: `commit_inline_edit(..., None)` when db_tx is required for dispatch but is None -- should either succeed gracefully (no backend) or return `Err(DispatchFailed)` if backend required
- **VIOLATES P2**: `commit_inline_edit(..., edit_value="new")` when target node was deleted -- should produce `Err(TargetNotFound)`
- **VIOLATES P4**: Calling commit with `new_label == current_label` -- should NOT dispatch event (Q2 violated if dispatch happens)
- **VIOLATES Q1**: Label changed but no EventEnvelope sent to db_tx -- Q1 violated
- **VIOLATES Q2**: Label unchanged but EventEnvelope IS sent -- Q2 violated (unwanted behavior)

---

## Ownership Contracts

- `doc_signal: Signal<DiagramDocument>` - Exclusive borrow, mutates `document.nodes` or `document.edges` and `revision`
- `history_signal: Signal<History>` - Exclusive borrow, may push new state for undo
- `editing_node: Signal<Option<NodeId>>` - Exclusive borrow, resets to `None` after commit
- `editing_edge: Signal<Option<EdgeId>>` - Exclusive borrow, resets to `None` after commit
- `edit_value: Signal<String>` - Shared read access to current edit value
- `db_tx: Option<Coroutine<EventEnvelope>>` - Shared borrow, may send but never mutates

---

## Non-goals

- [ ] Handle label validation (length limits, character restrictions) - future work
- [ ] Undo/redo for label edits - handled by existing history system
- [ ] Real-time sync while typing - only commit on blur/Enter
