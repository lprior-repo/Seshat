# Martin Fowler Test Plan: Store Strict DDD Refactoring

## Core Directives (Testing Trophy & Farley ATDD)
1. **No Mocks**: Every test MUST run against a REAL SQLite database. Mocking `StoreConnection` or `rusqlite::Connection` is strictly banned.
2. **Isolation**: Every test MUST run in complete isolation using a fresh in-memory SQLite database (`:memory:`) or a unique temporary file per test to guarantee deterministic execution.
3. **DSL / Protocol Driver**: Tests must use a Domain Specific Language (DSL) driver (e.g., `StoreTestDriver`) that translates domain intent into exact implementation calls. Test names map to behavior, not function names.
4. **Mutation Testing**: The test suite MUST be subjected to mutation testing (e.g., using `cargo-mutants`) to ensure that logic alterations or flipped conditionals are caught.

## Combinatorial & Property-Based Coverage
- `proptest_valid_operation_id`: Generates random valid strings and invalid strings to rigorously verify `ValidOperationId` rejects null bytes, pure whitespace, empty strings, and values exceeding the 255-character upper bound limit.
- `proptest_monotonically_increasing_revisions`: Generates random batches of sizes 1 to 1000 and asserts the `Revision` always strictly increments exactly by the batch size.
- `proptest_timestamp_validation`: Fuzzes timestamps to ensure 0 is rejected and valid `u64` values are accepted.
- `proptest_exact_idempotent_append_never_changes_state`: A proptest that generates a random valid batch, appends it, and then appends the exact same batch again to assert as a universal property that the database revision and state never change (Idempotency Property Test).

## Happy Path Tests
- `test_driver_appends_valid_batch_and_increments_revision_correctly`
- `test_driver_idempotent_append_with_identical_operation_id_succeeds_without_duplication`
- `test_driver_append_different_operation_id_with_identical_payload_succeeds`
- `test_driver_reads_events_in_exact_inserted_order`

## Error Path Tests
- `test_returns_database_locked_error_when_sqlite_is_busy_by_another_process`
- `test_returns_revision_overflow_error_when_exceeding_max_i64_at_batch_boundary`
- `test_returns_duplicate_with_conflict_when_same_operation_id_has_different_payload`
- `test_returns_read_only_violation_when_recovery_session_bypasses_runtime_protection`
- `test_mid_batch_failure_rolls_back_entire_transaction`: Verifies that if a batch of N events encounters an error (e.g., `DuplicateWithConflict`) on the k-th event, the ENTIRE batch transaction is rolled back and no partial writes occur (Mid-Batch Failure Atomicity).

## Edge Case Tests
- `test_handles_minimum_allowed_batch_size_of_one_successfully`
- `test_handles_maximum_allowed_batch_size_successfully`
- `test_rejects_empty_batch_with_specific_error`
- `test_rejects_batch_exceeding_maximum_size_with_specific_error`
- `test_operation_id_at_exactly_maximum_length_succeeds`
- `test_handles_minimum_valid_timestamp_of_one_successfully`
- `test_handles_maximum_valid_timestamp_successfully`
- `test_rejects_batch_overflowing_revision_space_when_starting_near_max`
- `test_batch_append_landing_exactly_on_i64_max_succeeds`: Proves a batch can successfully append exactly up to and landing on `i64::MAX` without an off-by-one boundary error (Exact Boundary Success).
- `test_payload_combinations_empty_and_excessive`: Includes tests for empty payloads (`""`) and excessively large payloads to ensure SQLite handles them cleanly without truncation or unbounded memory errors (Payload Combinations).

