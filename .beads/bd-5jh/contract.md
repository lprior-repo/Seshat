bead_id: bd-5jh
bead_title: tests: Implement SUB subgraph tests 3/4
phase: p0
updated_at: 2026-03-01T17:00:00Z

# Contract: SUB Subgraph Tests 3/4

## Summary

Implement 4 subgraph (container) behavior tests for the Seshat diagram tool:
1. Container auto-expand when child crosses boundary
2. Container resize behavior (children keep size vs scale)
3. Container overflow handling
4. Container padding alignment

## Context

This bead is part of the SUB (subgraph) test series. The existing codebase has:
- `diagram_tool/e2e/diagram.subgraph-resize.spec.ts` - proportional resize tests
- `diagram_tool/e2e/diagram.subgraph-save-reload.spec.ts` - save/reload stability tests

## Preconditions

- Playwright test infrastructure exists
- Helper functions available: `freshStart`, `clearCanvasOverlays`, `createTextNode`, `nodeFrameByLabel`, `runEffect`, `runEffectsSequential`, `trapPageErrors`, `waitForUiReady`, `nodeCount`, `selectedCount`
- Subgraph creation tool available via "Subgraph" button

## Test Specifications

### SUB-011: Container auto-expand when child crosses boundary

**Given**: A subgraph container with a child text node inside
**When**: The child node is dragged toward and past the container boundary
**Then**:
- The container auto-expands to contain the child (if auto-expand is implemented)
- OR the child is clipped at the boundary (if clipping is implemented)
- No errors occur
- Relative positioning remains valid

**Acceptance Criteria**:
- Test creates subgraph with child node
- Drags child toward/ past boundary
- Verifies system handles boundary crossing gracefully
- No page errors

### SUB-012: Container resize behavior (children keep size vs scale)

**Given**: A subgraph container with child nodes inside
**When**: The container is resized (not via proportional select-all resize)
**Then**:
- Child nodes maintain their absolute size (do not scale)
- Child relative positions within container adjust appropriately
- Layout remains stable

**Acceptance Criteria**:
- Test creates subgraph with multiple child nodes
- Resizes just the container (not via select-all)
- Verifies children keep their size
- Verifies children positions adjust appropriately

### SUB-013: Container overflow handling

**Given**: A subgraph container with children that exceed its bounds
**When**: The container is made smaller than needed to contain all children
**Then**:
- Children remain visible (overflow visible) OR are clipped consistently
- No rendering artifacts
- No console errors
- Layout remains valid

**Acceptance Criteria**:
- Test creates subgraph with children
- Shrinks container smaller than children bounds
- Verifies overflow behavior is handled gracefully
- No page errors

### SUB-014: Container padding alignment

**Given**: A subgraph container with child nodes
**When**: Children are positioned within the container
**Then**:
- Children maintain proper padding from container edges
- Alignment is consistent when container is resized
- Minimum padding constraints are respected

**Acceptance Criteria**:
- Test creates subgraph with child nodes at specific positions
- Verifies padding/alignment relationships
- Tests resizing and verifies padding is maintained
- No page errors

## Technical Requirements

1. Create new test file: `diagram_tool/e2e/diagram.subgraph-container-behavior.spec.ts`
2. Use existing helper functions from `helpers.ts`
3. Follow existing test patterns from `diagram.subgraph-resize.spec.ts`
4. Each test should:
   - Call `trapPageErrors(page)` and verify `pageErrors` is empty at end
   - Use `freshStart(page)` for clean state
   - Use `clearCanvasOverlays(page)` to dismiss panels
   - Use `runEffectsSequential` for multi-step operations

## Postconditions

- All 4 tests pass
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

- Edge binding tests (covered by EDG series)
- Save/reload tests (already covered)
- Selection tests (covered by SEL series)
