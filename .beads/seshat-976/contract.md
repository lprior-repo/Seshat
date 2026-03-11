# Contract Specification: Subgraph Events (SUB-019 to SUB-024)

## Context
- **Feature**: Subgraph event handling and lifecycle management (bounds, z-index, add/remove nodes).
- **Domain terms**:
  - **Subgraph**: A container node that holds child nodes.
  - **Bounds**: The bounding box `(x, y, width, height)` enclosing all child nodes plus padding.
  - **z-index**: The rendering order of nodes. Subgraphs and their children must render cohesively.
  - **Parent Reference**: A child node's `parent_id` field pointing to the Subgraph's ID.
- **Assumptions**:
  - Nodes have unique IDs (`NodeId`).
  - The diagram state maintains a flat list or map of nodes, where hierarchy is determined by `parent_id`.
- **Open questions**:
  - What happens if a node added to a subgraph is already in another subgraph? (Assuming it is atomically moved and the old subgraph's bounds are recalculated).

## Preconditions
- [P1] `NodeId` for the subgraph must exist in the diagram state.
- [P2] `NodeId` for the child node(s) being added/removed must exist in the diagram state.
- [P3] When adding a node to a subgraph, the operation must not create a cyclical parent-child relationship (e.g., setting a node's parent to its own child).

## Postconditions
- [Q1] **SUB-019 (Bounds)**: After a child is added, removed, or moved, the subgraph's bounds must accurately enclose all child nodes plus any defined padding.
- [Q2] **SUB-020 (Z-index)**: Subgraph children must inherit or dynamically align their effective z-index to be strictly above the subgraph container but below intersecting higher-z subgraphs.
- [Q3] **SUB-021 (Add node)**: The added child node's `parent_id` is updated to the subgraph's `NodeId`.
- [Q4] **SUB-022 (Remove node)**: The removed child node's `parent_id` is set to `None`.
- [Q5] **SUB-023 (Batch add)**: All specified nodes have their `parent_id` updated, and the subgraph bounds are recalculated exactly once for efficiency.
- [Q6] **SUB-024 (Remove all)**: If all children are removed, the subgraph container node remains in the diagram with empty bounds or minimum dimensions.

## Invariants
- [I1] A child node can have at most one `parent_id` at any time.
- [I2] A subgraph's bounds can never be strictly smaller than the bounding box of its children.

## Error Taxonomy
- `Error::NodeNotFound(NodeId)` - when a requested child or subgraph node does not exist.
- `Error::CycleDetected(NodeId, NodeId)` - when adding a node would create a parent-child loop.
- `Error::InvalidBounds(Rect)` - when a calculated bound is invalid (e.g., negative width).

## Contract Signatures
```rust
fn calculate_subgraph_bounds(subgraph_id: NodeId, state: &DiagramState) -> Result<Rect, Error>;
fn update_z_index_ordering(subgraph_id: NodeId, state: &mut DiagramState) -> Result<(), Error>;
fn add_node_to_subgraph(child_id: NodeId, subgraph_id: NodeId, state: &mut DiagramState) -> Result<(), Error>;
fn remove_node_from_subgraph(child_id: NodeId, state: &mut DiagramState) -> Result<(), Error>;
fn batch_add_nodes_to_subgraph(child_ids: &[NodeId], subgraph_id: NodeId, state: &mut DiagramState) -> Result<(), Error>;
fn remove_all_nodes_from_subgraph(subgraph_id: NodeId, state: &mut DiagramState) -> Result<(), Error>;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Node existence | Result error variant | `Result<T, Error::NodeNotFound>` |
| No cycles | Result error variant | `Result<T, Error::CycleDetected>` |
| Valid bounds | Compile-time | `struct BoundingBox { width: NonZeroU32, ... }` (if applicable) or strict initialization |

## Violation Examples
- **VIOLATES P1**: `add_node_to_subgraph(child_id, missing_subgraph_id, state)` -- should produce `Err(Error::NodeNotFound(missing_subgraph_id))`
- **VIOLATES P2**: `add_node_to_subgraph(missing_child_id, subgraph_id, state)` -- should produce `Err(Error::NodeNotFound(missing_child_id))`
- **VIOLATES P3**: `add_node_to_subgraph(subgraph_id, subgraph_id, state)` -- should produce `Err(Error::CycleDetected(subgraph_id, subgraph_id))`
- **VIOLATES Q1**: After adding a node outside current bounds, `calculate_subgraph_bounds` returns bounds smaller than child's position -- should produce `Err(Error::InvalidBounds(rect))` (or caught in tests).

## Ownership Contracts
- `state: &mut DiagramState`: Exclusive borrow. Mutates the nodes' `parent_id` and bounds fields.
- `child_ids: &[NodeId]`: Shared borrow. Read-only list of IDs for batch operations. No cloning required unless extracting into an event log.
