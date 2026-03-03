# Contract Specification: Subgraph Tests (SUB-001 to SUB-034)

**Bead ID**: bd-1b9
**Title**: subgraph: Implement subgraph tests (SUB-001 to SUB-034)
**Phase**: rust-contract
**Created**: 2026-03-03T00:00:00Z

## Summary

Implement 34 comprehensive subgraph tests covering:
- Subgraph creation and management
- Adding/removing nodes to/from subgraphs
- Nested subgraphs
- Subgraph selection behavior
- Subgraph operations (resize, collapse, expand)
- Edge routing across subgraph boundaries

## Test Categories

### Category 1: Subgraph Selection (SUB-001 to SUB-005)
- **SUB-001**: Click inside container selects child vs container with modifier
- **SUB-002**: Box-select across container boundary
- **SUB-003**: Collapse/expand container behavior
- **SUB-004**: Locked container with unlocked children interactions
- **SUB-005**: Parent-child relationship preservation during selection

### Category 2: Reparenting Operations (SUB-006 to SUB-010)
- **SUB-006**: Delete container reparents children
- **SUB-007**: Duplicate container remaps IDs
- **SUB-008**: Drag child into container
- **SUB-009**: Drag child out becomes root
- **SUB-010**: Drag across overlapping containers

### Category 3: Container Behavior (SUB-011 to SUB-014)
- **SUB-011**: Container auto-expand when child crosses boundary
- **SUB-012**: Container resize behavior (children keep size vs scale)
- **SUB-013**: Container overflow handling
- **SUB-014**: Container padding alignment

### Category 4: Subgraph Creation (SUB-015 to SUB-020)
- **SUB-015**: Create empty subgraph container
- **SUB-016**: Create subgraph with pre-selected nodes
- **SUB-017**: Create nested subgraphs
- **SUB-018**: Subgraph inherits viewport transforms
- **SUB-019**: Subgraph bounds calculation
- **SUB-020**: Subgraph z-index ordering

### Category 5: Node Addition/Removal (SUB-021 to SUB-025)
- **SUB-021**: Add node to subgraph updates parent reference
- **SUB-022**: Remove node from subgraph clears parent reference
- **SUB-023**: Add multiple nodes in batch
- **SUB-024**: Remove all nodes preserves container
- **SUB-025**: Remove last node deletes empty container

### Category 6: Nested Subgraphs (SUB-026 to SUB-029)
- **SUB-026**: Create subgraph within subgraph
- **SUB-027**: Drag node from parent to child subgraph
- **SUB-028**: Drag node from child to parent subgraph
- **SUB-029**: Collapse parent hides nested children

### Category 7: Edge Routing (SUB-030 to SUB-034)
- **SUB-030**: Edge internal to subgraph
- **SUB-031**: Edge crosses subgraph boundary
- **SUB-032**: Edge between nested subgraphs
- **SUB-033**: Edge updates when nodes reparented
- **SUB-034**: Edge routing respects collapsed state

## Design by Contract Obligations

### P1: Test Coverage
- All 34 tests must be implemented
- Each test must be independently runnable
- Tests must cover both happy path and edge cases

### P2: Error Handling
- Zero `unwrap()` calls in production code
- Zero `expect()` calls in production code
- Zero `panic!()` calls in production code
- All errors must be handled explicitly with `Result` types

### P3: Deterministic Behavior
- Tests must be reproducible
- No arbitrary timeouts
- No flaky race conditions
- Consistent ordering of operations

### P4: Isolation
- Each test must clean up state
- No shared state between tests
- Tests must run in any order

### P5: Performance
- Each test must complete in < 5 seconds
- No memory leaks
- No excessive allocations

## Pre-conditions

1. Rust test harness infrastructure exists (`diagram_tool/src/test_harness.rs`)
2. Playwright E2E test infrastructure exists (`diagram_tool/e2e/`)
3. Subgraph data model supports:
   - `parent: Option<NodeId>` field
   - `collapsed: Option<bool>` field
   - `z_index: i64` field
   - `locked: bool` field
4. Helper functions available:
   - `freshStart()`, `clearCanvasOverlays()`
   - `createTextNode()`, `nodeCount()`, `nodeFrameByLabel()`
   - `runEffect()`, `runEffectsSequential()`
   - `trapPageErrors()`, `waitForUiReady()`

## Post-conditions

1. All 34 tests pass
2. Code compiles without warnings (except allowed lints)
3. Zero `unwrap_used`, `expect_used`, `panic` violations
4. Test coverage report shows > 90% coverage for subgraph code
5. Performance benchmarks meet criteria

## Acceptance Criteria

1. **Functional Requirements**
   - All 34 tests execute successfully
   - Tests validate documented behavior
   - Edge cases are covered

2. **Quality Requirements**
   - Zero panics/unwraps/expects in production code paths
   - All test assertions are meaningful
   - Test names follow `given_X_when_Y_then_Z` convention

3. **Documentation Requirements**
   - Each test has a descriptive comment
   - Complex logic has inline documentation
   - Martin Fowler test patterns documented

4. **Integration Requirements**
   - Tests integrate with existing harness
   - Tests can run via `cargo test`
   - E2E tests can run via Playwright

## Test Implementation Locations

### Rust Tests (Unit/Integration)
- Location: `diagram_tool/src/ui/canvas/interaction_reducer.rs`
- Module: `mod subgraph_tests`
- Test count: ~20 tests (SUB-001 to SUB-005, SUB-015 to SUB-029)

### E2E Tests (Playwright)
- Location 1: `diagram_tool/e2e/diagram.subgraph-behavior.spec.ts`
- Test count: ~6 tests (SUB-006 to SUB-010)
- Location 2: `diagram_tool/e2e/diagram.subgraph-container-behavior.spec.ts`
- Test count: ~5 tests (SUB-011 to SUB-014)
- Location 3: `diagram_tool/e2e/diagram.edge-routing.spec.ts` (NEW)
- Test count: ~5 tests (SUB-030 to SUB-034)

## Out of Scope

- Visual regression testing (covered by separate bead)
- Performance stress testing (covered by perf tests)
- Persistence/serialization (covered by existing tests)
- Cross-browser compatibility testing

## Version History

- **2026-03-03**: Initial contract specification created
