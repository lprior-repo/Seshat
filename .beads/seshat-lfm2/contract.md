# Contract Specification: seshat-lfm2 - Unit Tests for HIS-003..HIS-008

## Context

- **Bead ID**: seshat-lfm2
- **Title**: Write unit tests for HIS-003..HIS-008
- **Domain**: History (undo/redo) system for diagram editor
- **Dependencies**: `diagram_tool/src/history.rs`, `diagram_tool/src/core/history.rs`, `DiagramDocument`, `History`

### Domain Terms
| Term | Definition |
|------|------------|
| History | Persistent undo/redo stack using immutable Rpds data structures |
| undo_stack | Stack of document states that can be restored |
| redo_stack | Stack of document states that can be re-applied after undo |
| push | Add new document state to history (creates new timeline branch) |
| undo | Restore previous document state from undo_stack |
| redo | Restore next document state from redo_stack |

### Assumptions
1. History uses persistent (immutable) data structures - all operations return new History
2. History is bounded at MAX_HISTORY (100) entries
3. Each push creates exactly one history entry (not per-frame for drag/edit)
4. push after undo clears redo stack (new timeline branch)
5. Undo/redo preserve complete document state including nodes, edges, metadata

### Open Questions
- Should tests verify exact field-level restoration or just structural correctness?
- Are there any specific error conditions to test beyond empty stack scenarios?

## Preconditions

| ID | Precondition | Enforcement Level | Type/Pattern |
|----|--------------|-------------------|--------------|
| P1 | `history.push(doc)` requires valid `DiagramDocument` | Compile-time | `DiagramDocument` is already validated type |
| P2 | `history.undo(current)` requires non-empty undo_stack | Runtime | Returns `Option`, caller checks `.is_some()` |
| P3 | `history.redo(current)` requires non-empty redo_stack | Runtime | Returns `Option`, caller checks `.is_some()` |
| P4 | Document pushed must have valid revision | Compile-time | `Revision` type ensures valid state |

## Postconditions

| ID | Postcondition | Enforcement Level |
|----|---------------|-------------------|
| Q1 | `push` returns new History with doc added to undo_stack | Runtime - test verifies |
| Q2 | `push` clears redo_stack (new timeline branch) | Runtime - test verifies |
| Q3 | `undo` returns previous document from undo_stack | Runtime - test verifies |
| Q4 | `undo` moves current document to redo_stack | Runtime - test verifies |
| Q5 | `redo` returns next document from redo_stack | Runtime - test verifies |
| Q6 | `redo` moves current document to undo_stack | Runtime - test verifies |
| Q7 | History bounded at MAX_HISTORY (100) entries | Runtime - test verifies |
| Q8 | Each push creates exactly ONE history entry | Runtime - test verifies |

## Invariants

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| I1 | Undo stack contains documents in reverse chronological order | Test verifies order |
| I2 | Redo stack contains documents in chronological order | Test verifies order |
| I3 | After push: redo_stack is empty | Test verifies |
| I4 | After undo: can_redo() returns true | Test verifies |
| I5 | After redo: can_undo() returns true | Test verifies |

## Error Taxonomy

| Error Variant | Condition | Recovery |
|---------------|-----------|----------|
| None (Option-based) | undo_stack empty on undo | Caller checks `.is_none()` |
| None (Option-based) | redo_stack empty on redo | Caller checks `.is_none()` |

### HistoryError Enum (for apply_undo/apply_redo)

| Error Variant | Condition | Recovery |
|---------------|-----------|----------|
| HistoryError::EmptyUndoStack | apply_undo called with no history available | Caller checks `can_undo()` first |
| HistoryError::EmptyRedoStack | apply_redo called with no redo available | Caller checks `can_redo()` first |

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryError {
    /// Attempted to undo when undo_stack is empty
    EmptyUndoStack,
    /// Attempted to redo when redo_stack is empty
    EmptyRedoStack,
}
```

Note: The History API uses `Option` instead of `Result` since missing history is a valid/expected state (not an error). The `apply_undo` and `apply_redo` functions in `core/history.rs` convert to `Result<(), HistoryError>` for API consumers.

## Contract Signatures

```rust
// From diagram_tool/src/history.rs
impl History {
    pub fn new() -> Self;
    pub fn push(&self, doc: DiagramDocument) -> Self;
    pub fn undo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)>;
    pub fn redo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
}

// From diagram_tool/src/core/history.rs
// Note: Implementation uses &'static str for errors, not HistoryError enum
pub fn apply_undo(doc: &mut DiagramDocument, history: &mut History) -> Result<(), &'static str>;
pub fn apply_redo(doc: &mut DiagramDocument, history: &mut History) -> Result<(), &'static str>;
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|--------------|-------------------|----------------|
| undo_stack not empty | Runtime-checked | `history.undo()` returns `Option` |
| redo_stack not empty | Runtime-checked | `history.redo()` returns `Option` |
| Document valid | Compile-time | `DiagramDocument` is a validated struct |
| Revision valid | Compile-time | `Revision` is a newtype ensuring validity |

## Violation Examples

### P2 Violation: Undo on Empty History
- **VIOLATES_P2**: `History::new().undo(doc)` -- should produce `None` (not panic)

### P3 Violation: Redo on Empty Redo Stack
- **VIOLATES_P3**: After `push(A)` then `undo(A)` then `redo()`, calling `redo()` again -- should produce `None` (not panic)

### Q2 Violation: Push Does Not Clear Redo Stack
- **VIOLATES_Q2**: After undo creates redo entries, push should clear redo_stack. Violation: redo_stack not empty after push.

### Q8 Violation: Multiple Entries for Single Operation
- **VIOLATES_Q8**: Drag gesture that calls push per frame creates multiple entries. Violation: undo_stack.len() > 1 after single logical operation.

## Ownership Contracts

- **push(&self, doc: DiagramDocument)**: Takes owned `DiagramDocument`, caller loses ownership. Document cloned into immutable history stack.
- **undo(&self, current: DiagramDocument)**: Takes owned current, returns owned previous document and new History. No mutation - returns new immutable state.
- **redo(&self, current: DiagramDocument)**: Same as undo - ownership transfer pattern.
- **apply_undo/apply_redo**: Take `&mut` parameters - mutations postconditions: `doc` is replaced, `history` is replaced with new state.

## Non-goals

- [ ] Testing Kani formal verification properties (already covered in existing #[cfg(kani)] tests)
- [ ] Performance benchmarking (covered elsewhere)
- [ ] Concurrent access patterns (single-threaded UI context)
- [ ] Persistence/serialization of history (future feature)
