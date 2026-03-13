# Test Defects Report - seshat-9t4

## STATUS: REJECTED

## Review Doctrine
- Testing Trophy (Real Execution)
- Dan North (BDD)
- Dave Farley (ATDD)
- Kent Beck (TDD)

---

## Critical Defects

### DEFECT-001: Open Question Not Resolved (Specification Gap)
**Severity**: CRITICAL  
**Location**: contract.md:16, martin-fowler-tests.md (entire file)

**Issue**: The contract explicitly lists "Which behavior is the SPECIFIED correct behavior: screen-space or world-space?" as an OPEN QUESTION. The test plan assumes screen-space behavior throughout without specification resolution.

**Evidence**:
- contract.md:16 - "Which behavior is the SPECIFIED correct behavior: screen-space or world-space?"
- contract.md:14 - "Current implementation uses screen-space" (implementation detail leaked into spec)
- All tests in martin-fowler-tests.md assume `MarginBehavior::ScreenSpace` without business justification

**Required Fix**: Either:
1. Resolve the open question with business stakeholder input, OR
2. Add both behaviors as separate feature flags with explicit acceptance criteria

---

### DEFECT-002: Missing DSL / ATDD WHAT-HOW Separation
**Severity**: HIGH  
**Location**: martin-fowler-tests.md:43-52, 70-73, 90-94

**Issue**: Dave Farley ATDD requires strict separation of WHAT (intent) from HOW (implementation). Tests leak implementation details.

**Evidence**:
- Line 50: `hit_test_with_zoom_margin(point, &rect, 0.1, MarginBehavior::ScreenSpace)`
- Line 70-72: Direct function calls `screen_to_world_margin(5.0, 0.1)`
- These are HOW (implementation), not WHAT (behavior)

**Required Fix**: Create a DSL layer:
```gherkin
Feature: Hit test margin behavior
  Scenario: User clicks near node edge when zoomed out
    Given a node at screen position (100, 100) with size 100x100
    And zoom level is 0.1 (far away)
    When user clicks 5 pixels from the node edge
    Then the hit test succeeds (larger hit area compensates for zoom)
```

---

### DEFECT-003: No Real Execution Evidence / Testing Trophy Violation
**Severity**: HIGH  
**Location**: martin-fowler-tests.md (entire file)

**Issue**: Testing Trophy requires "tremendous amounts of integration and end-to-end tests that validate the system actually works." This test plan is purely specification-only with no execution proof.

**Evidence**:
- No integration test files referenced
- No E2E test files referenced
- No test execution output
- No indication tests are implemented

**Required Fix**: Add:
1. Integration tests that run against actual viewport/diagram system
2. E2E tests that simulate user clicks at various zoom levels
3. Execution proof (test output showing tests pass)

---

### DEFECT-004: BDD Tests Are Implementation-Focused
**Severity**: MEDIUM  
**Location**: martin-fowler-tests.md:11-34, 43-63

**Issue**: Dan North BDD requires tests to be executable specifications of behavior. Current tests describe algorithms, not user behavior.

**Evidence**:
- T001: "Given: screen_margin = 5.0, zoom = MIN_ZOOM" - this is input, not behavior
- "When: computing world margin from screen margin" - technical operation, not user action
- "Then: returns 50.0" - expected value, not business outcome

**Required Fix**: Rewrite as user-facing scenarios:
```gherkin
Scenario: Easy node selection when zoomed out
  Given I have a diagram with a node at position (0,0)
  And I am zoomed out (zoom = 0.1)
  When I click near the node edge (5 pixels away)
  Then the node is selected (hit test succeeds because system accounts for zoom)
```

---

### DEFECT-005: Missing World-Space Invariant Testing
**Severity**: MEDIUM  
**Location**: martin-fowler-tests.md:235-249

**Issue**: Contract defines two invariants (I1 for screen-space, I2 for world-space). Only I1 is partially tested. No explicit test for world-space consistency.

**Evidence**:
- contract.md:31-32 defines both I1 and I2
- T033 tests screen-space invariant only
- No test verifies `world_margin_constant` produces consistent results across zoom levels

**Required Fix**: Add T034:
```rust
/// GEO-020-T034: verify_invariant_i2_world_space_consistency
/// Given: A point at fixed world distance from edge, varying zoom
/// When: performing hit test at zoom 0.1, 1.0, 4.0
/// Then: all return same hit result
let hit_low = hit_test_with_zoom_margin(Point::new(15.0, 50.0), &rect, 0.1, MarginBehavior::WorldSpace)?;
let hit_mid = hit_test_with_zoom_margin(Point::new(15.0, 50.0), &rect, 1.0, MarginBehavior::WorldSpace)?;
let hit_high = hit_test_with_zoom_margin(Point::new(15.0, 50.0), &rect, 4.0, MarginBehavior::WorldSpace)?;
assert_eq!(hit_low, hit_mid);
assert_eq!(hit_mid, hit_high);
```

---

### DEFECT-006: Duplicate Tests
**Severity**: LOW  
**Location**: martin-fowler-tests.md:88-110 vs 253-278

**Issue**: Tests T010/T011/T012 are functionally identical to V001/V002.

**Evidence**:
- T010 = V001 (invalid_zoom_below_min)
- T011 = V002 (invalid_zoom_above_max)  
- T012 duplicates negative zoom testing
- T013/T014 = V003/V004 (invalid margin)

**Required Fix**: Consolidate into single test per precondition, or clarify distinction (contract verification vs contract violation)

---

### DEFECT-007: Missing Edge Cases
**Severity**: LOW  
**Location**: martin-fowler-tests.md:128-144

**Issue**: Missing edge cases for:
- `-INFINITY` as point coordinate
- Subnormal (denormalized) floating point numbers
- Very close to zero zoom (but valid, e.g., 0.1000001)

**Required Fix**: Add:
```rust
/// GEO-020-T017: invalid_point_negative_infinity
/// GEO-020-T018: invalid_point_subnormal
/// GEO-020-T019: zoom_near_boundary
```

---

## Summary

| Defect | Severity | Doctrine Violated |
|--------|----------|-------------------|
| DEFECT-001 | CRITICAL | ATDD (Open Specification) |
| DEFECT-002 | HIGH | Dave Farley ATDD |
| DEFECT-003 | HIGH | Testing Trophy |
| DEFECT-004 | MEDIUM | Dan North BDD |
| DEFECT-005 | MEDIUM | ATDD (Invariant Coverage) |
| DEFECT-006 | LOW | TDD (Duplication) |
| DEFECT-007 | LOW | TDD (Edge Cases) |

---

## Required Actions Before Re-review

1. **Resolve DEFECT-001**: Get business stakeholder to specify correct behavior (screen-space vs world-space)
2. **Resolve DEFECT-002**: Create Gherkin/DSL layer for behavior specification
3. **Resolve DEFECT-003**: Implement integration tests and provide execution proof
4. **Resolve DEFECT-004**: Rewrite Given-When-Then as user-facing scenarios
5. **Resolve DEFECT-005**: Add world-space invariant test
6. **Optional**: Fix DEFECT-006 and DEFECT-007 for completeness