## Contract Verification Tests
- `test_precondition_rejects_timestamp_exactly_zero`
- `test_precondition_rejects_operation_id_empty_string`
- `test_precondition_rejects_operation_id_with_only_whitespace`
- `test_precondition_rejects_operation_id_with_null_bytes`
- `test_precondition_rejects_operation_id_exceeding_maximum_length`
- `test_postcondition_revision_matches_exact_batch_size_increment`
- `test_invariant_detects_and_rejects_revision_gap_caused_by_corruption`
- `test_invariant_schema_version_mismatch_returns_schema_version_mismatch_error`
- `test_compile_time_precondition_recovery_session_cannot_write` (Verified via module structure/typestate).

## Contract Violation Tests
- `test_empty_batch_violation_returns_empty_batch_error`
  Given: `BoundedBatch::try_from(vec![])`
  When: constructor is called
  Then: returns `Err(StoreError::EmptyBatch)`
- `test_large_batch_violation_returns_batch_too_large_error`
  Given: `BoundedBatch::try_from(vec![...1001 items])`
  When: constructor is called
  Then: returns `Err(StoreError::BatchTooLarge)`
- `test_zero_timestamp_violation_returns_invalid_timestamp_error`
  Given: `ValidTimestamp::new(0)`
  When: constructor is called
  Then: returns `Err(StoreError::InvalidTimestamp)`
- `test_empty_string_operation_id_violation_returns_invalid_operation_id_error`
  Given: `ValidOperationId::new("")`
  When: constructor is called
  Then: returns `Err(StoreError::InvalidOperationId)`
- `test_whitespace_operation_id_violation_returns_invalid_operation_id_error`
  Given: `ValidOperationId::new("   ")`
  When: constructor is called
  Then: returns `Err(StoreError::InvalidOperationId)`
- `test_null_byte_operation_id_violation_returns_invalid_operation_id_error`
  Given: `ValidOperationId::new("id\0test")`
  When: constructor is called
  Then: returns `Err(StoreError::InvalidOperationId)`
- `test_operation_id_too_long_violation_returns_operation_id_too_long_error`
  Given: `ValidOperationId::new("a".repeat(256))`
  When: constructor is called
  Then: returns `Err(StoreError::OperationIdTooLong)`
- `test_database_locked_violation_returns_database_locked_error`
  Given: A locked SQLite database (e.g. exclusive transaction held by another connection)
  When: append_batch is called
  Then: returns `Err(StoreError::DatabaseLocked)`
- `test_duplicate_conflict_violation_returns_duplicate_with_conflict_error`
  Given: An existing event with `OperationId(A)` and `Payload(X)`
  When: append_batch is called with `OperationId(A)` and `Payload(Y)`
  Then: returns `Err(StoreError::DuplicateWithConflict)`
- `test_batch_boundary_revision_overflow_violation_returns_revision_overflow_error`
  Given: A database with the latest revision exactly at `i64::MAX - 1`
  When: append_batch is called with a batch of 2 events
  Then: returns `Err(StoreError::RevisionOverflow)` and revision remains `i64::MAX - 1` (rollback)
- `test_read_only_violation_returns_read_only_violation_error`
  Given: A `RecoverySession` attempting to bypass typestate and write directly
  When: an internal write is triggered
  Then: returns `Err(StoreError::ReadOnlyViolation)`
- `test_schema_version_mismatch_violation_returns_schema_version_mismatch_error`
  Given: A database initialized with schema version 999
  When: `ReadWriteSession::open()` is called
  Then: returns `Err(StoreError::SchemaVersionMismatch)`
- `test_payload_too_large_violation_returns_payload_too_large_error`
  Given: A payload string of 100MB
  When: `ValidPayload::new()` is called
  Then: returns `Err(StoreError::PayloadTooLarge)`

## Given-When-Then Scenarios (Executable Specifications)

### Scenario 1: Idempotent Event Append (Same OpId, Same Payload)
Given: A database containing an event with `OperationId("op-123")` and payload `{"action": "create"}`.
When: The user attempts to append a new batch containing an event with `OperationId("op-123")` and payload `{"action": "create"}`.
Then: 
- The system accepts the operation as successful.
- The database revision is NOT incremented.
- No duplicate event is stored.

