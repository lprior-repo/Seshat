bead_id: bd-38a
bead_title: sync-watcher: watch db and wal files for external writes
phase: p1
updated_at: 2026-03-01T20:35:00Z

# Implementation: sync-watcher

## Summary

Implemented contract-compliant file watcher functions for monitoring SQLite database files and their WAL files for external writes.

## Changes Made

### File: `/home/lewis/src/seshat/diagram_tool/src/models/sync.rs`

#### 1. Updated `SyncError` enum

Added new error variants to match the contract:
- `WatchInit` - Unit variant for watcher initialization failures
- `WatchRuntime` - Unit variant for runtime errors
- `Io(String)` - I/O error variant for file access issues

Added `From<io::Error>` implementation for `SyncError`.

#### 2. Updated `WatcherHandle` struct

Added fields:
- `active: Arc<AtomicBool>` - Flag to track if the watcher is still active
- `watch_path: PathBuf` - The path being watched (for unwatch)

Added method:
- `is_active()` - Check if the watcher is still active

#### 3. Added `start_store_watcher` function

Contract-compliant function:
```rust
pub fn start_store_watcher(path: PathBuf) -> Result<WatcherHandle, SyncError>
```

Features:
- Verifies database file exists
- Uses `notify` crate for file watching
- Watches both `.db` and `.db-wal` files
- Returns `SyncError::Io` for missing files
- Returns `SyncError::WatchInit` for watcher initialization failures

#### 4. Added `stop_store_watcher` function

Contract-compliant function:
```rust
pub fn stop_store_watcher(handle: WatcherHandle) -> Result<(), SyncError>
```

Features:
- Sets active flag to false
- Unwatches the path
- Returns `SyncError::WatchRuntime` on failure

#### 5. Updated `start_event_tail_watcher` function

- Updated to use new error variants
- Added active flag tracking
- Added watch_path field

## Tests Added

- `test_start_store_watcher_fails_for_nonexistent_path` - Verifies Io error for missing files
- `test_start_store_watcher_succeeds_for_existing_db` - Verifies successful watcher creation
- `test_stop_store_watcher_succeeds` - Verifies successful watcher stop
- `test_watcher_handle_is_active_flag` - Verifies active flag functionality

## Test Results

All 16 sync module tests pass:
- 12 existing tests (updated for new error variants)
- 4 new tests for contract-compliant functions

All 719 project tests pass.

## Contract Compliance

| Contract Requirement | Status |
|---------------------|--------|
| `fn start_store_watcher(path: PathBuf) -> Result<WatcherHandle, SyncError>` | Implemented |
| `enum SyncError { WatchInit, WatchRuntime, Io }` | Implemented |
| `fn stop_store_watcher(handle: WatcherHandle) -> Result<(), SyncError>` | Implemented |
| Watch .db and .db-wal files | Implemented |
| Emit sync tick events on write activity | Implemented |
| Zero unwrap/expect usage | Verified |
| All fallible operations use typed Result errors | Verified |
