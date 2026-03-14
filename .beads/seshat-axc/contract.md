# Contract Specification: seshat-axc (SUB-013 to SUB-017)

## Context
- **Feature:** Nested subgraph validation
- **Bead ID:** `seshat-axc`
- **Domain terms:**
    - **Subgraph:** A node of kind `Subgraph` that can contain other nodes as children.
    - **Nesting Depth:** The number of ancestors of a node that are of kind `Subgraph`.
    - **MAX_SUBGRAPH_NESTING_DEPTH:** Constant limit for nesting (currently 5).

## Preconditions
- [P1] Any operation that sets or changes a node's parent (e.g., `Group`, `NodeMove` into a subgraph, or future `Reparent`) must ensure that the resulting nesting depth of any node in the affected subtree does not exceed `MAX_SUBGRAPH_NESTING_DEPTH`.
- [P2] The `DiagramProjection` state must be valid before applying any operation.

## Postconditions
- [Q1] Every node in the `DiagramProjection` after an operation must have a nesting depth <= `MAX_SUBGRAPH_NESTING_DEPTH`.
- [Q2] If an operation would violate [Q1], it must return `Err(ReplayError::NestedSubgraphLimitExceeded(MAX_SUBGRAPH_NESTING_DEPTH))` and leave the state unchanged.

## Invariants
- [I1] At all times, no node in the diagram has more than `MAX_SUBGRAPH_NESTING_DEPTH` ancestors of kind `Subgraph`.

## Error Taxonomy
- `ReplayError::NestedSubgraphLimitExceeded(usize)` - Returned when an operation would cause a node to exceed the maximum nesting depth.

## Contract Signatures
- `fn count_nesting_depth(nodes: &HashMap<NodeId, Node>, node_id: &NodeId) -> usize`
- `fn check_nesting_depth(nodes: &HashMap<NodeId, Node>, affected_nodes: &[NodeId], target_parent: Option<&NodeId>) -> Result<(), ReplayError>`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Nesting depth <= 5 | Result error variant | `Result<T, ReplayError::NestedSubgraphLimitExceeded>` |

## Violation Examples
- VIOLATES <P1>: `DomainOp::Group { ids: ["n1", "n2"] }` where `n1` is already at depth 5 -- should produce `Err(ReplayError::NestedSubgraphLimitExceeded(5))`
- VIOLATES <P1>: `DomainOp::NodeMove` into a subgraph at depth 5 -- should produce `Err(ReplayError::NestedSubgraphLimitExceeded(5))` (if Move is used for reparenting)

## Ownership Contracts
- `fn apply_group(state: DiagramProjection, ids: &[String]) -> Result<DiagramProjection, ReplayError>`: Mutates the nodes in the projection by adding a new `Subgraph` and updating the `parent` field of the specified nodes.

## Non-goals
- Validating the size or position of subgraphs (handled by other beads).
- Cycle detection (handled by `CyclePolicy`).
