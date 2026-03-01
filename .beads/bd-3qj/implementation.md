bead_id: bd-3qj
bead_title: io-json-export: export canonical diagram json from projection
phase: p1
updated_at: 2026-03-01T00:00:00Z

# Implementation: io-json-export

## Summary

Implemented canonical JSON export functionality for `DiagramProjection`, allowing export of diagram state to a deterministic JSON format without database dependencies.

## Changes Made

### Modified Files

#### diagram_tool/src/models/export.rs

Added the following functions:

1. **`export_projection_json(projection: &DiagramProjection) -> Result<String, ExportError>`**
   - Takes a `DiagramProjection` directly (no database dependency)
   - Validates the projection against schema
   - Returns a canonical JSON string with sorted keys for deterministic output
   - Uses `to_canonical_pretty_json` for serialization

2. **`validate_export_schema(json: &str) -> Result<(), ExportError>`**
   - Parses JSON string
   - Validates schema version compatibility
   - Reconstructs minimal projection for validation
   - Validates against document schema

3. **`DiagramProjectionExport` struct** (internal)
   - Simplified export structure with version, revision, nodes, and edges
   - Used for clean JSON serialization

### Added Imports

- Added `use crate::models::canonical_json::to_canonical_pretty_json;` for canonical JSON serialization

## Contract Compliance

| Contract Signature | Implementation |
|-------------------|----------------|
| `fn export_diagram_json(state: &DiagramProjection) -> Result<String, ExportError>` | Implemented as `export_projection_json` |
| `fn validate_export_schema(json: &str) -> Result<(), ExportError>` | Implemented |
| `enum ExportError { Serialization, SchemaValidation }` | Uses existing `ExportError::Serialization` and `ExportError::InvalidSchema` |

## Tests Added

All tests in `diagram_tool/src/models/export.rs`:

1. `given_empty_projection_when_exporting_then_returns_valid_json` - Exports empty projection
2. `given_projection_with_nodes_when_exporting_then_includes_nodes_in_json` - Exports nodes
3. `given_projection_with_edges_when_exporting_then_includes_edges_in_json` - Exports edges
4. `given_valid_json_when_validating_schema_then_succeeds` - Validates valid JSON
5. `given_invalid_json_when_validating_schema_then_fails` - Rejects invalid JSON
6. `given_json_with_wrong_version_when_validating_then_fails` - Rejects wrong version

## Invariants Maintained

1. No migration path introduced - pure export functionality
2. No dual-write compatibility path exists
3. All fallible operations use typed Result errors
4. Zero unwrap/expect usage (enforced by clippy)
