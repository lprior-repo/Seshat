# Test Plan Defects: seshat-k40

The test plan (`martin-fowler-tests.md`) and contract (`contract.md`) have been evaluated against Testing Trophy, Dan North BDD, and Dave Farley ATDD doctrines. **REJECTED** due to the following critical defects:

## 1. Contract-Implementation Parity Violation

**CRITICAL**: The contract specifies refactoring `store_async.rs` to use DDD types:
- `ValidEvent` instead of `EventEnvelope` (contract.md lines 127-129)
- `BoundedBatch<MIN, MAX>` instead of `Vec<EventEnvelope>` (contract.md lines 131-136)
- `Option<Revision>` instead of `Option<i64>` (contract.md lines 128, 134)

**Actual Implementation** (store_async.rs lines 305-359):
```rust
pub async fn append_event_async(
    pool: &SqlitePool,
    envelope: EventEnvelope,      // ❌ Should be ValidEvent
    expected_revision: Option<i64>,  // ❌ Should be Option<Revision>
)
```

The test plan ASSUMES this refactoring is done, but it hasn't been implemented. Tests cannot execute.

## 2. Missing Boundary Parsing Functions

Contract specifies (lines 139-154):
- `parse_valid_event(op_id, timestamp, payload) -> Result<ValidEvent>`
- `parse_bounded_batch::<MIN, MAX>(events) -> Result<BoundedBatch>`
- `parse_revision(rev: i64) -> Result<Revision>`

These functions do NOT exist in the codebase. Callers cannot convert primitives to DDD types.

## 3. Testing Trophy Violations (Real Execution)

### No Integration Tests Implemented
- Current tests in store_async.rs (lines 701-1177) are unit tests using `TempDir`
- Test plan mentions `test_e2e_full_workflow_with_ddd_types` (martin-fowler-tests.md line 276) but no actual E2E test exists
- No tests run against REAL database with concurrent connections
- No tests validate WAL behavior under concurrent writes

### Over-Mocking
- All tests use in-memory temp directories
- No integration with actual StoreBridge, sync module, or export module (lines 256-274)

## 4. Dan North BDD Violations

### Non-Behavioral Test Names
Test names follow xUnit imperative style:
- `test_append_event_async_with_valid_event_returns_success` (line 20)
- `test_append_batch_async_with_valid_bounded_batch_returns_success` (line 34)

Should be behavioral:
- `given_valid_event_when_appending_to_store_then_returns_incremented_revision`

### Incomplete GWT Structure
Main test lists (lines 18-157) use imperative naming, not Given-When-Then. Only "Given-When-Then Scenarios" section (lines 288-323) uses proper BDD structure.

## 5. Dave Farley ATDD Violations

### No DSL / Implementation Coupling
Test plan directly couples to API signatures:
```rust
append_event_async(&pool, valid_event, None).await  // Line 22
append_batch_async(&pool, batch, None).await       // Line 37
```

Missing Domain Specific Language to insulate tests from refactoring. Tests will break when function signatures change.

## 6. Combinatorial Permutations Violations

### Missing Edge Cases
- BoundedBatch: Tests MAX (line 49) but not exact boundary (MIN vs MIN-1)
- Revision: Missing tests for revision=0 initial state with expected=0 vs None
- Concurrent appends: No tests for concurrent revision monotonicity

### Missing Advanced Paradigms
- **No Property-Based Testing**: Invariant I3 ("revision strictly monotonically increases") demands property-based testing
- **No Fuzzing**: No fuzz tests for boundary inputs
- **No Mutation Testing**: Error taxonomy not validated for completeness

## Required Actions

1. **IMPLEMENT the contract**: Refactor `store_async.rs` to use DDD types
2. **Create boundary parsing functions**: Implement `parse_valid_event`, `parse_bounded_batch`, `parse_revision`
3. **Add REAL integration tests**: Tests that run against actual database with concurrent connections
4. **Add DSL layer**: Abstract test intent from API details
5. **Add property-based tests**: Use `quickcheck` or `proptest` for invariants
6. **Add concurrency tests**: Verify WAL under concurrent writes
