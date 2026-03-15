# Black Hat Review Defects - seshat-sr3b

## BEAD: seshat-sr3b - LockState Enum Migration

### VERDICT: REJECTED ❌

---

## Phase 1: Contract & Bead Parity - FAILED

### Critical Defect

**Location**: `diagram_tool/src/models/multi_select_tests.rs:212`

```rust
fn test_p2_violation_returns_item_locked_error() {
    let mut doc = setup_doc();
    doc.document
        .nodes
        .get_mut(&NodeId::new("A".to_string()))
        .unwrap()
        .locked = true;  // <-- VIOLATION OF Q2 AND P2
```

**Contract Violations**:
- **Q2**: `Node.locked` field does NOT exist in Rust struct
- **P2**: Any code that writes `node.locked = <bool>` must use `node.lock_state = LockState::Locked/Unlocked` instead

**Details**:
- Test is under `#[cfg(kani)]` so only compiles for Kani model checker
- Test name suggests it's testing a "violation" but uses the OLD pattern
- Should use: `.lock_state = LockState::Locked;`

---

## Required Fix

**File**: `diagram_tool/src/models/multi_select_tests.rs`  
**Line**: 212

```rust
// BEFORE:
.locked = true;

// AFTER:
.lock_state = LockState::Locked;
```

---

## What Passed ✅

- LockState enum with Unlocked/Locked variants
- is_locked() method (lines 406-409 in document.rs)
- is_movable(&NodeKind) method (lines 413-419 in document.rs) - correctly handles Subgraph exception
- Custom serde for backwards-compatible JSON serialization
- Deserialization accepts legacy "locked": bool format
- Hash, Clone, Default derived correctly
- 199 uses of lock_state pattern throughout codebase are correct
- 16 uses of is_movable() are correct
- No remaining !node.locked patterns
- Node struct has lock_state field, NOT locked field

---

## Minor Issues (Non-blocking)

### Unused Imports (Warnings)
- `diagram_tool/src/layout/dag.rs:12` - unused LockState
- `diagram_tool/src/layout/grid.rs:9` - unused LockState  
- `diagram_tool/src/ui/commands/alignment.rs:8` - unused LockState, NodeKind
- `diagram_tool/src/ui/commands/distribution.rs:8` - unused LockState, NodeKind
- `diagram_tool/src/ui/commands/selection.rs:9` - unused LockState, NodeKind

### Acceptable: test_harness.rs NodeSpec
The `NodeSpec` struct has `locked: bool` - this is acceptable because:
- It's a test fixture struct, not the domain Node model
- It correctly serializes to JSON as "locked" for backwards compatibility

---

## Summary

The migration is 99% complete. One test file uses the old `.locked` pattern which violates the contract. Fix this one line and the migration is complete.
