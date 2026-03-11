# Implementation Summary for seshat-dc3: qa-layer: Build Combinatorial Headless Test Harness

## Constraints Enforced & Implemented
1. **Data->Calc->Actions Architecture**: The headless driver isolates the pure calculation of VirtualDom events from the physical I/O WAL appending action. This provides strict determinism.
2. **Zero Mutability / Zero Panics/Unwraps**: We avoid panics in the proptests and correctly handle the Results returning them safely back down the test.
3. **Make Illegal States Unrepresentable**: We parse states within the boundaries (no generic Strings where explicit NodeIds or EdgeIds should exist, handled via the new harness DSL).

## Changes
- Created `PerformanceDriver` DSL in `diagram_tool/src/perf/harness.rs` that encapsulates a real `VirtualDom` and `SqlitePool`.
- Implemented `simulate_concurrent_session` that tests 60Hz UI inputs and concurrently fires Restate logs, asserting an 8ms frame budget and ghosting diff generation.
- Expanded `diagram_tool/src/tests/contracts.rs` to include a `proptest!` test suite testing concurrent combinations of human interactions vs Restate WAL operations. 
- Integrated real SQLite WAL, avoiding any mocking of `SqlitePool` to satisfy the "Do not mock the WAL" constraint.
- Resolved module resolution issues with duplicate files and old exports in `diagram_tool/src/ui/dispatch/`.

## Files Changed
- `diagram_tool/src/perf/harness.rs`
- `diagram_tool/src/tests/contracts.rs`
- `diagram_tool/src/ui/dispatch/mod.rs` (fixed broken module references)