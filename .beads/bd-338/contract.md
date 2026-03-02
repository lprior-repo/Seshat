bead_id: bd-338
bead_title: tests: Implement GEO geometry tests - transforms
phase: p0
updated_at: 2026-03-02T00:39:00Z

# Contract: GEO Geometry Transform Tests

## Overview
Implement comprehensive test coverage for geometry transform operations in the GEO (Geometry Engine) module.

## Acceptance Criteria

### Required Test Cases (5 total)

1. **Scale Around Anchor Point (NW/NE/SE/SW)**
   - Test scaling operations that use corner anchor points
   - Verify correct scaling behavior for each corner (NorthWest, NorthEast, SouthEast, SouthWest)
   - Validate that the anchor point remains fixed during scaling

2. **Rotate Around Selection Center**
   - Test rotation operations centered on the selection's centroid
   - Verify rotation calculations are accurate
   - Ensure selection center is computed correctly before rotation

3. **Rotate Around Custom Pivot**
   - Test rotation operations using a user-defined pivot point
   - Verify pivot point is respected during rotation
   - Test various pivot positions relative to the geometry

4. **Minimum Size Clamp**
   - Test that geometry cannot be scaled below minimum bounds
   - Verify clamping behavior prevents invalid geometry states
   - Test edge cases at boundary values

5. **Negative Scaling: Flip vs Clamp**
   - Test behavior when scale factors become negative
   - Verify correct handling (flip or clamp) based on geometry constraints
   - Test transition through zero scale

## Preconditions
- GEO geometry module exists with transform operations
- Transform operations support scaling and rotation
- Test framework is properly configured

## Postconditions
- All 5 test cases pass
- Code coverage for transform operations increases
- No regressions in existing tests

## Invariants
- Tests must be deterministic
- Tests must not depend on external state
- Tests must complete in reasonable time (< 1s per test)
