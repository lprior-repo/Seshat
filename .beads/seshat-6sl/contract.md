# Contract Specification: Store Module Strict DDD Refactoring

## Context
- **Feature**: Refactor `diagram_tool/src/store.rs` (4000+ lines) into a strict DDD-compliant module structure where all files are < 300 LOC.
- **Domain terms**: Event Store, Revision, OperationId, Payload, OCC, EventTimestamp, Append, Recovery.
- **Assumptions**: The SQLite database runs locally, filesystem supports WAL, single active writer pattern is preferred.
- **Open questions**: Maximum safe batch size is set to 1000.

## 1) Refactor Contract (Target Invariants + States)
- **Parse, Don't Validate**: All raw inputs (Strings, i64, u64) are parsed into strictly constrained domain types (`ValidOperationId`, `ValidTimestamp`, `BoundedBatch`, `ValidPayload`, `Revision`) at the boundary.
- **Illegal States Unrepresentable**: A database connection cannot be locked and writeable at the same time. Recovery mode is represented by a strictly read-only typestate.
- **Explicit Workflows**: Appending is a state transition `State<Connected> -> State<AppendSuccess> | State<AppendFailed>`. Batch atomicity means transactions never partially commit.

## Preconditions
- [x] P1: Batch size must be strictly > 0 and <= MAX_BATCH_SIZE (1000).
- [x] P2: `EventTimestamp` must be strictly positive (not 0).
- [x] P3: `OperationId` must not be empty (e.g., `""`), must not be whitespace-only, must not contain null bytes, and must not exceed 255 characters.
- [x] P4: The database connection must not be locked by another process (SQLITE_BUSY).
- [x] P5: `RecoverySession` must NOT be capable of write operations.
- [x] P6: `ValidPayload` handles edge cases explicitly (empty string `""` is allowed, excessively large strings up to SQLite limits are correctly bounded/validated).

## Postconditions
- [x] Q1: After a successful append, the database `Revision` is strictly incremented by exactly the batch size. 
- [x] Q2: Events appended are immediately retrievable in the exact order they were submitted, without gaps.
- [x] Q3: `OperationId` idempotency:
      1) Same `OperationId`, Same Payload -> Success (No-op, no state change).
      2) Same `OperationId`, Different Payload -> `DuplicateWithConflict`.
      3) Different `OperationId`, Same Payload -> Success (New event appended).
- [x] Q4: Mid-Batch Failure Atomicity: If a batch of `N` events encounters an error on the `k`-th event, the ENTIRE batch transaction is rolled back and no partial writes occur.
- [x] Q5: Exact Boundary Success: A batch can successfully append exactly up to and landing on `i64::MAX` without an off-by-one boundary error.

## Invariants
- [x] I1: Revisions are strictly monotonically increasing, gapless, and never exceed `i64::MAX`.
- [x] I2: Schema version is strictly checked upon initialization and mismatched/unmigratable versions are rejected.

## 2) Typed Model Diffs (Before/After)
**Before (Primitive Obsession & Boolean Soup):**
```rust
pub fn append_event(conn: &Connection, op_id: &str, timestamp: u64, payload: String) -> Result<i64, StoreError>;
```

**After (Strict DDD Types):**
```rust
pub fn append_batch(
    session: &mut ReadWriteSession, 
    events: BoundedBatch<1, 1000>
) -> Result<Revision, StoreError>;
```

## 3) Boundary Parsing Plan & Type Encoding
All primitive parameters must be parsed at the CLI/HTTP boundary before entering the core domain logic:
| Domain Concept | Primitive | Parsed Domain Type | Enforcement Level |
|---|---|---|---|
| Batch size | `Vec<T>` | `BoundedBatch<1, 1000>` | Compile-time |
| Timestamp | `u64` | `ValidTimestamp` | Constructor Result |
| Operation ID | `String` | `ValidOperationId` | Constructor Result |
| Payload | `String` | `ValidPayload` | Constructor Result |
| Recovery Mode | `&Connection` | `ReadOnlySession` | Compile-time (Typestate) |

