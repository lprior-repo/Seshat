# Test Plan Defects: seshat-k40

The test plan (`martin-fowler-tests.md`) has been evaluated against the core testing doctrines and has been **REJECTED** due to the following critical defects:

## 1. Dan North (BDD) Violations
- **Non-Behavioral Naming**: Test names follow an imperative/xUnit style (e.g., `test_append_event_async_succeeds_with_valid_event`) instead of expressive, behavioral descriptions (e.g., `given_valid_event_when_appending_then_succeeds`).
- **Incomplete GWT Structure**: While some scenarios use Given/When/Then, the main lists of tests do not reflect executable specifications.

## 2. Dave Farley (ATDD) Violations
- **Missing DSL / Implementation Coupling**: There is no separation of test intent (WHAT) from execution (HOW). The plan directly couples to the API signatures (e.g., calling `append_event_async(pool, valid_event, ...)`). It lacks a Domain Specific Language (DSL) to insulate tests from refactoring and implementation details.

## 3. Combinatorial Permutations Violations
- **Incomplete Edge Cases**: 
  - Misses boundary conditions for `BoundedBatch<MIN, MAX>`. It tests the `MAX` limit but fails to test the exact `MIN` allowed elements.
  - Misses edge cases for revisions (e.g., appending when current revision is `0`, expected is `0` vs `None`).
- **Missing Concurrency Permutations**: SQLite WAL is highly dependent on concurrent behavior, yet there are no tests ensuring monotonic revision increments and conflict resolution under highly concurrent async appends.

## 4. Advanced Testing Paradigms Violations
- **No Property-Based Testing**: The invariants (e.g., "[I1] The `revision` column strictly monotonically increases without gaps") demand property-based testing to verify behavior across hundreds of permutations, which is completely absent.
- **No Fuzzing or Mutation Testing**: The plan fails to specify fuzzing for inputs or mutation testing to ensure the robustness of the error taxonomy and contract enforcement.