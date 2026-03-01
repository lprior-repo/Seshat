bead_id: bd-2dg
bead_title: tests: Implement SEL selection tests 1/5
phase: p1
updated_at: 2026-03-01T21:30:00Z

# Contract: Selection Tests (bd-2dg)

## Summary
Add 5 selection tests covering core selection behaviors in the diagram tool.

## Acceptance Criteria

### Test 1: Click node selects @baseline
- GIVEN a canvas with at least one node
- WHEN the user clicks on a node
- THEN the node becomes selected (selected count = 1)

### Test 2: Click empty clears selection @baseline
- GIVEN a canvas with at least one selected node
- WHEN the user clicks on an empty area of the canvas (with select tool active)
- THEN the selection is cleared (selected count = 0)

### Test 3: Shift-click adds to selection @baseline
- GIVEN a canvas with multiple nodes and one already selected
- WHEN the user shift-clicks on an unselected node
- THEN that node is added to the selection (selected count = 2)

### Test 4: Marquee select contains nodes @baseline
- GIVEN a canvas with multiple nodes
- WHEN the user drags a marquee rectangle right-to-left (intersect mode)
- THEN nodes intersecting the marquee become selected

### Test 5: Marquee direction switches selection mode @baseline
- GIVEN a canvas with nodes
- WHEN the user drags left-to-right (contain mode) vs right-to-left (intersect mode)
- THEN the selection behavior differs based on drag direction

## Reference Patterns
- File: `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- Use helpers: `freshStart`, `clearCanvasOverlays`, `createTextNode`, `expectSelectedCount`, `runEffect`, `runEffectsSequential`, `canvas`
- Follow existing test naming and structure conventions
- Use `@baseline` tag for stability-critical tests

## Technical Requirements
- Must use Playwright test framework
- Must use existing helper functions from `./helpers`
- Must trap page errors where appropriate
- Tests must be deterministic (no flaky behavior)
- Tests should be in the same test.describe block as existing selection tests

## Preconditions
- E2E test infrastructure is functional
- `freshStart` helper provides clean state
- Selection counter (`data-testid="counter-selected"`) is available

## Postconditions
- All 5 new tests pass
- No regression in existing tests
- `moon run :test` passes
