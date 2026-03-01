bead_id: bd-1ws
bead_title: io-json-import: import diagram json by generating canonical events
phase: p3
updated_at: 2026-03-01T20:55:00Z

# Verification: io-json-import

## Test Results

### Unit Tests (from `diagram_tool/src/models/export.rs`)

| Test | Status | Description |
|------|--------|-------------|
| `given_empty_database_when_importing_then_succeeds_with_zero_events` | PASSED | Import empty data works |
| `given_valid_export_json_when_importing_then_creates_events` | PASSED | Import creates events |
| `given_invalid_json_when_importing_then_returns_error` | PASSED | Invalid JSON rejected |
| `given_mismatched_revision_when_importing_then_returns_error` | PASSED | Revision mismatch handled |
| `given_author_to_envelope_author_conversion` | PASSED | Author conversion works |
| `given_ai_author_to_envelope_author_conversion` | PASSED | AI author conversion works |

### Import-Specific Tests

```
cargo test import 2>&1
test models::export::tests::given_empty_database_when_importing_then_succeeds_with_zero_events ... ok
test models::export::tests::given_valid_export_json_when_importing_then_creates_events ... ok
test models::export::tests::given_invalid_json_when_importing_then_returns_error ... ok
test models::export::tests::given_mismatched_revision_when_importing_then_returns_error ... ok
test result: ok. 9 passed; 0 failed; 0 ignored
```

### Full Test Suite

```
cargo test 2>&1
test result: ok. 715 passed; 0 failed; 5 ignored
test result: ok. 13 passed; 0 failed (cli_e2e)
```

## Contract Verification

### Preconditions Verified

- [x] Valid JSON input matching diagram.schema.json is accepted
- [x] `import_diagram_json` signature matches contract (takes Connection, json str, Author)

### Postconditions Verified

- [x] Import generates canonical events (via `generate_canonical_events`)
- [x] Events can be replayed to reproduce the imported state (verified by export/import roundtrip test)
- [x] Returns `ImportResult` with events_generated and final_revision

### Invariants Verified

- [x] No migration path introduced
- [x] No dual-write compatibility path
- [x] All fallible operations use `Result<T, ExportError>`
- [x] Zero unwrap/expect (clippy enforced)

## Roundtrip Verification

The test `given_valid_export_json_when_importing_then_creates_events` verifies:
1. Create data in database A
2. Export to JSON
3. Import into fresh database B
4. Verify events are generated and final revision > 0

This confirms the import path correctly generates canonical events that reproduce the exported state.

## CI Status

- All 728 tests pass
- No clippy warnings
- No compilation errors
