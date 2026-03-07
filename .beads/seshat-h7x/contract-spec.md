# Contract Specification: Async Snapshot & Events Migration

## Context

- **Feature**: Port `diagram_tool/src/models/snapshot.rs` and `diagram_tool/src/models/events.rs` from sync `rusqlite` to async `sqlx`
- **Domain terms**:
  - **Snapshot**: Serialized `DiagramProjection` at a specific revision for fast recovery
  - **Tail replay**: Replaying events after a snapshot to reach current state
  - **Schema version**: Database schema version tracking
- **Assumptions**:
  - Async `store.rs` already exists with `SqlitePool` and `StoreError` patterns
  - Tests will use `#[tokio::test]` with in-memory or temp databases
  - All functions become async, accepting `&SqlitePool` or `&mut Transaction`
- **Open questions**: None

---

## Module: `models/snapshot.rs` (Async Port)

### Preconditions

#### P1: Valid pool reference
- **Enforcement**: Rust type system (compile-time)
- **Type**: `pool: &SqlitePool` - non-null reference guaranteed by Rust
- **Invariant**: Pool must be initialized and connected

#### P2: Projection revision matches current revision (write_snapshot)
- **Enforcement**: Runtime check returning `Result::Err`
- **Check**: `projection.revision == current_revision` from events table
- **Error**: `SnapshotError::SnapshotStale`

#### P3: Valid JSON serialization
- **Enforcement**: Runtime check returning `Result::Err`
- **Check**: `serde_json::to_string(projection)` succeeds
- **Error**: `SnapshotError::Serialization`

#### P4: Valid JSON deserialization
- **Enforcement**: Runtime check returning `Result::Err`
- **Check**: `serde_json::from_str(&payload)` succeeds
- **Error**: `SnapshotError::Serialization`

#### P5: Event envelope parsing
- **Enforcement**: Runtime check returning `Result::Err`
- **Check**: `parse_event_envelope(&payload)` succeeds
- **Error**: `SnapshotError::Serialization`

---

### Postconditions

#### Q1: write_snapshot creates valid snapshot
- **Enforcement**: Database constraint + transaction
- **Guarantee**: After successful return:
  - `snapshots` table contains row with `revision = projection.revision`
  - `payload` contains valid JSON
  - `created_at` is set to current timestamp
  - Transaction committed atomically

#### Q2: load_projection returns valid projection
- **Enforcement**: Transaction + replay validation
- **Guarantee**: After successful return:
  - `projection.revision == current_revision` from events table
  - All events after snapshot revision have been replayed
  - If no snapshot exists, full replay from revision 0

#### Q3: latest_snapshot returns None when no snapshots exist
- **Enforcement**: Query result handling
- **Guarantee**: Returns `Ok(None)` when `snapshots` table is empty

#### Q4: load_tail_events returns ordered events
- **Enforcement**: SQL ORDER BY clause
- **Guarantee**: Events returned in ascending revision order

---

### Invariants

#### I1: Revision monotonicity
- All events have monotonically increasing revision numbers
- Snapshots reference valid revisions

#### I2: Snapshot idempotency
- Writing same revision twice succeeds (INSERT OR REPLACE)
- No duplicate snapshot rows for same revision

#### I3: Transaction atomicity
- All write operations are atomic
- On failure, no partial state is committed

#### I4: Async non-blocking
- All I/O operations use `await`
- No blocking calls on async threads

---

### Error Taxonomy

```rust
pub enum SnapshotError {
    SnapshotStale { expected: u64, found: u64 },
    Serialization(String),
    Sqlx(String),
    Replay(String),
}
```

- **SnapshotStale**: Projection revision doesn't match current events revision
- **Serialization**: JSON encode/decode failure
- **Sqlx**: Database operation failure (wraps `sqlx::Error`)
- **Replay**: Event replay logic failure (wraps `ReplayError`)

---

### Contract Signatures

```rust
// Write snapshot at current revision
pub async fn write_snapshot(
    pool: &SqlitePool,
    projection: &DiagramProjection,
) -> Result<SnapshotMeta, SnapshotError>;

// Get latest snapshot metadata
pub async fn latest_snapshot(
    pool: &SqlitePool,
) -> Result<Option<SnapshotMeta>, SnapshotError>;

// Load projection from latest snapshot + tail replay
pub async fn load_projection(
    pool: &SqlitePool,
) -> Result<DiagramProjection, SnapshotError>;

// Load events after given revision
pub async fn load_tail_events(
    pool: &SqlitePool,
    after_revision: u64,
) -> Result<Vec<EventRecord>, SnapshotError>;
```

