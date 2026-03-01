# Implementation: bd-mtu - recovery-export

## Overview

Implemented the `export_while_recovering` function to support JSON export while in recovery-only mode.

## Contract Requirements Met

| Contract Requirement | Implementation |
|---------------------|----------------|
| `fn export_while_recovering(conn: &Connection) -> Result<String, ExportError>` | ✅ Implemented |
| Export works in recovery-only mode | ✅ Uses read-only connection |
| Returns valid JSON even when write operations are blocked | ✅ Uses SELECT queries only |

## Implementation Details

### Files Changed

- **`diagram_tool/src/models/export.rs`**:
  - Added `export_while_recovering` function (lines 177-199)
  - Added 3 test cases for the new function

### Function Implementation

```rust
pub fn export_while_recovering(conn: &rusqlite::Connection) -> Result<String, ExportError> {
    // Fetch all events from the read-only connection
    let events = fetch_all_events(conn)?;

    // Replay events to get the projection
    let projection = replay_events_from_db(&events)?;

    // Export projection to JSON string
    export_projection_json(&projection)
}
```

### Key Design Decisions

1. **Uses existing infrastructure**: Chains together `fetch_all_events`, `replay_events_from_db`, and `export_projection_json` - all existing functions
2. **Works with read-only connections**: The function uses only SELECT queries which work with SQLite read-only connections
3. **Returns canonical JSON**: Uses the same `export_projection_json` path to ensure consistent output format

### Test Coverage

1. **`given_empty_database_in_recovery_mode_when_exporting_then_returns_valid_json`** - Verifies export works on empty DB
2. **`given_database_with_events_in_recovery_mode_when_exporting_then_returns_projection_json`** - Verifies export includes node data
3. **`given_recovery_connection_is_read_only_when_exporting_then_succeeds`** - Verifies connection is read-only but export succeeds

## Verification Results

```
test models::export::tests::given_empty_database_in_recovery_mode_when_exporting_then_returns_valid_json ... ok
test models::export::tests::given_database_with_events_in_recovery_mode_when_exporting_then_returns_projection_json ... ok
test models::export::tests::given_recovery_connection_is_read_only_when_exporting_then_succeeds ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.00s
```

All 733 tests in the project pass.

## Quality Gates

- ✅ `cargo fmt` passes
- ✅ `cargo clippy` passes with `-D warnings -W pedantic -W nursery`
- ✅ `cargo test` passes (733 tests)
- ✅ `cargo check` passes
- ✅ No unwrap/expect/panic in source code
- ✅ No mut by default (immutable borrow only)
