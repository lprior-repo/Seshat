# ADR-007: Result<T, E> for Explicit Error Handling

## Status
Accepted

## Date
2026-03-08

## Context
The project requires zero panics in production code. All errors must be explicit, recoverable, and provide actionable user feedback.

## Decision
We will use **Result<T, E>** for all error handling with:
- **No unwrap()/expect()** - Denied at compile time via clippy
- **No panic!** - Forbidden in production code
- **Typed errors** - Each module defines its error enum
- **User-facing messages** - Errors include UI-ready text

## Error Taxonomy

```
SeshatError
├── SelectionError
│     ├── NodeNotFound(NodeId)
│     ├── EdgeNotFound(EdgeId)
│     └── EmptySelection
├── ClipboardError
│     ├── NothingToCopy
│     ├── EmptyClipboard
│     └── InvalidData(String)
├── HistoryError
│     ├── NothingToUndo
│     └── NothingToRedo
├── ValidationError
│     ├── DagCycle { source, target }
│     ├── InvalidParent { node, parent }
│     └── CircularParent { node }
├── ViewportError
│     ├── InvalidZoom(f64)
│     └── ZoomOutOfBounds { min, max, attempted }
└── StoreError
      ├── RevisionMismatch { expected, found }
      ├── ValidationFailed(String)
      └── Io(std::io::Error)
```

## Consequences

### Positive
- **No silent failures** - Every error must be handled
- **Type-safe** - Compiler enforces error handling
- **Testable** - Easy to assert error variants
- **User feedback** - Typed errors enable specific messages

### Negative
- **Boilerplate** - More code than unwrap()
- **Propigation** - Must use ? or match at call sites

### Clippy Enforcement

```rust
// lib.rs - compile-time enforcement
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
```

## Alternatives Considered
- **panic with catch** - Rejected: hides errors
- **Option<T>** - Rejected: loses error context
- **thiserror** - Used for derive macro convenience
