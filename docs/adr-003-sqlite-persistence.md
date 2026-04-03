# ADR-003: SQLite with sqlx for Persistence (AMENDED)

## Status
Accepted (Amended 2026-03-15)

## Date
2026-03-08 (Original), 2026-03-15 (Amendment)

## Context
Seshat needs reliable document persistence with support for 3000+ nodes, concurrent access, and event sourcing for sync capabilities.

## Decision
We will use **SQLite with sqlx** for persistence, configured with:
- **WAL mode** - Better concurrent read/write performance
- **Async connection pooling** - Non-blocking I/O with tokio
- **Event sourcing** - Store events, not just final state

## Schema Overview

```sql
-- Events table (append-only)
-- Note: id is auto-incrementing INTEGER, operation_id stores the unique op_id
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_id TEXT NOT NULL UNIQUE,
    revision INTEGER NOT NULL,
    payload TEXT NOT NULL,
    timestamp TEXT NOT NULL
);

-- Index for revision-ordered event retrieval
CREATE INDEX idx_events_revision ON events(revision);

-- Index for idempotency checks by operation_id
CREATE INDEX idx_events_operation_id ON events(operation_id);

-- Snapshots table for document state at given revision
CREATE TABLE snapshots (
    id INTEGER NOT NULL PRIMARY KEY,
    revision INTEGER NOT NULL UNIQUE,
    payload TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Index for revision-ordered snapshot retrieval (newest first)
CREATE INDEX idx_snapshots_revision ON snapshots(revision DESC);

-- AI Documents table for persistent document metadata
CREATE TABLE ai_documents (
    id TEXT NOT NULL PRIMARY KEY,
    key TEXT NOT NULL,
    json_payload TEXT NOT NULL,
    location_type TEXT NOT NULL,
    location_data TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

-- Index for key-based lookups
CREATE INDEX idx_ai_documents_key ON ai_documents(key);
```

## Serialization Format (EXPLICIT SPECIFICATION)

### Format: JSON
The `payload` TEXT column contains **JSON-encoded** operation data representing the full `EventEnvelope`.

**EventEnvelope Structure:**
```json
{
  "op_id": "unique-operation-id",
  "operation": { ... },
  "author": { "id": "...", "name": "...", "email": null },
  "timestamp": 1700000000
}
```

**Justification:**
- Schema is already defined in JSON for Dioxus serialization
- Human-readable for debugging and audits
- AI-friendly: easy to generate, parse, and test
- Wide tooling support
- The `operation` field contains the domain-specific operation (NodeAdd, EdgeConnect, etc.)

**Example payload:**
```json
{
  "op_id": "node-move-1700000000-abc123",
  "operation": {
    "NodeMove": {
      "id": "node-abc123",
      "x": 150.5,
      "y": 200.0
    }
  },
  "author": {
    "id": "user-1",
    "name": "Alice",
    "email": "alice@example.com"
  },
  "timestamp": 1700000000
}
```

## Conflict Resolution (EXPLICIT SPECIFICATION)

### Algorithm: CRDT (Conflict-Free Replicated Data Types)

We use **LWW-Element-Set (Last-Writer-Wins Element Set)** CRDT semantics.

**Why CRDT:**
- **Human-favoring**: Automatic merge without user intervention
- **AI-friendly**: Compositional, testable, deterministic
- **Offline-capable**: Works without network connectivity
- **Eventually consistent**: All replicas converge

### CRDT Implementation

```rust
/// Each operation carries a logical timestamp (Hybrid Logical Clock)
pub struct CrdtOperation {
    pub op_id: OpId,
    pub node_id: NodeId,
    pub author: AuthorId,
    pub hlc_timestamp: HlcTimestamp,  // Hybrid Logical Clock
    pub payload: OperationPayload,
}

/// Conflict resolution rule: highest HLC timestamp wins
/// Ties broken by author_id lexicographic comparison (deterministic)
pub fn resolve_conflict(a: &CrdtOperation, b: &CrdtOperation) -> CrdtOperation {
    match a.hlc_timestamp.cmp(&b.hlc_timestamp) {
        Ordering::Greater => a.clone(),
        Ordering::Less => b.clone(),
        Ordering::Equal => {
            // Tie-breaker: deterministic lexicographic comparison
            if a.author.0 < b.author.0 { a.clone() } else { b.clone() }
        }
    }
}
```

### Conflict Scenario Example

**Given:** Two users edit the same node at revision 5

```
User A (author: "alice"): 
  - Updates node position to (100, 200)
  - HLC timestamp: 2026-03-15T10:00:00.001Z

User B (author: "bob"):
  - Updates node position to (150, 250)  
  - HLC timestamp: 2026-03-15T10:00:00.002Z
```

**Resolution:**
- User B's change wins (later timestamp)
- No user intervention required
- Both replicas converge to same state

### Error Taxonomy for Conflicts

```rust
pub enum ConflictError {
    /// Revision mismatch - client must refresh and retry
    StaleRevision { expected: u64, found: u64 },
    /// CRDT merge failed (should never happen with correct implementation)
    MergeFailed { op_id: OpId, reason: String },
    /// Clock skew detected - HLC adjustment required
    ClockSkewDetected { local: HlcTimestamp, remote: HlcTimestamp },
}
```

## Consequences

### Positive
- **Proven reliability** - SQLite is battle-tested
- **ACID compliance** - Transaction support
- **WAL mode** - Concurrent reads during writes
- **Async** - sqlx provides non-blocking operations
- **Event sourcing** - Enables CRDT-based sync and automatic conflict resolution
- **Human-friendly** - JSON payloads are debuggable
- **AI-friendly** - CRDTs are compositional and testable

### Negative
- **Single-file** - Not distributed (acceptable for MVP)
- **File locking** - Concurrent writes need coordination
- **Migration complexity** - Schema changes require migrations
- **JSON overhead** - Larger than binary formats

### Risks
- 3000-node reads must complete in <16ms - requires indexing
- CRDT tombstones accumulate - need compaction strategy

## Alternatives Considered
- **redb** - Rejected in favor of sqlx's async support
- **PostgreSQL** - Overkill for MVP single-user
- **Bincode** - Rejected in favor of JSON debuggability
- **Operational Transformation** - Rejected due to complexity
- **Last-Write-Wins (naive)** - Rejected due to data loss risk
