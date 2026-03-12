# Test Defects - Bead seshat-w1j

## Summary

**CRITICAL FINDING**: The martin-fowler-tests.md is a TEST PLAN WITHOUT ANY IMPLEMENTED TESTS. All referenced test files do not exist in the codebase. This is a complete failure to produce executable specifications.

---

## Critical Defects

### DEFECT-001: Test Files Do Not Exist
**Severity**: CRITICAL  
**Category**: Missing Implementation
**Doctrine Violated**: All (Testing Trophy, BDD, ATDD, TDD)

The test plan references files that don't exist:

| Planned File | Issue |
|-------------|-------|
| `diagram_tool/src/ui/dispatch/send/edge_connect_tests.rs` | File does not exist |
| `diagram_tool/src/ui/dispatch/send/edge_connect_integration_tests.rs` | File does not exist |
| `diagram_tool/src/ui/dispatch/send/edge_connect_property_tests.rs` | File does not exist |
| `diagram_tool/tests/edge_drawing_e2e_tests.rs` | File does not exist |
| `diagram_tool/src/ui/dispatch/test_helpers/edge_dsl.rs` | File does not exist |

**Impact**: Zero tests can be executed. Testing Trophy requires "Real Execution" - there is nothing to execute. This violates ALL doctrines:
- **Dan North BDD**: No executable specifications exist
- **Dave Farley ATDD**: Cannot evaluate separation of WHAT/HOW
- **Kent Beck TDD**: No tests to evaluate isolation
- **Testing Trophy**: No real execution possible

---

### DEFECT-002: Contract Function Does Not Exist
**Severity**: CRITICAL  
**Category**: Contract-Implementation Mismatch

The contract.md (lines 139-144) specifies:
```rust
pub fn handle_edge_drawing_complete(
    db_tx: Option<Coroutine<EventEnvelope>>,
    doc: &DiagramDocument,
    source_id: String,
    target_id: String,
) -> Result<DispatchResult, DispatchError>;
```

This function **does not exist** in the codebase. The martin-fowler-tests.md references this function in:
- Line 318: `handle_edge_drawing_complete(db_tx, doc, "", "node-2")`
- Line 325: `handle_edge_drawing_complete(db_tx, doc, "node-1", "")`
- Line 332: `handle_edge_drawing_complete(db_tx, doc, "nonexistent-node", "node-2")`
- Line 339: `handle_edge_drawing_complete(db_tx, doc, "node-1", "nonexistent-node")`

**Impact**: Tests cannot run because the function under test doesn't exist.

---

### DEFECT-003: Implementation Missing Preconditions
**Severity**: HIGH  
**Category**: Contract Violation

The implementation in `diagram_tool/src/ui/dispatch/send/edge.rs` (lines 21-57) is **missing validation** for:

| Precondition | Contract Requires | Implementation (edge.rs:21-57) |
|--------------|-------------------|--------------------------------|
| P1: Non-empty source | Return `EdgeNotFound` | ❌ Not checked |
| P2: Non-empty target | Return `EdgeNotFound` | ❌ Not checked |
| P3: Source exists in document | Return `EdgeNotFound` | ❌ Not checked |
| P4: Target exists in document | Return `EdgeNotFound` | ❌ Not checked |

Even if tests were implemented, they would FAIL against the current implementation because:
- `test_precondition_p3_source_exists_in_document` expects `Err(DispatchError::EdgeNotFound)` but would return `Ok(...)` 
- `test_precondition_p4_target_exists_in_document` expects `Err(DispatchError::EdgeNotFound)` but would return `Ok(...)`

**Evidence**: Looking at `dispatch_edge_connect` (edge.rs lines 21-57):
- Line 29 only checks `source == target` (self-loop), not empty strings
- No check for `doc.document.nodes.contains_key(&NodeId::new(source))`
- No check for `doc.document.nodes.contains_key(&NodeId::new(target))`

---

### DEFECT-004: DSL Layer Not Implemented
**Severity**: MEDIUM  
**Category**: ATDD Violation

Per Dave Farley ATDD: Tests must separate WHAT (intent) from HOW (implementation) via DSL.

The martin-fowler-tests.md (lines 28-48) describes a DSL:
```rust
fn doc_with_nodes(node_ids: &[&str]) -> DiagramDocument;
fn when_user_draws_edge(from: &str, to: &str) -> EdgeDrawingContext;
fn then_edge_operation_dispatched(ctx: &EdgeDrawingContext) -> DispatchResult;
```

This DSL module (`edge_dsl.rs`) **does not exist**. Tests directly call internal functions (`dispatch_edge_connect`, `handle_edge_drawing_complete`) exposing implementation details.

**Impact**: Tests are brittle and leak implementation details.

---

## Required Actions

1. **Implement test files** at the paths specified in martin-fowler-tests.md (Test File Locations table)
2. **Implement `handle_edge_drawing_complete`** function as specified in contract.md
3. **Add precondition validation** for P1-P4 in `dispatch_edge_connect`:
   - Check source is non-empty: `if source.is_empty() { return Err(DispatchError::EdgeNotFound); }`
   - Check target is non-empty: `if target.is_empty() { return Err(DispatchError::EdgeNotFound); }`
   - Check source exists: `if !doc.document.nodes.contains_key(&NodeId::new(source)) { return Err(DispatchError::EdgeNotFound); }`
   - Check target exists: `if !doc.document.nodes.contains_key(&NodeId::new(target)) { return Err(DispatchError::EdgeNotFound); }`
4. **Implement DSL layer** at `diagram_tool/src/ui/dispatch/test_helpers/edge_dsl.rs`

---

## Doctrinal Violations Summary

| Doctrine | Status | Evidence |
|----------|--------|----------|
| Dan North BDD | ❌ FAIL | No executable tests exist - cannot evaluate Given-When-Then |
| Dave Farley ATDD | ❌ FAIL | DSL not implemented (DEFECT-004), no test files exist |
| Kent Beck TDD | ❌ FAIL | No tests to evaluate isolation/determinism |
| Testing Trophy | ❌ FAIL | No real execution possible (DEFECT-001) |
| Combinatorial Coverage | ❌ FAIL | Tests don't exist; even if they did, implementation mismatches (DEFECT-003) |

---

## Verdict

**STATUS: REJECTED**

This test plan is a specification without implementation. Per Testing Trophy: "Run the REAL thing first" - there is nothing to run. Per Dave Farley: "Separate WHAT from HOW" - the DSL doesn't exist. Per Dan North: "Executable Specifications" - these are not executable. Per Kent Beck: Tests must be isolated and deterministic - none exist.

---

*Generated: 2026-03-11*  
*Reviewer: test-reviewer skill*