# Architecture Refactor: document.rs Module Split

**Date:** 2026-03-15  
**Bead:** seshat-sr3b  
**Status:** REFACTORED

## Summary

Split the monolithic `document.rs` (1522 lines) into a properly modular structure with all files under 300 lines.

## Original Issues

1. **File Length Violation**: `document.rs` was 1522 lines, far exceeding the 300-line limit
2. **Module Cohesion**: Multiple domain concepts were co-located without clear separation

## Refactoring Actions

### Module Structure

Created `diagram_tool/src/models/document/` directory with the following files:

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 160 | Main re-exports, DiagramDocument struct and methods |
| `types.rs` | 274 | NewTypes: NodeId, EdgeId, AuthorId, Timestamp, Revision, OrderedFloat |
| `node.rs` | 252 | Node, NodeKind, LockState, NodeStyle, FontWeight |
| `edge.rs` | 208 | Edge, EdgeStyle, ArrowType, Point |
| `editor.rs` | 118 | EditorState, EditorTheme |
| `error.rs` | 43 | DocumentError taxonomy |
| `tests.rs` | 242 | DiagramDocument tests |
| `types_tests.rs` | 81 | Type validation tests |

### LockState Implementation (Preserved)

The LockState implementation was **already excellent DDD** and required no changes:

```rust
/// Lock state for nodes - makes illegal states unrepresentable
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum LockState {
    #[default]
    Unlocked,
    Locked,
}

impl LockState {
    pub fn is_locked(&self) -> bool {
        matches!(self, LockState::Locked)
    }

    pub fn is_movable(&self, node_kind: &NodeKind) -> bool {
        match node_kind {
            NodeKind::Subgraph => true,
            _ => !self.is_locked(),
        }
    }
}
```

**DDD Compliance:**
- ✅ Enum makes illegal states unrepresentable
- ✅ Explicit state transitions via methods
- ✅ Custom serde for backwards compatibility
- ✅ No primitive obsession

### Other DDD Patterns Preserved

All NewTypes have proper validation:
- `NodeId::try_new()` - validates non-empty
- `EdgeId::try_new()` - validates non-empty
- `AuthorId::try_new()` - validates non-empty
- `Timestamp::try_new()` - validates non-negative
- `OrderedFloat::new()` - validates not NaN/Infinity
- `Revision::increment()` - explicit state transition

## Files Modified

- Created: `diagram_tool/src/models/document/mod.rs`
- Created: `diagram_tool/src/models/document/types.rs`
- Created: `diagram_tool/src/models/document/node.rs`
- Created: `diagram_tool/src/models/document/edge.rs`
- Created: `diagram_tool/src/models/document/editor.rs`
- Created: `diagram_tool/src/models/document/error.rs`
- Created: `diagram_tool/src/models/document/tests.rs`
- Created: `diagram_tool/src/models/document/types_tests.rs`
- Deleted: `diagram_tool/src/models/document.rs` (original monolithic file)
- Updated: `diagram_tool/src/models/mod.rs` (changed to use directory)

## Test Results

All 37 tests pass:
- Document edge operations
- Node operations  
- Serialization roundtrips
- Type validations
- Error handling

## Compliance Status

| Requirement | Status |
|------------|--------|
| <300 lines per file | ✅ PASS |
| No primitive obsession | ✅ PASS |
| Explicit state transitions | ✅ PASS |
| Parse don't validate | ✅ PASS |
| Domain error taxonomy | ✅ PASS |
