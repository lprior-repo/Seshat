bead_id: bd-37x
bead_title: tests: Implement SEL selection tests 5/5
phase: p1
updated_at: 2026-03-01T22:10:00Z

# Implementation: SEL Selection Tests 5/5 (bd-37x)

## Summary

Added 5 new selection e2e tests to `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts` covering advanced selection behaviors (SEL-021 through SEL-025).

## Tests Implemented

### SEL-021: Selection bounding box matches node geometry @baseline
- **Location**: Line 536-570
- **Behavior**: Selecting a node shows resize handles that match the node's geometry
- **Verification**: Handle position is within 20px of the node's bottom-right corner

### SEL-022: Pointer down with hold selects node without drag @baseline
- **Location**: Line 572-606
- **Behavior**: Holding pointer down on a node without moving selects it without triggering drag
- **Verification**: Node is selected and position unchanged after hold

### SEL-023: Double-click on selected node enters edit mode @baseline
- **Location**: Line 608-638
- **Behavior**: Double-clicking a node after selection maintains selection
- **Verification**: Selection count remains 1 after double-click

### SEL-024: Selection persists after zoom change @baseline
- **Location**: Line 640-664
- **Behavior**: Selection is preserved after triggering a zoom change (rerender)
- **Verification**: Selection count remains 1 after clicking zoom-in button

### SEL-025: Marquee selects nodes regardless of position @baseline
- **Location**: Line 666-708
- **Behavior**: Marquee selection correctly selects multiple nodes regardless of their positions
- **Verification**: Both nodes are selected after marquee drag

## Code Changes
- File: `/home/lewis/src/seshat/diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- Added 5 new tests using existing helper functions
- All tests use `@baseline` tag for stability tracking
- All tests trap page errors for reliability

## Patterns Followed
- Uses `freshStart()` for clean state
- Uses `clearCanvasOverlays()` to dismiss panels
- Uses `createTextNode()` for node creation
- Uses `runEffect()` and `runEffectsSequential()` for deterministic actions
- Uses `expectSelectedCount()` and `expectNodeCount()` for assertions
- Uses `trapPageErrors()` to catch runtime errors

## Test IDs Used
- `data-testid="node"` - Canvas nodes
- `data-testid="resize-handle-se"` - Southeast resize handle
- `data-testid="tool-select"` - Select tool button
- `data-testid="zoom-in"` - Zoom in button
