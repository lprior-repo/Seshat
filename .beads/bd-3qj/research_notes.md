# Research Notes: bd-3qj io-json-export

## Files Analyzed

### diagram_tool/src/models/export.rs
- Contains existing `export_diagram_json(conn: &Connection) -> Result<DiagramJsonExport, ExportError>`
- Contains `ExportError` enum with variants: InvalidSchema, Serialization, Sqlite, Validation
- Contains `DiagramJsonExport` struct with metadata, data, and events fields
- Contains `import_diagram_json` for the import path

### diagram_tool/src/models/projection.rs
- Contains `DiagramProjection` struct with nodes, edges, revision, and other fields
- Contains `replay_events` function for deterministic replay
- Contains `projection_to_document` and `document_to_projection` conversion functions

### diagram_tool/src/models/canonical_json.rs
- Contains `to_canonical_pretty_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error>`
- Provides canonical (deterministic) JSON serialization with sorted keys

### diagram_tool/src/models/document.rs
- Contains `DiagramDocument`, `Node`, `Edge`, and related types
- Uses `OrderedFloat` for float handling

### diagram_tool/src/ui/toolbar/export_actions.rs
- Uses `to_canonical_pretty_json` for JSON export
- Current `export_json` function exports a `DiagramDocument`

## Contract Requirements

The contract specifies:
1. `fn export_diagram_json(state: &DiagramProjection) -> Result<String, ExportError>`
2. `fn validate_export_schema(json: &str) -> Result<(), ExportError>`
3. `enum ExportError { Serialization, SchemaValidation }`

## Gap Analysis

The existing `export_diagram_json`:
- Takes `&Connection` (database-dependent)
- Returns `DiagramJsonExport` struct
- Uses `ExportError` with 4 variants

The contract requires:
- Taking `&DiagramProjection` directly (no database)
- Returning `String` (JSON string)
- Using `ExportError` with 2 variants (Serialization, SchemaValidation)

## Implementation Plan

1. Add new function `export_projection_json(projection: &DiagramProjection) -> Result<String, ExportError>`
2. Add new function `validate_export_schema(json: &str) -> Result<(), ExportError>`
3. Ensure the existing ExportError enum can support the new variants or create a new error type
4. Write tests for the new functions
