# ADR-004: Persistent Data Structures for History

## Status
Accepted

## Date
2026-03-08

## Context
Undo/redo is a core MVP requirement. Users expect "perfect inverse" restoration - the document must be exactly as it was before.

## Decision
We will use **rpds (Persistent Data Structures)** for history management:
- **List<DiagramDocument>** for undo stack
- **List<DiagramDocument>** for redo stack
- **100 entry limit** - Bounded memory usage
- **Full snapshots** - Each entry is complete document state

## Implementation

```rust
pub struct History {
    undo_stack: List<DiagramDocument>,  // Reverse chronological
    redo_stack: List<DiagramDocument>,  // Chronological
}

impl History {
    pub fn push(&self, doc: DiagramDocument) -> Self {
        // New entry + clear redo = new timeline
        Self {
            undo_stack: self.undo_stack.push_front(doc),
            redo_stack: List::new(),
        }
    }
    
    pub fn undo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        // Pop from undo, push current to redo
        self.undo_stack.first().map(|prev| (prev.clone(), Self {
            undo_stack: drop_first(&self.undo_stack),
            redo_stack: self.redo_stack.push_front(current),
        }))
    }
}
```

## Consequences

### Positive
- **Perfect inverse** - Full snapshot guarantees exact restoration
- **Immutable** - No mutation bugs
- **Efficient sharing** - rpds uses structural sharing
- **Simple** - No command pattern needed

### Negative
- **Memory** - Full document copy per entry
- **Large doc cost** - 3000 nodes × 100 entries = significant memory

### Mitigations
- Bound to 100 entries (configurable)
- Consider delta compression post-MVP
- Monitor memory with large diagrams
