# Contract Specification

## Context
- **Feature**: UI Dispatch: Double-click Node Add
- **Bead ID**: seshat-6pi
- **Domain terms**:
  - `DomainOp::NodeAdd` - diagram operation containing `id`, `x`, `y`, `width`, `height`, `label`
  - `EventEnvelope` - wrapper containing `op_id`, `operation` (DomainOp), `author`, `timestamp`
  - `db_tx` - `Option<Coroutine<EventEnvelope>>` for dispatching to WAL backend
  - `Author` - struct with `id`, `name`, `email: Option<String>`
- **Assumptions**:
  - Double-click on empty canvas with Select tool triggers node creation
  - `db_tx` is provided via Dioxus context (may be None when WAL disconnected)
  - Node ID generated via `Uuid::new_v4()`
  - Default node dimensions: 64x64 pixels
  - Default node label: "Node"
  - Coordinates snapped to grid if snap_to_grid is enabled
- **Open questions**: None

## EARS Requirements
| Requirement | Type | Description |
|---|---|---|
| U1 | Ubiquitous | UI shall notify backend of human-created nodes |
| E1 | Event-driven | Double-click triggers NodeAdd |
| U2 | Unwanted | No dispatch if WAL disconnected |

## Preconditions
- [P1] **Canvas click is double-click**: User performs double-click gesture on canvas element
- [P2] **Tool is Select mode**: Current tool must be `ToolMode::Select`
- [P3] **Click target is empty canvas**: No node or edge exists at click coordinates (hit test returns None)
- [P4] **Valid coordinates**: Canvas coordinates (x, y) must be finite (not NaN/Infinity)
- [P5] **WAL connected**: `db_tx` context must be `Some(coroutine)` (not None)

## Postconditions
- [Q1] **Event dispatched**: `db_tx.send()` called with valid `EventEnvelope` containing `DomainOp::NodeAdd`
- [Q2] **Node created locally**: Document's nodes map contains new node with generated ID
- [Q3] **Selection updated**: New node ID inserted into `editor_state.selected_items`
- [Q4] **Revision incremented**: Document revision incremented by 1
- [Q5] **Edit state cleared**: `editing_node` and `editing_edge` set to None, `edit_value` cleared

## Invariants
- [I1] **Envelope validity**: All `EventEnvelope` fields must be non-empty/valid
- [I2] **Node uniqueness**: New node ID must not exist in document.nodes
- [I3] **Consistency**: Local node state must match dispatched operation data

## Error Taxonomy
- `DispatchError::WalDisconnected` - when `db_tx` is `None` (WAL disconnected)
- `DispatchError::SendFailed` - when `db_tx.send()` returns `Err` (channel full/closed)
- `DispatchError::InvalidCoordinates` - when x or y is NaN/Infinity
- `DispatchError::PreconditionNotMet` - when P1-P4 not satisfied (should not dispatch)

## Contract Signatures
```rust
// In canvas.rs - double-click handler
fn handle_canvas_double_click(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    tool_signal: &Signal<ToolMode>,
    db_tx: Option<Coroutine<EventEnvelope>>,
    coords: (f64, f64),
) -> Result<(), DispatchError>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Double-click event | Compile-time | Dioxus `ondoubleclick` event handler |
| P2: Select tool | Runtime-checked | `if tool == ToolMode::Select` guard |
| P3: Empty canvas | Runtime-checked | `hit_node.is_none() && hit_edge.is_none()` |
| P4: Valid coords | Runtime-checked | `x.is_finite() && y.is_finite()` |
| P5: WAL connected | Runtime-checked | `db_tx.is_some()` check before send |

## Violation Examples
- VIOLATES P5: `db_tx = None` -- should produce `Err(DispatchError::WalDisconnected)` but still create node locally (best-effort UI responsiveness)
- VIOLATES P4: `coords = (f64::NAN, 100.0)` -- should produce `Err(DispatchError::InvalidCoordinates)` and NOT create node
- VIOLATES Q1: `db_tx = Some(tx)` but send fails -- should propagate as `Err(DispatchError::SendFailed)`
- VIOLATES Q2: Node ID collision (impossible with UUID) -- would violate I2

## Ownership Contracts
- `doc_signal: Signal<DiagramDocument>` - exclusive borrow via `with_mut()`, mutates `document.nodes`, `editor_state.selected_items`, `revision`
- `history_signal: Signal<History>` - exclusive borrow via `write()`, mutates history stack
- `db_tx: Option<Coroutine<EventEnvelope>>` - borrowed, no ownership transfer, cloned for send
- `coords: (f64, f64)` - copied (f64 is Copy), no ownership concerns

## Non-goals
- [ ] Node creation via other input methods (keyboard shortcut, menu)
- [ ] Drag-to-create node (different interaction mode)
- [ ] Undo/redo integration (handled by history signal)
- [ ] Remote sync conflict resolution (WAL handles this)
