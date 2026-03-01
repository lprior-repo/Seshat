bead_id: bd-nbm
bead_title: tests: Implement SEL selection tests 3/5
phase: p1
updated_at: 2026-03-01T22:25:00Z

# Implementation: SEL Selection Tests 3/5

## Summary
Added 5 selection tests to `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`:

### SEL-010: Right-click context menu preserves selection
- Tests that right-clicking on a selected node preserves the selection
- Uses `page.mouse.click(x, y, { button: "right" })` for right-click simulation

### SEL-011: Alt-click selects parent container
- Tests Alt-click behavior for selecting parent containers
- Creates nodes and attempts to group them, then tests Alt-click selection
- Gracefully handles cases where grouping may not be available

### SEL-012: Locked element not selectable
- Tests that locked elements cannot be selected (or have restricted selection)
- Uses properties panel lock toggle or `__seshatSetNodeLocked` hook
- Verifies locked node behavior is different from unlocked nodes

### SEL-013: Hidden element not hit-testable
- Tests that hidden elements (display: none) are not hit-testable
- Creates overlapping nodes and hides one via CSS
- Verifies click passes through to visible node underneath

### SEL-014: Right-click on unselected node selects it first
- Tests that right-clicking on an unselected node may select it first
- Creates two nodes, selects one, then right-clicks the other
- Verifies selection behavior is consistent

## Files Modified
- `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts` - Added 5 new tests (SEL-010 through SEL-014)

## Test Patterns Used
- `freshStart(page)` for clean test state
- `clearCanvasOverlays(page)` to dismiss panels
- `createTextNode(page, diagramCanvas, x, y)` for node creation
- `expectSelectedCount(page, n)` for selection assertions
- `expectNodeCount(page, n)` for node count assertions
- `trapPageErrors(page)` for error tracking
- `runEffect()` and `runEffectsSequential()` for async operations

## Notes
- All tests follow existing patterns in the test file
- All tests have `@baseline` tag
- Tests handle cases where UI features may not be fully implemented
- Tests verify no page errors occur during execution
