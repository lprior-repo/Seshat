---
bead_id: seshat-dfs
bead_title: UI Dispatch: Node Creation
phase: contract-synthesis
updated_at: 2026-03-12T00:00:00Z
---

# Contract Specification

## Context
- **Feature**: UI Dispatch: Node Creation
- **Bead ID**: seshat-dfs
- **Parent of**: seshat-6pi (double-click), seshat-8xu (toolbar button)
- **Source files**:
  - Canvas double-click handler: `diagram_tool/src/ui/canvas.rs` (line ~1694-1788)
  - Envelope creation: `diagram_tool/src/ui/dispatch/create.rs` (create_node_add_envelope)
  - Dispatch function: `diagram_tool/src/ui/dispatch/send/node.rs` (dispatch_node_add)
  - DomainOp definition: `diagram_tool/src/models/envelope/domain_ops.rs` (NodeAdd variant)
- **Domain terms**:
  - `DomainOp::NodeAdd` - diagram operation containing `id: NodeId`, `x: f64`, `y: f64`, `width: f64`, `height: f64`, `label: String`
  - `EventEnvelope` - wrapper containing `op_id: String`, `operation: DomainOp`, `author: Author`, `timestamp: i64`
  - `db_tx` - `Option<Coroutine<EventEnvelope>>` obtained via `use_context()` from Dioxus context
  - `dispatch_node_add` - function in `ui/dispatch/send/node.rs` that sends envelope to db_tx
  - `create_node_add_envelope` - function in `ui/dispatch/create.rs` that creates the envelope with validation
- **Assumptions**:
  - Double-click on empty canvas triggers node creation in Select mode
  - Toolbar button (to be added in `toolbar.rs`) triggers node creation at viewport center
  - `db_tx` is provided via Dioxus context (may be None when WAL disconnected)
  - Node ID generated via `Uuid::new_v4()`
  - Default node dimensions: 64x64 pixels (see canvas.rs line 1766-1767)
  - Default node label: "Node" (see canvas.rs line 1763)
  - Coordinates snapped to grid if snap_to_grid is enabled (see canvas.rs line 1753-1757)
- **Open questions**:
  - Should toolbar add node at center of viewport or at mouse position? (Defaulting to center of viewport for now)

## EARS Requirements
| Requirement | Type | Description |
|---|---|---|
| U1 | Ubiquitous | UI shall notify backend of human-created nodes |
| E1 | Event-driven | Double-click on empty canvas triggers NodeAdd |
| E2 | Event-driven | Toolbar Add Node button triggers NodeAdd |
| U2 | Unwanted | No dispatch if WAL disconnected (fire-and-forget, log warning) |

## Preconditions
- [P1] **Canvas double-click triggers node creation**: User double-clicks on empty canvas with Select tool, OR
- [P1b] **Toolbar button triggers node creation**: User clicks Add Node button in toolbar
- [P2] **Tool is Select mode** (for double-click path): Current tool must be `ToolMode::Select`
- [P3] **Click target is empty canvas** (for double-click path): No node or edge exists at click coordinates (hit test returns None)
- [P4] **Valid coordinates**: Canvas coordinates (x, y) must be finite (not NaN/Infinity)
- [P5] **Valid dimensions**: Width and height must be positive finite numbers

## Postconditions
- [Q1] **Event dispatched**: `db_tx.send()` called with valid `EventEnvelope` containing `DomainOp::NodeAdd`
- [Q2] **Node created locally**: Document's nodes map contains new node with generated ID
- [Q3] **Selection updated**: New node ID inserted into `editor_state.selected_items`
- [Q4] **Revision incremented**: Document revision incremented by 1
- [Q5] **Edit state cleared**: `editing_node` and `editing_edge` set to None, `edit_value` cleared
- [Q6] **Envelope data matches local node**: Dispatched DomainOp::NodeAdd fields (id, x, y, width, height, label) exactly match local node

## Invariants
- [I1] **Envelope validity**: All `EventEnvelope` fields must be non-empty/valid
- [I2] **Node uniqueness**: New node ID must not exist in document.nodes (guaranteed by UUID)
- [I3] **Consistency**: Local node state must match dispatched operation data
- [I4] **WAL notification**: Every user-initiated node creation must dispatch to db_tx (non-blocking)

## Error Taxonomy
- `DispatchError::WalDisconnected` - when `db_tx` is `None` (WAL disconnected) - MUST NOT fail the operation, log and continue
- `DispatchError::SendFailed` - when `db_tx.send()` returns `Err` (channel full/closed) - MUST NOT fail local operation
- `DispatchError::InvalidCoordinates` - when x or y is NaN/Infinity OR width/height not positive
- `DispatchError::PreconditionNotMet` - when P2-P5 not satisfied (should not dispatch)

## Contract Signatures
```rust
// In canvas.rs - double-click handler (existing inline logic, needs dispatch call added)
// Lines ~1745-1787: inline node creation logic with local state updates

// New function to be added in canvas.rs or as helper
fn handle_canvas_double_click_node_creation(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    tool_signal: &Signal<ToolMode>,
    db_tx: Option<Coroutine<EventEnvelope>>,
    coords: (f64, f64),
) -> Result<DispatchResult, DispatchError>

// In toolbar.rs - add node button handler (to be added)
fn handle_toolbar_add_node(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
    viewport_center: (f64, f64),
) -> Result<DispatchResult, DispatchError>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Double-click event | Compile-time | Dioxus `ondoubleclick` event handler |
| P1b: Toolbar button click | Compile-time | Dioxus `onclick` event handler |
| P2: Select tool | Runtime-checked | `if tool == ToolMode::Select` guard |
| P3: Empty canvas | Runtime-checked | `hit_node.is_none() && hit_edge.is_none()` |
| P4: Valid coords | Runtime-checked | `x.is_finite() && y.is_finite()` |
| P5: Valid dimensions | Runtime-checked | `width > 0.0 && height > 0.0` in create_node_add_envelope |

## Violation Examples
- VIOLATES P4: `coords = (f64::NAN, 100.0)` -- should produce `Err(DispatchError::InvalidCoordinates)` and NOT create node
- VIOLATES P5: `width = 0.0` -- should produce `Err(DispatchError::InvalidCoordinates)` from create_node_add_envelope
- VIOLATES Q1: `db_tx = Some(tx)` but send fails -- MUST still create node locally, log warning for dispatch failure
- VIOLATES Q6: Local node x differs from envelope x -- violates consistency invariant I3

## Ownership Contracts
- `doc_signal: Signal<DiagramDocument>` - exclusive borrow via `with_mut()`, mutates:
  - `document.nodes` (insert new node)
  - `editor_state.selected_items` (insert new node ID)
  - `revision` (increment by 1)
- `history_signal: Signal<History>` - exclusive borrow via `write()`, mutates history stack
- `db_tx: Option<Coroutine<EventEnvelope>>` - borrowed, no ownership transfer, cloned for send
- `coords: (f64, f64)` - copied (f64 is Copy), no ownership concerns

## Non-goals
- [ ] Drag-to-create node (different interaction mode)
- [ ] Undo/redo integration (handled by history signal)
- [ ] Remote sync conflict resolution (WAL handles this)
- [ ] Node creation via keyboard shortcut or menu
