bead_id: bd-2p8
bead_title: tests: Implement MUL multi-select tests 2/4
phase: p0
updated_at: 2026-03-01T22:30:00Z

# Contract: MUL Multi-Select Tests 2/4

## Overview

Implement 5 multi-select resize tests for the diagram tool e2e test suite. These tests verify resize behavior when multiple items are selected.

## Test Cases

### MUL-006: Resize from NW/NE/SE/SW corners
- **Given**: 2 nodes selected together
- **When**: User drags corner resize handles (NW, NE, SE, SW)
- **Then**: Selection bounding box resizes from the dragged corner

### MUL-007: Multi-select resize maintains relative positions
- **Given**: 2 nodes selected together at different positions
- **When**: User drags SE resize handle to make selection larger
- **Then**: Both nodes remain within the resized selection bounds

### MUL-008: Resize clamps to minimum size
- **Given**: 2 nodes selected together
- **When**: User attempts to resize selection below minimum size
- **Then**: Selection clamps to minimum dimensions without errors

### MUL-009: Resize expands selection bounds
- **Given**: 2 nodes selected together
- **When**: User drags SE handle to expand selection
- **Then**: Selection bounds expand and both nodes remain selected

### MUL-010: Resize with text nodes
- **Given**: 2 text nodes selected together
- **When**: User resizes the selection via corner handle
- **Then**: Selection bounding box resizes correctly, no page errors

## Preconditions

- Test file follows existing e2e test patterns
- Uses helpers from `diagram_tool/e2e/helpers.ts`
- Tests use `freshStart`, `createTextNode`, `runEffect`, `runEffectsSequential`
- Available resize handles: `resize-handle-nw`, `resize-handle-ne`, `resize-handle-se`, `resize-handle-sw`

## Postconditions

- All 5 tests pass with `moon run :test`
- Tests follow naming convention: `test("MUL-NNN: description @baseline", ...)`
- No page errors during test execution
- Tests are deterministic and not flaky

## Invariants

- Tests do not use `waitForTimeout` for synchronization (except minimal waits for UI state)
- Tests use `trapPageErrors` to verify no console errors
- Selection count assertions use `expectSelectedCount`
- Node count assertions use `expectNodeCount`

## Implementation Notes

- Tests should be added to a new file: `diagram_tool/e2e/diagram.multi-select-resize.spec.ts`
- Follow the patterns from `diagram.nodes-and-selection.spec.ts` and `diagram.transform-invariants.spec.ts`
- Note: Aspect lock (Shift) and rotation features are not yet implemented in the UI
