# Contract: Document and Scene Graph Invariants (DOC-001 to DOC-020)

## Overview

This contract specifies 20 test cases verifying document invariants for the Seshat diagram tool.
All tests operate on the `DiagramDocument` type and its sub-structures (`Node`, `Edge`, `NodeId`, `EdgeId`, `Revision`).

## Test Environment

- **Module**: `diagram_models::document`
- **Types**: `DiagramDocument`, `DocumentData`, `Node`, `Edge`, `NodeId`, `EdgeId`, `Revision`, `OrderedFloat`, `DocumentError`
- **Validation**: `diagram_models::validation::rules::validate_document_data`
- **DAG**: `diagram_models::dag::validate_dag`

## Error Taxonomy

| Error | Condition |
|---|---|
| `DocumentError::NodeNotFound(NodeId)` | Node does not exist in document |
| `DocumentError::EdgeNotFound(EdgeId)` | Edge does not exist in document |
| `DocumentError::EdgeAlreadyExists(EdgeId)` | Edge ID already present |
| `DocumentError::InvalidMarqueeBounds` | Negative width/height on marquee |
| `OrderedFloatError::NaN` | NaN coordinate |
| `OrderedFloatError::Infinite` | Infinite coordinate |

## Test Case Specifications

### DOC-001: Create node with valid data

- **Precondition**: `DiagramDocument` is empty
- **Action**: Insert a `Node` with valid fields (finite coords, positive dims, non-empty label)
- **Postcondition**: Node is present in `document.nodes`, count == 1
- **Invariant**: I1 (unique NodeId)

### DOC-002: Create node with duplicate ID rejected

- **Precondition**: A node with ID "N1" already exists
- **Action**: Attempt to insert another node with the same `NodeId("N1")`
- **Postcondition**: Second insert overwrites (im::HashMap semantics) OR is rejected
- **Implementation**: `im::HashMap::insert` replaces; test verifies the document state after two inserts has exactly 1 node with the last value
- **Invariant**: I1 (unique NodeId) - after duplicate insert, only one entry exists

### DOC-003: Delete existing node

- **Precondition**: Document contains node "N1"
- **Action**: `remove_node(&NodeId::new("N1"))`
- **Postcondition**: Node "N1" removed, `document.nodes` is empty, `Result::Ok(())`

### DOC-004: Delete non-existent node returns error

- **Precondition**: Document is empty
- **Action**: `remove_node(&NodeId::new("N999"))`
- **Postcondition**: `Err(DocumentError::NodeNotFound(NodeId::new("N999")))`

### DOC-005: Update node position

- **Precondition**: Node "N1" at position (0, 0)
- **Action**: Replace node with updated x, y values
- **Postcondition**: Node "N1" has new coordinates, other fields unchanged

### DOC-006: Create edge between valid nodes

- **Precondition**: Nodes "N1" and "N2" exist
- **Action**: `add_edge(EdgeId::new("E1"), Edge{source: N1, target: N2, ...})`
- **Postcondition**: Edge "E1" present in `document.edges`, count == 1

### DOC-007: Create edge with non-existent source rejected

- **Precondition**: Only node "N2" exists
- **Action**: `add_edge` with source "N1" (non-existent)
- **Postcondition**: `Err(DocumentError::NodeNotFound(NodeId::new("N1")))`, edges unchanged

### DOC-008: Create edge that would form cycle rejected

- **Precondition**: Nodes N1->N2->N3 chain with edges E1(N1,N2), E2(N2,N3)
- **Action**: Add edge E3(N3,N1) to form cycle
- **Postcondition**: Validation detects DAG cycle error (via `validate_dag`)
- **Note**: This is a validation-level check, not an add_edge rejection. Test creates the cycle then validates.

### DOC-009: Delete edge

- **Precondition**: Edge "E1" exists
- **Action**: `remove_edge(&EdgeId::new("E1"))`
- **Postcondition**: Edge removed, nodes unchanged, `Ok(())`

### DOC-010: Delete node cascades connected edges

- **Precondition**: N1, N2, N3 with edges E1(N1,N2) and E2(N2,N3)
- **Action**: `remove_node(&NodeId::new("N2"))`
- **Postcondition**: N2 removed, E1 and E2 removed, N1 and N3 remain

