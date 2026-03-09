# Contract Specification

## Context

- **Feature**: DDD Refactor: Edges and Subgraphs Extraction
- **Bead ID**: seshat-hly
- **Domain terms**:
  - `Edge` - A directed connection between two nodes in the diagram
  - `Node` - A visual element in the diagram (Text, Subgraph, etc.)
  - `NodeId` - Unique identifier for nodes
  - `EdgeId` - Unique identifier for edges
  - `Subgraph` - A grouping node that contains child nodes
  - `DiagramDocument` - The entire diagram data structure
  - `RoutingError` - Errors from edge routing operations
  - `GroupingError` - Errors from grouping/ungrouping operations

- **Assumptions**:
  - The codebase has `routing.rs` with `create_edge()` returning `Result<T, RoutingError>`
  - The codebase has `grouping.rs` with `group_selection()` and `ungroup_selection()` returning `Result<T, GroupingError>`
  - These functions are already pure domain logic (no UI dependencies)

- **Open questions**:
  - What specific "edge routing bounds checking" exists in UI components that needs extraction? (None currently found - coordinates are not validated)
  - What specific "topological subgraph constraints" exist in UI components that needs extraction? (Currently none beyond empty selection)

## Preconditions

### Edge Creation (`create_edge`)

- **P1**: Source node must exist in document
  - Enforcement: Runtime check via `doc.document.nodes.contains_key(&source)`
  - Violation: Returns `Err(RoutingError::SourceNotFound(NodeId))`

- **P2**: Target node must exist in document
  - Enforcement: Runtime check via `doc.document.nodes.contains_key(&target)`
  - Violation: Returns `Err(RoutingError::TargetNotFound(NodeId))`

- **P3**: Source and target must be different (no self-loops)
  - Enforcement: Runtime check `source != target`
  - Violation: Returns `Err(RoutingError::SelfLoop(NodeId))`

- **P4**: Edge must not create a cycle in the DAG
  - Enforcement: Runtime check via BFS cycle detection algorithm
  - Violation: Returns `Err(RoutingError::CycleDetected)`

### Group Selection (`group_selection`)

- **P5**: Selection must not be empty
  - Enforcement: Runtime check `selected.is_empty()`
  - Violation: Returns `Err(GroupingError::EmptySelection)`

- **P6**: No selected nodes can be locked
  - Enforcement: Runtime check `node.locked`
  - Violation: Returns `Err(GroupingError::LockedNode(NodeId))`

### Ungroup Selection (`ungroup_selection`)

- **P7**: At least one Subgraph must be selected
  - Enforcement: Runtime check `!target_subgraphs.is_empty()`
  - Violation: Returns `Err(GroupingError::EmptySelection)`

## Postconditions

### Edge Creation (`create_edge`)

- **Q1**: Edge is inserted into document with correct source/target
  - State: `doc.document.edges.contains_key(edge_id)` is true
  - State: `doc.document.edges.get(edge_id).source == source`
  - State: `doc.document.edges.get(edge_id).target == target`

- **Q2**: Edge has default styling
  - State: Edge has default `label` (empty), `style`, `arrow_type`, `directed=true`, `bend_points=empty`

### Group Selection (`group_selection`)

- **Q3**: New subgraph node is created with correct bounding box
  - State: `doc.document.nodes.contains_key(group_id)`
  - State: `group_node.kind == NodeKind::Subgraph`
  - State: Group bounds include all selected nodes plus padding (20.0 units)

- **Q4**: Children are reparented to the new group
  - State: All selected nodes have `parent == Some(group_id)`

- **Q5**: Selection is updated to contain only the new group
  - State: `doc.editor_state.selected_items == {group_id.as_str()}`

### Ungroup Selection (`ungroup_selection`)

- **Q6**: Subgraph is removed from document
  - State: `doc.document.nodes.get(subgraph_id)` is None for all selected subgraphs

- **Q7**: Children are orphaned (parent set to inherited parent)
  - State: All former children have `parent` updated appropriately

- **Q8**: Connected edges are removed
  - State: Any edge with source or target in deleted subgraphs is removed

- **Q9**: Former children are selected
  - State: All orphaned children are in `doc.editor_state.selected_items`

## Invariants

### Document Integrity

- **I1**: Document DAG remains acyclic after any edge operation
  - Check: Cycle detection passes for all edges

- **I2**: No edge references non-existent nodes
  - Check: For all edges, both source and target exist in nodes

- **I3**: Subgraph children are within parent's bounding box (including padding)
  - Check: Child nodes' bounds are contained within parent bounds

- **I4**: NodeId and EdgeId are non-empty
  - Check: Inner string has length > 0

### Type System Invariants (Compile-Time Enforcement)

- **I5**: `NodeId` is non-empty - enforced via `NodeId::try_new()`
- **I6**: `EdgeId` is non-empty - enforced via `EdgeId::try_new()`

## Error Taxonomy

### RoutingError
```rust
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoutingError {
    #[error("Source node {0} not found")]
    SourceNotFound(NodeId),           // P1 violation

    #[error("Target node {0} not found")]
    TargetNotFound(NodeId),           // P2 violation

    #[error("Cannot create self-loop on node {0}")]
    SelfLoop(NodeId),                 // P3 violation

    #[error("Adding this edge creates a cycle")]
    CycleDetected,                    // P4 violation
}
```

