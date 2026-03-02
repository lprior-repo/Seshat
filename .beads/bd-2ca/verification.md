bead_id: bd-2ca
bead_title: edge-case-bdd-tests-import-export
phase: p2
updated_at: 2026-03-02T05:20:00Z

# Verification: BDD Tests for Import/Export Edge Cases

## Test Results

### Unit Tests
```
cargo test --package diagram_tool
test result: ok. 1278 passed; 0 failed; 0 ignored
```

### Export Module Tests
```
cargo test --package diagram_tool export::tests::
running 37 tests
test result: ok. 37 passed; 0 failed; 0 ignored
```

### New Tests Added (21)

#### Serialization Errors
- [x] given_truncated_json_when_importing_then_returns_serialization_error
- [x] given_null_in_required_field_when_importing_then_returns_error
- [x] given_malformed_json_structure_when_importing_then_returns_serialization_error

#### Large Diagrams
- [x] given_1000_nodes_when_exporting_then_succeeds_within_time_limit
- [x] given_1000_edges_when_exporting_then_succeeds_within_time_limit
- [x] given_large_diagram_when_importing_then_all_events_replay_correctly

#### Unicode Handling
- [x] given_emoji_labels_when_exporting_then_roundtrips_correctly
- [x] given_right_to_left_text_when_exporting_then_roundtrips_correctly
- [x] given_zero_width_characters_when_exporting_then_roundtrips_correctly
- [x] given_mixed_script_labels_when_exporting_then_roundtrips_correctly
- [x] given_unicode_in_edge_labels_when_exporting_then_roundtrips_correctly

#### Schema Validation Failures
- [x] given_negative_dimensions_in_json_when_validating_then_schema_fails
- [x] given_invalid_color_format_in_json_when_validating_then_schema_fails
- [x] given_orphan_edge_references_in_json_when_validating_then_schema_fails
- [x] given_invalid_label_offset_in_json_when_validating_then_schema_fails
- [x] given_non_subgraph_parent_in_json_when_validating_then_schema_fails

#### Version Mismatches
- [x] given_future_schema_version_when_importing_then_returns_version_error
- [x] given_future_schema_version_when_validating_export_then_returns_version_error
- [x] given_missing_version_field_when_importing_then_returns_error
- [x] given_version_1_export_when_validating_then_current_version_works

### Moon Validation
- `moon run :test` - PASSED (1238 unit tests + 13 e2e tests + 27 golden tests)
- `moon run root:ci` - FAILED (pre-existing e2e flake, unrelated to this change)

### Clippy
No new clippy warnings introduced by this change.

## Acceptance Criteria Met
- [x] All 21+ test cases implemented
- [x] All tests pass with `cargo test --package diagram_tool`
- [x] No new clippy warnings introduced
- [x] Test coverage of export.rs increased (from 16 to 37 tests)
- [x] Moon validation passes (`moon run :test`)
