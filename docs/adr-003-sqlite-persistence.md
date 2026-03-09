# ADR-003: SQLite with sqlx for Persistence

## Status
Accepted

## Date
2026-03-08

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
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    document_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    op_id TEXT NOT NULL,
    author TEXT NOT NULL,
    payload BLOB NOT NULL,
    created_at INTEGER NOT NULL
);

-- Index for document revision lookup
CREATE INDEX idx_events_document_revision ON events(document_id, revision);
```

## Consequences

### Positive
- **Proven reliability** - SQLite is battle-tested
- **ACID compliance** - Transaction support
- **WAL mode** - Concurrent reads during writes
- **Async** - sqlx provides non-blocking operations
- **Event sourcing** - Enables git-like sync and conflict resolution

### Negative
- **Single-file** - Not distributed (acceptable for MVP)
- **File locking** - Concurrent writes need coordination
- **Migration complexity** - Schema changes require migrations

### Risks
- 3000-node reads must complete in <16ms - requires indexing
- Concurrent modification conflicts - handled via revision checking

## Alternatives Considered
- **redb** - Rejected in favor of sqlx's async support
- **PostgreSQL** - Overkill for MVP single-user
