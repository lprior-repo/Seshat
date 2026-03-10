# Contract Specification

## Context
- **Bead ID**: oya-4ye
- **Title**: multi-select: Drag selection across container boundary triggers reparent (MUL-003)
- **Domain**: Diagram tool - multi-selection drag operations
- **Assumptions**: Multi-select drag currently moves nodes but does not trigger reparent when crossing container boundaries. The feature needs to be implemented/tested.
- **Open Questions**: None - behavior is defined in generate_all_tests.py

## Preconditions
- **P1**: At least one node is selected in the multi-selection
- **P2**: A container (Subgraph) exists that is NOT part of the selection
- **P3**: The drag operation moves the selection such that ALL selected nodes are now visually contained within a target container

## Postconditions
- **Q1**: All nodes in the selection that are dragged into a container become children of that container
- **Q2**: Nodes maintain their screen position after reparenting (world transform preserved)
- **Q3**: Undo reverses the reparent operation correctly
- **Q4**: Selection state is preserved after reparenting

## Invariants
- **I1**: No node becomes its own ancestor (no cycles)
- **I2**: Edges connected to reparented nodes remain valid

## Error Taxonomy
- **Error::CycleDetected** - If reparenting would create a cycle
- **Error::InvalidTarget** - If target container is part of the selection

## Contract Signatures
- For multi-select drag operations, the drag_end handler should check if selection intersects with a container
- `fn calculate_reparent_targets(selection: &[NodeId], target_container: &NodeId) -> Vec<NodeId>`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| At least one node selected | Runtime | `!selection.is_empty()` |
| Target not in selection | Runtime | `!selection.contains(target)` |
| Cycle detection | Runtime | `!is_ancestor(node, target)` |

## Violation Examples
- **VIOLATES P1**: Drag with empty selection -- should be no-op (handled by select mode)
- **VIOLATES P3**: Drag that doesn't cross container boundary -- should NOT reparent

## Ownership Contracts
- The selection is borrowed (read-only) during drag operations
- Nodes are mutated via `doc.document.nodes.get_mut(id).parent = new_parent`

## Non-goals
- Single-node reparent (already implemented via grouping.rs)
- Reparent via keyboard shortcuts
- Reparent via context menu
