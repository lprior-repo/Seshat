# Implementation: bd-2ik - json-roundtrip

## Overview

Implemented schema-valid JSON import and export pipeline for diagram data. The implementation follows functional Rust patterns with zero unwrap/expect usage.

## Contract Requirements Met

- **Function Signature**: `fn export_diagram_json(conn: &Connection) -> Result<DiagramJsonExport, ExportError>`
- **Error Enum**: `enum ExportError { InvalidSchema, Serialization, Sqlite, Validation }`
- **Import Function**: `fn import_diagram_json(conn: &mut Connection, input: &str, actor: Author) -> Result<ImportResult, ExportError>`

## Implementation Details

### Export Pipeline (`export_diagram_json`)

1. **Fetch Events**: Reads all events from the SQLite database in revision order
2. **Replay Events**: Converts DB events to `EventRecord` and replays using deterministic projection replay
3. **Validate Schema**: Validates the resulting document against schema version 2
4. **Serialize**: Converts projection to JSON with metadata (name, revision, version)
5. **Include Events**: Optionally includes event bundle for replay capability

### Import Pipeline (`import_diagram_json`)

1. **Parse Input**: Deserializes JSON input to `DiagramJsonExport`
2. **Validate Schema**: Checks schema version compatibility
3. **Validate Data**: Validates projection data against schema
4. **Generate Events**: Either uses provided events or generates canonical events from projection data
5. **Append Events**: Uses store's OCC (Optimistic Concurrency Control) to append events
6. **Handle Idempotency**: Skips duplicate events gracefully

### Key Design Decisions

1. **Revision Handling**: DB revisions start at 1, but replay expects starting at 0. Implementation adjusts by subtracting 1 from each event's revision during export.

2. **Schema Version**: Projection uses version 1 internally, but document requires version 2. Conversion sets correct version.

3. **Author Conversion**: Contract's `Author` (id, is_human) is converted to envelope's `Author` (id, name, email) by prefixing human IDs with "human-".

4. **Idempotent Import**: Uses OCC to handle revision conflicts - duplicate operations are skipped gracefully.

## Functional Patterns Used

- `map`, `and_then`, `filter_map` for transformations
- `try_fold` for event replay
- `Result<T, E>` for all error handling
- No unwrap/expect/panic anywhere

## Tests

8 tests implemented covering:
- Empty database export
- Database with events export
- Empty import
- Import with valid JSON
- Invalid JSON error handling
- Revision mismatch handling
- Author conversion (human and AI)

## Files Modified

- `/home/lewis/src/seshat/diagram_tool/src/models/export.rs` - Complete implementation

## Verification

All export tests pass:
```
cargo test models::export::tests
```
