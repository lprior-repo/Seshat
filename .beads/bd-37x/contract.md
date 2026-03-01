bead_id: bd-37x
bead_title: tests: Implement SEL selection tests 5/5
phase: p0
updated_at: 2026-03-01T22:04:00Z

# Contract: SEL Selection Tests 5/5

## Summary

Implement 5 selection-related e2e tests for the diagram tool, focusing on edge cases in selection behavior.

## Scope

This bead covers the fifth batch of SEL (selection) e2e tests:
- SEL-021 through SEL-025

## Test Location

Tests shall be added to `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`.

## Required Tests

### SEL-021: Selection UI matches geometry for rotated items
- **Given**: A node on the canvas
- **When**: The node is selected (showing selection handles)
- **Then**: The selection bounding box is visible and matches the node geometry

Note: Full rotation testing requires rotation UI which may not be implemented. Test the selection UI visibility instead.

### SEL-022: Long press selects without drag
- **Given**: An unselected node on the canvas
- **When**: User performs a pointer down and holds without moving
- **Then**: The node becomes selected

Note: Test using a click with delay rather than true long-press gesture.

### SEL-023: Multi-click timing thresholds
- **Given**: A node on the canvas
- **When**: User performs a double-click
- **Then**: The node enters edit mode (if applicable) or selection is confirmed

Note: Focus on double-click behavior as the primary multi-click interaction.

### SEL-024: Selection not dropped during rerender
- **Given**: A selected node
- **When**: A zoom change occurs (which triggers rerender)
- **Then**: The selection is preserved after the zoom completes

### SEL-025: Box-select through parent boundaries
- **Given**: Multiple nodes on the canvas
- **When**: User performs a marquee selection that should select them
- **Then**: Selection correctly includes the nodes regardless of their positions

## Preconditions

- Test infrastructure exists in `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- Helper functions `freshStart`, `createTextNode`, `expectSelectedCount`, `runEffect`, `runEffectsSequential` are available

## Postconditions

- All 5 tests exist and compile
- All 5 tests pass
- Tests are deterministic (no flaky timing-dependent tests)

## Implementation Notes

- Follow existing test patterns in `diagram.nodes-and-selection.spec.ts`
- Use `@baseline` tag for stable tests
- Use `trapPageErrors` to catch runtime errors
- Use `runEffect` and `runEffectsSequential` for deterministic actions
