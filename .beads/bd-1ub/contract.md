bead_id: bd-1ub
bead_title: tests: Implement EDG edge binding tests 2/4
phase: p0
updated_at: 2026-03-01T18:10:00Z

# Contract: EDG Edge Binding Tests 2/4

## Summary

Implement 5 edge binding behavior tests for the Seshat diagram tool:
1. Rotate node keeps binding (EDG-011)
2. Rotate selection with edges (EDG-012)
3. Resize selection with edges (EDG-013)
4. Multi-select includes edge but not nodes (EDG-014)
5. Edge endpoint follows node during drag (EDG-015)

## Context

This bead is part of the EDG (edge binding) test series 2/4:
- Part 1/4 (bd-2cb): Basic edge creation, reconnect, delete with bound edge, move node, resize node
- Part 2/4 (bd-1ub): Rotate node, rotate selection, resize selection, multi-select edge, edge endpoint follow

The existing codebase has:
- `diagram_tool/e2e/diagram.edges-and-routing.spec.ts` - edge routing and hit detection tests
- `diagram_tool/e2e/diagram.multi-select-resize.spec.ts` - multi-select resize tests (pattern reference)

## Preconditions

- Playwright test infrastructure exists
- Helper functions available: `freshStart`, `clearCanvasOverlays`, `createTextNode`, `nodeFrameByLabel`, `runEffect`, `runEffectsSequential`, `trapPageErrors`, `waitForUiReady`, `nodeCount`, `edgeCount`, `selectedCount`, `expectNodeCount`, `expectEdgeCount`, `expectSelectedCount`, `nodeCenters`
- Edge tool available via "Edge" button
- Select tool available via "Select" button

## Test Specifications

### EDG-011: Rotate node keeps binding

**Given**: Two nodes connected by an edge
**When**: One node is rotated (if rotation is implemented)
**Then**:
- The edge endpoint remains attached to the rotated node
- No errors occur
- Edge is still selectable

**Acceptance Criteria**:
- Test creates two nodes with connecting edge
- Attempts rotation operation on one node
- Verifies edge remains attached
- No page errors

### EDG-012: Rotate selection with edges

**Given**: A selection containing multiple nodes with edges between them
**When**: The selection is rotated (if rotation is implemented)
**Then**:
- All edges remain connected to their respective nodes
- Edge positions update correctly
- No errors occur

**Acceptance Criteria**:
- Test creates multiple nodes with edges
- Selects all nodes
- Attempts rotation operation
- Verifies all edges remain connected
- No page errors

### EDG-013: Resize selection with edges

**Given**: A selection containing nodes with edges
**When**: The selection is resized using corner handles
**Then**:
- Edges remain attached to their nodes
- Edge visual routing updates appropriately
- No errors occur

**Acceptance Criteria**:
- Test creates nodes with connecting edges
- Selects all and resizes via corner handle
- Verifies edge count remains unchanged
- Verifies edges are still connected
- No page errors

### EDG-014: Multi-select includes edge but not nodes

**Given**: Two nodes connected by an edge
**When**: The edge is clicked in select mode
**Then**:
- Only the edge is selected (not the nodes)
- Selected count reflects 1 item
- Nodes remain unselected

**Acceptance Criteria**:
- Test creates two nodes with connecting edge
- Switches to select mode
- Clicks on the edge (not near endpoints)
- Verifies selectedCount is 1
- Verifies nodes are not selected
- No page errors

### EDG-015: Edge endpoint follows node during drag

**Given**: Two nodes connected by an edge
**When**: One node is dragged to a new position
**Then**:
- The edge endpoint follows the node
- Edge remains connected
- Edge visual updates correctly

**Acceptance Criteria**:
- Test creates two nodes with connecting edge
- Drags one node to new position
- Verifies edge is still connected
- Verifies edge visual reflects new positions
- No page errors

## Technical Requirements

1. Create new test file: `diagram_tool/e2e/diagram.edge-binding-2.spec.ts`
2. Use existing helper functions from `helpers.ts`
3. Follow existing test patterns from `diagram.edges-and-routing.spec.ts` and `diagram.multi-select-resize.spec.ts`
4. Each test should:
   - Call `trapPageErrors(page)` and verify `pageErrors` is empty at end
   - Use `freshStart(page)` for clean state
   - Use `clearCanvasOverlays(page)` to dismiss panels
   - Use `runEffectsSequential` for multi-step operations
   - Use `@baseline` tag for stable tests

## Postconditions

- All 5 tests pass
- No page errors in any test
- `moon run :quick` passes
- `moon run :test` passes
- Test file follows project conventions

## Invariants

- Tests must be deterministic (no flaky behavior)
- Tests must use existing helper functions
- Tests must not use arbitrary timeouts (use `waitForUiReady` instead)
- Each test must be independent (no shared state between tests)

## Out of Scope

- Edge routing algorithm tests (covered in edges-and-routing.spec.ts)
- Edge hit detection tests (covered in edges-and-routing.spec.ts)
- Save/reload tests
- Subgraph tests
