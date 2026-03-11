# Defects: seshat-o8u

## Black Hat Review Defect

### Issue
- **CONTRACT DEVIATION**: Contract specifies `ReplayError::NodeNotFound` but implementation returns `ReplayError::InvariantViolation`. Error type mismatch.

### Location
- **File**: `diagram_tool/src/models/projection/ops/node_ops.rs`
- **Function**: `apply_update_node_style`
- **Line**: ~421

### Contract Specification
- **Clause Q5**: Apply to non-existent node returns NodeNotFound error
- **Error Taxonomy**: ReplayError::NodeNotFound - Expected error for non-existent node test

### Implementation (Before Fix)
```rust
.ok_or_else(|| ReplayError::InvariantViolation(format!("node not found: {id}")))
```

### Contract Violation
- **VIOLATES Q5**: Expected: Err(ReplayError::NodeNotFound)
- Actual: Err(ReplayError::InvariantViolation("node not found: {id}"))

### Fix Applied
Changed error type to match contract:
```rust
.ok_or_else(|| ReplayError::NodeNotFound(format!("node not found: {id}")))
```

Also added `UpdateNodeStyle` case in `project_operation` with proper error mapping:
```rust
DomainOp::UpdateNodeStyle { id, style } => apply_update_node_style(projection, id, *style)
    .map_err(|e| match e {
        ReplayError::NodeNotFound(msg) => ProjectionError::NodeNotFound(msg),
        _ => ProjectionError::InvalidOperation(e.to_string()),
    })?,
```

### Status
- [x] Fixed in implementation
- [x] Updated implementation.md
- [ ] Verified in test suite (pre-existing tests check for error, not specific type)
