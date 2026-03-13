# Test Defects: seshat-lfm2 (Re-Review)

## Status: REJECTED

---

## Previously Reported Defects (Now Fixed)

| Defect ID | Description | Status |
|-----------|-------------|--------|
| 1 | Tests for apply_undo/apply_redo public API | ✅ FIXED - Tests added (lines 93-143) |
| 2 | Invariant verification tests I1, I2, I4, I5 | ✅ FIXED - All present (lines 308-356) |
| 3 | Positive case test for multiple entries | ✅ FIXED - test_multiple_operations_create_multiple_entries (lines 198-211) |

---

## NEW Critical Defects Found

### DEFECT-001: Contract Mismatch for apply_undo/apply_redo Error Types

**Severity**: CRITICAL  
**Location**: martin-fowler-tests.md lines 107-118, 132-143

**Issue**: Contract specifies `Result<(), &'static str>` but tests expect `HistoryError` enum.

**Contract** (contract.md lines 86-88):
```rust
pub fn apply_undo(doc: &mut DiagramDocument, history: &mut History) -> Result<(), &'static str>;
pub fn apply_redo(doc: &mut DiagramDocument, history: &mut History) -> Result<(), &'static str>;
```

**Test Expectations** (Incorrect):
- Line 117: `Returns Err(HistoryError::EmptyUndoStack)`
- Line 143: `Returns Err(HistoryError::EmptyRedoStack)`

**Required Fix**: Tests must expect `Err("EmptyUndoStack")` and `Err("EmptyRedoStack")` (string literals) OR contract must be updated to define `HistoryError` enum.

---

### DEFECT-002: Missing Integration/E2E Tests (Testing Trophy Violation)

**Severity**: HIGH  
**Location**: martin-fowler-tests.md (entire file)

**Issue**: All tests are unit tests. No integration or E2E tests exist to validate the system actually works end-to-end per Testing Trophy philosophy.

**Testing Trophy Doctrine**: "Focus on running the REAL thing first. Demand tremendous amounts of integration and end-to-end tests that validate the system actually works."

**Required Fix**: Add integration tests that:
- Exercise full UI → history → undo workflow
- Test drag gesture → single history entry → undo restores position
- Verify multi-component interactions (document + history + UI state)

---

### DEFECT-003: Missing Invariant I3 Verification

**Severity**: MEDIUM  
**Location**: martin-fowler-tests.md

**Issue**: Contract specifies I3: "After push: redo_stack is empty" (contract.md line 59), but no explicit test verifies this invariant.

**Required Fix**: Add test_invariant_i3_after_push_redo_stack_is_empty

---

## Evaluation Summary

| Criterion | Status | Notes |
|-----------|--------|-------|
| Dan North BDD (Given-When-Then) | ✅ PASS | Well-structured GWT format |
| Dave Farley ATDD (WHAT/HOW separation) | ⚠️ PARTIAL | Contract mismatch (DEFECT-001) |
| Kent Beck TDD (Isolation/Fast/Deterministic) | ✅ PASS | Tests are isolated |
| Testing Trophy (Real Execution) | ⚠️ PARTIAL | No integration/E2E tests (DEFECT-002) |
| Combinatorial Permutations | ⚠️ PARTIAL | Missing I3 (DEFECT-003) |
| Advanced Paradigms | ⚠️ PARTIAL | No property-based testing |

---

## Recommendation

**STATUS: REJECTED**

The test plan must be revised to address:
1. Contract mismatch for apply_undo/apply_redo error types (Defect 001)
2. Add integration/E2E tests (Defect 002)  
3. Add invariant I3 verification test (Defect 003)

Once these defects are addressed, the test plan can be re-reviewed for approval.
