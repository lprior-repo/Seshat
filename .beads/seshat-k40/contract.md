# Contract Specification: seshat-k40

## Context
- **Feature**: Refactor `store_async.rs` to use Scott Wlaschin DDD types instead of primitive inputs
- **Domain terms**: ValidEvent, BoundedBatch, Revision, ValidOperationId, ValidTimestamp, ValidPayload
- **Assumptions**: 
  - DDD types in `diagram_tool/src/store/types.rs` are stable and validated
  - BoundedBatch constants MIN/MAX will be defined for typical batch operations (e.g., MIN=1, MAX=1000)
  - Parse-at-boundary pattern means validation happens at entry points, not inside the store
- **Open questions**:
  - What exact values for BoundedBatch MIN/MAX bounds should be used?
  - Should append_idempotent_async also be refactored?

## Preconditions

### P1: append_event_async - ValidEvent input
- **Requirement**: The `event` parameter must be a fully validated `ValidEvent` type
- **Enforcement Level**: Compile-time (type-level guarantee via `ValidEvent`)
- **Violation Example**: Passing raw primitives like `(op_id: String, timestamp: u64, payload: String)` directly to the function

### P2: append_event_async - expected_revision type
- **Requirement**: If provided, `expected_revision` must be a non-negative `Revision` (i64 >= 0)
- **Enforcement Level**: Compile-time (use `Option<Revision>` instead of `Option<i64>`)
- **Violation Example**: Passing `Some(-1)` as expected_revision should be rejected at type level

### P3: append_batch_async - BoundedBatch input
- **Requirement**: The `batch` parameter must be a `BoundedBatch<MIN, MAX>` where MIN <= len <= MAX
- **Enforcement Level**: Compile-time (BoundedBatch is a newtype that can only be constructed via TryFrom)
- **Violation Example**: Passing `Vec<ValidEvent>` directly without bounded construction

### P4: append_batch_async - batch non-empty
- **Requirement**: BoundedBatch guarantees non-empty (MIN >= 1)
- **Enforcement Level**: Compile-time (BoundedBatch MIN constraint enforced at construction)
- **Violation Example**: Attempting to create BoundedBatch with empty Vec<ValidEvent>

### P5: Boundary Parsing Contract
- **Requirement**: All call sites must parse primitives to DDD types at the boundary (entry point)
- **Enforcement Level**: Runtime-checked (callers must explicitly use TryFrom/TryInto to construct DDD types)
- **Violation Example**: Callers passing raw EventEnvelope or i64 without parsing

### P6: Pool validity
- **Requirement**: The `pool` parameter must be a valid, connected SqlitePool
- **Enforcement Level**: Runtime (pool connection validity checked by SQLx at runtime)
- **Violation Example**: Passing an uninitialized or closed pool

## Postconditions

### Q1: append_event_async returns valid AppendOutcome
- **Requirement**: On success, returns `AsyncAppendResult` with:
  - `revision` = previous_revision + 1 (monotonically increasing)
  - `op_id` matches the input ValidEvent's op_id
  - `timestamp` matches the input ValidEvent's timestamp
- **Enforcement Level**: Debug assertion + database constraint
- **Violation Example**: After append, fetching by op_id returns different revision

### Q2: append_batch_async returns valid AsyncBatchAppendResult
- **Requirement**: On success, returns `AsyncBatchAppendResult` with:
  - `start_revision` = previous_revision + 1
  - `end_revision` = start_revision + batch.len() - 1
  - `count` = batch.len()
  - `op_ids` = all op_ids from the batch in order
  - `last_timestamp` = timestamp of the last event in batch
- **Enforcement Level**: Debug assertion
- **Violation Example**: start_revision != expected or count mismatch

### Q3: Revision monotonicity
- **Requirement**: After any append operation, subsequent reads must show strictly increasing revisions
- **Enforcement Level**: Database transaction + SQL constraint (revision INTEGER NOT NULL)
- **Violation Example**: Gap detected in revision sequence

### Q4: Event record integrity
- **Requirement**: Stored event can be retrieved with matching op_id, revision, timestamp, and payload
- **Enforcement Level**: Database query validation
- **Violation Example**: Stored record differs from input

## Invariants

### I1: Store revision is always >= 0
- **Requirement**: revision column in events table is always non-negative
- **Enforcement**: Database schema constraint (INTEGER NOT NULL) + Revision::new() validation

### I2: No duplicate op_ids
- **Requirement**: operation_id column has UNIQUE constraint
- **Enforcement**: Database schema + unique constraint

### I3: BoundedBatch always respects MIN/MAX
- **Requirement**: Once constructed, BoundedBatch::len() is always in [MIN, MAX]
- **Enforcement**: TryFrom<Vec<ValidEvent>> guarantees this at construction time

## Error Taxonomy

| Error Variant | Condition | Recovery |
|---|---|---|
| AsyncStoreError::EmptyBatch | Batch size < MIN | Retry with valid batch |
| AsyncStoreError::BatchTooLarge | Batch size > MAX | Split into smaller batches |
| AsyncStoreError::InvalidTimestamp | ValidTimestamp::new() fails | Fix input timestamp |
| AsyncStoreError::InvalidOperationId | ValidOperationId::new() fails | Fix input op_id |
| AsyncStoreError::OperationIdTooLong | op_id.len() > 255 | Shorten operation ID |
| AsyncStoreError::PayloadTooLarge | payload.len() > 100MB | Reduce payload size |
| AsyncStoreError::RevisionMismatch | expected_revision != current_revision | Retry with correct revision |
| AsyncStoreError::RevisionGap | Non-sequential revision detected | Data integrity issue |
| AsyncStoreError::DuplicateWithConflict | Same op_id with different payload | Resolve conflict |
| AsyncStoreError::Serialization | JSON encode/decode fails | Fix envelope format |
| AsyncStoreError::Sqlx | Database operation fails | Check DB connection |

