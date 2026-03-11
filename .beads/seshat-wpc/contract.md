# Contract Specification: Node Grouping (SUB-001 to SUB-006)

## Context
- Feature: Node Grouping and Subgraph Behaviors (SUB-001 to SUB-006)
- Domain terms:
  - Container / Subgraph: A node (`NodeKind::Subgraph`) that encapsulates other nodes.
  - Grouping: Creating a container that bounds a set of child nodes.
  - Reparenting: Changing a node's parent relationship, specifically when a container is deleted or nodes are moved.
  - Collapse/Expand: Toggling the visibility of child nodes while maintaining the grouping relationship.
- Assumptions:
  - SUB-001 to SUB-005 involve selection, modifier keys, and container behaviors (expand/collapse, locked states).
  - SUB-006 covers container deletion, which reparents children instead of deleting them.
  - Nodes maintain global coordinate space.
- Open questions:
  - Does box-select across boundary (SUB-002) select the container, the children, or both? (Assuming it depends on intersection depth per diagram tool norms).

## Preconditions
- P1: `child_ids` provided to group creation must not be empty.
- P2: All nodes referenced in `child_ids` must exist in the document state.
- P3: Grouping operations must not create a circular dependency.
- P4: Cannot mutate or group nodes that are `locked` unless the operation specifically targets locked interaction validation.
- P5: When ungrouping or deleting a container (SUB-006), the target `group_id` must exist and be of type `NodeKind::Subgraph`.

## Postconditions
- Q1: `group_nodes` returns a new container node whose bounds strictly encapsulate all selected children plus padding.
- Q2: After grouping, all `child_ids` have their `parent` field correctly set to the new `group_id`.
- Q3: Deleting a container (SUB-006) preserves all child nodes, resetting their `parent` to the deleted container's parent.
- Q4: Collapsing a group sets its `collapsed` state to `Some(true)` and hides children from standard hit-tests.
- Q5: Parent-child relationship invariant is maintained during nested selection interactions (SUB-005).

## Invariants
- I1: A child node's graphical bounds must always be fully contained within its expanded parent's bounds.
- I2: Node hierarchy must remain strictly acyclic (a node cannot be its own ancestor or descendant).
- I3: Operations on locked containers do not modify their child nodes' locked states, but enforce immutable bounds.

## Error Taxonomy
- `Error::EmptySelection` - when trying to group zero nodes.
- `Error::NodeNotFound(NodeId)` - when a requested child or group node does not exist in the state.
- `Error::CircularDependency` - when a hierarchy operation would create a cycle.
- `Error::NodeLocked(NodeId)` - when attempting to mutate a locked node.
- `Error::InvalidNodeType` - when attempting to perform a container operation on a non-container node.
- `Error::InvariantViolation` - when a postcondition or invariant fails safety checks.

## Contract Signatures
- `fn group_nodes(canvas: &mut CanvasState, group_id: NodeId, child_ids: &[NodeId]) -> Result<Node, Error>`
- `fn ungroup_nodes(canvas: &mut CanvasState, group_id: NodeId) -> Result<Vec<NodeId>, Error>`
- `fn toggle_collapse(canvas: &mut CanvasState, group_id: NodeId) -> Result<(), Error>`
- `fn evaluate_selection(canvas: &CanvasState, click_pos: Point, modifiers: SelectionModifiers) -> Result<SelectionResult, Error>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Non-empty children | Compile-time | `&NonEmpty<NodeId>` or `Vec1<NodeId>` |
| P2: Nodes exist | Error variant | `Result<T, Error::NodeNotFound>` |
| P3: No circular deps | Error variant | `Result<T, Error::CircularDependency>` |
| P4: Node not locked | Error variant | `Result<T, Error::NodeLocked>` |
| P5: Target is subgraph | Error variant | `Result<T, Error::InvalidNodeType>` |

## Violation Examples (REQUIRED)
- VIOLATES P1: `group_nodes(canvas, id, &[])` -- should produce `Err(Error::EmptySelection)`
- VIOLATES P2: `group_nodes(canvas, id, &["missing".into()])` -- should produce `Err(Error::NodeNotFound("missing"))`
- VIOLATES P3: `set_parent(canvas, "A".into(), "B".into())` where B is ancestor of A -- should produce `Err(Error::CircularDependency)`
- VIOLATES P4: `group_nodes(canvas, id, &["locked_id".into()])` -- should produce `Err(Error::NodeLocked("locked_id"))`
- VIOLATES P5: `ungroup_nodes(canvas, "text_node".into())` -- should produce `Err(Error::InvalidNodeType)`
- VIOLATES Q1: `group_nodes` completes but container bounds are smaller than a child's bounds after call -- should produce `Err(Error::InvariantViolation)`
- VIOLATES Q2: `group_nodes` completes but a child's parent is not `group_id` after call -- should produce `Err(Error::InvariantViolation)`
- VIOLATES Q3: `ungroup_nodes` completes but children are missing from canvas state after call -- should produce `Err(Error::InvariantViolation)`

## Ownership Contracts (Rust-specific)
- Exclusive borrow: `fn group_nodes(canvas: &mut CanvasState, ...)` -- mutates `nodes` to insert the new group and updates the `parent` reference of all specified children.
- Exclusive borrow: `fn ungroup_nodes(canvas: &mut CanvasState, ...)` -- removes the container from `nodes` and mutates the `parent` reference of all its children.
- Shared borrow: `child_ids: &[NodeId]` -- read-only slice of IDs used to identify targets without taking ownership.
- Ownership transfer: `group_id: NodeId` -- caller provides the ID for the new container, giving up ownership.
- Clone policy: Nodes are cloned during mutation as `im::HashMap` is used for `CanvasState` (purely functional update pattern).

## Non-goals
- Deep visual regression testing of subgraph borders.
- Implementing cross-document dragging (only intra-document grouping).
