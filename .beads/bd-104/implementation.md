# Implementation: bd-104 - recovery-mode: add integrity check and read-only recovery workflow

## Files Changed

### diagram_tool/src/store.rs
- Added `RecoveryError` enum with variants: `CorruptDatabase`, `BackupUnavailable`, `Io`, `Sqlite`
- Added `IntegrityStatus` struct with fields: `is_valid`, `page_count`, `free_pages`, `corrupted_pages`, `schema_version`, `event_count`, `latest_revision`, `error_message`
- Added `RecoveryHandle` struct with fields: `conn`, `db_path`, `export_path`
- Added `startup_integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError>` function
- Added `open_recovery_mode(db_path: &Path) -> Result<RecoveryHandle, RecoveryError>` function
- Added `RecoveryHandle::export_to_json(output_path: &Path) -> Result<(), RecoveryError>` method

### diagram_tool/src/models/envelope.rs
- Fixed pre-existing borrow checker error in `parse_event_envelope` function

## Contract Fulfillment

### Preconditions (Contract Clause 1-2)
- ✅ `fn startup_integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError>` - Implemented
- ✅ `enum RecoveryError { CorruptDatabase, BackupUnavailable, Io, Sqlite }` - Implemented

### Postconditions (Contract Clause 3-4)
- ✅ `fn open_recovery_mode(db_path: &Path) -> Result<RecoveryHandle, RecoveryError>` - Implemented

### Invariants (Contract Clause 5-7)
- ✅ No migration path introduced
- ✅ No dual-write compatibility path
- ✅ All fallible operations use typed Result errors

## Implementation Details

### startup_integrity_check
- Checks if database file exists
- Opens database in read-only mode
- Runs SQLite `PRAGMA integrity_check`
- Returns detailed status including page counts, schema version, event count, latest revision
- Returns appropriate error message if invalid

### open_recovery_mode
- Opens database in read-only mode
- Performs basic validation that database is readable
- Returns RecoveryHandle for further operations

### RecoveryHandle::export_to_json
- Exports all events to JSON format
- Writes pretty-printed JSON to specified output path
- Updates export_path in handle

## Tests Added
- `test_startup_integrity_check_on_valid_database` - Verifies valid database passes integrity check
- `test_startup_integrity_check_on_nonexistent_database` - Verifies nonexistent database is marked invalid
- `test_open_recovery_mode_on_valid_database` - Verifies read-only mode opens successfully
- `test_recovery_handle_export_to_json` - Verifies JSON export functionality
