bead_id: bd-38a
bead_title: sync-watcher: watch db and wal files for external writes
phase: p0
updated_at: 2026-03-01T20:20:00Z

# Contract: sync-watcher

## Overview

Implement a file watcher that monitors the SQLite database file (.db) and its WAL file (.db-wal) for external writes. When changes are detected, the watcher should emit sync tick events to notify the application that external CLI tools have modified the database.

## Preconditions

### System State
- Rust Contract Signature: `fn start_store_watcher(path: PathBuf) -> Result<WatcherHandle, SyncError>`
- Rust Error Contract: `enum SyncError { WatchInit, WatchRuntime, Io }`
- Legacy code path for this slice is identified and removable in one commit

### Required Inputs
- `path: PathBuf` - Path to the SQLite database file to watch

## Postconditions

### State Changes
- Rust Postcondition Signature: `fn stop_store_watcher(handle: WatcherHandle) -> Result<(), SyncError>`
- Legacy path is deleted or unreachable by compile-time guarantees
- Replacement path passes focused tests with no fallback to removed code

### Return Guarantees
- `start_store_watcher` returns `Ok(WatcherHandle)` on successful watcher initialization
- `start_store_watcher` returns `Err(SyncError::WatchInit)` if watcher cannot be initialized
- `start_store_watcher` returns `Err(SyncError::Io)` if the path doesn't exist or is inaccessible
- `stop_store_watcher` returns `Ok(())` when watcher is successfully stopped
- `stop_store_watcher` returns `Err(SyncError::WatchRuntime)` if watcher fails to stop cleanly

## Invariants

1. No migration path is introduced
2. No dual-write compatibility path exists
3. All fallible operations use typed Result errors
4. Zero unwrap/expect usage

## Implementation Tasks

1. Watch .db and .db-wal files with notify crate
2. Emit sync tick events on write activity

## Acceptance Criteria

- [ ] `start_store_watcher(path)` creates a watcher that monitors the database file and its WAL file
- [ ] Watcher detects modifications to both .db and .db-wal files
- [ ] `WatcherHandle` can be used to stop the watcher via `stop_store_watcher`
- [ ] All error paths return typed `SyncError` variants
- [ ] No unwrap or expect calls in implementation
- [ ] All tests pass via `moon run :ci`
