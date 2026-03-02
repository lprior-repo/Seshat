bead_id: bd-2ca
bead_title: edge-case-bdd-tests-import-export
phase: p1
updated_at: 2026-03-02T04:55:00Z

# Implementation: BDD Tests for Import/Export Edge Cases

## Summary

Added 21 new BDD-style tests to `diagram_tool/src/models/export.rs` covering edge cases in the import/export pipeline.

## Changes Made

### File Modified
- `diagram_tool/src/models/export.rs`

### Test Categories Implemented

#### 1. Serialization Errors (4 tests)
- `given_truncated_json_when_importing_then_returns_serialization_error` - Tests incomplete JSON handling
- `given_null_in_required_field_when_importing_then_returns_error` - Tests null in required fields
- `given_malformed_json_structure_when_importing_then_returns_serialization_error` - Tests wrong JSON structure

#### 2. Large Diagrams (3 tests)
- `given_1000_nodes_when_exporting_then_succeeds_within_time_limit` - Performance test with 1000 nodes
- `given_1000_edges_when_exporting_then_succeeds_within_time_limit` - Performance test with 1000 edges
- `given_large_diagram_when_importing_then_all_events_replay_correctly` - Import test with 100 events

#### 3. Unicode Handling (5 tests)
- `given_emoji_labels_when_exporting_then_roundtrips_correctly` - Emoji preservation
- `given_right_to_left_text_when_exporting_then_roundtrips_correctly` - Arabic/Hebrew text
- `given_zero_width_characters_when_exporting_then_roundtrips_correctly` - ZWJ and invisible chars
- `given_mixed_script_labels_when_exporting_then_roundtrips_correctly` - CJK + Latin + Cyrillic
- `given_unicode_in_edge_labels_when_exporting_then_roundtrips_correctly` - Unicode in edges

#### 4. Schema Validation Failures (5 tests)
- `given_negative_dimensions_in_json_when_validating_then_schema_fails` - Negative width/height
- `given_invalid_color_format_in_json_when_validating_then_schema_fails` - Bad hex colors
- `given_orphan_edge_references_in_json_when_validating_then_schema_fails` - Dangling edges
- `given_invalid_label_offset_in_json_when_validating_then_schema_fails` - Out-of-range offset
- `given_non_subgraph_parent_in_json_when_validating_then_schema_fails` - Wrong parent type

#### 5. Version Mismatches (4 tests)
- `given_future_schema_version_when_importing_then_returns_version_error` - Version 999
- `given_future_schema_version_when_validating_export_then_returns_version_error` - Version 999
- `given_missing_version_field_when_importing_then_returns_error` - No version field
- `given_version_1_export_when_validating_then_current_version_works` - Backward compatibility

## Test Count
- Original tests: 16
- New tests: 21
- Total: 37

## Verification
All tests pass with `cargo test --package diagram_tool export::tests::`
