# Contract Specification for seshat-1m9: Z-index Ordering (DOC-016 to DOC-020)

## Context
- **Feature**: Strict z-index handling when nodes overlap or are brought to front.
- **Domain terms**:
  - **z_index**: i64 value determining render order (higher = in front)
  - **ZOrderOp**: Enum (BringForward, SendBackward, BringToFront, SendToBack)
  - **NodeKind**: Types of nodes (Node, Subgraph)
  - **locked**: Boolean flag preventing node movement
  - **overlap**: Visual intersection of node bounding boxes
- **Assumptions**:
  - The core z-order logic already exists in `core/z_order.rs`
  - Projection layer z-order ops exist in `projection/ops/z_order.rs`
  - Locked nodes should be excluded from z-order operations
  - Subgraphs and regular nodes are handled in separate z-order layers
- **Open questions**:
  - What is the exact behavior for nodes with equal z-index that overlap?
  - Should z-index gaps be eliminated after each operation?

## Preconditions
- [P1] `ValidZIndexRange`: z_index values must be within i64::MIN to i64::MAX after any operation.
- [P2] `NoOverflow`: Z-order operations must not cause integer overflow when assigning new z-indexes.
- [P3] `NodesExist`: All node IDs in the selection must exist in the document.
- [P4] `LayerSeparation`: Z-order operations must respect node kind separation (subgraphs vs nodes).
- [P5] `LockedNodeHandling`: Locked nodes should be filtered from selection before z-order operations.

## Postconditions
- [Q1] `UniqueZIndexes`: After any z-order operation, all nodes in the same layer have unique z_index values.
- [Q2] `SequentialIndexes`: After any z-order operation, z-indexes are sequential (no gaps) within each layer.
- [Q3] `RelativeOrderPreserved`: The relative order of selected nodes is preserved after BringToFront/SendToBack.
- [Q4] `BringForwardSwapCount`: BringForward swaps each selected node at most once with the next non-selected node.
- [Q5] `SendBackwardSwapCount`: SendBackward swaps each selected node at most once with the previous non-selected node.
- [Q6] `ZIndexAssignment`: After operation, min_z + index is assigned to each node in sorted order.
- [Q7] `SelectionNotEmpty`: If no valid nodes are selected, operation returns false (no change).
- [Q8] `LockedNodesExcluded`: Only unlocked nodes (or subgraphs) are affected by z-order operations.

## Invariants
- [I1] `ZIndexUniqueness`: At any time, no two nodes of the same kind have identical z_index values.
- [I2] `LayerIntegrity`: Subgraphs and regular nodes maintain separate z-order sequences.
- [I3] `BoundedZIndex`: All z_index values remain within reasonable bounds (i64::MIN to i64::MAX).

## Error Taxonomy
- `ZOrderError::NoNodesSpecified` - when the input node ID slice is empty
- `ZOrderError::AllNodesInvalid` - when none of the specified node IDs exist in the document
- `ZOrderError::ZIndexOverflow` - when node count exceeds i64 capacity for z-index assignment
- `ZOrderError::NoChange` - when operation would not change any z-indexes (returned as bool from apply_z_order_operation)

## Contract Signatures

### Core Z-order Functions
```rust
/// Apply z-order operation to document
/// Returns true if any changes were made, false if no change
pub fn apply_z_order_operation(doc: &mut DiagramDocument, op: ZOrderOp) -> bool;

/// Apply z-order to sorted node IDs
/// Modifies ids in-place based on selected set and operation
pub fn apply_z_order_to_ids(ids: &mut Vec<NodeId>, selected: &BTreeSet<NodeId>, op: ZOrderOp);

/// Sort node IDs by z-index within a layer
fn ordered_layer_node_ids(doc: &DiagramDocument, subgraph_layer: bool) -> Vec<NodeId>;
```

### Projection Layer Functions
```rust
/// Apply BringForward to projection
pub fn apply_bring_forward(
    state: DiagramProjection,
    ids: &[NodeId],
) -> Result<DiagramProjection, ReplayError>;

/// Apply SendBackward to projection
pub fn apply_send_backward(
    state: DiagramProjection,
    ids: &[NodeId],
) -> Result<DiagramProjection, ReplayError>;

/// Apply BringToFront to projection
pub fn apply_bring_to_front(
    state: DiagramProjection,
    ids: &[NodeId],
) -> Result<DiagramProjection, ReplayError>;

/// Apply SendToBack to projection
pub fn apply_send_to_back(
    state: DiagramProjection,
    ids: &[NodeId],
) -> Result<DiagramProjection, ReplayError>;
```

### Convenience Functions
```rust
pub fn bring_forward(doc: &mut DiagramDocument) -> bool;
pub fn send_backward(doc: &mut DiagramDocument) -> bool;
pub fn bring_to_front(doc: &mut DiagramDocument) -> bool;
pub fn send_to_back(doc: &mut DiagramDocument) -> bool;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| ValidZIndexRange | Compile-time | i64 is always valid (no overflow in normal use) |
| NoOverflow | Runtime check | `i64::try_from(idx)` with unwrap_or fallback |
| NodesExist | Runtime check | Filter via `doc.document.nodes.contains_key(id)` |
| LayerSeparation | Compile-time | Separate iterators for subgraph vs node layers |
| LockedNodeHandling | Runtime check | Filter locked nodes before processing |

## Violation Examples
- VIOLATES P2: `apply_z_order_to_ids` with more than i64::MAX nodes -- should handle overflow gracefully
- VIOLATES P3: `apply_bring_forward(state, &[NodeId("nonexistent")])` -- returns Err(ReplayError::AllNodesInvalid)
- VIOLATES Q1: Two nodes in same layer with identical z_index after operation -- indicates bug
- VIOLATES Q2: Gaps in z_index sequence after operation -- indicates bug
- VIOLATES Q3: Selected nodes [B, D] reordered to [D, B] after BringToFront -- relative order not preserved
- VIOLATES Q4: BringForward swaps node multiple times -- each selected node should swap at most once
- VIOLATES Q7: apply_z_order_operation returns true when no selection exists -- should return false

## Ownership Contracts

### apply_z_order_to_ids
- **Input**: `ids: &mut Vec<NodeId>` - exclusive borrow, mutated in-place
- **Input**: `selected: &BTreeSet<NodeId>` - shared borrow, no mutation
- **Output**: Returns (), modifies `ids` vector in-place
- **Mutation**: `ids` is reordered based on selected set and operation

### apply_z_order_operation
- **Input**: `doc: &mut DiagramDocument` - exclusive borrow
- **Output**: Returns `bool` indicating if changes were made
- **Mutation**: `doc.document.nodes[id].z_index` values are modified for affected nodes

### Projection functions
- **Input**: `state: DiagramProjection` - ownership transferred, new projection returned
- **Output**: Returns `Result<DiagramProjection, ReplayError>`
- **Mutation**: Creates new nodes HashMap with updated z_index values (functional update)

## Non-goals
- Adding new z-order operations beyond the four existing ones.
- Visual collision detection (handled by separate hit-testing module).
- Automatic z-index assignment on node creation (handled by creation logic).
- Cross-document z-index management (single document scope).
