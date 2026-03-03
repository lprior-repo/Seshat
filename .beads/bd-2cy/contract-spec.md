# Contract Specification: Multi-Select Tests (MUL-001 to MUL-037)

**Bead ID**: bd-2cy
**Title**: multi-select: Implement multi-selection tests
**Category**: Functional Testing - Multi-Select Operations

## Overview

This contract specifies the expected behavior for multi-selection operations in the Seshat diagram tool. Multi-select allows users to select and manipulate multiple diagram elements simultaneously.

## Test Categories

The multi-select functionality is tested across 37 test cases (MUL-001 to MUL-037) organized into these categories:

### 1. Marquee Selection (MUL-001 to MUL-005)
- Drag-selection rectangle (marquee) selects multiple nodes
- Right-to-left marquee uses intersection mode
- Left-to-right marquee uses containment mode
- Marquee respects z-order and container boundaries

### 2. Shift-Click Selection (MUL-006 to MUL-010)
- Shift-click adds to selection
- Shift-click on selected item removes it
- Shift-click works across containers
- Preserves existing selection state

### 3. Select All (MUL-011 to MUL-015)
- Ctrl/Cmd+A selects all nodes in current container
- Select all respects locked state
- Nested containers handled correctly
- Deselect on empty canvas

### 4. Deselection (MUL-016 to MUL-020)
- Click on empty space deselects all
- Escape key deselects all
- Single click replaces selection
- Selection state persistence

### 5. Selection Bounds (MUL-021 to MUL-025)
- Selection bounds calculated correctly
- Bounds update on move/resize
- Bounds visible for multi-select
- Bounds handle positioning
- Bounds minimum size

### 6. Selection Handles (MUL-026 to MUL-030)
- Corner handles (NW, NE, SE, SW)
- Edge handles (N, E, S, W)
- Handle visibility conditions
- Handle hit testing
- Handle cursor feedback

### 7. Selection Operations (MUL-031 to MUL-035)
- **Move**: All selected items move together preserving relative positions
- **Resize**: Selection bounds resize, items scale proportionally
- **Delete**: All selected items deleted together
- **Copy/Paste**: Multi-item clipboard operations
- **Undo/Redo**: Selection state in history

### 8. Selection Constraints (MUL-036 to MUL-037)
- Locked items cannot be selected
- Parent-child selection rules
- Minimum selection size (1 item)
- Maximum selection limits

## Current Test Coverage

### Implemented Tests (18 total)

#### diagram.multi-select.spec.ts (10 tests)
1. **MUL-001**: Drag 3 selected nodes preserves relative spacing
2. **MUL-002**: Mixed selection drag (nodes at different positions)
3. **MUL-003**: Drag across container boundary reparents
4. **MUL-004**: One locked item stays put during multi-select drag
5. **MUL-005**: Grid snapping with multi-select preserves alignment
6. **MUL-011**: Resize 2-point line endpoints
7. **MUL-012**: Resize curved arrow (edge updates when node moves)
8. **MUL-013**: Resize past minimum clamps
9. **MUL-014**: Resize past inversion flips or clamps
10. **MUL-015**: Resize container+children

#### diagram.multi-select-resize.spec.ts (8 tests)
1. **MUL-006**: Resize from NW corner handle
2. **MUL-006**: Resize from NE corner handle
3. **MUL-006**: Resize from SE corner handle
4. **MUL-006**: Resize from SW corner handle
5. **MUL-007**: Multi-select resize maintains relative positions
6. **MUL-008**: Resize clamps to minimum size
7. **MUL-009**: Resize expands selection bounds
8. **MUL-010**: Resize with text nodes

### Missing Tests (19 tests)

The following test categories need implementation:
- Marquee selection modes (left-to-right vs right-to-left)
- Shift-click selection edge cases
- Select all operations (Ctrl/Cmd+A)
- Deselection behaviors (Escape key, empty space click)
- Selection bounds calculation and display
- Selection handle visibility and interaction
- Delete multi-selection
- Copy/paste multi-selection
- Undo/redo with multi-selection
- Locked item selection constraints

## Precondition Violations

The implementation must handle these error conditions gracefully:
- **P1**: Selection cannot exceed maximum items
- **P2**: Selection cannot be empty for move/resize/delete operations
- **P3**: Locked items cannot be moved/resized
- **P4**: Selection bounds must have minimum size (e.g., 10x10 pixels)
- **P5**: Resize cannot invert selection (negative width/height)

## Postcondition Guarantees

After any multi-select operation:
- **G1**: Relative positions of selected items are preserved (within 2px tolerance)
- **G2**: Selection bounds accurately enclose all selected items
- **G3**: Locked items maintain their position during move operations
- **G4**: Container relationships are preserved during move/resize
- **G5**: No duplicate items in selection
- **G6**: Selection state is visible to user (bounds/handles displayed)

## Invariants

- **I1**: Selection count ≥ 0 and ≤ maximum allowed
- **I2**: All selected items must be valid nodes in the document
- **I3**: Selection bounds = bounding box of all selected items
- **I4**: Locked items are never moved/resized
- **I5**: Parent-child relationships are preserved

## Error Handling

All multi-select operations must:
- Return Result types (no panics)
- Provide clear error messages
- Validate preconditions before execution
- Handle edge cases (empty selection, single item, all items)
- Never corrupt document state

## Test Execution Requirements

All tests must:
- Use `@baseline` tag for smoke test suite
- Execute in < 45 seconds per test
- Have zero page errors (console, network, unhandled rejections)
- Use deterministic waits (no arbitrary timeouts)
- Clean up state between tests (freshStart)
- Trap and verify zero page errors

## Quality Gates

Before this bead can be marked complete:
- [ ] All 18 existing tests pass
- [ ] Zero unwrap/panic in production code
- [ ] All tests tagged @baseline
- [ ] Test execution time < 45s per test
- [ ] Zero page errors in all tests
- [ ] Martin Fowler test patterns documented
- [ ] Verification artifacts created

## References

- Test harness: `diagram_tool/src/test_harness.rs` (TestCategory::Mul)
- Test files:
  - `diagram_tool/e2e/diagram.multi-select.spec.ts` (782 lines, 10 tests)
  - `diagram_tool/e2e/diagram.multi-select-resize.spec.ts` (350 lines, 8 tests)
- Playwright config: `playwright.config.ts`
