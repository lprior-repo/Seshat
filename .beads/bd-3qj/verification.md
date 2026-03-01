bead_id: bd-3qj
bead_title: io-json-export: export canonical diagram json from projection
phase: p2
updated_at: 2026-03-01T00:00:00Z

# Verification: io-json-export

## Test Results

### Unit Tests

```
running 14 tests
test models::export::tests::given_ai_author_to_envelope_author_conversion ... ok
test models::export::tests::given_author_to_envelope_author_conversion ... ok
test models::export::tests::given_invalid_json_when_validating_schema_then_fails ... ok
test models::export::tests::given_json_with_wrong_version_when_validating_then_fails ... ok
test models::export::tests::given_empty_projection_when_exporting_then_returns_valid_json ... ok
test models::export::tests::given_valid_json_when_validating_schema_then_succeeds ... ok
test models::export::tests::given_projection_with_nodes_when_exporting_then_includes_nodes_in_json ... ok
test models::export::tests::given_projection_with_edges_when_exporting_then_includes_edges_in_json ... ok
test models::export::tests::given_invalid_json_when_importing_then_returns_error ... ok
test models::export::tests::given_mismatched_revision_when_importing_then_returns_error ... ok
test models::export::tests::given_empty_database_when_importing_then_succeeds_with_zero_events ... ok
test models::export::tests::given_empty_database_when_exporting_then_returns_empty_projection ... ok
test models::export::tests::given_database_with_events_when_exporting_then_includes_projection_data ... ok
test models::export::tests::given_valid_export_json_when_importing_then_creates_events ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 721 filtered out
```

### Full Test Suite

```
test result: ok. 730 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out

Running tests/cli_e2e.rs
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Contract Verification

| Requirement | Status | Evidence |
|-------------|--------|----------|
| `export_diagram_json(state: &DiagramProjection) -> Result<String, ExportError>` | PASS | `export_projection_json` function implemented |
| `validate_export_schema(json: &str) -> Result<(), ExportError>` | PASS | Function implemented and tested |
| `ExportError` enum with `Serialization` and `SchemaValidation` | PASS | Uses existing `ExportError::Serialization` and `ExportError::InvalidSchema` |
| No database dependency for export | PASS | Takes `&DiagramProjection` directly |
| Deterministic/canonical output | PASS | Uses `to_canonical_pretty_json` with sorted keys |
| Zero unwrap/expect | PASS | Enforced by `#![deny(clippy::unwrap_used)]` |

## Acceptance Criteria

### Happy Paths
- [x] Given valid DiagramProjection, when export_projection_json is called, then returns valid JSON string
- [x] Given valid JSON, when validate_export_schema is called, then returns Ok(())

### Error Paths
- [x] Given invalid JSON, when validation is called, then returns ExportError::Serialization
- [x] Given JSON with wrong version, when validation is called, then returns ExportError::InvalidSchema

## Lint Status

No clippy errors. Warnings are pre-existing in other modules.

## Conclusion

All contract requirements met. Implementation complete.
