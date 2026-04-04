# ADR-018: Load File Implementation

## Status

**Accepted** (2026-04-03)

## Context

Loading files must handle corruption, schema evolution, and provide recovery options when files are damaged. The existing `physical_io::load_document` in `diagram_models/src/physical_io.rs` provides basic loading but lacks LKG (Last Known Good) fallback. The CLI has `cli_persistence::read::load_workspace_with_lkg` that should be the reference implementation.

## Decision

### Load with LKG Fallback (Native)

Native file loads MUST implement Last Known Good fallback:

1. **Try primary file first** - Attempt to load and validate the requested file
2. **Fall back to LKG on failure** - If primary fails, try `.lkg/<filename>.lkg`
3. **Return first valid document** - If either succeeds, return that document

LKG files are created automatically on each successful save (see ADR-017).

### WASM Limitation

**WASM does NOT support LKG** due to browser security constraints. The browser's FileReader API provides a one-shot read of a user-selected file with no filesystem access for LKG fallback. This is a fundamental architectural constraint, not a bug.

### LKG Directory Structure

```
project/
├── my-diagram.seshat.json    ← Primary file
└── .lkg/
    └── my-diagram.seshat.json.lkg  ← Last Known Good backup
```

### Implementation Location

```
diagram_tool/src/cli_persistence/read.rs  → load_workspace_with_lkg (reference implementation)
diagram_tool/src/ui/toolbar/persistence/open.rs  → open_workspace (native: LKG enabled, WASM: no LKG)
diagram_tool/src/ui/toolbar/persistence/common.rs  → prepare_import_transition (shared validation)
diagram_models/src/physical_io.rs  → Base implementation (no LKG)
```

### Validation Pipeline

On load, the following validations run in order:

1. **Read file** - Read entire file contents into memory
2. **UTF-8 validation** - Ensure file is valid UTF-8
3. **JSON parse** - Parse as JSON, fail on syntax error
4. **Depth check** - Reject if nesting depth > 100 (DOS protection)
5. **Structure validation** - Check required fields (version, document, nodes, edges)
6. **Schema validation** - Run `validate_schema()` from `diagram_models/src/schema/mod.rs`
7. **Schema migration** - If version < current, run migration pipeline
8. **Return migrated document** - Fully migrated, validated document

### Schema Migration

Per ADR-014 and `diagram_models/src/physical_io/migration.rs`:

- Version 1 → Version 2: Add new fields with defaults
- Future migrations: Incremental, additive only (no breaking changes)

If file version is **newer** than current supported version:
- Load with warnings
- Preserve unknown fields
- Show "File from newer Seshat version"

### Error Handling

| Error | User Message | Recovery |
|-------|-------------|----------|
| File not found | "File not found: <filename>" | Show file picker |
| Permission denied | "Cannot read file: permission denied" | Show file picker |
| UTF-8 invalid | "File is not valid UTF-8" | Offer LKG or cancel |
| JSON syntax error | "File is corrupted (invalid JSON)" | Offer LKG |
| Schema validation fails | "File is corrupted (failed validation)" | Offer LKG |
| Version too new | "File from newer Seshat version" | Load with warnings |
| Both primary and LKG fail | "Cannot open file. Both primary and backup are corrupted." | Show file picker |

### File Picker Integration

| Platform | Implementation |
|----------|---------------|
| Native (desktop) | `rfd::FileDialog` for open |
| WASM (browser) | HTML `<input type="file">` element |

### Contracts

**Preconditions:**
- File exists at path
- File is readable
- File contains valid UTF-8

**Postconditions:**
- In-memory state matches file content
- SQLite overwritten with file content (per ADR-016)
- Dirty flag cleared
- `file_path` set to loaded path

## Consequences

### Positive
- Robust recovery from corruption via LKG
- Graceful handling of schema evolution
- Clear error messages with recovery options

### Negative
- LKG files consume additional disk space (mitigated by single-file policy)
- Migration complexity for schema changes

### Risks
- LKG could also be corrupted if corruption happened during save - mitigated by atomic save
- Large files could cause memory pressure - mitigated by depth limits

## Machine-Readable Spec (JSONL)

```jsonl
{"type":"adr","id":"adr-018","title":"Load File Implementation","status":"proposed","date":"2026-04-03"}
{"type":"contract","domain":"load","preconditions":[{"field":"file_exists","constraint":"path_exists"},{"field":"file_readable","constraint":"is_readable"},{"field":"encoding","constraint":"utf8"}],"postconditions":[{"field":"in_memory","constraint":"matches_file"},{"field":"sqlite","constraint":"overwritten"},{"field":"dirty_flag","constraint":"cleared"},{"field":"file_path","constraint":"set"}]}
{"type":"implementation","component":"load","locations":[{"path":"diagram_tool/src/cli_persistence/read.rs","function":"load_workspace_with_lkg","status":"implemented"},{"path":"diagram_tool/src/ui/toolbar/persistence/open.rs","function":"open_workspace","status":"needs_lkg_integration"},{"path":"diagram_models/src/physical_io.rs","function":"load_document","status":"base_implementation"}],"validation_pipeline":["read_file","utf8_validation","json_parse","depth_check","structure_validation","schema_validation","migration"]}
{"type":"error_recovery","scenario":"primary_file_corrupted","fallback":"lkg_file","lkg_path":".lkg/<filename>.lkg"}
```
