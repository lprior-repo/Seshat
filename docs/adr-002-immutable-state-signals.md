# ADR-002: Immutable State with Dioxus Signals

## Status
Accepted

## Date
2026-03-08

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

## Consequences

### Positive
- **Testable** - Pure functions easy to unit test
- **Reliable undo/redo** - History stores full document snapshots
- **No hidden state** - All changes visible in data flow
- **Concurrency-safe** - Immutable data has no race conditions
- **Time-travel debugging** - Can replay state changes

### Negative
- **Memory overhead** - Each change creates new document copy
- **Copy cost** - Large documents may be slow to copy
- **Learning curve** - Team unfamiliar with immutable patterns

### Mitigations
- Use `rpds` (persistent data structures) for efficient sharing
- `im::HashMap` for structural sharing
- Full snapshots for undo (bounded to 100 entries)
