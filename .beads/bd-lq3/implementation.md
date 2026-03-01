---
bead_id: bd-lq3
bead_title: tests: Implement SUB subgraph tests 2/4
phase: p1
updated_at: 2026-03-01T22:45:00Z
---

# Implementation: SUB Subgraph Tests 2/4

## Summary
Implemented 5 subgraph behavior tests in `/home/lewis/src/seshat/diagram_tool/e2e/diagram.subgraph-behavior.spec.ts`.

## Tests Implemented

### SUB-006: delete container handles children gracefully @baseline
- **File**: `diagram_tool/e2e/diagram.subgraph-behavior.spec.ts:110`
- **Description**: Tests that when a subgraph container is deleted, the system handles child nodes gracefully (either reparents to root or deletes consistently).
- **Verification**: Checks that no page errors occur and all remaining nodes have valid dimensions.

### SUB-007: duplicate container produces valid copies @baseline
- **File**: `diagram_tool/e2e/diagram.subgraph-behavior.spec.ts:153`
- **Description**: Tests that duplicating a container with children via copy-paste produces valid copies with remapped IDs.
- **Verification**: Node count increases, all nodes have valid dimensions, no page errors (ID conflicts would cause errors).

### SUB-008: drag node into container area produces valid state @baseline
- **File**: `diagram_tool/e2e/diagram.subgraph-behavior.spec.ts:190`
- **Description**: Tests dragging a root-level node into a container bounds.
- **Verification**: Node count remains consistent, node and container have valid dimensions after drag.

### SUB-009: drag child out of container produces valid state @baseline
- **File**: `diagram_tool/e2e/diagram.subgraph-behavior.spec.ts:251`
- **Description**: Tests dragging a child node outside of its container bounds.
- **Verification**: Node count remains 3, all nodes have valid dimensions.

### SUB-010: drag node between overlapping containers produces valid state @baseline
- **File**: `diagram_tool/e2e/diagram.subgraph-behavior.spec.ts:296`
- **Description**: Tests dragging a node between two overlapping containers.
- **Verification**: Node count remains 3, all nodes have valid dimensions.

## Helper Functions Created

### `setupSubgraphWithNodes(page: Page): Promise<Locator>`
Creates a fresh canvas with 2 text nodes inside a subgraph container. Used by multiple tests.

### `createSubgraphContainer(page, canvasLocator, startX, startY, endX, endY)`
Creates a subgraph container by dragging a rectangle on the canvas.

### `nodeBoxesSorted(canvasLocator: Locator): Promise<Box[]>`
Returns all node bounding boxes sorted by x position.

### `dragMouse(page, from, to)`
Performs a mouse drag operation from one point to another.

## Test Patterns Used
- All tests use `freshStart()` for clean state
- All tests use `trapPageErrors()` to verify no console errors
- All tests verify valid node dimensions (finite, positive)
- Tests are behavioral rather than prescriptive - they verify valid states rather than specific outcomes

## Notes
- Tests use `@baseline` tag to match the playwright project configuration
- Tests include `waitForNoRebuildOverlay` and `waitForUiReady` calls for stability
- The `setupSubgraphWithNodes` helper reduces code duplication across tests