## 4) Transition Map
- `[Raw Input] --(parse)--> [Domain Types]`
- `[ValidBatch, ReadWriteSession] --(append)--> [Revision] | [StoreError]`

## 5) Error Taxonomy
- `StoreError::EmptyBatch` - when batch size is 0.
- `StoreError::BatchTooLarge` - when batch size exceeds 1000.
- `StoreError::InvalidOperationId` - when OperationId is empty, whitespace-only, or contains null bytes.
- `StoreError::OperationIdTooLong` - when OperationId exceeds the 255 character limit.
- `StoreError::InvalidTimestamp` - when EventTimestamp is 0 or unrepresentable.
- `StoreError::RevisionGap` - when a write would create a non-sequential revision.
- `StoreError::DuplicateWithConflict` - when an OperationId is reused for a different payload.
- `StoreError::RevisionOverflow` - when appending any part of a batch would cause the revision to exceed `i64::MAX`.
- `StoreError::DatabaseLocked` - when SQLite is locked by another process.
- `StoreError::ReadOnlyViolation` - when a write is attempted on a read-only RecoverySession (if typestate is bypassed).
- `StoreError::SchemaVersionMismatch` - when opening a database with a schema version that cannot be safely processed.
- `StoreError::PayloadTooLarge` - when the payload exceeds predefined boundaries.

## 6) Violation Examples (REQUIRED)
- VIOLATES P1: `BoundedBatch::try_from(vec![])` -- should produce `Err(StoreError::EmptyBatch)`
- VIOLATES P1: `BoundedBatch::try_from(vec![...1001 items])` -- should produce `Err(StoreError::BatchTooLarge)`
- VIOLATES P2: `ValidTimestamp::new(0)` -- should produce `Err(StoreError::InvalidTimestamp)`
- VIOLATES P3: `ValidOperationId::new("")` -- should produce `Err(StoreError::InvalidOperationId)`
- VIOLATES P3: `ValidOperationId::new("   ")` -- should produce `Err(StoreError::InvalidOperationId)`
- VIOLATES P3: `ValidOperationId::new("id\0test")` -- should produce `Err(StoreError::InvalidOperationId)`
- VIOLATES P3: `ValidOperationId::new("a".repeat(256))` -- should produce `Err(StoreError::OperationIdTooLong)`
- VIOLATES P4: `append_batch()` while SQLite locked -- should produce `Err(StoreError::DatabaseLocked)`
- VIOLATES P5: Calling `.append_batch()` on `ReadOnlySession` -- Compile Error (method doesn't exist). Runtime bypass produces `Err(StoreError::ReadOnlyViolation)`.
- VIOLATES P6: `ValidPayload::new("a".repeat(100 * 1024 * 1024))` -- should produce `Err(StoreError::PayloadTooLarge)`
- VIOLATES Q1: Revision skips a number after successful insert -- `proptest` validation failure.
- VIOLATES Q4: `append_batch()` with `[ValidEvent, ConflictingEvent, ValidEvent]` -- returns `Err(StoreError::DuplicateWithConflict)` and first event is NEVER written.
- VIOLATES I1: Manual SQL injection deleting a row -- subsequent read should produce `Err(StoreError::RevisionGap)`.
- VIOLATES I1: `append_batch()` with 2 events when current revision is `i64::MAX - 1` -- should produce `Err(StoreError::RevisionOverflow)` and rollback (state is unchanged).
- VIOLATES I2: Opening database with schema version 999 -- should produce `Err(StoreError::SchemaVersionMismatch)`.
- VIOLATES Q3: Append same `OperationId` with different payload -- should produce `Err(StoreError::DuplicateWithConflict)`.

## 7) Ownership Contracts
- `BoundedBatch` takes ownership of events to consume them, requires exclusive borrow `&mut ReadWriteSession` to serialize writes.
- `ReadOnlySession` provides only shared borrows `&self` for reading. No mutation methods exposed.

## 8) Migration Notes
- Split `store.rs` into `store/mod.rs`, `store/types.rs`, `store/error.rs`, `store/append.rs`, `store/session.rs`, and `store/read.rs` to keep files under 300 LOC.