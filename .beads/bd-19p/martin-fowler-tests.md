# Martin Fowler-Style Tests: bd-19p Import/Export/Persistence

These tests follow the Given-When-Then pattern with emphasis on edge cases and error handling.

## Category K: Import/Export/Persistence (IO-001 to IO-015)

### IO-001: Malformed JSON Import Rejection

```
Given a database connection
And malformed JSON input (syntax errors)
When I call import_diagram_json()
Then I receive ExportError::Serialization
And the error message describes the syntax error
And no data is written to the database
```

**Test Implementation**: `given_truncated_json_when_importing_then_returns_serialization_error`

---

### IO-002: Empty Document Export

```
Given an empty database with no events
When I call export_diagram_json()
Then I receive a valid DiagramJsonExport
And metadata.revision equals 0
And data.nodes is empty
And data.edges is empty
And events is an empty array
```

**Test Implementation**: `given_empty_database_when_exporting_then_returns_empty_projection`

---

### IO-003: Invalid Schema Version Rejection

```
Given a database connection
And JSON input with version 999 (future version)
When I call import_diagram_json()
Then I receive ExportError::InvalidSchema
And the error message indicates unsupported version
```

**Test Implementation**: `given_future_schema_version_when_importing_then_returns_version_error`

---

### IO-004: Valid Round-Trip Preservation

```
Given a database with NodeAdd and EdgeConnect events
When I export to JSON
And import to a fresh database
Then the imported document has the same nodes
And the imported document has the same edges
And the revision matches
```

**Test Implementation**: `given_valid_export_json_when_importing_then_creates_events`

---

### IO-005: Large Document Export Performance

```
Given a projection with 1000 nodes
When I call export_projection_json()
Then the operation completes within 5 seconds
And the JSON contains all 1000 nodes
```

**Test Implementation**: `given_1000_nodes_when_exporting_then_succeeds_within_time_limit`

---

### IO-006: Large Document Import Performance

```
Given an export with 100 events
When I call import_diagram_json()
Then all events are replayed
And the final revision matches expected
```

**Test Implementation**: `given_large_diagram_when_importing_then_all_events_replay_correctly`

---

### IO-007: Unicode Label Round-Trip

```
Given a projection with emoji labels
And RTL (Arabic, Hebrew) labels
And zero-width characters
And mixed script labels
When I export to JSON and re-import
Then all labels are preserved exactly
```

**Test Implementations**:
- `given_emoji_labels_when_exporting_then_roundtrips_correctly`
- `given_right_to_left_text_when_exporting_then_roundtrips_correctly`
- `given_zero_width_characters_when_exporting_then_roundtrips_correctly`
- `given_mixed_script_labels_when_exporting_then_roundtrips_correctly`
- `given_unicode_in_edge_labels_when_exporting_then_roundtrips_correctly`

---

### IO-008: Atomic Save Crash Safety

```
Given an existing document file
When I save a new document atomically
Then no temp files remain after completion
And the file is either fully written or original
```

**Test Implementation**: `given_atomic_save_when_crash_during_write_then_original_untouched`

---

### IO-009: Last Known Good Fallback

```
Given a corrupted primary file
And a valid .lkg backup file
When I call load_workspace_with_lkg()
Then I receive the valid document from LKG
And StageDetails.fallback_used is true
```

**Test Implementation**: `given_lkg_fallback_file_when_primary_fails_then_uses_lkg`

---

### IO-010: Schema Validation Rejection

```
Given JSON with schema violations:
  - Negative dimensions
  - Invalid color format
  - Orphan edge references
  - Invalid label offset
  - Non-subgraph parent
When I call validate_export_schema()
Then I receive ExportError::InvalidSchema
```

**Test Implementations**:
- `given_negative_dimensions_in_json_when_validating_then_schema_fails`
- `given_invalid_color_format_in_json_when_validating_then_schema_fails`
- `given_orphan_edge_references_in_json_when_validating_then_schema_fails`
- `given_invalid_label_offset_in_json_when_validating_then_schema_fails`
- `given_non_subgraph_parent_in_json_when_validating_then_schema_fails`

---

### IO-011: Recovery Mode Export

```
Given a database opened in read-only mode
When I call export_while_recovering()
Then I receive valid JSON
And no write operations are attempted
```

**Test Implementations**:
- `given_empty_database_in_recovery_mode_when_exporting_then_returns_valid_json`
- `given_database_with_events_in_recovery_mode_when_exporting_then_returns_projection_json`
- `given_recovery_connection_is_read_only_when_exporting_then_succeeds`

---

### IO-012: Version Backward Compatibility

```
Given JSON with version 1 (older than current)
When I call validate_export_schema()
Then validation passes
And the document is processed successfully
```

**Test Implementation**: `given_version_1_export_when_validating_then_current_version_works`

---

### IO-013: Null in Required Field Rejection

```
Given JSON with null in a required string field
When I call import_diagram_json()
Then I receive ExportError::Serialization
```

**Test Implementation**: `given_null_in_required_field_when_importing_then_returns_error`

---

### IO-014: Truncated JSON Rejection

```
Given JSON cut off mid-string
When I call import_diagram_json()
Then I receive ExportError::Serialization
And no panic occurs
```

**Test Implementation**: `given_truncated_json_when_importing_then_returns_serialization_error`

---

### IO-015: Missing Required Field Rejection

```
Given JSON missing the version field
When I call import_diagram_json()
Then I receive ExportError::Serialization
```

**Test Implementation**: `given_missing_version_field_when_importing_then_returns_error`

---

## Additional Persistence Tests (cli_persistence)

### P-001: Valid Document Save
```
Given a valid DiagramDocument
When I call save_workspace_atomic()
Then the file exists at the target path
And the file contains valid JSON
```

### P-002: Saved Document Load
```
Given a saved document file
When I call load_workspace_with_lkg()
Then I receive the same document
And version matches
And revision matches
```

### P-003: Missing File Error
```
Given a non-existent file path
When I call load_workspace_with_lkg()
Then I receive CliPersistenceError::NoValidDocument
```

### P-004: Invalid JSON Error
```
Given a file with invalid JSON
When I call load_workspace_with_lkg()
Then I receive CliPersistenceError::NoValidDocument
```

### P-005: Schema Validation Error
```
Given a file with invalid schema (version 1)
When I call load_workspace_with_lkg()
Then I receive CliPersistenceError::ValidationError
```

## Test Coverage Summary

| Module | Tests | Pass | Fail |
|--------|-------|------|------|
| export.rs | 37 | 37 | 0 |
| cli_persistence.rs | 9 | 9 | 0 |
| **Total IO Tests** | **46** | **46** | **0** |

All 15 IO test cases (IO-001 to IO-015) are covered by existing test implementations.
