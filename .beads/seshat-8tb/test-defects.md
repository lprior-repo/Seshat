# Test Defects - Bead seshat-8tb

**Reviewer:** test-reviewer skill  
**Date:** 2026-03-12  
**Status:** REJECTED

---

## Summary

The martin-fowler-tests.md file has **8 critical defects** that violate the Testing Trophy, Dan North BDD, and Dave Farley ATDD doctrines.

---

## Defect List

### D1: Missing Precondition P2 Test Coverage (CRITICAL)
- **Contract Requirement:** P2 - "ai_conflict_state signal must be initialized in app.rs context"
- **Current State:** No test case verifies this precondition
- **Doctrine Violation:** ATDD requires all preconditions be verified as executable specifications
- **Recommended Fix:** Add `test_precondition_p2_ai_conflict_state_initialized`

### D2: Missing Postcondition Q4 Test Coverage (CRITICAL)
- **Contract Requirement:** Q4 - "User can manually dismiss the toast, which also clears the conflict state"
- **Current State:** Scenario 2 mentions this but no dedicated test exists
- **Doctrine Violation:** Dave Farley ATDD requires explicit verification of ALL postconditions
- **Recommended Fix:** Add `test_postcondition_q4_manual_dismiss_clears_conflict_state`

### D3: Missing Invariant I1 Test (HIGH)
- **Contract Requirement:** I1 - "Only one conflict toast can be displayed at a time"
- **Current State:** Scenario 3 describes behavior but no explicit test
- **Doctrine Violation:** Combinatorial permutations must exhaustively cover invariants
- **Recommended Fix:** Add `test_invariant_i1_only_one_toast_at_a_time`

### D4: Missing Invariant I2 Test (HIGH)
- **Contract Requirement:** I2 - "ai_conflict_state is Some when conflict exists, None otherwise"
- **Current State:** Implicitly tested but not explicit
- **Doctrine Violation:** Invariants must be explicitly verified
- **Recommended Fix:** Add `test_invariant_i2_conflict_state_reflects_conflict_existence`

### D5: Missing Error::InvalidReason Test (MEDIUM)
- **Contract Requirement:** Error taxonomy includes Error::InvalidReason
- **Current State:** No test covers this error path
- **Doctrine Violation:** All error types in taxonomy must have test coverage
- **Recommended Fix:** Add `test_error_invalid_reason_when_conflict_reason_empty`

### D6: BDD Naming Convention Not Followed (MEDIUM)
- **Current State:** Test names like `test_conflict_toast_displays_when_ai_event_dropped`
- **Expected:** Given-When-Then structure: `test_given_poller_detects_dropped_ai_when_conflict_detected_then_toast_displays`
- **Doctrine Violation:** Dan North BDD requires executable specifications with GWT naming

### D7: No Property-Based or Fuzzing Tests (MEDIUM)
- **Current State:** Only deterministic unit tests
- **Doctrine Violation:** Testing Trophy and advanced paradigms require property-based testing for edge cases
- **Recommended Fix:** Consider adding quickcheck/proptest for AiConflictState generation

### D8: No Integration/E2E Test Coverage (MEDIUM)
- **Current State:** All tests are unit-level
- **Doctrine Violation:** Testing Trophy emphasizes "Real Execution" - tests should run against actual components
- **Recommended Fix:** Add integration test verifying poller → conflict detection → toast display pipeline

---

## Coverage Matrix

| Contract Item | Test Coverage | Status |
|---------------|---------------|--------|
| P1 (ToastApi available) | test_precondition_p1_toast_api_available | ✓ |
| P2 (ai_conflict_state initialized) | - | **MISSING** |
| P3 (Conflict detected) | test_precondition_p3_conflict_detected | ✓ |
| Q1 (Toast displays) | test_postcondition_q1_toast_displayed | ✓ |
| Q2 (Auto-dismiss 3s) | test_postcondition_q2_auto_dismiss_after_3_seconds | ✓ |
| Q3 (Clear on dismiss) | test_postcondition_q3_conflict_state_cleared_on_dismiss | ✓ |
| Q4 (Manual dismiss clears) | - | **MISSING** |
| I1 (One toast max) | - | **MISSING** |
| I2 (State reflects conflict) | - | **PARTIAL** |
| Error::NoConflictState | test_show_conflict_toast_returns_error_when_no_conflict_state | ✓ |
| Error::QueueFull | test_show_conflict_toast_returns_error_when_queue_full | ✓ |
| Error::InvalidReason | - | **MISSING** |
| Error::SignalNotFound | - | **IMPLICIT** |

---

## Conclusion

**STATUS: REJECTED**

The test plan requires fixes for D1-D8 before implementation. Address all critical and high-priority defects before proceeding to code generation.
