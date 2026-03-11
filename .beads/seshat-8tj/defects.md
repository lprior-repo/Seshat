# 🔴 Black Hat Review: seshat-8tj

## 🔴 PHASE 1: Contract Violations

### Contract Parity
- **CONTRACT DEVIATION**: Contract specifies `project_operation(doc: &mut DiagramDocument, operation: &DomainOp)` but implementation uses `apply_update_label(state: DiagramProjection, id: &str, label: &str)`. The contract uses `DiagramDocument` and mutable reference; implementation uses `DiagramProjection` with persistent/immutable state.
- **CONTRACT DEVIATION**: Contract specifies `ProjectionError::TargetNotFound(String)` but implementation returns `ReplayError::InvariantViolation`. Error type mismatch.

### Test Coverage
- ✅ EARS-1 (UpdateLabel applies): Covered by `given_update_label_when_applying_then_updates_label`
- ✅ EARS-2 (Label replaced): Covered by same test
- ✅ EARS-3 (Error on missing target): Covered by `given_update_label_nonexistent_node_returns_error`  
- ✅ EARS-4 (Empty string valid): Covered by `given_update_label_with_empty_string_clears_label`
- ✅ Q1 (Label updated): Verified in multiple tests
- ✅ Q2-Q4 (Position/dimensions/other nodes unchanged): Verified
- ✅ Q5 (Revision incremented): Covered by `given_update_label_increments_revision`

**Verdict**: Tests are comprehensive. Implementation deviates from contract signature but matches implementation.md description.

---

## 🟠 PHASE 2: Farley Rigor Flaws

### Hard Constraints Violation

**FAIL**: `apply_update_label` at line 253-284 is **32 lines**, exceeds 25-line limit.

```rust
// Line 253-284: 32 lines total
pub fn apply_update_label(
    state: DiagramProjection,
    id: &str,
    label: &str,
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());  // Line 259

    // Check node exists  // Line 261
    let node = state     // Line 262
        .nodes
        .get(&node_id)
        .ok_or_else(|| ReplayError::InvariantViolation(format!("node not found: {id}")))?
        .clone();

    // Create updated node with new label, preserving all other fields  // Line 268
    let updated_node = Node {  // Line 269
        label: label.to_string(),  // Line 270
        ..node  // Line 271
    };  // Line 272

    let new_nodes = state.nodes.update(node_id, updated_node);  // Line 274

    Ok(DiagramProjection {  // Line 276
        version: state.version,  // Line 277
        revision: state.revision,  // Line 278
        nodes: new_nodes,  // Line 279
        edges: state.edges,  // Line 280
        author_priority: state.author_priority,  // Line 281
        cycle_policy: state.cycle_policy,  // Line 282
    })  // Line 283
}  // Line 284
```

### Parameter Count
- ✅ 3 parameters (state, id, label) - passes <5 limit

### I/O Separation
- ✅ Pure function, no I/O hidden inside

---

## 🟡 PHASE 3: Functional Rust Flaws (The Big 6)

| Requirement | Status |
|-------------|--------|
| Make illegal states unrepresentable | ✅ `ReplayError` enum handles all error cases |
| Parse, Don't Validate | ✅ `DomainOp::UpdateLabel` already parsed at boundary |
| Types as Documentation | ✅ Clear function signatures |
| Workflows as explicit state transitions | ✅ `apply_event` → `apply_operation` → `apply_update_label` |
| Newtypes | ✅ `NodeId` newtype used |

---

## 🔵 PHASE 4: Simplicity & DDD Failures

| Issue | Status |
|-------|--------|
| No unwraps | ✅ None in apply_update_label |
| No mut | ✅ Uses persistent `im::HashMap` |
| No Option-based state machines | ✅ Uses `Result<DiagramProjection, ReplayError>` |
| Primitive obsession | ✅ `NodeId` newtype wraps String |

---

## 🟣 PHASE 5: The Bitter Truth (Cleverness & Bloat)

- ✅ No clever one-liners
- ✅ No unreadable formatting  
- ✅ No YAGNI future-proofing
- ✅ Code is "painfully obvious" - follows exact same pattern as other node operations

---

## Verdict

**REJECTED** - The `apply_update_label` function exceeds the 25-line hard constraint (32 lines). While the implementation is functionally correct, well-tested, and follows functional Rust patterns, it violates Phase 2's hard constraint on function length. The function must be refactored to reduce line count, ideally by extracting some logic or using a macro/builder pattern similar to other operations in the codebase.

**Required Action**: Refactor `apply_update_label` to ≤25 lines while maintaining all behavior and tests.
