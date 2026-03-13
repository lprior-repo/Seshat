# Test Defects: seshat-shb (UI Dispatch - Z-Index Layering)

## Status: REJECTED

---

## Critical Defects Found

### DEFECT-001: Testing Trophy Violation - Heavy Reliance on Mocks (CRITICAL)

**Severity**: CRITICAL  
**Location**: martin-fowler-tests.md line 235

**Issue**: The test plan explicitly mandates mocking db_tx: "Use mocked db_tx coroutine for unit testing the dispatch logic". This directly violates the Testing Trophy philosophy which demands "Focus on running the REAL thing first" and "tremendous amounts of integration and end-to-end tests that validate the system actually works."

**Testing Trophy Doctrine**: "Keep setups simple" and test the real system, not mocks.

**Required Fix**: Add integration tests that:
- Use real db_tx coroutine (or test channel that behaves like the real thing)
- Test the full pipeline: toolbar action → dispatch → db_tx → store bridge
- Verify persistence actually occurs

---

### DEFECT-002: ATDD WHAT/HOW Separation Violation (HIGH)

**Severity**: HIGH  
**Location**: martin-fowler-tests.md (entire file)

**Issue**: Test names reference implementation details (HOW) rather than behavior (WHAT):

| Current Test Name | Problem |
|-------------------|---------|
| `test_happy_dispatch_bring_to_front_constructs_valid_envelope` | Tests "dispatch_bring_to_front" function (HOW) |
| `test_happy_dispatch_send_to_back_contains_all_selected_ids` | Tests "dispatch_send_to_back" function (HOW) |

**Dave Farley ATDD Doctrine**: Tests must express WHAT (intent/behavior) separate from HOW (implementation).

**Required Fix**: Rename tests to express behavior:
- "test_when_user_clicks_to_front_with_selection_then_envelope_contains_selected_ids"
- "test_given_document_with_selected_nodes_when_bring_to_front_then_nodes_appear_at_highest_z_index"

---

### DEFECT-003: Missing End-to-End Integration Test (HIGH)

**Severity**: HIGH  
**Location**: martin-fowler-tests.md lines 35-37

**Issue**: While line 36 mentions `test_happy_toolbar_button_triggers_full_dispatch_pipeline`, there's no actual implementation of this test. The test plan only describes what should be tested but doesn't provide executable test code.

**Required Fix**: Add actual E2E test that:
1. Creates a DiagramDocument with nodes
2. Selects nodes
3. Calls toolbar action (or directly invokes the action function)
4. Verifies envelope is sent to db_tx
5. Verifies local document state is updated (z-order changed)

---

### DEFECT-004: Missing Combinatorial Permutations (MEDIUM)

**Severity**: MEDIUM  
**Location**: martin-fowler-tests.md (Edge Case Tests section)

**Issue**: Missing critical edge cases:

| Missing Test | Description |
|--------------|-------------|
| Rapid successive clicks | What happens if user clicks To Front multiple times quickly? |
| db_tx channel closed | What happens if channel is closed mid-send? |
| Concurrent operations | What if BringToFront and SendToBack are called simultaneously? |

**Kent Beck TDD**: Tests must cover all permutations including edge cases.

**Required Fix**: Add these test cases:
- `test_edge_rapid_successive_bring_to_front_clicks`
- `test_edge_db_tx_channel_closed_returns_error`
- `test_edge_concurrent_bring_to_front_and_send_to_back`

---

### DEFECT-005: Contract Postcondition Q5 Not Fully Verified (MEDIUM)

**Severity**: MEDIUM  
**Location**: martin-fowler-tests.md line 105-106

**Issue**: Contract Q5 states "NoPanicOnEmptySelection" - empty selection returns early without panicking. However:
1. Test name says "no_panic_on_empty_selection" but doesn't verify the return value
2. Should verify `DispatchResult { nodes_affected: 0, dispatches_sent: 0 }` is returned

**Contract** (contract.md line 36):
```
[Q5] `NoPanicOnEmptySelection`: When selection is empty, the action returns early without panicking (no-op).
```

**Required Fix**: Update test to verify:
```rust
let result = dispatch_bring_to_front(&Some(tx), &[]);
assert_eq!(result, Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 }));
```

---

### DEFECT-006: No Property-Based Testing (MEDIUM)

**Severity**: MEDIUM  
**Location**: martin-fowler-tests.md (Advanced Paradigms section)

**Issue**: No property-based tests to verify invariants across many inputs.

**Advanced Paradigms Doctrine**: Property-based testing exhaustively verifies behavior.

**Required Fix**: Add property-based tests:
- For any non-empty set of node IDs, dispatch should include all IDs
- For empty IDs, result should always be {0, 0}
- Metadata (op_id, timestamp) should always be valid

---

### DEFECT-007: No Mutation Testing Consideration (LOW)

**Severity**: LOW  
**Location**: martin-fowler-tests.md

**Issue**: No mention of mutation testing to verify test quality.

**Advanced Paradigms**: Mutation testing ensures tests actually catch bugs.

**Recommended Fix**: Consider adding mutation testing (e.g., using `mutagen` or similar Rust mutation testing framework).

---

## Evaluation Summary

| Criterion | Status | Notes |
|-----------|--------|-------|
| Dan North BDD (Given-When-Then) | ⚠️ PARTIAL | GWT scenarios present but test names reference HOW not WHAT |
| Dave Farley ATDD (WHAT/HOW separation) | ❌ FAIL | Tests tightly coupled to implementation (DEFECT-002) |
| Kent Beck TDD (Isolation/Fast/Deterministic) | ✅ PASS | Tests are isolated and fast |
| Testing Trophy (Real Execution) | ❌ FAIL | Heavy mocking, no real E2E tests (DEFECT-001, DEFECT-003) |
| Combinatorial Permutations | ⚠️ PARTIAL | Missing edge cases (DEFECT-004) |
| Advanced Paradigms | ❌ FAIL | No property-based or mutation testing (DEFECT-006, DEFECT-007) |

---

## Recommendation

**STATUS: REJECTED**

The test plan must be revised to address:
1. Remove mocking requirement, add real integration tests (DEFECT-001)
2. Separate WHAT from HOW in test names (DEFECT-002)
3. Implement actual E2E test for full pipeline (DEFECT-003)
4. Add missing combinatorial edge cases (DEFECT-004)
5. Verify Q5 fully (DEFECT-005)
6. Add property-based testing (DEFECT-006)

Once these defects are addressed, the test plan can be re-reviewed for approval.

---

## Contract Alignment Check

| Contract Requirement | Test Coverage |
|---------------------|---------------|
| E1: BringToFront dispatch | ✅ Covered |
| E2: SendToBack dispatch | ✅ Covered |
| E3: Contains selected IDs | ✅ Covered (but via HOW, not WHAT) |
| E4: No panic on empty | ⚠️ Partial (DEFECT-005) |
| E5: Handle db_tx unavailable | ✅ Covered |
| P1: SelectionNotEmpty | ✅ Covered |
| P2: DbTxAvailable | ✅ Covered |
| Q1-Q5: Postconditions | ⚠️ Partial coverage |
| I1-I2: Invariants | ❌ Not covered (no property tests) |

