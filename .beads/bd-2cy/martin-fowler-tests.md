# Martin Fowler Test Patterns: Multi-Select Tests

**Bead ID**: bd-2cy
**Test Category**: Multi-Select Transform (MUL)
**Test Count**: 18 implemented, 19 missing (target: 37)

## Test Smell Analysis

### Current Test Characteristics

#### Strengths
1. **Deterministic State Management**: All tests use `freshStart()` and `clearCanvasOverlays()` for clean state
2. **Explicit Assertions**: Clear expected vs actual comparisons with tolerance values
3. **Error Trapping**: `trapPageErrors()` ensures zero console errors
4. **Descriptive Names**: Test names clearly indicate the behavior being tested
5. **Baseline Tagging**: All tests use `@baseline` for smoke test inclusion

#### Areas for Improvement
1. **Magic Numbers**: Hard-coded coordinates (e.g., `400, 200`) without named constants
2. **Test Duplication**: MUL-006 has 4 variants for corners that could be parameterized
3. **Incomplete Coverage**: Only 18/37 tests implemented (48.6% coverage)
4. **Missing Edge Cases**: No tests for maximum selection limits, rapid operations
5. **No Property Tests**: Geometry calculations not tested with property-based approach

## Test Patterns Applied

### 1. State Verification Pattern
**Usage**: Verify state after operations

```typescript
// Before state
const initialBoxes = await nodeBoxes(diagramCanvas);

// Perform operation
await dragMouse(page, dragStart, dragEnd);

// After state verification
const finalBoxes = await nodeBoxes(diagramCanvas);
expect(finalBoxes).toHaveLength(3);
```

**Rating**: Good - Clear before/after state comparison

### 2. Invariant Preservation Pattern
**Usage**: Verify invariants hold during operations

```typescript
// Verify relative distances preserved (invariant)
const initialGap01 = { dx: ..., dy: ... };
const finalGap01 = { dx: ..., dy: ... };
expect(Math.abs(finalGap01.dx - initialGap01.dx)).toBeLessThan(2);
```

**Rating**: Excellent - Tests critical multi-select invariant

### 3. Boundary Value Pattern
**Usage**: Test minimum/maximum constraints

```typescript
// MUL-008: Resize clamps to minimum size
// MUL-013: Resize past minimum clamps
// MUL-014: Resize past inversion flips or clamps
```

**Rating**: Good - Covers boundary conditions

### 4. Fixture Setup Pattern
**Usage**: Reusable test data creation

```typescript
async function selectMultipleNodes(page: Page, canvas: Locator, count: number) {
  const nodes = canvas.getByTestId("node");
  await runEffect(() => nodes.first().click());
  for (let i = 1; i < count; i++) {
    await runEffectsSequential([
      () => page.keyboard.down("Shift"),
      () => nodes.nth(i).click(),
      () => page.keyboard.up("Shift"),
    ]);
  }
}
```

**Rating**: Good - Reduces duplication, clear intent

## Missing Test Patterns

### 1. Parameterized Tests
**Opportunity**: MUL-006 corner resize tests

```typescript
// Current: 4 separate tests for NW, NE, SE, SW
// Better: Parameterized test
const corners = ["nw", "ne", "se", "sw"] as const;
for (const corner of corners) {
  test(`MUL-006: resize from ${corner.toUpperCase()} corner`, async ({ page }) => {
    // Single implementation for all corners
  });
}
```

### 2. Property-Based Tests
**Opportunity**: Geometry invariants

```typescript
// Property: Relative positions always preserved
test("relative positions invariant", async ({ page }) => {
  // For random selections, verify relative positions preserved
  // Use fuzzing for drag distances and directions
});
```

### 3. State Transition Tests
**Opportunity**: Selection lifecycle

```typescript
test("selection state transitions", async ({ page }) => {
  // empty -> single -> multi -> empty
  // Verify each transition is valid
});
```

### 4. Performance Tests
**Opportunity**: Large selection handling

