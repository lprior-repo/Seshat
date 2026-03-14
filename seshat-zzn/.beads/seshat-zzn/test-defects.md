# Test Defects Report: seshat-zzn

**Bead ID**: seshat-zzn  
**Review Date**: 2026-03-14  
**Status**: REJECTED  
**Reviewer**: Test Reviewer (Updated)

---

## Executive Summary

The contract.md and martin-fowler-tests.md have **multiple critical defects** that violate the Testing Trophy, Dan North BDD, and Dave Farley ATDD doctrines. The tests cannot be implemented as specified.

**NEW CRITICAL FINDING:** The actual test implementation in `selection_ops_tests.rs` contains `#[cfg(kani)]` attributes on ALL tests, making them **non-executable with cargo test**.

---

## Critical Defects

### DEFECT-001: Contract-Implementation Mismatch (Touch Hit Radius Value)

**Severity**: CRITICAL  
**Location**: contract.md lines 82-84, martin-fowler-tests.md lines 60-78

**Issue**: The contract specifies incorrect touch hit radius values:
- Contract Q8: `max(base_radius, 8.0)` for nodes
- Contract Q9: `max(base_radius, TOUCH_HIT_RADIUS_MIN)` for handles

**Actual Implementation** (canvas_view.rs line 1503, 1522):
- `TOUCH_HIT_RADIUS_PX = 44.0`
- `touch_hit_radius(base_radius, is_touch)` returns `base_radius.max(44.0)`

**Impact**: Tests written to contract specifications will fail against actual implementation. The contract value (8.0) is incorrect - the code correctly uses 44.0 (WCAG touch target guideline).

**Required Fix**: Update contract.md to use `44.0` instead of `8.0` for touch hit radius.

---

### DEFECT-002: Undefined TOUCH_HIT_RADIUS_MIN Constant

**Severity**: CRITICAL  
**Location**: contract.md line 83, 158

**Issue**: Contract references `TOUCH_HIT_RADIUS_MIN` but never defines its value.

**Required Fix**: Either define the constant value (should be 44.0 per implementation), or remove the distinction between nodes and handles if not applicable.

---

### DEFECT-003: Function Signature Cannot Support Contract Logic

**Severity**: HIGH  
**Location**: contract.md lines 129-130, 82-84

**Issue**: The contract specifies two different behaviors for `touch_hit_radius`:
- Q8: For nodes - returns `max(base_radius, 8.0)`
- Q9: For handles - returns `max(base_radius, TOUCH_HIT_RADIUS_MIN)`

But the function signature is:
```rust
fn touch_hit_radius(base_radius: f64, is_touch: bool) -> f64
```

There is no parameter to distinguish nodes from handles. The actual implementation treats both identically.

**Required Fix**: Either:
1. Add a parameter to distinguish element type, OR
2. Update contract to state both use same logic

---

### DEFECT-004: Testing Trophy Violation - No Real Execution

**Severity**: HIGH  
**Location**: martin-fowler-tests.md entire document

**Issue**: The Testing Trophy emphasizes integration and E2E tests that validate the system actually works. This test plan specifies ONLY unit tests (all marked `#[cfg(kdni)]`).

**Missing Test Categories**:
1. No integration tests (testing component interactions)
2. No E2E tests (testing complete user workflows)
3. No tests against actual DOM/browser
4. No Playwright/WebDriver tests

**Required Fix**: Add integration and E2E test specifications for:
- Hover visual feedback rendering
- Touch selection in actual browser
- Resize handle dragging in UI

---

### DEFECT-005: Open Questions Remain Unanswered

**Severity**: MEDIUM  
**Location**: contract.md lines 31-34

**Issue**: Three open questions (Q1, Q2, Q3) are listed but never resolved:
- Q1: Are tests expected to test UI rendering or only model logic?
- Q2: Should tests verify hover performance (<16ms) or just behavior?
- Q3: Should touch hit area tests verify exact pixel values or just behavior?

**Impact**: Test implementation cannot proceed without resolving these questions.

**Required Fix**: Resolve all open questions before implementing tests.

---

### DEFECT-006: Redundant Test Coverage for SEL-005

**Severity**: LOW  
**Location**: martin-fowler-tests.md lines 180-189

**Issue**: The test plan includes tests for SEL-005 (Marquee Direction), but contract.md line 40 states SEL-005 is "Already implemented in seshat-9n1".

**Required Fix**: Remove SEL-005 tests from martin-fowler-tests.md or clarify they are reference/comparison tests.

---

## Doctrinal Violations Summary

| Doctrine | Violation | Severity |
|----------|-----------|----------|
| Testing Trophy | No integration/E2E tests | HIGH |
| Testing Trophy | No real execution validation | HIGH |
| Dave Farley ATDD | Contract specifies non-existent behavior | CRITICAL |
| Kent Beck TDD | Tests depend on unfixed contract | HIGH |
| Dan North BDD | Open questions block implementation | MEDIUM |

---

## Required Actions Before Approval

1. **Update contract.md**: Change touch hit radius from 8.0 to 44.0 (line 82)
2. **Define or remove TOUCH_HIT_RADIUS_MIN**: Either set to 44.0 or remove distinction (line 83)
3. **Resolve Q1, Q2, Q3**: Answer all open questions in contract.md
4. **Add integration tests**: Specify E2E tests for visual feedback, touch, and resize interactions
5. **Clarify SEL-005**: Remove redundant tests or mark as reference only

---

## Test Implementation Status

**DO NOT IMPLEMENT** - Tests written to current specifications will:
1. Fail against actual implementation (wrong touch radius value)
2. Be incomplete (missing E2E coverage)
3. Be blocked (unanswered questions)

---

## NEW DEFECT (2026-03-14): Tests Not Executable with cargo test

**Severity:** CRITICAL  
**Doctrine Violated:** Testing Trophy - Real Execution  
**Location:** `diagram_tool/src/models/selection_ops_tests.rs` (entire file)

**Issue:** All 274 lines of tests are wrapped with `#[cfg(kani)]` and `#[kani::proof]` attributes. Running `cargo test` shows **zero tests execute**:

```
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 295 filtered out
```

**Problem:** This violates the Testing Trophy philosophy which demands **Real Execution**. Tests should be runnable with standard `cargo test`, not require Kani model checker.

**Evidence:** Test file line 27 reads `#[cfg(kani)]`, line 28 reads `#[kani::proof]`, and this pattern repeats for every test function (lines 30, 50, 69, 88, 112, 126, 147, 163, 175, 191, 210, 233, 249, 265).

---

## NEW DEFECT (2026-03-14): Missing Test Coverage for SEL-006..SEL-009

**Severity:** CRITICAL  
**Doctrine Violated:** Dan North BDD  
**Location:** `diagram_tool/src/models/selection_ops_tests.rs`

**Issue:** The contract specifies tests for SEL-005 through SEL-009, but the test file only contains tests for SEL-001 through SEL-005. **No tests exist for:**
- SEL-006: Hover shows visual affordances
- SEL-007: Resize handles are clickable
- SEL-008: Touch has larger hit area (WCAG 44px)
- SEL-009: Drag threshold prevents accidental drag

**Missing Test Functions (from contract.md lines 127-137):**
- `touch_hit_radius()` - NOT TESTED
- `has_drag_threshold()` - NOT TESTED
- `touch_handle_hit_test()` - NOT TESTED

---

*Generated by Test Reviewer*

*Updated 2026-03-14: Added critical defects for non-runnable tests and missing SEL-006..SEL-009 coverage*
