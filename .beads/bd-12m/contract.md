# Contract: bd-12m - edge-case-bdd-tests-error-handling

bead_id: bd-12m
bead_title: edge-case-bdd-tests-error-handling
phase: p0
updated_at: 2026-03-02T04:45:00Z

---

## EARS Requirements

### Ubiquitous
- THE SYSTEM SHALL return structured, typed errors for all edge case scenarios

### Event-Driven
- WHEN an error condition is detected, THE SYSTEM SHALL return the appropriate typed error variant with meaningful context

### Unwanted
- IF an error path is triggered, THE SYSTEM SHALL NOT panic, unwrap, or produce unstructured error messages

---

## Preconditions

- auth_required: false
- required_inputs: []
- system_state:
  - Error types already defined in `diagram_tool/src/store.rs`:
    - `StoreError::InvalidPragma(String)`
    - `StoreError::SchemaVersionMismatch { expected: i32, found: i32 }`
    - `StoreError::MigrationForbidden { version: i32 }`
    - `StoreError::RevisionMismatch { expected: i64, found: i64 }`
    - `StoreError::RevisionGap { expected: i64, found: i64 }`
    - `StoreError::EmptyBatch`
  - Recovery error types in `diagram_tool/src/store.rs`:
    - `RecoveryError::CorruptDatabase(String)`
    - `RecoveryError::BackupUnavailable(String)`

---

## Postconditions

- state_changes:
  - BDD-style tests added for all 8 error paths
  - Each test verifies:
    - Error is returned under correct conditions
    - Error variant matches expected type
    - Error message contains contextual information
    - No side effects occur on error (atomicity)
- return_guarantees: []

---

## Invariants

- All error paths have dedicated test coverage
- Tests follow Given-When-Then pattern where applicable
- Tests verify both error type and error message content
- No production code changes required (test-only bead)

---

## Error Paths to Test

### 1. InvalidPragma
- Trigger: WAL journal mode not set correctly
- Trigger: Synchronous mode not set to FULL (2)
- Expected: `StoreError::InvalidPragma(String)` with descriptive message

### 2. SchemaVersionMismatch
- Trigger: Database schema version differs from expected
- Expected: `StoreError::SchemaVersionMismatch { expected, found }`

### 3. MigrationForbidden
- Trigger: Attempt to migrate from unsupported schema version
- Expected: `StoreError::MigrationForbidden { version }`

### 4. RevisionMismatch
- Trigger: Expected revision doesn't match current revision on append
- Expected: `StoreError::RevisionMismatch { expected, found }`

### 5. RevisionGap
- Trigger: Non-sequential revision detected
- Expected: `StoreError::RevisionGap { expected, found }`

### 6. EmptyBatch
- Trigger: Empty ops vector passed to append_batch
- Expected: `StoreError::EmptyBatch`

### 7. CorruptDatabase
- Trigger: SQLite integrity check fails
- Expected: `RecoveryError::CorruptDatabase(String)`

### 8. BackupUnavailable
- Trigger: Backup file does not exist or cannot be read
- Expected: `RecoveryError::BackupUnavailable(String)`

---

## Implementation Tasks

### Phase 0: Research
- Review existing error handling tests in store.rs
- Identify gaps in test coverage for the 8 error types

### Phase 1: Tests
- Add BDD-style tests for each error path
- Ensure tests are isolated and deterministic

### Phase 2: Verification
- Run all tests to verify coverage
- Ensure no regression in existing tests

---

## AI Hints

- DO: Follow existing test patterns in store.rs
- DO: Use TempDir for isolated test databases
- DO: Verify error message content, not just error type
- DO NOT: Modify production code (test-only bead)
- DO NOT: Use unwrap/expect in test assertions (use match or assert_eq!)

---

## Completion Checklist

- [ ] InvalidPragma error path tested
- [ ] SchemaVersionMismatch error path tested
- [ ] MigrationForbidden error path tested
- [ ] RevisionMismatch error path tested
- [ ] RevisionGap error path tested
- [ ] EmptyBatch error path tested
- [ ] CorruptDatabase error path tested
- [ ] BackupUnavailable error path tested
- [ ] All tests pass
- [ ] No clippy warnings
