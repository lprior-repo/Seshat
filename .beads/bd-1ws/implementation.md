bead_id: bd-1ws
bead_title: io-json-import: import diagram json by generating canonical events
phase: p2
updated_at: 2026-03-01T20:55:00Z

# Implementation: io-json-import

## Location

`diagram_tool/src/models/export.rs`

## Implementation Summary

The import functionality has been implemented with the following components:

### Types

```rust
/// Result of an import operation
pub struct ImportResult {
    /// Number of events generated
    pub events_generated: u64,
    /// Final revision after import
    pub final_revision: u64,
}

/// Author of operations (contract version)
pub struct Author {
    /// Author identifier
    pub id: String,
    /// Whether author is human
    pub is_human: bool,
}
```

### Functions

1. **`import_diagram_json(conn: &mut Connection, input: &str, actor: Author) -> Result<ImportResult, ExportError>`**
   - Parses the JSON input into `DiagramJsonExport`
   - Validates schema version compatibility
   - Deserializes projection from data field
   - Converts to document and validates against schema
   - Gets events to import (from export or generates via `generate_canonical_events`)
   - Appends each event to the store with OCC
   - Returns `ImportResult` with events generated and final revision

2. **`generate_canonical_events(projection: &DiagramProjection) -> Vec<serde_json::Value>`**
   - Generates `NodeAdd` events for all nodes in projection
   - Generates `EdgeConnect` events for all edges in projection
   - Creates `EventRecord` structures with proper metadata
   - Returns events as JSON values for storage

3. **`Author::to_envelope_author(&self) -> EnvelopeAuthor`**
   - Converts contract Author to envelope Author
   - Prefixes human authors with "human-"
   - Preserves AI author IDs as-is

## Contract Compliance

- [x] `import_diagram_json` - Returns `Result<ImportResult, ExportError>`
- [x] Canonical event generation from projection
- [x] Events can be replayed to reproduce imported state
- [x] All fallible operations use typed Result errors
- [x] Zero unwrap/expect usage (enforced by clippy)

## Related Files

- `diagram_tool/src/models/projection.rs` - DiagramProjection and replay
- `diagram_tool/src/models/envelope.rs` - EventEnvelope, Author, DomainOp
- `diagram_tool/src/models/schema.rs` - Schema validation
- `diagram_tool/src/store.rs` - Database operations (append_event, fetch_latest_revision)
