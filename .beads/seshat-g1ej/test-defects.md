# Test Defects: seshat-g1ej

## STATUS: REJECTED

---

## Critical Defects

### DEFECT-001: Graph Routing Error Variant Tests Claimed But Non-Existent
**Location**: martin-fowler-tests.md Scenarios 37-42
**Doctrine Violated**: Testing Trophy (Real Execution), Kent Beck TDD
**Severity**: CRITICAL

The test plan claims coverage for 6 graph routing error variants in Scenarios 37-42:
- Scenario 37: SourceNotFound
- Scenario 38: TargetNotFound
- Scenario 39: SelfLoop
- Scenario 40: CycleDetected
- Scenario 41: InvalidNodeCoordinates
- Scenario 42: DuplicateEdge

**Evidence**: grep across entire codebase shows these variants ONLY exist as type definitions in `routing.rs` lines 15-25. No test files contain tests for these error variants. The routing_tests.rs file contains no tests for these errors.

**Required Fix**: Implement actual tests for all 6 graph routing error variants.

---

### DEFECT-002: StoreBridge Tests Only Run Under Kani Formal Verification
**Location**: store_bridge.rs lines 167-265; martin-fowler-tests.md Scenarios 6-11, 17, 19-20, 25, 43-45
**Doctrine Violated**: Testing Trophy (Real Execution), Dave Farley ATDD
**Severity**: CRITICAL

The test plan describes regular runtime tests for StoreBridge (Scenarios 6-11, 17, 19-20, 25, 43-45):
- Scenario 6: spawn_async_pool success
- Scenario 7: append_event_sync success
- Scenario 8: fetch_events_since_sync
- Scenario 9: append_batch_sync
- Scenario 10: append_idempotent_sync
- Scenario 11: shutdown
- Scenario 17: RuntimeSpawn error
- Scenario 19: PoolNotInitialized error (instead of panic)
- Scenario 20: RuntimeNotRunning error after shutdown
- Scenario 25: revision conflict
- Scenario 43: PoolLockError

**Evidence**: ALL StoreBridge tests in store_bridge.rs are gated behind `#[cfg(kani)]`:
- Line 167: `#[cfg(kani)]` on test_spawn_and_shutdown
- Line 178: `#[cfg(kani)]` on test_append_event_sync
- Line 196: `#[cfg(kani)]` on test_fetch_events_since_sync
- Line 220: `#[cfg(kani)]` on test_append_batch_sync
- Line 243: `#[cfg(kani)]` on test_append_idempotent_sync

These tests will NOT run with `cargo test` - they only run under Kani model checker for formal verification. The test plan claims they are regular runtime tests.

**Required Fix**: Implement runtime tests (not kani-gated) for StoreBridge functionality.

---

### DEFECT-003: E2E Integration Tests Claimed But Non-Existent
**Location**: martin-fowler-tests.md Scenarios 44-45
**Doctrine Violated**: Testing Trophy (Real Execution)
**Severity**: CRITICAL

The test plan claims:
- Scenario 44: "append_event_then_fetch_events_end_to_end"
- Scenario 45: "spawn_bridge_append_batch_shutdown_verify_clean"

**Evidence**: No E2E tests exist in the codebase for StoreBridge lifecycle. The store_bridge_test.rs file is an 8-line placeholder component, not actual tests.

**Required Fix**: Implement actual E2E integration tests.

---

### DEFECT-004: PoolLockError Test Claimed But Non-Existent
**Location**: martin-fowler-tests.md Scenario 43; contract.md line 86
**Doctrine Violated**: Combinatorial Permutations, Kent Beck TDD
**Severity**: HIGH

The test plan and contract claim coverage for `PoolLockError`, but no test exists in the codebase.

**Required Fix**: Implement test for PoolLockError variant.

---

## Medium-Severity Defects

### DEFECT-005: Multiple Assertions Per Test
**Location**: routing_tests.rs lines 35-44, 265-281, 284-291
**Doctrine Violated**: Kent Beck TDD (one logical assertion per test)
**Severity**: MEDIUM

Examples:
- `test_returns_success_l_shape_when_points_are_diagonal` (lines 35-44): asserts `route.points.len() == 3` AND `is_orthogonal(&route)`
- `test_postcondition_start_and_end_points_match_input` (lines 265-281): asserts first point AND last point match
- `test_postcondition_route_never_intersects_obstacle_interior` (lines 284-291): asserts route.ok AND non-intersection

Kent Beck's TDD doctrine requires one logical assertion per test for precise failure diagnosis.

**Required Fix**: Split tests into single-assertion tests.

---

## Previously Fixed Issues

The following defects from the previous review have been addressed:
- ~~DEFECT-003 (Precondition Misuse)~~ - Contract.md correctly describes CODE defect, not test defect
- ~~DEFECT-009 (BDD Naming Inconsistency)~~ - martin-fowler-tests.md now uses proper Scenario format

---

## Summary

| Category | Count |
|----------|-------|
| Critical | 3 |
| High | 1 |
| Medium | 1 |
| **Total** | **5** |

**Blocking Issues**: DEFECT-001, DEFECT-002, DEFECT-003

The test plan cannot be approved until:
1. Graph routing error variant tests are implemented (SourceNotFound, TargetNotFound, SelfLoop, CycleDetected, InvalidNodeCoordinates, DuplicateEdge)
2. StoreBridge tests run under `cargo test` (not just kani)
3. E2E integration tests are implemented for StoreBridge lifecycle

**Note**: The test plan itself is well-written (proper BDD Given-When-Then structure, comprehensive coverage claims, good error taxonomy). However, the claims of test coverage are not backed by actual runnable tests. Per Testing Trophy doctrine, tests must "ACTUALLY execute" - not just be specified in documentation.
