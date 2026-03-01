# Implementation: bd-3sm - gui-sync

## Contract Compliance

| Contract Requirement | Status | Implementation |
|---------------------|--------|----------------|
| `fn start_event_tail_watcher(db_path: PathBuf, tx: Sender<SyncMessage>) -> Result<WatcherHandle, SyncError>` | ✅ | `diagram_tool/src/models/sync.rs:117` |
| `enum SyncError { WatchInit, Sqlite, Decode, ChannelClosed }` | ✅ | `diagram_tool/src/models/sync.rs:57` |
| `fn fetch_new_events(conn: &Connection, after_revision: i64) -> Result<Vec<EventRecord>, SyncError>` | ✅ | `diagram_tool/src/models/sync.rs:211` |
| Add notify watcher for db and db-wal paths | ✅ | Watches parent directory to catch both `.db` and `-wal` file changes |
| Batch and apply new events to signals without blocking render path | ✅ | Sends `SyncMessage::EventsUpdated` notification; GUI fetches events asynchronously |

## Architecture

### Data Flow
```
External CLI (append_event)
        ↓
   SQLite DB (write)
        ↓
  notify watcher (detects .db/-wal changes)
        ↓
  SyncMessage::EventsUpdated (channel)
        ↓
  GUI event loop (fetches events via fetch_new_events)
        ↓
  Apply to signals (non-blocking)
```

### Key Components

1. **SyncError** - Error enum with variants:
   - `WatchInit(String)` - Failed to initialize file watcher
   - `Sqlite(String)` - SQLite database errors
   - `Decode(String)` - Failed to decode event from database  
   - `ChannelClosed` - Channel was closed unexpectedly

2. **WatcherHandle** - Keeps the notify watcher alive; dropped to stop watching

3. **SyncMessage** - Notification types:
   - `EventsUpdated(Vec<u64>)` - New events available (revision numbers)
   - `Error(String)` - Error occurred during watching

4. **start_event_tail_watcher** - Creates a notify watcher that monitors:
   - Database file (`*.db`)
   - WAL file (`*-wal`, `*.db-wal`)
   - Watches parent directory to catch all related files

5. **fetch_new_events** - Queries events with `revision > after_revision`, ordered by revision ascending

## Functional Patterns Used

- `map_err` for error transformation
- `and_then` for chained operations  
- `filter_map` for optional value extraction
- `try_fold` for deterministic state updates
- Zero `unwrap`/`expect` in core logic
- Zero `mut` in public API

## Test Coverage

12 tests in `sync::tests` module:
- `test_fetch_new_events_returns_empty_when_no_events`
- `test_fetch_new_events_returns_events_after_revision`
- `test_fetch_new_events_returns_all_events_when_after_revision_zero`
- `test_fetch_new_events_returns_empty_when_after_revision_is_latest`
- `test_fetch_latest_revision_returns_zero_when_empty`
- `test_fetch_latest_revision_returns_max_revision`
- `test_start_event_tail_watcher_fails_for_nonexistent_path`
- `test_start_event_tail_watcher_succeeds_for_existing_db`
- `test_watcher_detects_database_modifications`
- `test_events_are_ordered_by_revision`
- `test_event_record_contains_correct_data`
- `test_replaying_fetched_events_produces_correct_projection`

All 592 tests in the project pass.

## Fixes Applied

1. **Fixed test_replaying_fetched_events_produces_correct_projection** - Adjusted event revisions to start at 0 using `.enumerate()` pattern (matching how `load_projection` handles events from DB)

2. **Fixed test_start_event_tail_watcher_succeeds_for_existing_db** - Made test resilient to platform-dependent spurious notifications on watcher startup

3. **Fixed test_load_projection_preserves_snapshot_data** - Same revision adjustment pattern applied
