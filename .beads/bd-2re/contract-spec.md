# Contract Specification: bd-2re - Edge Binding Tests

**Bead ID**: bd-2re
**Title**: edges: Implement edge binding tests (EDG-001 to EDG-035)
**Status**: In Progress

## Overview

This bead implements comprehensive edge binding tests covering 35 test scenarios (EDG-001 to EDG-035) for the diagram tool's edge functionality. The tests verify edge creation, binding, routing, hit-testing, and advanced operations.

## Test Categories

### EDG-001 to EDG-010: Basic Edge Operations
- **EDG-001**: Create edge between two nodes
- **EDG-002**: Edge rejects self-loop (source == target) in DAG mode
- **EDG-003**: Edge rejects cycle-forming connections in DAG mode
- **EDG-004**: Edge creation updates document state
- **EDG-005**: Edge deletion removes from document
- **EDG-006**: Edge stores source and target node IDs
- **EDG-007**: Edge serializes/deserializes correctly
- **EDG-008**: Edge supports different arrow types (default, sharp, curved, step, straight)
- **EDG-009**: Edge supports different styles (solid, dashed, dotted)
- **EDG-010**: Edge label renders at correct position

### EDG-011 to EDG-020: Edge Binding and Selection
- **EDG-011**: Rotate node keeps edge binding
- **EDG-012**: Rotate selection with edges maintains bindings
- **EDG-013**: Resize selection with edges maintains bindings
- **EDG-014**: Clicking edge selects edge only, not nodes
- **EDG-015**: Edge endpoint follows node during drag
- **EDG-016**: Edge hit-testing is deterministic
- **EDG-017**: Overlapping edge hit-selection stays deterministic
- **EDG-018**: Thin horizontal edge remains selectable across zoom levels
- **EDG-019**: Thin vertical edge remains selectable across zoom levels
- **EDG-020**: Endpoint-near clicks keep selecting same edge endpoint

### EDG-021 to EDG-030: Edge Routing and Containers
- **EDG-021**: Edge between nodes in same container
- **EDG-022**: Edge crossing container boundary
- **EDG-023**: Reparent node with connected edge produces valid state
- **EDG-024**: Horizontal edge overlap hit-selection is deterministic
- **EDG-025**: Vertical edge overlap hit-selection is deterministic
- **EDG-026**: Curved edge hit-testing along bezier path
- **EDG-027**: Step-routed edge hit-testing at midpoint segments
- **EDG-028**: Sharp diagonal edge hit-testing along line
- **EDG-029**: Edge routing stable on overlapping nodes (horizontal)
- **EDG-030**: Edge routing stable on overlapping nodes (vertical)

### EDG-031 to EDG-035: Advanced Edge Operations
- **EDG-031**: Edge undo/redo maintains binding
- **EDG-032**: Edge copy/paste preserves properties
- **EDG-033**: Edge with custom bend points renders correctly
- **EDG-034**: Edge thickness variation renders correctly
- **EDG-035**: Edge color customization works

## Data Model

### Edge Structure
```typescript
interface Edge {
  source: NodeId;
  target: NodeId;
  label: string;
  style: EdgeStyle; // solid | dashed | dotted
  arrow_type: ArrowType; // default | sharp | curved | step | straight
  label_offset_t: OrderedFloat;
  color?: string;
  thickness: OrderedFloat;
  directed: boolean;
  bend_points: Point[];
  tags: string[];
  metadata: Record<string, unknown>;
  font_size?: OrderedFloat;
}
```

### NodeId and EdgeId
- Newtype pattern to prevent primitive obsession
- String-based identifiers with type safety
- Display and AsRef implementations

## Quality Requirements

### Zero Unwrap/Panic Policy
- **CRITICAL**: No `unwrap()`, `expect()`, or `panic!()` in production code
- All edge operations must use graceful error handling
- Test code may use unwrap with `#[allow(clippy::unwrap_used)]`

### Test Execution Requirements
1. All tests must execute successfully (exit code 0)
2. No console errors or page errors during test execution
3. Edge count must match expected values
4. Node count must match expected values
5. Selection behavior must be deterministic

### Coverage Requirements
- All 35 test scenarios (EDG-001 to EDG-035) must be implemented
- Tests must cover happy path and error cases
- Tests must verify edge bindings under node transformations
- Tests must verify hit-testing accuracy across zoom levels

## Verification Criteria

### Functional Requirements
- [x] Existing edge tests in `diagram.edges-and-routing.spec.ts` (38k, 11 tests)
- [x] Existing edge binding tests in `diagram.edge-binding-2.spec.ts` (12k, 5 tests)
- [ ] Additional tests needed for EDG-021 to EDG-035 coverage
- [ ] All tests pass with zero errors
- [ ] No unwrap/panic in production code paths

### Non-Functional Requirements
- Test execution time < 60 seconds per test file
- No flaky tests (deterministic behavior)
- Clear test names following EDG-XXX pattern
- Proper test isolation (fresh start per test)

## Acceptance Criteria

1. **Contract Adherence**: All tests follow the EDG-XXX naming convention
2. **Code Quality**: Zero `unwrap()`, `expect()`, `panic!` in production code
3. **Test Coverage**: All 35 test scenarios implemented
4. **Test Execution**: All tests pass with exit code 0
5. **Verification**: `.beads/bd-2re/verification.md` with test results
6. **Receipts**: `.beads/bd-2re/receipts.jsonl` with execution evidence

## References

- Test file 1: `/home/lewis/src/seshat/diagram_tool/e2e/diagram.edges-and-routing.spec.ts`
- Test file 2: `/home/lewis/src/seshat/diagram_tool/e2e/diagram.edge-binding-2.spec.ts`
- Model: `/home/lewis/src/seshat/diagram_tool/src/models/document.rs`
- Helpers: `/home/lewis/src/seshat/diagram_tool/e2e/helpers.ts`