## Contract Signatures

### Original (Primitive-based)
```rust
pub async fn append_event_async(
    pool: &SqlitePool,
    envelope: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AsyncAppendResult, AsyncStoreError>

pub async fn append_batch_async(
    pool: &SqlitePool,
    ops: Vec<EventEnvelope>,
    expected_revision: Option<i64>,
) -> Result<AsyncBatchAppendResult, AsyncStoreError>
```

### Refactored (DDD Type-based)
```rust
pub async fn append_event_async(
    pool: &SqlitePool,
    event: ValidEvent,
    expected_revision: Option<Revision>,
) -> Result<AsyncAppendResult, AsyncStoreError>

pub async fn append_batch_async(
    pool: &SqlitePool,
    batch: BoundedBatch<MIN, MAX>,
    expected_revision: Option<Revision>,
) -> Result<AsyncBatchAppendResult, AsyncStoreError>
```

### Boundary Parsing Functions (New - for call sites)
```rust
/// Parse raw inputs into ValidEvent at boundary
pub fn parse_valid_event(
    op_id: String,
    timestamp: u64,
    payload: String,
) -> Result<ValidEvent, AsyncStoreError>

/// Parse raw inputs into BoundedBatch at boundary
pub fn parse_bounded_batch<MIN, MAX>(
    events: Vec<ValidEvent>,
) -> Result<BoundedBatch<MIN, MAX>, AsyncStoreError>

/// Parse raw revision input into Revision type
pub fn parse_revision(rev: i64) -> Result<Revision, AsyncStoreError>
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| ValidEvent required | Compile-time | `ValidEvent` type parameter |
| Batch non-empty | Compile-time | `BoundedBatch<MIN, MAX>` where MIN >= 1 |
| Batch size bounded | Compile-time | `BoundedBatch<MIN, MAX>` with MAX constraint |
| Revision non-negative | Compile-time | `Revision` (wraps i64, validates >= 0) |
| Timestamp non-zero | Compile-time | `ValidTimestamp` (wraps NonZeroU64) |
| op_id valid | Compile-time | `ValidOperationId` (validated at construction) |
| payload bounded | Compile-time | `ValidPayload` (max 100MB) |
| Pool valid | Runtime | SQLx connection validation |

## Violation Examples

### VIOLATES P1: Invalid event type passed
```rust
// Given: Raw primitive tuple
let bad_event = ("op_id".to_string(), 1700000000u64, "payload".to_string());
// When: Calling append_event_async with primitive
append_event_async(&pool, bad_event, None).await
// Then: Should produce compile error (type mismatch)
```

### VIOLATES P2: Negative revision passed
```rust
// Given: Option<Revision> with negative value
let bad_revision = Some(Revision::new(-1));
// When: Constructing Revision with negative value  
let result = Revision::new(-1);
// Then: Returns Err(AsyncStoreError::ValidationFailed("Revision cannot be negative"))
```

### VIOLATES P3: Unbounded batch passed
```rust
// Given: Raw Vec<ValidEvent> without bounded construction
let events = vec![valid_event1, valid_event2, /* ... 10000 events */];
// When: Calling append_batch_async directly with Vec
append_batch_async(&pool, events, None).await
// Then: Should produce compile error (type mismatch - needs BoundedBatch)
```

### VIOLATES P4: Empty batch construction
```rust
// Given: Empty Vec<ValidEvent>
let empty_events: Vec<ValidEvent> = vec![];
// When: Attempting to construct BoundedBatch
let result: Result<BoundedBatch<1, 1000>, _> = BoundedBatch::try_from(empty_events);
// Then: Returns Err(AsyncStoreError::EmptyBatch)
```

### VIOLATES Q1: Revision not incremented correctly
```rust
// Given: Store at revision 5, append event
let result = append_event_async(&pool, valid_event, None).await?;
// When: Checking returned revision
// Then: result.revision == 6 (not 5, not 7)
```

### VIOLATES Q2: Batch result mismatch
```rust
// Given: Store at revision 0, batch of 3 events
let batch: BoundedBatch<1, 1000> = BoundedBatch::try_from(vec![e1, e2, e3])?;
let result = append_batch_async(&pool, batch, None).await?;
// Then: result.count == 3 && result.start_revision == 1 && result.end_revision == 3
```

## Ownership Contracts

### append_event_async
- **Input**: `event: ValidEvent` - ownership transfer, caller gives up ownership
- **Rationale**: Event is consumed and stored; no need to retain original
- **Mutation**: None on caller's data (immutable reference to pool)

### append_batch_async
- **Input**: `batch: BoundedBatch<MIN, MAX>` - ownership transfer
- **Rationale**: Batch is consumed sequentially; no need to retain
- **Mutation**: None on caller's data

### Boundary Parsing Functions
- **Input**: Primitives (String, u64, Vec)
- **Output**: DDD types (ValidEvent, BoundedBatch, Revision)
- **Clone Policy**: Parsing creates new DDD types; callers should not expect original primitives to be preserved

## Non-goals
- [ ] Changing the underlying database schema
- [ ] Adding new async store functionality
- [ ] Modifying the envelope serialization format
- [ ] Supporting other databases beyond SQLite
