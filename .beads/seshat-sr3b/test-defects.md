# Test Defects Report: seshat-sr3b Test Plan

**Review Date**: 2026-03-15  
**Reviewer**: Test Reviewer (BDD/TDD/ATDD/Testing Trophy)  
**Status**: REJECTED

---

## Critical Defects

### 1. Testing Trophy Violation: No Real Execution (CRITICAL)
**Doctrine**: Testing Trophy demands integration/E2E tests that validate the system actually works. Mocks should be minimized.

**Evidence**:
- martin-fowler-tests.md contains ONLY unit tests for the `LockState` enum
- `test_70_plus_references_updated` (lines 157-160) is just a `grep` command, NOT an actual test
- No integration tests that run the real diagram tool with the new enum
- No E2E tests that verify user workflows work end-to-end

**Required Fix**: Add real integration tests:
- Run actual document loading/saving with locked nodes
- Test canvas interactions with locked nodes
- Test selection/movement with actual Node instances in a real diagram

---

### 2. Dave Farley ATDD Violation: No DSL / WHAT vs HOW Separation (CRITICAL)
**Doctrine**: ATDD requires strict separation of WHAT (intent/behavior) from HOW (implementation). Tests should use a DSL that describes business value, not implementation details.

**Evidence**:
- Tests directly call implementation methods: `node.lock_state.is_locked()` (line 21)
- Tests describe HOW to check: "When `node.lock_state.is_movable(&node.kind)` is called" (line 30)
- No user-facing DSL that describes business behavior

**Required Fix**: Add DSL-style acceptance tests like:
```
Scenario: User cannot move a locked node
  Given a diagram with a locked regular node
  When the user attempts to drag the node
  Then movement is blocked
  And the node remains at its original position
```

---

### 3. Specification Contradiction: Backwards Compatibility (CRITICAL)
**Doctrine**: Contract and tests must be consistent.

**Evidence**:
- contract.md line 112: "must remain `locked: bool` in JSON for backwards compatibility"
- martin-fowler-tests.md line 65: expects `"lock_state": "unlocked"` in JSON
- martin-fowler-tests.md line 70: expects `"lock_state": "locked"` in JSON

**Conflict**: Tests expect `lock_state` field but contract says backward compatibility requires `locked` field.

**Required Fix**: Either:
1. Update tests to expect `"locked": true/false` for backward compatibility, OR
2. Add transformation layer in deserialization to convert `locked` → `lock_state`

---

### 4. Missing Happy Path: User-Facing Behavior Tests
**Doctrine**: Dan North BDD emphasizes testing behavior, not state. Tests should describe user-visible behavior.

**Evidence**:
- All tests are about the enum implementation, not user workflows
- No tests for: "User locks a node via UI", "User tries to move locked node"

**Required Fix**: Add BDD scenarios for:
- User toggling lock state in properties panel
- User attempting operations on locked nodes
- User experience with locked/unlocked nodes

---

### 5. Missing Unhappy Path: Real Error Scenarios
**Doctrine**: Comprehensive test coverage requires unhappy paths with actual runtime behavior.

**Evidence**:
- Error path tests (lines 49-58) only test compile errors, not runtime errors
- No tests for edge cases like corrupted serialization data
- No tests for invalid state transitions

**Required Fix**: Add runtime error path tests:
- Deserialize JSON with invalid lock_state value
- Handle missing lock_state field gracefully

---

### 6. Missing Edge Case: NodeKind::Text with Unlocked
**Evidence**:
- `test_text_node_movable_behavior_matches_regular_node` (line 43) only tests when locked
- Missing: Text node with Unlocked should return true for is_movable()

---

### 7. Incomplete Invariant Testing
**Evidence**:
- `test_invariant_subgraph_always_movable` (line 94) only tests Subgraph
- Missing: Invariant tests for ALL NodeKind variants with BOTH LockState variants

---

## Summary

| Defect | Severity | Doctrine Violated |
|--------|----------|-------------------|
| No real integration/E2E tests | CRITICAL | Testing Trophy |
| No DSL / WHAT vs HOW separation | CRITICAL | Dave Farley ATDD |
| Backwards compatibility contradiction | CRITICAL | Contract Consistency |
| Missing user-facing BDD scenarios | HIGH | Dan North BDD |
| No runtime error path tests | HIGH | TDD Completeness |
| Missing edge cases | MEDIUM | Combinatorial Coverage |

---

## 8. CRITICAL: Test Cannot Compile - Implementation Missing
**Location**: `diagram_tool/tests/lock_state_test.rs:7`
**Issue**: Test imports `diagram_tool::models::document::LockState` but this type does NOT exist in the codebase

**Evidence**:
- `grep -r "enum LockState" diagram_tool/src/models/` returns no results
- `document.rs:210` still shows `pub locked: bool,` (not `lock_state: LockState`)
- The entire Node struct migration has NOT been implemented

**Impact**: Tests cannot run at all - fundamental failure

---

## 9. Contract Implementation Status: NOT STARTED
**Location**: `diagram_tool/src/models/document.rs:197-225`
**Issue**: Node struct still uses boolean `locked` field, not LockState enum

**Evidence**: Line 210 shows `pub locked: bool,` - contract Q1/Q2 require `lock_state: LockState`

**Impact**: All 50+ tests in martin-fowler-tests.md cannot be executed

**Recommendation**: Tests must be rewritten to include:
1. Real integration tests that execute the actual diagram tool
2. DSL-style acceptance tests describing user behavior
3. Resolution of the backwards compatibility contradiction
4. Complete combinatorial coverage of all NodeKind × LockState permutations
