# Event Sourcing & Concurrency (`sqlx` + SQLite)

Seshat’s backend persistence is built on `sqlx` and SQLite in **WAL (Write-Ahead Log) mode**. We do not use ORMs (like Diesel or SeaORM) because we need absolute control over our event-sourcing JSON payloads.

## Why Event Sourcing?
Instead of a standard `UPDATE nodes SET x = 100` query, we append immutable events to an `events` table. 

```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    document_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    author TEXT NOT NULL,
    payload BLOB NOT NULL -- JSON
);
```

**Benefits:**
1. **Time Travel**: We can reconstruct the exact state of a diagram at any millisecond in history.
2. **Conflict Resolution**: Two users modifying the diagram don't lock the database. They both append events. Our CRDT logic reads the events and deterministic merges them.
3. **AI Context**: AI agents can read the *diffs* of how a diagram was built, rather than just staring at the final state.

## SQLite WAL Mode
By default, SQLite locks the entire database file during a write, blocking readers. In a multiplayer app with an 8ms frame budget, this is unacceptable.

We configure SQLite to use `PRAGMA journal_mode=WAL;`. This allows concurrent readers to continue reading the database seamlessly while a writer is appending a new event.

## `sqlx` and `tokio` (The Imperative Shell)
`sqlx` provides compile-time checked SQL queries and asynchronous I/O via the `tokio` runtime. 

**Constraints (As per ADR-006):**
- `sqlx` and `tokio` are strictly isolated to `#[cfg(not(target_arch = "wasm32"))]`.
- They live exclusively in the Tier 3 "Actions" layer (`store_sqlx.rs`).

### Example Action
```rust
#[cfg(not(target_arch = "wasm32"))]
pub async fn append_event(pool: &sqlx::SqlitePool, doc_id: &str, event: &CrdtOperation) -> anyhow::Result<()> {
    let payload = serde_json::to_vec(event)?;
    
    sqlx::query!(
        "INSERT INTO events (document_id, revision, op_id, author, payload, created_at) 
         VALUES (?, ?, ?, ?, ?, ?)",
        doc_id, event.revision, event.op_id.0, event.author.0, payload, event.hlc_timestamp
    )
    .execute(pool)
    .await?;
    
    Ok(())
}
```

Because `sqlx` uses macros that verify queries against the database at compile time, if a human or AI agent modifies the schema but forgets to update a query, **the pipeline fails to compile**. This is a massive safety net.