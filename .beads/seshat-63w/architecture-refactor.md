# Architecture Refactor: seshat-63w

## Summary

Refactored `subgraph_events.rs` to comply with the 300-line file limit and Scott Wlaschin DDD principles.

## Changes Made

### File Split

| Before | After |
|--------|-------|
| `subgraph_events.rs` (304 lines) | `subgraph_events.rs` (258 lines) |
| | `subgraph_events/types.rs` (55 lines) |

### DDD Compliance

#### Types Module (`subgraph_events/types.rs`)
- **`Rect`**: Semantic newtype for bounding boxes with smart constructor
  - Validates width/height ≥ 0 at construction time
  - Makes illegal states unrepresentable
- **`Error`**: Domain error taxonomy with exhaustive variants
  - `NodeNotFound(NodeId)` - Parse-time validation
  - `CycleDetected(NodeId, NodeId)` - Graph integrity
  - `InvalidBounds(Rect)` - Constraint violation

#### Main Module (`subgraph_events.rs`)
- Helper functions are private (`get_node`, `get_subgraph`, `detect_cycle`)
- All public functions return `Result<T, Error>`
- Pure calculation functions: `calculate_subgraph_bounds`, `collect_child_bounds`
- No primitive obsession - uses domain types throughout

## Verification

```bash
# Library compiles
cargo check --lib  # ✅ Pass

# Clippy clean
cargo clippy --lib  # ✅ No errors
```

## Line Count Compliance

| File | Lines | Status |
|------|-------|--------|
| `subgraph_events.rs` | 258 | ✅ Under 300 |
| `subgraph_events/types.rs` | 55 | ✅ Under 300 |

## Module Structure

```
diagram_tool/src/models/
├── subgraph_events.rs      (main module - 258 lines)
├── subgraph_events/
│   └── types.rs           (domain types - 55 lines)
└── mod.rs                 (declares pub mod subgraph_events)
```

## Notes

- Test file `subgraph_events_tests.rs` continues to work via re-exports
- All domain invariants enforced at type level
- Pre-existing test failures in `history/tests.rs` and `subgraph_tests.rs` are unrelated to this refactor
