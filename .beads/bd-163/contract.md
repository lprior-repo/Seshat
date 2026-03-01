bead_id: bd-163
bead_title: tests: Implement SUB subgraph tests 1/4
phase: p0
updated_at: 2026-03-01T22:25:00Z

# Contract: SUB Subgraph Tests 1/4

## Scope

Implement 5 subgraph unit tests covering:
1. Group selection creates group with correct bounds and parent relationships
2. Ungroup restores children to canvas root (parent = None)
3. Nested groups work correctly (subgraph inside subgraph)
4. Container/frame creation via DrawingSubgraph interaction mode
5. Prevent parent cycles in schema validation

## Acceptance Criteria

### Test 1: Group Selection Creates Group
- GIVEN: Document with 2+ selected non-subgraph nodes
- WHEN: `apply_group_selection` is called
- THEN:
  - A new Subgraph node is created with `NodeKind::Subgraph`
  - The subgraph bounds encompass all selected nodes with 24px padding
  - All selected nodes have their `parent` field set to the new group's NodeId
  - Selection is updated to contain only the new group
  - Document revision is incremented

### Test 2: Ungroup Restores Positions
- GIVEN: Document with a selected subgraph containing child nodes
- WHEN: `apply_ungroup_selection` is called
- THEN:
  - The subgraph node is removed from the document
  - All child nodes have their `parent` field set to `None`
  - Children remain at their absolute canvas positions
  - Document revision is incremented

### Test 3: Nested Groups Work
- GIVEN: Document with an outer subgraph containing an inner subgraph
- WHEN: Inner subgraph contains child nodes
- THEN:
  - Inner subgraph's `parent` points to outer subgraph
  - Child nodes' `parent` points to inner subgraph
  - Schema validation passes for valid nesting

### Test 4: Container/Frame Creation
- GIVEN: Canvas in DrawingSubgraph interaction mode
- WHEN: User draws a subgraph rectangle
- THEN:
  - `InteractionMode::DrawingSubgraph` captures start and current coordinates
  - Subgraph is created with correct bounds on finalization
  - Created subgraph has `locked: true` and `z_index: -1`

### Test 5: Prevent Parent Cycles
- GIVEN: Document with nodes forming a potential parent cycle (A -> B -> C -> A)
- WHEN: Schema validation is performed
- THEN:
  - `validate_schema` returns an error
  - Error message contains "circular" or "cycle"

## Invariants

- All tests must use `#![deny(clippy::unwrap_used)]` pattern
- All tests must be pure unit tests (no file I/O, no network)
- All tests must follow existing test naming convention: `given_X_when_Y_then_Z`
- Tests must not use `expect()` or `panic!()`

## Test Location

Tests should be added to:
- `diagram_tool/src/ui/commands.rs` (tests module) for group/ungroup tests
- `diagram_tool/src/models/schema.rs` (tests module) for cycle prevention tests
- `diagram_tool/src/ui/canvas/interaction_reducer.rs` (tests module) for DrawingSubgraph tests

## Dependencies

- Existing `apply_group_selection` function in commands.rs
- Existing `apply_ungroup_selection` function in commands.rs
- Existing `validate_schema` function in schema.rs
- Existing `InteractionMode::DrawingSubgraph` variant in interaction_reducer.rs
