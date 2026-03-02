bead_id: bd-2ca
bead_title: edge-case-bdd-tests-import-export
phase: p0
updated_at: 2026-03-02T04:47:00Z

# Contract: BDD Tests for Import/Export Edge Cases

## Scope

Add comprehensive BDD-style tests for the import/export pipeline in `diagram_tool/src/models/export.rs`. Tests must cover edge cases that could cause data loss, corruption, or silent failures during serialization, deserialization, and validation.

## Test Categories

### 1. Serialization Errors

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_circular_json_structure_when_importing_then_returns_serialization_error` | JSON with circular reference patterns | Import is attempted | Returns `ExportError::Serialization` with descriptive message |
| `given_truncated_json_when_importing_then_returns_serialization_error` | Incomplete JSON (cut off mid-string) | Import is attempted | Returns `ExportError::Serialization` with parse error |
| `given_null_in_required_field_when_importing_then_returns_validation_error` | JSON with null where node id expected | Import is attempted | Returns appropriate error (validation or serialization) |
| `given_infinity_in_coordinates_when_exporting_then_handles_gracefully` | Projection with f64::INFINITY coordinates | Export is called | Either succeeds with valid JSON or returns explicit error |

### 2. Large Diagrams

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_10000_nodes_when_exporting_then_succeeds_within_time_limit` | Projection with 10,000 nodes | Export is called | Completes within 5 seconds and produces valid JSON |
| `given_10000_edges_when_exporting_then_succeeds_within_time_limit` | Projection with 10,000 edges | Export is called | Completes within 5 seconds and produces valid JSON |
| `given_deeply_nested_subgraph_tree_when_exporting_then_no_stack_overflow` | 100-level deep subgraph nesting | Export is called | Completes without stack overflow |
| `given_large_diagram_when_importing_then_all_events_replay_correctly` | Export with 1000 events | Import is called | All events are replayed and projection matches |

### 3. Unicode Handling

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_emoji_labels_when_exporting_then_roundtrips_correctly` | Nodes with emoji labels | Export then import | Labels are preserved exactly |
| `given_right_to_left_text_when_exporting_then_roundtrips_correctly` | Arabic/Hebrew labels | Export then import | Labels are preserved exactly |
| `given_zero_width_characters_when_exporting_then_roundtrips_correctly` | Labels with ZWJ sequences | Export then import | Labels are preserved exactly |
| `given_mixed_script_labels_when_exporting_then_roundtrips_correctly` | CJK, Latin, Cyrillic mixed | Export then import | Labels are preserved exactly |
| `given_unicode_in_edge_labels_when_exporting_then_roundtrips_correctly` | Edge labels with unicode | Export then import | Labels are preserved exactly |

### 4. Schema Validation Failures

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_negative_dimensions_when_validating_then_schema_fails` | Node with negative width/height | Schema validation runs | Returns validation error |
| `given_invalid_color_format_when_validating_then_schema_fails` | Edge with invalid hex color | Schema validation runs | Returns validation error |
| `given_orphan_edge_references_when_validating_then_schema_fails` | Edge referencing deleted node | Schema validation runs | Returns dangling reference error |
| `given_invalid_label_offset_when_validating_then_schema_fails` | Edge with label_offset_t > 1.0 | Schema validation runs | Returns validation error |
| `given_non_subgraph_parent_when_validating_then_schema_fails` | Node parent is regular node | Schema validation runs | Returns parent validation error |

### 5. Version Mismatches

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_future_schema_version_when_importing_then_returns_version_error` | Export with version 999 | Import is attempted | Returns `ExportError::InvalidSchema` mentioning version |
| `given_version_1_export_when_importing_then_returns_version_error` | Export with version 1 | Import is attempted | Returns appropriate error |
| `given_missing_version_field_when_importing_then_returns_serialization_error` | Export without version | Import is attempted | Returns serialization or validation error |

## Implementation Requirements

1. **Location**: Tests should be added to `diagram_tool/src/models/export.rs` in the `#[cfg(test)] mod tests` block, or in a new test file `diagram_tool/tests/import_export_edge_cases.rs` for integration-style tests.

2. **Naming Convention**: All tests must follow `given_X_when_Y_then_Z` BDD naming pattern.

3. **Assertions**: Each test must have clear assertions that verify the expected behavior:
   - Success cases: verify output structure and data integrity
   - Error cases: verify error type and message content

4. **No Unwrap/Expect**: Tests must not use `.unwrap()` or `.expect()` - use `assert!` on Result::is_ok/is_err or pattern match.

5. **Performance Tests**: Large diagram tests should have explicit time limits using `std::time::Instant`.

## Acceptance Criteria

- [ ] All 20+ test cases implemented
- [ ] All tests pass with `cargo test --package diagram_tool`
- [ ] No new clippy warnings introduced
- [ ] Test coverage of export.rs increases
- [ ] Moon validation passes (`moon run :test`)

## Out of Scope

- Fuzz testing (covered by proptests in schema.rs)
- CLI-level import/export commands
- Filesystem-related error handling
- Network/async operations