### Scenario 2: Conflicting Event Append (Same OpId, Different Payload)
Given: A database containing an event with `OperationId("op-123")` and payload `{"action": "create"}`.
When: The user attempts to append a new batch containing an event with `OperationId("op-123")` but payload `{"action": "delete"}`.
Then: 
- The system rejects the operation.
- The system returns `Err(StoreError::DuplicateWithConflict)`.
- The database state remains unchanged.

### Scenario 3: Allowed Duplication (Different OpId, Same Payload)
Given: A database containing an event with `OperationId("op-123")` and payload `{"action": "create"}`.
When: The user attempts to append a new batch containing an event with `OperationId("op-456")` and the exact same payload `{"action": "create"}`.
Then:
- The system accepts the operation as successful.
- The database revision is incremented by 1.
- The new event is stored with the new `OperationId`.

### Scenario 4: Monotonic Revision Enforcement
Given: A clean, isolated SQLite database initialized for testing.
When: A batch of 5 valid events is appended.
Then: 
- The returned revision is exactly 5.
When: A subsequent batch of 3 valid events is appended.
Then:
- The returned revision is exactly 8.
- Reading events from revision 1 to 8 returns exactly 8 sequentially numbered events without gaps.

### Scenario 5: Corruption / Gap Detection
Given: A database populated with 10 sequential events.
When: Event at revision 5 is manually deleted bypassing the domain layer (simulating corruption).
When: The user attempts to read events covering revision 5.
Then:
- The system detects the missing revision.
- The system returns `Err(StoreError::RevisionGap)`.

### Scenario 6: Exhaustion of Revision Space at Batch Boundary
Given: A database artificially set to have its current revision at `i64::MAX - 1`.
When: The user attempts to append a batch of 2 valid events.
Then:
- The system rejects the operation before any event in the batch is committed.
- The system returns `Err(StoreError::RevisionOverflow)`.
- The database revision remains at `i64::MAX - 1`.

### Scenario 7: Recovery Session Write Prevention
Given: A `RecoverySession` initialized against a valid database.
When: The user attempts to call write/append operations.
Then:
- The type system prevents compilation.
- (If runtime bypassed), the system instantly returns `Err(StoreError::ReadOnlyViolation)`.

### Scenario 8: Schema Version Guard
Given: A database file previously created with an incompatible future schema version (e.g., v999).
When: The user attempts to bootstrap or open the store.
Then:
- The system rejects the initialization.
- The system returns `Err(StoreError::SchemaVersionMismatch)`.

### Scenario 9: Mid-Batch Failure Atomicity
Given: A store at revision 5.
When: A batch of 3 events is appended, where the 2nd event has a conflicting `OperationId` returning `DuplicateWithConflict`.
Then:
- The system rejects the operation.
- The system returns `Err(StoreError::DuplicateWithConflict)`.
- The ENTIRE batch transaction is rolled back.
- The store remains exactly at revision 5, and the 1st event is NOT written.

### Scenario 10: Exact Boundary Success
Given: A store pre-populated to exactly revision `i64::MAX - 2`.
When: A batch of exactly 2 valid events is appended.
Then:
- The append succeeds.
- The database lands exactly on revision `i64::MAX` without an off-by-one boundary error.

### Scenario 11: Payload Combinations
Given: A clean, isolated SQLite database initialized for testing.
When: Appending an event with an empty payload `""`.
Then:
- The system accepts the operation as successful.
When: Appending an event with an excessively large payload (e.g., a very large valid string bounded by SQLite limits).
Then:
- The system accepts the operation as successful without truncation.

### Scenario 12: Idempotency Property Test
Given: A randomly generated valid batch of `N` events.
When: The batch is appended successfully, advancing the revision from `A` to `B`.
When: The EXACT same batch of `N` events is appended again.
Then:
- The system identifies the duplicate batch operations.
- The state never changes.
- The latest revision remains exactly `B`.