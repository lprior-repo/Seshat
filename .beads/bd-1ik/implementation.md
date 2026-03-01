# Implementation: bd-1ik - cli-errors: standardize structured rejection codes and output

## Files Changed

- `diagram_tool/src/store.rs`

## Changes Made

### 1. Added `HumanPriorityBlock` variant to `StoreError`

Added a new error variant to support human-priority conflict detection:
```rust
#[error("Human priority block: {0}")]
HumanPriorityBlock(String),
```

### 2. Created `CliErrorCode` enum

Added a new enum with structured error codes for CLI output:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliErrorCode {
    RevisionMismatch,
    HumanPriorityBlock,
    PolicyViolation,
    ValidationFailed,
    Unknown,
}
```

### 3. Implemented `map_error_code()`

Maps `StoreError` variants to `CliErrorCode`:
- `StoreError::RevisionMismatch` → `CliErrorCode::RevisionMismatch`
- `StoreError::HumanPriorityBlock` → `CliErrorCode::HumanPriorityBlock`
- `StoreError::ValidationFailed` → `CliErrorCode::ValidationFailed`
- All other variants → `CliErrorCode::Unknown`

### 4. Implemented `render_error_json()`

Renders error code and message as JSON:
```rust
pub fn render_error_json(code: CliErrorCode, message: &str) -> String
```

Returns JSON: `{"code": "...", "message": "..."}`

### 5. Added comprehensive tests

- `test_map_error_code_revision_mismatch`
- `test_map_error_code_human_priority_block`
- `test_map_error_code_validation_failed`
- `test_map_error_code_sqlite`
- `test_map_error_code_io`
- `test_render_error_json_*` (5 tests)
- `test_cli_error_code_serialization`

## Contract Compliance

| Contract Requirement | Implementation |
|---------------------|----------------|
| `fn map_error_code(err: &StoreError) -> CliErrorCode` | ✅ Implemented |
| `fn render_error_json(code: CliErrorCode, message: &str) -> String` | ✅ Implemented |
| `enum StoreError { Sqlite, RevisionMismatch, HumanPriorityBlock, ValidationFailed }` | ✅ All variants mapped |
| No unwrap/expect/panic | ✅ Zero unwrap used |
| Result<T, E> for errors | ✅ All functions return Result or have pure fallible signatures |
