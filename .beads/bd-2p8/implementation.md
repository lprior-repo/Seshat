bead_id: bd-2p8
bead_title: tests: Implement MUL multi-select tests 2/4
phase: p1
updated_at: 2026-03-01T22:45:00Z

# Implementation: MUL Multi-Select Tests 2/4

## Summary

Created new e2e test file `diagram_tool/e2e/diagram.multi-select-resize.spec.ts` containing 5 tests for multi-select resize behavior.

## Files Created

### diagram_tool/e2e/diagram.multi-select-resize.spec.ts

New test file with the following tests:

1. **MUL-006a**: `resize from NW corner handle resizes selection @baseline`
   - Tests NW corner resize on multi-select
   - Verifies selection shrinks from NW corner

2. **MUL-006b**: `resize from NE corner handle resizes selection @baseline`
   - Tests NE corner resize on multi-select
   - Verifies selection shrinks from NE corner

3. **MUL-006c**: `resize from SE corner handle resizes selection @baseline`
   - Tests SE corner resize on multi-select
   - Verifies selection expands from SE corner

4. **MUL-006d**: `resize from SW corner handle resizes selection @baseline`
   - Tests SW corner resize on multi-select
   - Verifies selection shrinks from SW corner

5. **MUL-007**: `resize maintains node positions within selection @baseline`
   - Tests that relative node positions are maintained during resize

6. **MUL-008**: `resize clamps to minimum size without errors @baseline`
   - Tests that aggressive shrinking clamps to minimum without errors

7. **MUL-009**: `resize expands selection bounds correctly @baseline`
   - Tests that selection expands correctly when dragging outward

8. **MUL-010**: `resize with text nodes works without errors @baseline`
   - Tests resize behavior specifically with text nodes

## Helper Functions

The implementation includes internal helper functions:

- `getSelectionBounds(canvas)`: Gets the selection bounding box
- `getResizeHandle(canvas, corner)`: Gets a specific resize handle's bounding box
- `dragHandle(page, handleBox, dx, dy, steps)`: Performs a drag on a resize handle
- `selectMultipleNodes(page, canvas, count)`: Selects multiple nodes with Shift-click

## Patterns Used

- Follows existing test patterns from `diagram.nodes-and-selection.spec.ts`
- Uses `@baseline` tag for test selection
- Uses `trapPageErrors` for error detection
- Uses `freshStart`, `createTextNode`, `runEffect`, `runEffectsSequential`
- Properly awaits all async operations

## Notes

- Side handles (N, E, S, W) are not currently rendered in the UI, so tests use corner handles only
- Aspect lock (Shift) and rotation features are not yet implemented in the UI
- Tests are designed to be deterministic and not flaky
