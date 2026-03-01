bead_id: bd-nbm
bead_title: tests: Implement SEL selection tests 3/5
phase: p0
updated_at: 2026-03-01T22:15:00Z

# Contract: SEL Selection Tests 3/5

## Overview
Implement 5 selection-related e2e tests for the diagram tool:
1. Right-click context menu preserves selection
2. Alt-click selects parent
3. Locked element not selectable
4. Hidden element not hit-testable

## Preconditions
- `diagram_tool/e2e/helpers.ts` exists with test utilities
- `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts` contains existing selection tests
- Playwright test infrastructure is functional
- `freshStart`, `createTextNode`, `expectSelectedCount`, `runEffect`, `runEffectsSequential` helpers available

## Required Tests

### SEL-010: Right-click context menu preserves selection
**Given**: A node is selected on the canvas
**When**: User right-clicks on the selected node to open context menu
**Then**: Selection count remains 1 (selection is preserved)

```typescript
test("right-click context menu preserves selection @baseline", async ({ page }) => {
  // Create and select a node
  // Right-click on the selected node
  // Verify selection count is still 1
});
```

### SEL-011: Alt-click selects parent
**Given**: A child node is inside a parent container (group/frame)
**When**: User Alt-clicks on the child node
**Then**: The parent container is selected instead of the child

```typescript
test("alt-click selects parent container @baseline", async ({ page }) => {
  // Create a parent container with a child node
  // Alt-click on the child
  // Verify parent is selected (not child)
});
```

### SEL-012: Locked element not selectable
**Given**: A node is locked (non-interactive)
**When**: User clicks on the locked node
**Then**: The node is not selected (selection count remains 0)

```typescript
test("locked element cannot be selected @baseline", async ({ page }) => {
  // Create a node
  // Lock the node (set locked property)
  // Click on the locked node
  // Verify selection count is 0
});
```

### SEL-013: Hidden element not hit-testable
**Given**: A node is hidden (visible=false)
**When**: User clicks on the area where the hidden node would be
**Then**: The hidden node is not selected (click passes through)

```typescript
test("hidden element is not hit-testable @baseline", async ({ page }) => {
  // Create a node
  // Hide the node (set visible property to false)
  // Click on the area where the node would be
  // Verify selection count is 0 (click passes through)
});
```

### SEL-014: Fifth test (selection preservation during interaction)
**Given**: A node is selected
**When**: User performs a no-op interaction (like clicking empty space then re-clicking)
**Then**: Selection state is correctly maintained

Note: The description mentions 5 tests but only lists 4 specific behaviors. The 5th test will be:
- Right-click on unselected node should select it AND preserve that selection when context menu opens

```typescript
test("right-click on unselected node selects it first @baseline", async ({ page }) => {
  // Create two nodes
  // Right-click on the second node (not selected)
  // Verify second node is now selected (right-click selects first)
});
```

## Postconditions
- All 5 tests exist in `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- All tests follow existing patterns (using `freshStart`, `runEffect`, `runEffectsSequential`)
- All tests have `@baseline` tag
- Tests use `trapPageErrors` for error tracking
- Tests verify both selection count and no page errors

## Invariants
- Existing tests continue to pass
- Moon `:test` target passes
- No TypeScript errors
- Code follows existing patterns in the test file

## Implementation Notes
- Use existing helpers from `helpers.ts`
- Follow the naming convention: `test("description @baseline", async ({ page }) => { ... })`
- Use `diagramCanvas` locator for canvas interactions
- Use `expectSelectedCount` for selection assertions
- Use `trapPageErrors` to catch any console/runtime errors

## Files to Modify
- `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts` - Add 5 new tests

## Verification
- `moon run :quick` passes
- `moon run :test` passes (including new tests)
- `moon run :ci` passes
