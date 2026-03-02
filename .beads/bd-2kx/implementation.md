bead_id: bd-2kx
bead_title: tests: Implement CAM viewport tests 2/3
phase: p1
updated_at: 2026-03-02T00:10:00Z

# Implementation: CAM Viewport Tests 2/3

## Summary

Added 2 new viewport tests to `/home/lewis/src/seshat/diagram_tool/e2e/diagram.viewport-cam.spec.ts`:

### Test 1: Edge Scrolling During Drag
**Location:** Line 296
**Name:** `edge scrolling during drag reveals more canvas space @baseline`

This test verifies that when a user drags a node near the canvas edge, the viewport scrolls to reveal more canvas space. The test:
1. Creates a node and selects it
2. Drags the node towards the right edge of the canvas
3. Holds near the edge briefly to allow edge-scrolling to trigger
4. Verifies the node position changed significantly

### Test 2: Fit to Content with Padding
**Location:** Line 343
**Name:** `fit to content centers nodes with appropriate padding @baseline`

This test verifies that the zoom reset functionality fits all content within the viewport with appropriate padding. The test:
1. Creates 3 nodes at various positions
2. Zooms in significantly to change the viewport
3. Clicks the zoom reset button to return to default view
4. Verifies zoom returns to ~100%
5. Verifies all nodes are visible within canvas bounds (with padding tolerance)

## Code Changes

### Modified Files
- `/home/lewis/src/seshat/diagram_tool/e2e/diagram.viewport-cam.spec.ts`

### Test Count
- Before: 8 tests
- After: 10 tests

## Dependencies
- Uses existing helper functions: `freshStart`, `clearCanvasOverlays`, `createTextNode`, `runEffect`, `runEffectsSequential`, `trapPageErrors`, `zoomPercent`, `nodeCount`, `selectedCount`, `waitForNoRebuildOverlay`, `canvasBox`
- No new dependencies added

## Notes
The existing tests already covered 4 of the 5 requirements from the bead description:
- spacebar pan (line 90)
- min zoom clamp (line 131)
- max zoom clamp (line 145)
- world-to-screen at extremes (line 159)

The 2 new tests complete the remaining requirements:
- edge scrolling during drag
- fit to content with padding
