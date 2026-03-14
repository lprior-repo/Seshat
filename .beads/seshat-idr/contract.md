# Contract Specification

bead_id: seshat-idr
bead_title: SUB-023 to SUB-027: Subgraph relative coordinates
phase: p1
updated_at: 2026-03-14T18:00:00Z

## Context
- Feature: Subgraph relative coordinates
- Domain terms:
  - World Space: Absolute coordinate system of the entire diagram.
  - Relative Space: Coordinate system relative to a parent node's top-left corner (x, y).
  - Subgraph: A node that can contain other nodes (has `NodeKind::Subgraph`).
- Assumptions:
  - `Node.x` and `Node.y` will now store relative coordinates.
  - Existing data might need migration or interpretation changes.
- Open questions:
  - Should `Node` store a separate `world_x/y` for caching? (No, keep it functional/calc-heavy).

## Preconditions
- [P1] All `Node` coordinate values must be finite `OrderedFloat`.
- [P2] `Node.parent` must reference a valid `NodeId` in the same `DocumentData`.
- [P3] The parent-child relationship must be acyclic (enforced by DAG).

## Postconditions
- [Q1] `Node.x` and `Node.y` are interpreted as relative to `parent` if `parent` is `Some`.
- [Q2] `Node.x` and `Node.y` are interpreted as world coordinates if `parent` is `None`.
- [Q3] Moving a parent node automatically shifts all its children in world space without changing their stored `x/y`.

## Invariants
- [I1] `WorldX(node) = (parent.WorldX + node.x)` recursive, base case `parent == None => WorldX(node) = node.x`.
- [I2] A node's bounding box in world space is `[WorldX, WorldY, width, height]`.

## Error Taxonomy
- Error::NodeNotFound - referenced node doesn't exist.
- Error::CycleDetected - reparenting would create a cycle.

## Contract Signatures
- `fn get_world_coords(nodes: &HashMap<NodeId, Node>, node_id: &NodeId) -> Result<(f64, f64), Error>`
- `fn set_world_coords(nodes: &mut HashMap<NodeId, Node>, node_id: &NodeId, world_x: f64, world_y: f64) -> Result<(), Error>`
- `fn reparent_node(nodes: &mut HashMap<NodeId, Node>, node_id: &NodeId, new_parent: Option<NodeId>, keep_world_pos: bool) -> Result<(), Error>`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Finite coords | Compile-time | `OrderedFloat` |
| Valid parent | Error variant | `Result<T, Error::NodeNotFound>` |

## Violation Examples
- VIOLATES <P1>: `Node { x: OrderedFloat(f64::NAN), ... }` -- should produce `Err(OrderedFloatError::NaN)`
- VIOLATES <P2>: `Node { parent: Some("invalid_id"), ... }` -- `get_world_coords` returns `Err(Error::NodeNotFound)`

## Ownership Contracts (Rust-specific)
- `get_world_coords`: Shared borrow of nodes, returns copy of coords.
- `set_world_coords`: Exclusive borrow of nodes, mutates `Node.x/y`.
- `reparent_node`: Exclusive borrow of nodes, mutates `Node.parent` and `Node.x/y` if `keep_world_pos` is true.

## Non-goals
- Performance optimization of coordinate calculation (caching).
- Coordinate systems for edges (they use node port coordinates).
