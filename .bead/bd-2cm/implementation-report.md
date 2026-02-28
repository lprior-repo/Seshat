# Implementation Report: bd-2cm - storage-sync: add atomic redb-plus-file persistence

## Summary

Successfully implemented atomic persistence for CLI workspace operations with crash-safe file writes and Last Known Good (LKG) fallback for recovery.

## Files Created

### `diagram_tool/src/cli_persistence.rs` (NEW)
- **CliPersistenceError** enum with thiserror for structured error handling:
  - `IoError` - File I/O operations
  - `ParseError` - JSON deserialization failures
  - `ValidationError` - Schema validation failures
  - `TempFileError` - Temp file creation issues
  - `AtomicRenameError` - Atomic rename failures
  - `NoValidDocument` - Both primary and LKG files failed (includes original error message)

- **save_workspace_atomic()** - Atomic write pattern:
  1. Creates temp file in same directory as target
  2. Writes JSON content to temp file
  3. fsync to ensure data is on disk
  4. Atomic rename temp -> target
  5. Emits `persisted` stage event on success

- **load_workspace_with_lkg()** - Load with fallback:
  1. Attempts to load and validate primary file
  2. On failure, tries `<path>.lkg` fallback
  3. Returns first successfully loaded document
  4. Emits stage events for validation and loading

- **emit_stage_event()** - JSONL output for structured logging
- **StageDetails** - Builder pattern for event details

## Files Modified

### `diagram_tool/src/cli.rs`
- Added import for `cli_persistence` module
- Replaced `save_doc()` calls with `save_workspace_atomic()`
- Replaced `load_doc()` to use `load_workspace_with_lkg()`
- Added `emit_stage_event()` calls before validation
- Updated `error_code()` to detect parse errors in wrapped messages

### `diagram_tool/src/main.rs`
- Added `mod cli_persistence;` declaration

### `diagram_tool/Cargo.toml`
- Moved `tempfile` from `[dev-dependencies]` to `[dependencies]`

## Code Changes Summary

### Atomic Write Pattern
```rust
pub fn save_workspace_atomic(doc: &DiagramDocument, path: &Path) -> Result<(), CliPersistenceError> {
    // Get parent directory, defaulting to current directory for relative paths
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let parent = parent.unwrap_or_else(|| Path::new("."));
    
    // Create temp file in same directory for atomic rename
    let temp_path = parent.join(format!(".{}.tmp.{}", ...));
    
    // Write, flush, fsync, then atomic rename
    ...
}
```

### LKG Fallback Pattern
```rust
pub fn load_workspace_with_lkg(path: &Path) -> Result<DiagramDocument, CliPersistenceError> {
    match load_and_validate(path) {
        Ok(doc) => Ok(doc),
        Err(primary_err) => {
            // Try LKG fallbacks
            if let Ok(doc) = load_and_validate(&lkg_path) {
                return Ok(doc);
            }
            Err(CliPersistenceError::NoValidDocument(primary_err.to_string()))
        }
    }
}
```

### JSONL Event Emission
Events are single-line valid JSON:
```json
{"event":"stage","name":"validating","details":{"path":"/path/to/file.json"}}
{"event":"stage","name":"persisted","details":{"path":"/path/to/file.json","bytes_written":1234}}
{"event":"stage","name":"error","details":{"path":"/path/to/file.json","code":"no_valid_document"}}
```

## Test Results

All 440 unit tests + 8 e2e tests pass:
```
test result: ok. 440 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Quality Gates

| Gate | Status |
|------|--------|
| `moon run :check` | ✅ PASSED |
| `moon run :clippy` | ✅ PASSED |
| `moon run :test-rust` | ✅ PASSED |

## Deviations from Spec

1. **No `NoParentDirectory` error** - Changed logic to default to current directory (`.`) for relative paths instead of returning an error. This makes the API more user-friendly.

2. **Error message preservation** - `NoValidDocument` now includes the original error message to enable proper error code classification in the CLI.

3. **Additional `with_temp_path` builder method** - Added but marked as `#[allow(dead_code)]` for future use.

## Hard Constraints Compliance

- ✅ ZERO unwrap/expect/panic
- ✅ All fallible functions return `Result<T, CliPersistenceError>`
- ✅ Atomic write: temp file in same directory, fsync, rename
- ✅ JSONL events are single-line valid JSON
- ✅ Functional Rust patterns (const fn, builder pattern, Result types)
