bead_id: bd-mtu
bead_title: recovery-export: support json export while in recovery-only mode
phase: p2
updated_at: 2026-03-01T21:20:00Z

# Verification: bd-mtu - recovery-export

## Contract Verification

| Contract Requirement | Status | Evidence |
|---------------------|--------|----------|
| `fn export_while_recovering(conn: &Connection) -> Result<String, ExportError>` | PASS | Implemented in `diagram_tool/src/models/export.rs:190-199` |
| Export works in recovery-only mode | PASS | Uses read-only connection via `open_recovery_mode` |
| Returns valid JSON even when write operations are blocked | PASS | Test `given_recovery_connection_is_read_only_when_exporting_then_succeeds` |

## Test Results

### Recovery Export Tests (bd-mtu specific)

```
test models::export::tests::given_empty_database_in_recovery_mode_when_exporting_then_returns_valid_json ... ok
test models::export::tests::given_database_with_events_in_recovery_mode_when_exporting_then_returns_projection_json ... ok
test models::export::tests::given_recovery_connection_is_read_only_when_exporting_then_succeeds ... ok
```

### All Export Tests

```
test models::export::tests::given_ai_author_to_envelope_author_conversion ... ok
test models::export::tests::given_author_to_envelope_author_conversion ... ok
test models::export::tests::given_invalid_json_when_validating_schema_then_fails ... ok
test models::export::tests::given_json_with_wrong_version_when_validating_then_fails ... ok
test models::export::tests::given_empty_projection_when_exporting_then_returns_valid_json ... ok
test models::export::tests::given_projection_with_nodes_when_exporting_then_includes_nodes_in_json ... ok
test models::export::tests::given_valid_json_when_validating_schema_then_succeeds ... ok
test models::export::tests::given_projection_with_edges_when_exporting_then_includes_edges_in_json ... ok
test models::export::tests::given_invalid_json_when_importing_then_returns_error ... ok
test models::export::tests::given_mismatched_revision_when_importing_then_returns_error ... ok
test models::export::tests::given_empty_database_when_exporting_then_returns_empty_projection ... ok
test models::export::tests::given_database_with_events_when_exporting_then_includes_projection_data ... ok
test models::export::tests::given_empty_database_in_recovery_mode_when_exporting_then_returns_valid_json ... ok
test models::export::tests::given_recovery_connection_is_read_only_when_exporting_then_succeeds ... ok
test models::export::tests::given_empty_database_when_importing_then_succeeds_with_zero_events ... ok
test models::export::tests::given_database_with_events_in_recovery_mode_when_exporting_then_returns_projection_json ... ok
test models::export::tests::given_valid_export_json_when_importing_then_creates_events ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 742 filtered out
```

## Quality Gates

| Gate | Status | Evidence |
|------|--------|----------|
| `cargo check` | PASS | Finished successfully |
| `cargo clippy` | PASS | No warnings with `-D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic` |
| `cargo test` (export tests) | PASS | 17/17 tests passed |

## Invariants Verification

1. **No migration path is introduced** - PASS: Function chains existing infrastructure
2. **No dual-write compatibility path exists** - PASS: Uses read-only operations only
3. **All fallible operations use typed Result errors** - PASS: Returns `Result<String, ExportError>`
4. **Zero unwrap/expect law** - PASS: No unwrap or expect in the implementation

## Implementation Location

- **File**: `diagram_tool/src/models/export.rs`
- **Function**: `export_while_recovering` (lines 177-199)
- **Tests**: Lines 913-1005

## Conclusion

All contract requirements met. Implementation verified with passing tests.
