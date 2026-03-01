# Contract: bd-3sm - gui-sync

bead_id: bd-3sm
bead_title: gui-sync: add file-watch tail ingestion for external cli writes
phase: p0
updated_at: 2026-03-01T19:10:00Z

## Overview

Add file-watch tail ingestion for external CLI writes to keep GUI in sync.

## Preconditions

- SQLite connection is open with WAL enabled and synchronous FULL
- Rust Contract Signature: `fn start_event_tail_watcher(db_path: PathBuf, tx: Sender<SyncMessage>) -> Result<WatcherHandle, SyncError>`
- Rust Error Contract: `enum SyncError { WatchInit, Sqlite, Decode, ChannelClosed }`

## Postconditions

- Rust Postcondition Signature: `fn fetch_new_events(conn: &Connection, after_revision: i64) -> Result<Vec<EventRecord>, SyncError>`
- Accepted operations increment revision monotonically by exactly one
- Rejected operations return structured error codes without side effects

## Invariants

- Event log remains append-only and replay deterministic
- Idempotent operation IDs never produce duplicate durable mutations
- Human-authored operations keep priority over conflicting AI operations

## Implementation Tasks

### Phase 2: Implementation
- Add notify watcher for db and db-wal paths
- Batch and apply new events to signals without blocking render path

### Phase 4: Verification
- Run moon run :ci
