bead_id: bd-81l
bead_title: tests: Implement MUL multi-select tests 3/4
phase: p1
updated_at: 2026-03-01T22:30:00Z

# Contract: MUL Multi-Select Tests 3/4

## Overview

Add 5 multi-select resize tests to `diagram_tool/e2e/diagram.multi-select.spec.ts`:
1. MUL-011: resize 2-point line endpoints
2. MUL-012: resize curved arrow
3. MUL-013: resize past minimum clamps
4. MUL-014: resize past inversion flips or clamps
5. MUL-015: resize container+children

## Preconditions

- `diagram_tool/e2e/diagram.multi-select.spec.ts` exists with MUL-001 through MUL-005
- `diagram_tool/e2e/helpers.ts` provides helper functions for canvas interaction
- Resize handles are exposed via `data-testid="resize-handle-{n,e,s,w,nw,ne,sw,se}"`
- Edge tool available via "Edge" button
- Subgraph tool available via "Subgraph" button

## Test Specifications

### MUL-011: Resize 2-point line endpoints

**Given**: Two nodes connected by an edge (line)
**When**: User selects the edge and drags one of its endpoints
**Then**:
- The edge endpoint moves to the new position
- The edge remains connected to both nodes
- No page errors occur

**Implementation Notes**:
- Create two nodes with an edge between them
- Select the edge
- This tests edge selection and resize handle behavior on edges
- If edges don't have resize handles, test that edge selection works and nodes can be repositioned to "resize" the edge

### MUL-012: Resize curved arrow

**Given**: A curved arrow (edge with arrow type) between two nodes
**When**: User resizes by moving one of the connected nodes
**Then**:
- The curved edge updates its curve to reflect new positions
- The arrow head remains at the target node
- No visual artifacts or errors

**Implementation Notes**:
- Create two nodes with a curved/directed edge
- Select one node and drag to resize the edge visually
- Verify edge routing updates correctly

### MUL-013: Resize past minimum clamps

**Given**: A selected node or multi-selection
**When**: User drags resize handle past the minimum size threshold (24px)
**Then**:
- Width and/or height clamp to minimum value (24px)
- Selection remains valid
- No negative dimensions
- No page errors

**Implementation Notes**:
- Create a node and select it
- Grab east or south-east resize handle
- Drag far enough to shrink below minimum
- Verify final dimensions are >= 24px

### MUL-014: Resize past inversion flips or clamps

**Given**: A selected node
**When**: User drags resize handle past the opposite edge (creating inverted dimensions)
**Then**:
- Either: Width/height clamp at minimum (no inversion)
- Or: Selection flips orientation (position changes, dimensions stay positive)
- No negative dimensions ever
- No NaN or Infinity values
- No page errors

**Implementation Notes**:
- Create a node and select it
- Grab west resize handle and drag past the east edge
- Or grab north handle and drag past south edge
- Verify behavior is deterministic and safe

### MUL-015: Resize container+children

**Given**: A subgraph (container) with child nodes inside
**When**: User selects the subgraph and resizes it
**Then**:
- Subgraph dimensions change
- Children scale proportionally OR maintain relative positions
- No children escape the container bounds unexpectedly
- No page errors

**Implementation Notes**:
- Create a subgraph with 2+ child nodes
- Select the subgraph
- Resize using corner or edge handle
- Verify children remain within or proportionally scaled

## Postconditions

- All 5 tests pass
- No page errors in any test
- Tests follow existing patterns in diagram.multi-select.spec.ts
- Tests use helper functions from helpers.ts

## Invariants

- Test isolation: each test uses freshStart()
- No flaky timeouts: use runEffect/runEffectsSequential
- Page errors tracked via trapPageErrors
- All assertions have meaningful tolerances for floating point

## Acceptance Criteria

1. `moon run :test` passes with new tests
2. `moon run :ci` passes
3. Tests are properly tagged (@baseline, @behavior, etc.)
4. Code follows existing test file patterns