### DOC-011: Revision increments on every mutation

- **Precondition**: `doc.revision == Revision::INITIAL` (value 0)
- **Action**: Perform add_edge (which should increment revision)
- **Postcondition**: `doc.revision == Revision::new(1)`
- **Implementation note**: `add_edge` currently does NOT auto-increment. We implement a `add_node` method that does, and a general mutation wrapper.

### DOC-012: Revision never decreases

- **Precondition**: `doc.revision == Revision::new(5)`
- **Action**: Set revision to a lower value
- **Postcondition**: Operation is rejected or revision remains at 5
- **Implementation**: `Revision` type provides only `increment()`, no `decrement()`. Document enforces monotonicity.

### DOC-013: Concurrent revision mismatch detected

- **Precondition**: Client has document at revision 5, server at revision 7
- **Action**: Client attempts mutation with expected revision 5
- **Postcondition**: `Err(RevisionMismatch { expected: 5, actual: 7 })`
- **Implementation**: New `DocumentError::RevisionMismatch` variant + `check_and_mutate` method.

### DOC-014: Multi-node delete is atomic

- **Precondition**: N1, N2, N3 exist with edges E1(N1,N2), E2(N2,N3)
- **Action**: Delete [N1, N2] atomically
- **Postcondition**: Only N3 remains, all edges removed, no partial state

### DOC-015: Schema rejects NaN coordinates

- **Precondition**: A node with NaN x or y coordinate
- **Action**: `validate_document_data(&doc)`
- **Postcondition**: Validation returns `INVALID_NUMERIC` issue for NaN coordinate

### DOC-016: Schema rejects negative dimensions

- **Precondition**: A node with negative width or height
- **Action**: `validate_document_data(&doc)`
- **Postcondition**: Validation returns `INVALID_NUMERIC` issue for negative dimension

### DOC-017: Schema rejects empty ID

- **Precondition**: Empty string attempted as NodeId
- **Action**: `NodeId::try_new(String::new())`
- **Postcondition**: `Err("NodeId cannot be empty")`

### DOC-018: Circular parent chain rejected

- **Precondition**: N1.parent = N2, N2.parent = N1 (cycle)
- **Action**: `validate_document_data(&doc)`
- **Postcondition**: Validation returns `PARENT_CYCLE` issue

### DOC-019: Edge with self-loop detected

- **Precondition**: Edge E1 with source == target (same node)
- **Action**: `validate_dag(&nodes, &edges)` (DAG check) or `validate_document_data`
- **Postcondition**: CycleError::CycleDetected or DAG_CYCLE validation issue

### DOC-020: Document serialization round-trip preserves all data

- **Precondition**: Fully populated document with nodes, edges, editor state, revision
- **Action**: `serde_json::to_string(&doc)` then `serde_json::from_str::<DiagramDocument>(&json)`
- **Postcondition**: Parsed document equals original (`PartialEq`)

## DAG Validation Rules

1. **Acyclicity**: The directed graph formed by edges must be acyclic (validated via `petgraph::algo::is_cyclic_directed`)
2. **Self-loops**: An edge where source == target is a cycle (rejected by `validate_dag`)
3. **Connectivity**: (Optional) All nodes must be reachable (currently `check_connectivity` is `#[allow(dead_code)]`)
4. **Dangling edges**: Edges must reference existing nodes (validated by `check_edge_dangling`)

## Implementation Plan

### New Methods on DiagramDocument

1. `add_node(node_id: NodeId, node: Node) -> Result<(), DocumentError>` - with duplicate detection and revision increment
2. `update_node(node_id: &NodeId, f: impl Fn(Node) -> Node) -> Result<(), DocumentError>` - with revision increment
3. `remove_nodes_batch(ids: &[NodeId]) -> Result<(), DocumentError>` - atomic multi-delete
4. `check_revision(expected: Revision) -> Result<(), DocumentError>` - optimistic concurrency check

### New Error Variants

1. `DocumentError::RevisionMismatch { expected: u64, actual: u64 }`
2. `DocumentError::NodeAlreadyExists(NodeId)`
