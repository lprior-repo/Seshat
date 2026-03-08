# Implementation: SEL-002 Edge Selection by Click

## Summary

Implemented test coverage for SEL-002: Select single edge by clicking.

## Files Changed

### 1. `diagram_tool/src/core/selection_tests.rs` (NEW)

Created new test module with comprehensive test coverage for edge selection:

#### Test Cases Implemented

**Happy Path Tests:**
- `test_sel_002_given_document_with_two_nodes_and_edge_when_clicking_edge_then_edge_is_selected` - Primary happy path test
- `test_sel_002_given_document_with_edge_when_clicking_at_edge_center_then_edge_selected` - Center click test

**Error Path Tests:**
- `test_sel_002_given_empty_document_when_clicking_then_no_edge_selected` - Empty document
- `test_sel_002_given_document_with_edge_when_clicking_far_from_edge_then_no_edge_selected` - Click far from edge
- `test_sel_002_given_document_when_clicking_with_nan_coordinates_then_no_edge_selected` - NaN coordinates

**Edge Case Tests:**
- `test_sel_002_given_horizontal_edge_when_clicking_at_endpoint_then_edge_selected` - Endpoint selection
- `test_sel_002_given_vertical_edge_when_clicking_along_edge_then_edge_selected` - Vertical edge
- `test_sel_002_given_diagonal_edge_when_clicking_along_edge_then_edge_selected` - Diagonal edge

**Contract Verification Tests:**
- `test_precondition_p1_document_contains_edge` - P1: Document has edge
- `test_precondition_p4_coordinates_finite` - P4: Coordinates finite  
- `test_postcondition_q1_selection_count_exactly_one` - Q1: Single selection
- `test_postcondition_q2_selection_contains_edge_id` - Q2: Correct edge selected
- `test_invariant_i1_selection_contains_valid_ids` - I1: Valid IDs only
- `test_invariant_i4_edge_selection_does_not_mutate_nodes` - I4: No mutation

## Contract Mapping

| Contract Clause | Test Coverage |
|-----------------|---------------|
| P1: Document contains edge | `test_precondition_p1_document_contains_edge` |
| P4: Coordinates finite | `test_precondition_p4_coordinates_finite` |
| Q1: Exactly one selected | `test_postcondition_q1_selection_count_exactly_one` |
| Q2: Correct edge ID | `test_postcondition_q2_selection_contains_edge_id` |
| Q3: No nodes selected | Happy path tests verify this |
| I1: Valid IDs | `test_invariant_i1_selection_contains_valid_ids` |
| I4: No mutation | `test_invariant_i4_edge_selection_does_not_mutate_nodes` |

## Technical Details

### Test Setup

The tests use `DiagramDocument` with:
- Two nodes: "node-a" at (0,0) and "node-b" at (100,0), each 10x10
- One edge: "edge-1" connecting node-a to node-b
- Default zoom of 1.0

### Hit Test (find_edge_at)

The `find_edge_at(doc, x, y)` function:
- Returns `Option<EdgeId>` - `None` when no edge hit
- Uses screen-consistent hit radius: 17px screen / zoom = world hit radius
- For endpoint clicks, uses 21px screen / zoom radius
- Returns closest edge when multiple edges are within hit radius

### Selection (Selection::select_edge)

The `Selection::select_edge(edge_id, doc)` method:
- Returns `Result<Selection, SelectionError>`
- Validates edge exists in document
- Returns new Selection with the edge in edges HashSet
- Uses default SelectionMode::Replace (single-select replaces previous)

### Invariant Verification

All tests verify:
- Selected edge ID exists in document
- Selection count is exactly 1
- No nodes in selection (edges only)
- Node positions unchanged after selection

## Dependencies

- `diagram_tool/src/models/selection.rs` - Selection struct
- `diagram_tool/src/models/document.rs` - DiagramDocument, Node, Edge types
- `diagram_tool/src/ui/canvas/canvas_view.rs` - find_edge_at function

## Test Execution

Run tests with:
```bash
cargo test --package diagram_tool sel_002
```
