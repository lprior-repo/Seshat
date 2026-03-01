# Verification: bd-1ik - cli-errors: standardize structured rejection codes and output

## Test Results

```
running 24 tests
test models::envelope::tests::given_node_restore_op_when_getting_kind_then_returns_node_kind ... ok
test store::tests::test_cli_error_code_serialization ... ok
test store::tests::test_map_error_code_sqlite ... ok
test store::tests::test_map_error_code_human_priority_block ... ok
test store::tests::test_map_error_code_revision_mismatch ... ok
test store::tests::test_map_error_code_validation_failed ... ok
test store::tests::test_bootstrap_store_with_invalid_path ... ok
test store::tests::test_map_error_code_io ... ok
test store::tests::test_render_error_json_human_priority_block ... ok
test store::tests::test_render_error_json_policy_violation ... ok
test store::tests::test_render_error_json_revision_mismatch ... ok
test store::tests::test_render_error_json_validation_failed ... ok
test store::tests::test_startup_integrity_check_on_nonexistent_database ... ok
test store::tests::test_render_error_json_unknown ... ok
test store::tests::test_bootstrap_store_enforces_synchronous_full ... ok
test store::tests::test_current_store_config_returns_pragmas_and_version ... ok
test store::tests::test_bootstrap_store_creates_schema_tables ... ok
test store::tests::test_bootstrap_store_enforces_wal_mode ... ok
test store::tests::test_bootstrap_store_creates_database_with_schema ... ok
test store::tests::test_startup_integrity_check_on_valid_database ... ok
test store::tests::test_open_store_with_existing_wal_database ... ok
test store::tests::test_bootstrap_idempotent_on_existing_schema ... ok
test store::tests::test_open_recovery_mode_on_valid_database ... ok
test store::tests::test_recovery_handle_export_to_json ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 508 filtered out
```

## Verification Checklist

- [x] `map_error_code()` correctly maps all StoreError variants
- [x] `render_error_json()` produces valid JSON output
- [x] CliErrorCode serializes correctly with serde
- [x] All StoreError variants specified in contract are mapped:
  - [x] Sqlite → Unknown
  - [x] RevisionMismatch → RevisionMismatch
  - [x] HumanPriorityBlock → HumanPriorityBlock
  - [x] ValidationFailed → ValidationFailed
- [x] Zero unwrap/expect/panic in source code
- [x] All functions return Result where appropriate
- [x] No legacy code path exists (new implementation only)
- [x] Tests compile and pass

## Quality Gates

- ✅ cargo fmt --check passes
- ✅ cargo clippy passes with no errors
- ✅ cargo test passes (24 tests)
- ✅ No unwrap/expect/panic in source (functional-rust compliant)
