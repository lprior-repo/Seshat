bead_id: bd-1jm
bead_title: history: Add can_undo and can_redo methods
phase: p2
updated_at: 2026-03-01T00:45:35Z

# Implementation: can_undo and can_redo

## Verification

Methods already exist in `diagram_tool/src/history.rs`:

```rust
#[must_use]
pub fn can_undo(&self) -> bool {
    !self.undo_stack.is_empty()
}

#[must_use]
pub fn can_redo(&self) -> bool {
    !self.redo_stack.is_empty()
}
```

## Acceptance Criteria Coverage
| Criteria | Status |
|----------|--------|
| can_undo() returns true if undo_stack is non-empty | ✅ |
| can_redo() returns true if redo_stack is non-empty | ✅ |
| Methods are O(1) time complexity | ✅ (just checks is_empty) |
| Methods do not modify state | ✅ (only reads) |
