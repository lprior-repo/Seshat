# seshat-c0j Defects - REJECTED

## Critical Contract Violations (PHASE 1)

### 1. Function Signature Mismatch
- **Contract**: `pub fn project_operation(doc: &mut DiagramDocument, operation: &DomainOp) -> Result<(), ProjectionError>`
- **Implementation**: `pub fn apply_node_resize(state: DiagramProjection, id: &str, width: f64, height: f64) -> Result<DiagramProjection, ReplayError>`
- **Severity**: BLOCKING - API contract broken

### 2. Error Type Mismatch
- **Contract specifies**: `ProjectionError` enum with variants:
  - `NodeNotFound(String)`
  - `InvalidDimensions(String)`  
  - `InvalidOperation(String)`
- **Implementation uses**: `ReplayError` with:
  - `InvariantViolation(String)` for missing node
  - `InvalidEvent(String)` for invalid dimensions
- **Severity**: BLOCKING - error taxonomy doesn't match contract

### 3. Missing Test Cases (Contract Violation Examples)
The contract explicitly requires tests for these cases:
- `width = f64::NAN` - NOT TESTED in projection tests
- `width = f64::INFINITY` - NOT TESTED in projection tests  
- Tests only cover negative width (-10.0)
- **Severity**: BLOCKING - test parity broken

### 4. Missing Error Type Verification
- **Contract requires**: Verify specific `ProjectionError::InvalidDimensions` variant returned
- **Implementation**: Tests only assert `result.is_err()` without checking error type
- **Severity**: BLOCKING

---

## Farley Rigor Violations (PHASE 2)

### 5. Function Exceeds 25-Line Limit
- **Location**: `diagram_tool/src/models/projection/ops/node_ops.rs:193-251`
- **Actual**: 59 lines
- **Limit**: 25 lines
- **Severity**: BLOCKING - hard constraint violated

---

## Required Fixes

1. Create `ProjectionError` enum matching contract spec
2. Rename function to `project_operation` with contract signature
3. Add tests for NaN and Infinity dimensions
4. Add error type assertions in tests (not just `is_err()`)
5. Refactor `apply_node_resize` to ≤25 lines - extract validation into separate function
