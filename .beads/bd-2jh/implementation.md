bead_id: bd-2jh
bead_title: tests: Implement CAM viewport tests 3/3
phase: p1
updated_at: 2026-03-02T01:15:00Z

# Implementation: CAM Viewport Tests 3/3

## Summary

Added 5 viewport tests to `/home/lewis/src/seshat/diagram_tool/e2e/diagram.viewport-cam.spec.ts`:

### Test 1: canvas embedded in scrollable parent handles coordinate offset (line 398)

**Behavior tested:** World-to-screen transforms account for scroll offset when canvas is embedded in a scrollable parent.

**Implementation:**
- Mounts scrollable harness via `mountScrollableHarness()`
- Scrolls to position (350, 280)
- Creates node at position (300, 200)
- Verifies node position is within canvas bounds
- Performs wheel zoom at node center
- Verifies zoom worked and node remains under cursor (within 30px tolerance)

### Test 2: viewport recalculates after DPR change (line 451)

**Behavior tested:** Zoom and coordinates remain consistent after devicePixelRatio change simulation.

**Implementation:**
- Creates initial node at (400, 280)
- Records initial zoom (expecting 100%)
- Dispatches custom `dprchange` event and `resize` event to simulate DPR change
- Waits for stabilization
- Verifies zoom remains ~100% (95-105% tolerance)
- Verifies node is still selectable
- Performs wheel zoom and verifies it still works

### Test 3: context menu focus loss mid-drag does not corrupt selection (line 504)

**Behavior tested:** Selection state remains intact after context menu opens during drag.

**Implementation:**
- Creates and selects a node
- Starts drag operation (mouse down + partial move)
- Right-clicks mid-drag to trigger context menu
- Presses Escape to dismiss
- Completes drag (mouse up)
- Verifies selection count >= 1 (not corrupted)
- Verifies node is still interactive

### Test 4: auto-save preserves camera position without stutter (line 562)

**Behavior tested:** Camera position does not jump or reset during auto-save cycle.

**Implementation:**
- Creates node at (400, 280)
- Pans viewport using spacebar + drag
- Records node position after pan (represents camera position)
- Dispatches `seshat-autosave` custom event and `storage` event
- Waits for any save-related processing
- Verifies node position is stable (within 5px tolerance)
- Performs zoom, triggers another save
- Verifies zoom level is preserved

### Test 5: pan inertia decays smoothly to stop (line 648)

**Behavior tested:** Camera gradually decelerates to a stop (if inertia implemented) or stops cleanly on mouse up.

**Implementation:**
- Creates node at (400, 280)
- Performs quick pan with spacebar + drag (150px movement)
- Records position immediately after pan ends
- Waits 200ms for any inertia to settle
- Records final position
- Waits another 100ms and verifies stability (within 3px)
- Verifies actual pan occurred (delta > 30px)
- Verifies interaction state is clean (node is selectable)

## Patterns Followed

All tests follow existing patterns in the file:
- Use `@baseline` tag convention
- Use `freshStart()` for isolation
- Use `trapPageErrors(page)` and assert `pageErrors` is empty
- Use `runEffect()` and `runEffectsSequential()` for async operations
- Use `waitForNoRebuildOverlay(page)` after operations that may trigger rebuilds
- Throw descriptive errors when bounds are unavailable
- Use existing helper functions: `createTextNode`, `canvasBox`, `zoomPercent`, `selectedCount`, `mountScrollableHarness`, `scrollHarnessTo`

## Files Modified

- `/home/lewis/src/seshat/diagram_tool/e2e/diagram.viewport-cam.spec.ts` - Added 5 new tests (lines 398-718)
