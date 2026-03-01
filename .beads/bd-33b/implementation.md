# Implementation: bd-33b - projection-replay

## Overview

Built a deterministic document projection replayer that replays events to produce a consistent DiagramProjection.

## Files Changed

### New File: `src/models/projection.rs`

Created new module with the following public API:

```rust
// Error enum per contract
pub enum ReplayError {
    InvalidEvent(String),
    InvariantViolation(String),
    UnsupportedVersion(u32),
}

// Event record for replay
pub struct EventRecord {
    pub op_id: String,
    pub revision: u64,
    pub operation: DomainOp,
    pub author: Author,
    pub timestamp: i64,
}

// Diagram projection - result of replaying events
pub struct DiagramProjection {
    pub version: u32,
    pub revision: u64,
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
    pub author_priority: HashMap<String, bool>,
}

// Main replay function per contract
pub fn replay_events(events: &[EventRecord]) -> Result<DiagramProjection, ReplayError>

// Apply single event per contract
pub fn apply_event(state: DiagramProjection, event: &EventRecord) -> Result<DiagramProjection, ReplayError>

// Conversion helpers
pub fn projection_to_document(projection: &DiagramProjection) -> DiagramDocument
pub fn document_to_projection(document: &DiagramDocument) -> DiagramProjection
```

### Modified: `src/models/mod.rs`

Added `pub mod projection;` to expose the new module.

## Contract Fulfillment

### Preconditions ✓
- `replay_events` takes `&[EventRecord]` as input
- `ReplayError` enum has exactly `InvalidEvent`, `InvariantViolation`, `UnsupportedVersion` variants

### Postconditions ✓
- `apply_event` takes `DiagramProjection` and `&EventRecord`, returns `Result<DiagramProjection, ReplayError>`
- Accepted operations increment revision by exactly one (verified in tests)
- Rejected operations return errors without side effects (verified in tests)

### Invariants ✓
- Event log is append-only: replay validates revision sequence is monotonically increasing
- Deterministic replay: multiple replays of same events produce identical results (tested)
- Idempotent op IDs: duplicate op_id returns error (tested)
- Human-authored operations priority: `author_priority` map tracks is_human per op_id

## Tests

17 tests covering:
- Empty events replay
- Single node add
- Multiple events with revision increment
- Revision gap detection
- Duplicate node ID detection
- Node move on nonexistent node error
- Edge connect to nonexistent source error
- Apply event with wrong revision error
- Duplicate op_id error
- Human vs AI author priority tracking
- Deterministic replay verification
- Projection <-> Document conversion
- Node delete cascades to connected edges
- Edge disconnect

## Design Decisions

1. **Pure Functions**: All core functions are pure - no I/O, no mutation
2. **Persistent Data Structures**: Uses `im::HashMap` for immutable updates
3. **No Unwrap/Expect**: All fallible operations use `Result<T, E>` with proper error handling
4. **No Mut by Default**: Uses functional patterns (fold, clone + update)
5. **Author Priority**: Tracks human vs AI authors for conflict resolution per contract