### GroupingError
```rust
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GroupingError {
    #[error("Selection is empty")]
    EmptySelection,                        // P5, P7 violation

    #[error("Node {0} is locked")]
    LockedNode(NodeId),                    // P6 violation
}
```

## Contract Signatures

```rust
// Edge Routing Domain Model (routing.rs)
pub fn create_edge(
    doc: &mut DiagramDocument,
    source: NodeId,
    target: NodeId,
    edge_id: EdgeId,
) -> Result<(), RoutingError>

// Subgraph Domain Model (grouping.rs)
pub fn group_selection(
    doc: &mut DiagramDocument,
    group_id: &NodeId,
) -> Result<(), GroupingError>

pub fn ungroup_selection(
    doc: &mut DiagramDocument,
) -> Result<(), GroupingError>
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Source exists | Runtime (Result) | `RoutingError::SourceNotFound` |
| P2: Target exists | Runtime (Result) | `RoutingError::TargetNotFound` |
| P3: No self-loop | Runtime (Result) | `RoutingError::SelfLoop` |
| P4: No cycle | Runtime (Result) | `RoutingError::CycleDetected` |
| P5: Selection non-empty | Runtime (Result) | `GroupingError::EmptySelection` |
| P6: No locked nodes | Runtime (Result) | `GroupingError::LockedNode` |
| P7: Subgraph selected | Runtime (Result) | `GroupingError::EmptySelection` |
| I5: NodeId non-empty | Compile-time | `NodeId::try_new()` |
| I6: EdgeId non-empty | Compile-time | `EdgeId::try_new()` |

**Note**: All preconditions require runtime enforcement because:
- Node existence depends on document state
- Selection state depends on editor state
- Cycle detection is inherently a graph algorithm

Compile-time enforcement is achieved for I5 and I6 via `try_new()` constructors.

## Violation Examples (REQUIRED)

### Edge Creation Violations

- **VIOLATES P1**: `create_edge(&mut doc, NodeId::new("nonexistent".into()), target, edge_id)` 
  - Expected: `Err(RoutingError::SourceNotFound(NodeId::new("nonexistent".into())))`

- **VIOLATES P2**: `create_edge(&mut doc, source, NodeId::new("nonexistent".into()), edge_id)` 
  - Expected: `Err(RoutingError::TargetNotFound(NodeId::new("nonexistent".into())))`

- **VIOLATES P3**: `create_edge(&mut doc, node_id.clone(), node_id.clone(), edge_id)` 
  - Expected: `Err(RoutingError::SelfLoop(node_id))`

- **VIOLATES P4**: Given document with edges A→B and B→C, `create_edge(&mut doc, C, A, edge_id)` 
  - Expected: `Err(RoutingError::CycleDetected)`

### Group Selection Violations

- **VIOLATES P5**: `group_selection(&mut doc, &group_id)` with empty selection 
  - Expected: `Err(GroupingError::EmptySelection)`

- **VIOLATES P6**: `group_selection(&mut doc, &group_id)` with locked node in selection 
  - Expected: `Err(GroupingError::LockedNode(locked_node_id))`

### Ungroup Selection Violations

- **VIOLATES P7**: `ungroup_selection(&mut doc)` with non-Subgraph nodes selected 
  - Expected: `Err(GroupingError::EmptySelection)`

## Ownership Contracts

### `create_edge(&mut doc, source, target, edge_id)`
- **Mutates**: `doc.document.edges` - inserts new edge
- **Does NOT mutate**: nodes or editor state
- **Clone policy**: None required, uses references and clones for ID comparison

### `group_selection(&mut doc, group_id)`
- **Mutates**: `doc.document.nodes` - inserts new Subgraph node
- **Mutates**: `doc.document.nodes[id].parent` - reparents selected nodes
- **Mutates**: `doc.editor_state.selected_items` - clears and sets to group_id
- **Clone policy**: `NodeId` is cloned when setting as parent - necessary for graph structure

### `ungroup_selection(&mut doc)`
- **Mutates**: `doc.document.nodes` - removes Subgraph nodes, updates parent of children
- **Mutates**: `doc.document.edges` - removes edges connected to deleted Subgraphs
- **Mutates**: `doc.editor_state.selected_items` - clears and sets orphaned children
- **Clone policy**: Same as `group_selection`

## Non-goals

- [ ] Implementing actual edge path/route calculation (rendering concern)
- [ ] Edge coordinate validation (NaN/Inf) - not in current implementation
- [ ] Subgraph minimum size validation - not in current implementation
- [ ] Implementing node dragging physics
- [ ] Implementing zoom/pan viewport logic
- [ ] UI-specific validation (e.g., "must be in edit mode")
- [ ] Implementing automatic layout algorithms (handled in layout module)
- [ ] Adding edge routing styles (orthogonal, curved, etc.)
- [ ] Edge deletion domain function (handled by delete module)
- [ ] DAG validation standalone function (embedded in create_edge)
