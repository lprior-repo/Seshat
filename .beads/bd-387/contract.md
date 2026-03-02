bead_id: bd-387
bead_title: tests: Implement MUL multi-select tests - resize edge cases
phase: p0
updated_at: 2026-03-02T00:39:10Z

# Contract: MUL Multi-Select Resize Edge Cases Tests

## Summary
Implement 5 multi-select resize tests covering edge cases in the seshat diagram editor.

## Requirements

### Test Cases (5 total)

1. **Resize selection containing rotated items**
   - Create a selection with one or more rotated shapes
   - Perform resize operation
   - Verify rotated items maintain their rotation while being resized correctly

2. **Resize selection with text**
   - Create a selection containing text elements
   - Perform resize operation
   - Verify text elements resize appropriately

3. **Resize selection with 2-point line**
   - Create a selection containing a 2-point line
   - Perform resize operation
   - Verify line endpoints scale correctly

4. **Resize selection with curved arrow**
   - Create a selection containing a curved arrow
   - Perform resize operation
   - Verify arrow curve and endpoints scale correctly

5. **Resize selection past inversion**
   - Create a selection and resize past the point of inversion
   - Verify the selection handles inversion correctly (negative scaling)

## Preconditions
- Test infrastructure exists in the seshat codebase
- Multi-select resize functionality is implemented
- Test utilities for creating selections, shapes, and verifying transforms are available

## Postconditions
- 5 new test functions exist in the appropriate test module
- All tests pass with `moon run :test`
- Tests cover the specified edge cases
- Code passes `moon run :quick` linting

## Acceptance Criteria
- [ ] Test for rotated items in selection exists and passes
- [ ] Test for text in selection exists and passes
- [ ] Test for 2-point line in selection exists and passes
- [ ] Test for curved arrow in selection exists and passes
- [ ] Test for inversion during resize exists and passes
- [ ] All tests run in CI without flakiness

## Technical Context
- Location: Likely in `crates/app/src/tests/` or similar test directory
- Framework: Rust test framework with app-specific test utilities
- Dependencies: Existing multi-select and resize implementations
