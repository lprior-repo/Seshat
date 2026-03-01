bead_id: bd-3qj
bead_title: io-json-export: export canonical diagram json from projection
phase: p0
updated_at: 2026-03-01T00:00:00Z

# Contract: io-json-export

## Summary

Export canonical diagram JSON from a DiagramProjection. This implements a hard-cutover rewrite with no legacy compatibility layer.

## Rust Contract Signatures

```rust
/// Export a diagram projection to canonical JSON format
fn export_diagram_json(state: &DiagramProjection) -> Result<String, ExportError>;

/// Validate exported JSON against the schema
fn validate_export_schema(json: &str) -> Result<(), ExportError>;

/// Error type for export operations
enum ExportError {
    Serialization,
    SchemaValidation,
}
```

## Preconditions

1. DiagramProjection type exists and is accessible
2. Legacy code path for this slice is identified and removable in one commit
3. No authentication required for this operation

## Postconditions

1. Returns valid JSON string on success
2. JSON conforms to canonical schema
3. Legacy path is deleted or unreachable by compile-time guarantees
4. Replacement path passes focused tests with no fallback to removed code

## Invariants

1. No migration path is introduced
2. No dual-write compatibility path exists
3. All fallible operations use typed Result errors
4. Zero unwrap/expect usage

## Acceptance Criteria

### Happy Paths
- Given valid DiagramProjection, when export_diagram_json is called, then returns valid JSON string
- Given valid JSON, when validate_export_schema is called, then returns Ok(())

### Error Paths
- Given invalid state, when export is called, then returns appropriate ExportError
- Given malformed JSON, when validation is called, then returns ExportError::SchemaValidation

## Related Files

- diagram_tool/src/backend.rs - Existing patterns
- diagram_tool/src/patch.rs - Existing patterns
- diagram_tool/src/cli.rs - Existing patterns
- diagram_tool/src/models/document.rs - Existing patterns

## Implementation Tasks

1. Serialize projection to canonical schema
2. Validate output against JSON schema
3. Ensure all tests pass with Result<T, Error> throughout