---

### Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Valid pool | Compile-time | `&SqlitePool` (Rust reference) |
| P2: Revision match | Runtime Result | `Result<T, SnapshotError::SnapshotStale>` |
| P3: Valid JSON encode | Runtime Result | `Result<T, SnapshotError::Serialization>` |
| P4: Valid JSON decode | Runtime Result | `Result<T, SnapshotError::Serialization>` |
| P5: Valid envelope | Runtime Result | `Result<T, SnapshotError::Serialization>` |

---

### Violation Examples (REQUIRED)

- **VIOLATES P2**: `write_snapshot(&pool, &stale_projection)` where `stale_projection.revision = 0` but current revision = 1 → `Err(SnapshotError::SnapshotStale { expected: 1, found: 0 })`

- **VIOLATES P3**: `write_snapshot(&pool, &projection)` with non-serializable projection (if manually constructed) → `Err(SnapshotError::Serialization("..."))`

- **VIOLATES P4**: `load_projection(&pool)` when snapshots table contains `'this is not json'` → `Err(SnapshotError::Serialization("..."))`

- **VIOLATES P5**: `load_tail_events(&pool, 0)` when events table contains malformed envelope → Warning logged, event skipped (no error, graceful degradation)

---

### Ownership Contracts

- **`pool: &SqlitePool`**: Shared immutable borrow - read/write through pool
- **`projection: &DiagramProjection`**: Shared immutable borrow - read for serialization
- **Returns `Result<T, SnapshotError>`**: No ownership transfer of inputs
- **Clones**: None required - all data is read-only

**Mutation Contract**: No `&mut` parameters - all writes go through pool transactions

---

---

## Module: `models/events.rs` (Async Port)

### Preconditions

#### P1: Valid pool reference
- **Enforcement**: Compile-time type system
- **Type**: `pool: &SqlitePool`

#### P2: Schema version compatibility
- **Enforcement**: Runtime check
- **Check**: Existing schema version matches expected version (1)
- **Error**: `StoreError::SchemaVersionMismatch` or `StoreError::MigrationForbidden`

---

### Postconditions

#### Q1: Schema tables created
- **Enforcement**: Transaction
- **Guarantee**: After `ensure_schema_v1`:
  - `events_schema_version` table exists
  - `events` table exists with proper schema
  - `snapshots` table exists with proper schema
  - All indexes created
  - Version record inserted

#### Q2: Idempotent schema creation
- **Enforcement**: Query check
- **Guarantee**: Calling `ensure_schema_v1` twice succeeds without error

---

### Invariants

#### I1: Single schema version
- Only one schema version row exists
- Version is immutable once created

#### I2: Migration forbidden
- No automatic migration between versions
- Explicit version mismatch errors

---

### Error Taxonomy

Uses existing `StoreError` from `store.rs`:

```rust
pub enum StoreError {
    SchemaVersionMismatch { expected: i32, found: i32 },
    MigrationForbidden { version: i32 },
    Sqlx(sqlx::Error),
    // ... other variants
}
```

---

### Contract Signatures

```rust
// Ensure schema v1 exists
pub async fn ensure_schema_v1(
    pool: &SqlitePool,
) -> Result<SchemaState, StoreError>;

// Read current schema state
pub async fn read_schema_state(
    pool: &SqlitePool,
) -> Result<SchemaState, StoreError>;
```

---

### Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Valid pool | Compile-time | `&SqlitePool` |
| P2: Version check | Runtime Result | `Result<T, StoreError::SchemaVersionMismatch>` |

---

### Violation Examples (REQUIRED)

- **VIOLATES P2 (higher)**: `ensure_schema_v1(&pool)` when schema version = 99 → `Err(StoreError::SchemaVersionMismatch { expected: 1, found: 99 })`

- **VIOLATES P2 (lower)**: `ensure_schema_v1(&pool)` when schema version = 0 → `Err(StoreError::MigrationForbidden { version: 0 })`

---

### Ownership Contracts

- **`pool: &SqlitePool`**: Shared immutable borrow
- **Returns `Result<T, StoreError>`**: No ownership transfer

**Mutation Contract**: No `&mut` parameters - schema writes through pool transactions

---

---

## Non-goals

- ❌ No schema migration logic (only v1 creation)
- ❌ No rusqlite compatibility layer (full replacement)
- ❌ No synchronous API (async only)
- ❌ No blocking operations in async context
