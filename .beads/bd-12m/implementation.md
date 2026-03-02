# Implementation: bd-12m - edge-case-bdd-tests-error-handling

## Metadata
- bead_id: bd-12m
- bead_title: edge-case-bdd-tests-error-handling
- phase: p1
- updated_at: 2026-03-02T04:50:00Z

## Summary

Added comprehensive BDD-style tests for 8 error paths in the storage layer:
1. InvalidPragma
2. SchemaVersionMismatch
3. MigrationForbidden
4. RevisionMismatch
5. RevisionGap
6. EmptyBatch
7. CorruptDatabase
8. BackupUnavailable

## Changes Made

### File: `diagram_tool/src/store.rs`

Added 20 new tests in the `tests` module, organized by error type:

#### InvalidPragma Error Path Tests (4 tests)
- `test_invalid_pragma_wal_mode_error_construction` - Verifies error message for WAL mode issues
- `test_invalid_pragma_synchronous_mode_error_construction` - Verifies error message for synchronous mode issues
- `test_invalid_pragma_error_display` - Verifies generic error display
- `test_invalid_pragma_readonly_database` - Tests pragma behavior on read-only database

#### SchemaVersionMismatch Error Path Tests (3 tests)
- `test_map_error_code_invalid_pragma` - Maps InvalidPragma to CliErrorCode::Unknown
- `test_schema_version_mismatch_error_display` - Verifies error message format
- `test_map_error_code_schema_version_mismatch` - Maps to CliErrorCode::Unknown

#### MigrationForbidden Error Path Tests (2 tests)
- `test_migration_forbidden_error_display` - Verifies error message contains version
- `test_map_error_code_migration_forbidden` - Maps to CliErrorCode::Unknown

#### RevisionMismatch Error Path Tests (2 tests)
- `test_revision_mismatch_error_display` - Verifies expected/found in message
- `test_map_error_code_revision_mismatch_variant` - Maps to CliErrorCode::RevisionMismatch

#### RevisionGap Error Path Tests (1 test)
- `test_revision_gap_full_error_path` - Verifies display and error code mapping

#### EmptyBatch Error Path Tests (2 tests)
- `test_empty_batch_error_display` - Verifies "zero events" in message
- `test_map_error_code_empty_batch` - Maps to CliErrorCode::ValidationFailed

#### CorruptDatabase Error Path Tests (2 tests)
- `test_corrupt_database_error_display` - Verifies "integrity check failed" in message
- `test_corrupt_database_on_invalid_file` - Tests corrupt file detection

#### BackupUnavailable Error Path Tests (2 tests)
- `test_backup_unavailable_error_display` - Verifies "Backup file unavailable" in message
- `test_backup_unavailable_on_missing_file` - Tests missing file handling

#### Comprehensive BDD Scenario Tests (3 tests)
- `test_bdd_revision_mismatch_atomicity` - Full scenario: revision mismatch with atomicity verification
- `test_bdd_empty_batch_rejection` - Full scenario: empty batch rejection with state verification
- `test_bdd_error_message_quality` - Verifies all error types have meaningful, contextual messages

## Test Pattern

Each test follows BDD-style documentation:
```rust
/// BDD: Given <condition>, when <action>,
/// then <expected result>.
#[test]
fn test_<descriptive_name>() {
    // Given: Setup conditions
    // When: Perform action
    // Then: Verify expected outcome
}
```

## Requirements Met

- ✅ InvalidPragma error path tested (4 tests)
- ✅ SchemaVersionMismatch error path tested (3 tests)
- ✅ MigrationForbidden error path tested (2 tests)
- ✅ RevisionMismatch error path tested (2 tests)
- ✅ RevisionGap error path tested (1 test)
- ✅ EmptyBatch error path tested (2 tests)
- ✅ CorruptDatabase error path tested (2 tests)
- ✅ BackupUnavailable error path tested (2 tests)
- ✅ All tests pass (109 store tests)
- ✅ No production code changes (test-only bead)
- ✅ No clippy warnings in new code

## Notes

- The geometry test failure (`prop_edge_negative_dimensions_aabb_positive`) is a pre-existing issue unrelated to this bead
- Tests verify both error type matching and error message content
- Tests verify atomicity on error conditions where applicable
- Tests follow the existing test patterns in the codebase