```typescript
test("MUL-037: handle maximum selection size", async ({ page }) => {
  // Select 100+ nodes, verify performance acceptable
  // Verify operations remain responsive
});
```

## Test Smells Detected

### 1. Mystery Guest (Minor)
**Issue**: Test helpers imported from `./helpers` without documentation

**Impact**: Harder to understand test dependencies

**Fix**: Document each imported helper's purpose

### 2. Duplicate Test Code (Minor)
**Issue**: MUL-006 corner tests have identical structure

**Impact**: Maintenance burden, code duplication

**Fix**: Extract to parameterized test or shared utility

### 3. Assertion Roulette (Minor)
**Issue**: Multiple assertions without descriptive messages

```typescript
// Current
expect(Math.abs(finalGap01.dx - initialGap01.dx)).toBeLessThan(2);

// Better
expect(Math.abs(finalGap01.dx - initialGap01.dx))
  .toBeLessThan(2, "relative x-spacing should be preserved within 2px");
```

**Fix**: Add assertion messages for failure clarity

## Test Coverage Gaps

### Critical Missing Tests (Priority 1)

1. **Marquee Selection Modes**
   - Left-to-right (containment) vs right-to-left (intersection)
   - Marquee with partial node overlap
   - Marquee crossing container boundaries

2. **Select All Operations**
   - Ctrl/Cmd+A selects all in current scope
   - Nested container handling
   - Deselect behavior

3. **Selection Bounds**
   - Bounds calculation accuracy
   - Bounds update timing
   - Bounds visibility conditions

4. **Selection Handles**
   - Handle visibility rules
   - Handle hit testing accuracy
   - Handle cursor feedback

### Important Missing Tests (Priority 2)

5. **Deselection Behaviors**
   - Click empty space deselects all
   - Escape key deselects all
   - Single click replaces selection

6. **Multi-Item Operations**
   - Delete multiple items
   - Copy/paste multiple items
   - Undo/redo multi-select state

### Nice-to-Have Tests (Priority 3)

7. **Performance**
   - Large selection handling
   - Rapid selection operations
   - Selection with many edges

8. **Accessibility**
   - Keyboard-only multi-selection
   - Screen reader announcements
   - Focus management

## Recommendations

### Immediate Actions (For This Bead)

1. **Run Existing Tests**: Verify all 18 tests pass
2. **Zero Panics Check**: Ensure no unwrap/panic in multi-select code
3. **Document Current Coverage**: Create receipt showing 18/37 tests passing

### Future Improvements (Separate Beads)

1. **Implement Missing Tests**: Prioritize critical gaps (marquee, select all, bounds)
2. **Refactor Duplication**: Extract parameterized tests for corner resize
3. **Add Property Tests**: Use property-based testing for geometry invariants
4. **Performance Tests**: Add tests for large selections
5. **Accessibility Tests**: Add keyboard/screen reader tests

## Test Metrics

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Test Count | 18 | 37 | 48.6% |
| @Baseline Tests | 18 | 18 | 100% |
| Average Test Length | ~40 lines | ~30 lines | Needs improvement |
| Test Helper Usage | Good | Good | ✓ |
| Error Trapping | 100% | 100% | ✓ |
| State Cleanup | 100% | 100% | ✓ |
| Property Tests | 0 | 5+ | Missing |
| Parameterized Tests | 0 | 3+ | Missing |

## Conclusion

The existing multi-select tests demonstrate good practices:
- Clean state management
- Clear assertions
- Error trapping
- Baseline tagging

However, coverage is incomplete (18/37 tests). The immediate priority is to verify existing tests pass and zero panics exist. Future work should focus on implementing missing tests and refactoring for maintainability.

**Recommendation**: APPROVE existing tests, but create follow-up beads for:
1. bd-2cz: Implement marquee selection tests (MUL-016 to MUL-020)
2. bd-2d0: Implement select all tests (MUL-021 to MUL-025)
3. bd-2d1: Implement selection bounds/handles tests (MUL-026 to MUL-030)
4. bd-2d2: Implement multi-item operations tests (MUL-031 to MUL-037)
