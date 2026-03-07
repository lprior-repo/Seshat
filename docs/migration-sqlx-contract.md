# Contract Specification: `sqlx` Unified Store Migration

## Context
- Feature: Migration from synchronous `rusqlite` to asynchronous `sqlx` for the entire `diagram_tool` backend storage.
- Domain terms: `DiagramProjection`, `EventEnvelope`, `SqlitePool`, `AsyncStoreError`.
- Assumptions:
  - SQLite is used with WAL mode and FULL synchronous durability.
  - The `store_async.rs` module already contains the majority of the `sqlx` logic, but must be generalized to replace `store.rs` completely.
  - The `models/` directory will exclusively take `&SqlitePool` or `&mut sqlx::Transaction`.
- Open questions:
  - Do we rename `AsyncStoreError` to `StoreError` after removing the synchronous version? (Yes).

## Preconditions
- [ ] P1: All file interactions (open/bootstrap) must receive a valid, writable path.
- [ ] P2: Appending an event must receive an `EventEnvelope` with a sequential revision number relative to the current state.
- [ ] P3: Snapshots written must match the current highest revision in the `events` table.

## Postconditions
- [ ] Q1: After `append_event`, the event is safely durable (fsynced) in the database via `sqlx`.
- [ ] Q2: After `bootstrap_store`, the database has WAL mode enabled, pragmas configured, and schema created.
- [ ] Q3: Writing a snapshot atomically replaces any older snapshot metadata without corrupting history.

## Invariants
- [ ] I1: Revision numbers are strictly monotonically increasing, starting from 1.
- [ ] I2: Every `op_id` in the `events` table is unique.
- [ ] I3: `sqlx::SqlitePool` manages all connections concurrently and safely without deadlocking.

## Error Taxonomy
- `StoreError::Sqlx(sqlx::Error)` - Database/connection level errors.
- `StoreError::Io(std::io::Error)` - File system access errors.
- `StoreError::RevisionMismatch { expected, found }` - Attempted to append out of order.
- `StoreError::Serialization(String)` - JSON payload could not be encoded/decoded.

## Contract Signatures
- `pub async fn bootstrap_store(db_path: &Path) -> Result<StoreBootstrap, StoreError>`
- `pub async fn append_event(pool: &SqlitePool, env: EventEnvelope, ...) -> Result<AppendResult, StoreError>`
- `pub async fn current_revision(pool: &SqlitePool) -> Result<i64, StoreError>`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Sequential Revision | Result error variant | `Result<T, StoreError::RevisionMismatch>` |
| Valid pool connection | Compile-time | `&SqlitePool` (ensures pool exists) |
| Transaction atomic | Compile-time | `&mut sqlx::Transaction<'_, sqlx::Sqlite>` |

## Violation Examples
- VIOLATES <P1>: `bootstrap_store(Path::new("/root/readonly"))` -- should produce `Err(StoreError::Sqlx(..))`
- VIOLATES <P2>: `append_event(..., envelope_with_gap_revision)` -- should produce `Err(StoreError::RevisionMismatch)`
- VIOLATES <Q1>: Hardware failure during await -- `Err(StoreError::Sqlx(..))`

## Ownership Contracts (Rust-specific)
- Shared borrow: `pool: &SqlitePool` -- used for reads and connection checkouts.
- Exclusive borrow: `tx: &mut sqlx::Transaction` -- used when modifying multiple tables (e.g. events + snapshots) atomically.
