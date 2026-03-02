bead_id: bd-1ub
bead_title: tests: Implement EDG edge binding tests 2/4
phase: p1
updated_at: 2026-03-01T18:20:00Z

# Implementation: EDG Edge Binding Tests 2/4

## Summary

Implemented 5 edge binding behavior tests for the Seshat diagram tool in a new test file `diagram_tool/e2e/diagram.edge-binding-2.spec.ts`.

## Tests Implemented

### EDG-011: Rotate node keeps binding (SKIPPED)
- **Status**: Skipped - Rotation feature not currently exposed in the UI
- **Test ID**: `EDG-011: rotate node keeps binding @baseline`
- **Rationale**: The rotation controls are not implemented in the current UI, so this test is marked with `test.skip()` until rotation is available.

### EDG-012: Rotate selection with edges (SKIPPED)
- **Status**: Skipped - Rotation feature not currently exposed in the UI
- **Test ID**: `EDG-012: rotate selection with edges @baseline`
- **Rationale**: Same as EDG-011 - rotation controls not yet available.

### EDG-013: Resize selection with edges maintains bindings
- **Status**: Implemented
- **Test ID**: `EDG-013: resize selection with edges maintains bindings @baseline`
- **Steps**:
  1. Create two text nodes at positions (450, 220) and (600, 320)
  2. Create an edge connecting the two nodes
  3. Switch to select mode
  4. Select both nodes using shift+click
  5. Resize selection using SE corner handle
  6. Verify edge count remains 1 (binding maintained)
  7. Verify selection still contains 2 nodes

### EDG-014: Clicking edge selects edge only not nodes
- **Status**: Implemented
- **Test ID**: `EDG-014: clicking edge selects edge only not nodes @baseline`
- **Steps**:
  1. Create two text nodes horizontally aligned at (400, 280) and (700, 280)
  2. Create an edge connecting the two nodes
  3. Switch to select mode
  4. Clear selection by clicking canvas whitespace
  5. Click on the midpoint of the edge
  6. Verify selectedCount is 1 (edge selected, not nodes)
  7. Verify edge still exists

### EDG-015: Edge endpoint follows node during drag
- **Status**: Implemented
- **Test ID**: `EDG-015: edge endpoint follows node during drag @baseline`
- **Steps**:
  1. Create two text nodes horizontally aligned at (400, 280) and (700, 280)
  2. Create an edge connecting the two nodes
  3. Switch to select mode
  4. Drag first node by offset (100, 80)
  5. Verify edge still exists (binding maintained)
  6. Verify node moved by approximately the drag offset
  7. Clear selection and click on new edge midpoint
  8. Verify edge is still selectable (visual updated correctly)

## Helper Functions

The tests use these internal helper functions:

1. `getSelectionBounds(canvas)` - Gets the bounding box of the selection rectangle
2. `getResizeHandle(canvas, corner)` - Gets the bounding box of a resize handle
3. `dragHandle(page, handleBox, dx, dy)` - Drags a resize handle by the given offset
4. `edgeClick(page, x, y)` - Performs a mouse click at the given coordinates
5. `clickCanvasWhitespace(page, canvasRoot)` - Clicks on empty canvas area to clear selection
6. `extrema(points)` - Finds left/right/top/bottom points from an array
7. `selectMultipleNodes(page, canvas, count)` - Selects multiple nodes using shift+click

## Patterns Followed

1. Each test follows the established pattern from `diagram.multi-select-resize.spec.ts`:
   - `freshStart(page)` for clean state
   - `clearCanvasOverlays(page)` to dismiss panels
   - `trapPageErrors(page)` to capture any page errors
   - Verification that `pageErrors` is empty at test end

2. Uses `waitForNoRebuildOverlay(page)` instead of `waitForUiReady(page)` after mode switches to avoid unnecessary delays

3. Uses `runEffectsSequential` for multi-step operations

4. Uses `@baseline` tag for stable test identification

## Files Created

- `diagram_tool/e2e/diagram.edge-binding-2.spec.ts` - New test file with 5 tests (2 skipped, 3 active)

## Files Modified

- None (new file only)

## Dependencies

- Uses existing helper functions from `helpers.ts`:
  - `clearCanvasOverlays`
  - `createTextNode`
  - `edgeCount`
  - `expectEdgeCount`
  - `expectNodeCount`
  - `expectSelectedCount`
  - `freshStart`
  - `nodeCenters`
  - `runEffectsSequential`
  - `runEffect`
  - `selectedCount`
  - `trapPageErrors`
  - `waitForNoRebuildOverlay`
