# Contract Specification

## Context
- Feature: SUB-001 Subgraph Parent Validation
- Domain terms:
  - Node: An element in the diagram (can be of kind Node, Subgraph, Text, etc.)
  - Subgraph: A container node that can have children
  - Parent: The container node that a child node belongs to
  - Reparenting: Changing the parent of a node to a different container
- Assumptions:
  - Validation applies to both reparenting operations and document validation
  - The system must reject any node whose parent is not a Subgraph
- Open questions:
  - None - validation already exists in codebase

## Preconditions
- [P1] Parent node must exist in the canvas
- [P2] Parent node must be of kind Subgraph
- [P3] Child node must exist in the canvas

## Postconditions
- [Q1] After successful reparenting, child's parent reference points to the new Subgraph
- [Q2] Validation returns no errors when parent is a valid Subgraph

## Invariants
- [I1] A node can only have one parent at a time
- [I2] The parent-child relationship must not create a cycle
- [I3] All nodes with a parent must have that parent be a Subgraph

## Error Taxonomy
- Error::InvalidNodeType - when parent is not a Subgraph
- Error::NodeNotFound - when child or parent node doesn't exist
- Error::CircularDependency - when reparenting would create a cycle

## Contract Signatures
- fn validate_parent_is_subgraph(canvas: &CanvasState, parent_id: &NodeId) -> Result<(), Error>
- fn set_node_parent(child_id: NodeId, parent_id: NodeId, canvas: &mut CanvasState) -> Result<(), Error>
- fn validate_document_data(document: &DocumentData) -> Vec<ValidationIssue>

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Parent exists | Runtime-checked constructor | Result error variant |
| P2: Parent is Subgraph | Runtime-checked constructor | validate_parent_is_subgraph returns Error::InvalidNodeType |
| P3: Child exists | Runtime-checked constructor | Result error variant |

## Violation Examples
- VIOLATES P2: set_node_parent(child, non_subgraph_parent) -- should produce Err(Error::InvalidNodeType)
- VIOLATES P2: validate_document with non-subgraph parent -- should produce ValidationIssue with code "invalid-parent"
- VIOLATES P1: set_node_parent(child, nonexistent_parent) -- should produce Err(Error::NodeNotFound)

## Ownership Contracts
- set_node_parent takes &mut CanvasState - mutates canvas.nodes field (updates parent reference)

## Non-goals
- Coordinate transforms during reparenting (explicitly excluded per bead)
- Adding new validation beyond subgraph parent check
