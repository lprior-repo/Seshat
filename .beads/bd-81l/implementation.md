bead_id: bd-81l
bead_title: tests: Implement MUL multi-select tests 3/4
phase: p1
updated_at: 2026-03-01T23:00:00Z

# Implementation: MUL Multi-Select Tests 3/4

## Changes Made

### File Modified
- `diagram_tool/e2e/diagram.multi-select.spec.ts`

### Imports Added
```typescript
import {
  // ... existing imports ...
  edgeCount,
  expectEdgeCount,
  nodeCenters,
  nodeFrameByLabel,
  selectedCount,
  waitForUiReady,
} from "./helpers";
```

### New Test Suite: "MUL multi-select resize behavior"

Added a new `test.describe` block containing 5 tests:

#### MUL-011: Edge endpoints update when connected nodes are resized
- Creates two nodes connected by an edge
- Selects first node and resizes via east handle
- Verifies node width increases
- Verifies edge count remains at 1

#### MUL-012: Edge routing updates when node position changes
- Creates two nodes connected by an edge
- Selects first node and drags to new position
- Calculates distance change between node centers
- Verifies edge distance changed significantly
- Verifies edge count remains at 1

#### MUL-013: Resize clamps to minimum dimensions
- Creates a single node
- Attempts to resize smaller than minimum (24px) by dragging east handle left
- Verifies width is clamped to >= 24px
- Verifies dimensions are finite

#### MUL-014: Resize past opposite edge clamps without inversion
- Creates a single node
- Drags west handle far past the east edge (would cause inversion)
- Verifies all dimensions remain positive and finite
- Verifies no NaN or Infinity values

#### MUL-015: Subgraph resize scales children proportionally
- Creates two nodes
- Creates a subgraph containing the nodes
- Selects all and resizes via SE handle
- Verifies subgraph dimensions increased
- Verifies relative position of child within parent is preserved (within 25% tolerance)

### Helper Functions Added

```typescript
async function getResizeHandleCenter(canvasLocator: Locator, handle: string): Promise<{ x: number; y: number }>
```
- Gets the center point of a resize handle by test ID

```typescript
async function createEdgeBetweenNodes(page: Page, canvasLocator: Locator): Promise<void>
```
- Creates an edge between the first two nodes using the Edge tool

```typescript
async function createSubgraphWithNodes(page: Page, canvasLocator: Locator): Promise<void>
```
- Creates a subgraph that encompasses existing nodes

## Test Patterns Used

1. **freshStart()** for test isolation
2. **trapPageErrors(page)** for error tracking
3. **runEffect()** and **runEffectsSequential()** for deterministic async operations
4. **expectNodeCount()**, **expectEdgeCount()**, **expectSelectedCount()** for assertions
5. **@baseline** tags for stable tests

## Verification

- TypeScript compilation passes: `npx tsc --noEmit --project diagram_tool/e2e/tsconfig.json`
- Tests follow existing patterns in the codebase
- All tests use helper functions from helpers.ts
