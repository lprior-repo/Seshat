# Contract Specification: SEL-002 Edge Selection by Click

## Context

- **Feature**: SEL-002 - Select single edge by clicking
- **Scenario**: Click selects edge
- **Given**: Diagram with nodes and an edge
- **When**: Click on edge
- **Then**: Edge in selection
- **Domain terms**:
  - `DiagramDocument` - contains nodes and edges
  - `Node` - rectangular element with position (x, y) and size (width, height)
  - `Edge` - connection between two nodes (source, target)
  - `find_edge_at(doc, x, y)` - finds edge at world coordinates
  - `select_single(item_id)` - returns HashSet containing single selected item
  - `selected_items` - HashSet<String> in editor_state containing selected node/edge IDs

## Preconditions

- **P1**: Document must contain at least two nodes connected by an edge
- **P2**: The edge must exist in `doc.document.edges`
- **P3**: Source and target nodes of the edge must exist in `doc.document.nodes`
- **P4**: Click coordinates (x, y) must be finite (not NaN or Infinity)
- **P5**: The edge must be within hit-test distance at click position (17px screen / scaled to world)
- **P6**: Document must have valid zoom level for hit radius calculation

## Postconditions

- **Q1**: After selection, `doc.editor_state.selected_items` contains exactly one element
- **Q2**: The selected item ID matches the edge ID from hit test
- **Q3**: No nodes are selected (only the edge)
- **Q4**: The edge ID exists in the document (selected items reference valid entities)
- **Q5**: Selection is independent of previous selection state (single-click replaces)

## Invariants

- **I1**: Selection set contains only IDs that exist in the document
- **I2**: Node and edge IDs are distinct namespaces (edge ID "edge-1" does not conflict with node ID "edge-1")
- **I3**: Empty document implies empty selection after any operation
- **I4**: Selecting an edge does not modify node positions or properties

## Error Taxonomy

Based on existing `SelectionError` enum from `selection.rs`:

- `SelectionError::EdgeNotFound(EdgeId)` - Edge does not exist in document (should NOT occur in happy path - hit test should only return existing edges)
- `SelectionError::InvalidCoordinates(String)` - Click coordinates are NaN or Infinity
- `SelectionError::NodeNotFound(NodeId)` - Source/target node missing (precondition violation)

Note: In the context of click-to-select, "not finding an edge" is NOT an error - it's the expected result when clicking empty space. The function returns `Option<EdgeId>` not `Result<EdgeId, Error>`.

## Contract Signatures

