# ADR-002: Immutable State with Dioxus Signals (AMENDED)

## Status
Accepted (Amended 2026-03-15)

## Date
2026-03-08 (Original), 2026-03-15 (Amendment)

## Context
The MVP requires reliable basic operations (move, copy, paste, undo/redo) with comprehensive test coverage. Mutable state makes testing difficult and introduces subtle bugs.

## Decision
We will use **immutable data structures + Dioxus Signals** for state management.

## Architecture

```
┌─────────────────────────────────────────┐
│         User Action (immutable)         │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│    Pure Function (Data → New Data)     │
│    - No mutation                        │
│    - Returns new document               │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│         Signal.update()                 │
│    - Immutable replacement              │
│    - Dioxus auto-re-renders affected   │
└─────────────────────────────────────────┘
```

## Undo History (EXPLICIT SPECIFICATION)

### Memory Budget
- **Maximum memory allocation**: 512 MB
- **Maximum snapshot count**: 100 snapshots
- **Eviction policy**: FIFO (First-In-First-Out) - drop the oldest snapshot

### Eviction Trigger
Eviction occurs when snapshot count reaches 100. The oldest snapshot is discarded.

### Memory Calculation (Worst Case)
```
3000 nodes × ~500 bytes/node = ~1.5 MB per snapshot
100 snapshots × 1.5 MB = ~150 MB (well under 512 MB budget)
```

### Error Handling
```rust
pub enum HistoryError {
    NothingToUndo,
    NothingToRedo,
    /// History capacity exhausted - oldest entry evicted
    CapacityExhausted { evicted_revision: u64 },
}
```

### User Experience
- When history reaches 100 entries, the oldest undo step is silently dropped
- User can still undo the most recent 99 operations
- No user-facing warning for eviction (expected behavior)

## Consequences

### Positive
- **Testable** - Pure functions easy to unit test
- **Reliable undo/redo** - History stores full document snapshots
- **No hidden state** - All changes visible in data flow
- **Concurrency-safe** - Immutable data has no race conditions
- **Time-travel debugging** - Can replay state changes
- **Bounded memory** - 512 MB cap prevents OOM

### Negative
- **Memory overhead** - Each change creates new document copy
- **Copy cost** - Large documents may be slow to copy
- **Learning curve** - Team unfamiliar with immutable patterns
- **Limited undo depth** - 100 operations maximum

### Mitigations
- Use `rpds` (persistent data structures) for efficient sharing
- `im::HashMap` for structural sharing
- Full snapshots for undo (bounded to 100 entries)
- FIFO eviction is predictable and debuggable
