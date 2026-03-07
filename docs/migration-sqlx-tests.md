# Martin Fowler Test Plan: `sqlx` Unified Store

## Happy Path Tests
- `test_returns_success_when_valid_pool_bootstrapped`
- `test_creates_event_when_preconditions_met_via_sqlx`
- `test_returns_correct_current_revision_after_async_append`

## Error Path Tests
- `test_returns_error_when_invalid_db_path_provided`
- `test_returns_error_when_appending_with_revision_gap`
- `test_returns_error_when_op_id_duplicate`

## Edge Case Tests
- `test_handles_concurrent_async_appends_gracefully`
- `test_recovers_state_from_sqlx_snapshot_correctly`
- `test_handles_zero_byte_database_initialization`

## Contract Verification Tests
- `test_precondition_sequential_revision_enforced`
- `test_postcondition_wal_mode_enabled_on_bootstrap`
- `test_invariant_unique_op_id_enforced_by_schema`

## Contract Violation Tests
- `test_revision_gap_violation_returns_revision_mismatch`
  Given: A store at revision 5
  When: `append_event` is called with revision 7
  Then: returns `Err(StoreError::RevisionMismatch)`

- `test_invalid_path_violation_returns_sqlx_error`
  Given: A read-only directory
  When: `bootstrap_store` is called
  Then: returns `Err(StoreError::Sqlx(_))`

## Given-When-Then Scenarios
### Scenario 1: Async Concurrent Appends
Given: An initialized `SqlitePool`
When: 10 asynchronous tasks concurrently attempt to `append_event`
Then: 
- The database correctly serializes them or returns conflict errors.
- The final `current_revision()` equals the number of successful appends.
- No `database is locked` panics occur due to correct `busy_timeout` pragmas.