```rust
// Primary hit-test function (returns Option - None when no edge hit)
pub(super) fn find_edge_at(doc: &DiagramDocument, x: f64, y: f64) -> Option<EdgeId>

// Selection function
pub fn select_single(item_id: String) -> HashSet<String>
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Document has edge | Runtime check | `doc.document.edges.contains_key(&edge_id)` before selection |
| Click coordinates finite | Runtime check | `x.is_finite() && y.is_finite()` in hit test |
| Edge within hit radius | Runtime algorithm | Distance calculation vs threshold |
| Valid zoom | Compile-time | `OrderedFloat` wrapper ensures non-NaN |

## Violation Examples

### Precondition Violations

- **VIOLATES P1**: `find_edge_at(&empty_doc, 100.0, 100.0)` where `empty_doc` has no edges
  - Returns: `None` (not an error - expected behavior for empty document)

- **VIOLATES P4**: `find_edge_at(&doc, f64::NAN, 100.0)` 
  - Returns: `None` (NaN coordinates cannot hit any edge - edge distance will be NaN)

- **VIOLATES P3**: Create edge with non-existent source node, call `find_edge_at`
  - Returns: `None` (edge cannot be hit if endpoints don't exist)

### Postcondition Violations

- **VIOLATES Q1**: After calling `select_single("edge-1")`, check `selected_items.len()`
  - Wrong: `len() != 1` - selection should contain exactly one item
  - Correct: `len() == 1`

- **VIOLATES Q2**: After selection, `selected_items` should contain the exact edge ID
  - Wrong: `!selected_items.contains("edge-1")` 
  - Correct: `selected_items.contains("edge-1")`

## Ownership Contracts

- `find_edge_at(&doc, x, y)` - takes `&DiagramDocument`, read-only, no mutation
- `select_single(item_id: String)` - takes owned `String`, returns owned `HashSet<String>`, no mutation of document
- `doc.editor_state.selected_items` - mutated by assignment: `doc.editor_state.selected_items = select_single(...)`

## Non-goals

- Multi-select (Shift+Click) - covered by SEL-006/SEL-007
- Toggle selection - covered by separate feature
- Selection of multiple edges - not in scope
- Edge selection via marquee/rubber-band - covered by separate selection tests

---

# Martin Fowler Test Plan: SEL-002 Edge Selection

## Happy Path Tests

### test_sel_002_given_document_with_two_nodes_and_edge_when_clicking_edge_then_edge_is_selected

**Given**: Document with two nodes (node-a at (0,0), node-b at (100,0)) and an edge between them

**When**: Click on edge at position (50, 0) - center of edge line

**Then**:
- `find_edge_at(doc, 50.0, 0.0)` returns `Some(EdgeId("edge-1"))`
- After selection, `selected_items` contains exactly "edge-1"
- `selected_items.len()` equals 1
- No nodes are selected

### test_sel_002_given_document_with_edge_when_clicking_at_edge_center_then_edge_selected

**Given**: Document with two nodes and one edge connecting them

**When**: Click at midpoint of edge (computed from node centers)

**Then**: Edge is found and selected

## Error Path Tests

### test_sel_002_given_empty_document_when_clicking_then_no_edge_selected

**Given**: Empty document (no nodes, no edges)

**When**: Click at any position (50, 50)

**Then**:
- `find_edge_at(doc, 50.0, 50.0)` returns `None`
- Selection remains empty

### test_sel_002_given_document_with_edge_when_clicking_far_from_edge_then_no_edge_selected

**Given**: Document with two nodes and one edge

**When**: Click far from the edge (e.g., (500, 500))

**Then**:
- `find_edge_at(doc, 500.0, 500.0)` returns `None`
- No selection change

### test_sel_002_given_document_when_clicking_with_nan_coordinates_then_no_edge_selected

**Given**: Document with two nodes and one edge

**When**: Click with NaN coordinates `find_edge_at(doc, f64::NAN, 50.0)`

**Then**:
- Returns `None` (NaN cannot hit any edge)

## Edge Case Tests

### test_sel_002_given_horizontal_edge_when_clicking_at_endpoint_then_edge_selected

**Given**: Document with two nodes at (0,0) and (100,0), edge between them

**When**: Click at source endpoint (0, 0) within endpoint hit radius (21px / zoom)

**Then**: Edge is found and selected

### test_sel_002_given_vertical_edge_when_clicking_along_edge_then_edge_selected

**Given**: Document with two nodes at (0,0) and (0,100), vertical edge

**When**: Click at (0, 50) - middle of vertical edge

**Then**: Edge is found and selected

### test_sel_002_given_diagonal_edge_when_clicking_along_edge_then_edge_selected

**Given**: Document with nodes at (0,0) and (100,100), diagonal edge

**When**: Click at (25, 25), (50, 50), (75, 75) along the edge

**Then**: Edge is found at all positions within hit radius

### test_sel_002_given_edge_with_bend_points_when_clicking_on_bend_then_edge_selected

**Given**: Document with edge that has bend points

**When**: Click on a bend point location

**Then**: Edge is found (polyline geometry handles bends)

## Contract Verification Tests

### test_precondition_p1_document_contains_edge

**Given**: Empty document

**When**: Call `find_edge_at(doc, x, y)`

**Then**: Returns `None` (P1 violation handled gracefully)

### test_precondition_p4_coordinates_finite

**Given**: Document with edge

**When**: Call `find_edge_at(doc, f64::NAN, 0.0)` or with Infinity

**Then**: Returns `None` (P4 violation handled gracefully)

### test_postcondition_q1_selection_count_exactly_one

**Given**: Document with edge

**When**: Select edge via `select_single("edge-1")`

**Then**: `selected_items.len() == 1`

### test_postcondition_q2_selection_contains_edge_id

**Given**: Document with edge "edge-1"

**When**: Select edge via `select_single("edge-1")`

**Then**: `selected_items.contains("edge-1")` is true

### test_invariant_i1_selection_contains_valid_ids

**Given**: Document with edge "edge-1"

**When**: Select edge

**Then**: Selected ID exists in `doc.document.edges`

### test_invariant_i4_edge_selection_does_not_mutate_nodes

**Given**: Document with nodes and edge

**When**: Select edge

**Then**: Node positions remain unchanged

## Given-When-Then Scenarios

### Scenario 1: Basic Edge Selection

**Given**: A document containing two nodes (node-a at position (0,0) with size 10x10, node-b at position (100,0) with size 10x10) and an edge (edge-1) connecting node-a to node-b

**When**: User clicks at position (50, 0) on the canvas, which is along the edge line between the two nodes

**Then**:
1. The hit test `find_edge_at(doc, 50.0, 0.0)` returns `Some(EdgeId("edge-1"))`
2. The selection operation `select_single("edge-1")` returns a HashSet containing exactly "edge-1"
3. The document's `selected_items` now contains one item: "edge-1"
4. No nodes are selected (selected_items does not contain "node-a" or "node-b")

### Scenario 2: Click on Empty Canvas

**Given**: A document containing two nodes and an edge

**When**: User clicks at position (500, 500), which is far from any edge

**Then**:
1. The hit test `find_edge_at(doc, 500.0, 500.0)` returns `None`
2. No selection change occurs
3. `selected_items` remains unchanged (empty or previous selection)

### Scenario 3: Edge Endpoint Selection

**Given**: A document with nodes at (0,0) and (100,0) with an edge between them

**When**: User clicks at the source endpoint (0, 0) - within the endpoint hit radius of 21px

**Then**:
1. The hit test finds the edge (endpoint hit is included)
2. The edge is selected
