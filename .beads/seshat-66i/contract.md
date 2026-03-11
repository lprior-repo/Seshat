# Contract Specification: Nested Graphs (SUB-013 to SUB-018)

## Context
- Feature: SUB-013 to SUB-018 covering Container Behavior (overflow, padding) and Subgraph Creation (empty, pre-selected, nested, viewport transforms).
- Domain terms: 
  - Subgraph Container: A node that acts as a parent, visually encapsulating child nodes.
  - Padding: The visual buffer between the container boundary and its children.
  - Nested Subgraph: A container node whose parent is also a container node.
  - Overflow: A scenario where a child attempts to render outside the padding constraints of the container.
- Assumptions:
  - Container minimum size is enforced even if empty.
  - Graph is acyclic (nodes cannot be their own ancestors).
- Open questions:
  - Is there a maximum depth limit for nested subgraphs?
  - Does padding scaling depend on viewport zoom level, or is it constant?

## Preconditions
- [P1] Container padding values must be non-negative.
- [P2] When creating a subgraph from pre-selected nodes, all node IDs must exist in the canvas.
- [P3] When creating a nested subgraph, the new parent must not introduce a cycle (acyclic property).
- [P4] Viewport transforms (scale) must be strictly greater than zero when subgraphs inherit them.

## Postconditions
- [Q1] A container's bounding box encapsulate all of its children plus the defined padding (handling overflow correctly).
- [Q2] Creating an empty subgraph returns a newly allocated container node with an empty child list and minimum dimensions.
- [Q3] Creating a subgraph from pre-selected nodes successfully sets the `parent` reference of all selected nodes to the new container's ID.
- [Q4] A nested subgraph inherits its viewport transforms relative to its parent without spatial distortion.

## Invariants
- [I1] A node can have at most one parent container.
- [I2] A container node's bounds are never smaller than its calculated bounds (children bounds + padding).
- [I3] The node hierarchy must remain an acyclic directed graph (no cycles).

## Error Taxonomy
- `Error::InvalidPadding` - when negative padding is specified for a container.
- `Error::NodeNotFound` - when attempting to reparent or create a subgraph with a non-existent node ID.
- `Error::CircularDependency` - when assigning a nested subgraph parent creates a cycle.
- `Error::InvalidTransform` - when a viewport transform applied to a subgraph has an invalid scale (e.g., zero or negative).

## Contract Signatures
- `fn calculate_container_bounds(children: &[Node], padding: Padding) -> Result<BoundingBox, Error>`
- `fn create_empty_subgraph(id: NodeId, position: Point) -> Result<Node, Error>`
- `fn create_subgraph_from_nodes(id: NodeId, child_ids: &[NodeId], canvas: &mut CanvasState) -> Result<Node, Error>`
- `fn set_node_parent(child_id: NodeId, parent_id: NodeId, canvas: &mut CanvasState) -> Result<(), Error>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| [P1] non-negative padding | Compile-time (strongest) | `struct Padding { top: u32, right: u32, bottom: u32, left: u32 }` |
| [P2] nodes exist | Result | `Result<Node, Error::NodeNotFound>` |
| [P3] acyclic hierarchy | Result | `Result<(), Error::CircularDependency>` |
| [P4] scale > 0 | Compile-time (strongest) | `NonZeroF64` or `PositiveScale` newtype |

## Violation Examples (REQUIRED -- one per precondition and postcondition)
- VIOLATES [P1]: `calculate_container_bounds(&nodes, Padding { top: -10, ... })` -- should produce type error or `Err(Error::InvalidPadding)`
- VIOLATES [P2]: `create_subgraph_from_nodes(new_id, &["non_existent_id"], &mut canvas)` -- should produce `Err(Error::NodeNotFound)`
- VIOLATES [P3]: `set_node_parent(container_a, container_b)` when `container_b` is already a child of `container_a` -- should produce `Err(Error::CircularDependency)`
- VIOLATES [P4]: `apply_viewport_transform(subgraph, Scale(0.0))` -- should produce type error or `Err(Error::InvalidTransform)`
- VIOLATES [Q1]: Container bounds after child insertion do not encompass child bounding box plus padding -- should produce `Err(Error::InvariantViolation)` (or test failure).
- VIOLATES [Q2]: `create_empty_subgraph` returns a container with smaller than minimum bounds -- should produce `Err(Error::InvariantViolation)`.
- VIOLATES [Q3]: After `create_subgraph_from_nodes(id, &[child_id])`, `child.parent != Some(id)` -- should produce `Err(Error::InvariantViolation)`.
- VIOLATES [Q4]: Nested subgraph renders at offset diverging from true inherited transform -- should produce `Err(Error::InvariantViolation)`.

## Ownership Contracts (Rust-specific)
- Shared borrow: `fn calculate_container_bounds(children: &[Node], padding: &Padding)` -- read-only, no mutation, calculates the bounding box derived from children.
- Exclusive borrow: `fn create_subgraph_from_nodes(..., canvas: &mut CanvasState)` -- mutation contract: modifies `canvas.nodes` to add the new container and mutates the `parent` field of the provided child nodes. Mutates `canvas` overall.
- Ownership transfer: `child_ids: Vec<NodeId>` -- ownership of the ID list transferred to the function to map state.
