# ADR-017: Save File Implementation

## Status

**Accepted** (2026-04-03)

## Context

We need a robust, atomic save mechanism for `.seshat.json` files that ensures no data loss on crash or power failure. The existing `physical_io::save_document` in `diagram_models/src/physical_io.rs` lacks atomic guarantees. The CLI already has atomic save via `cli_persistence::write::save_workspace_atomic`, but the UI path needs alignment.

## Decision

### Atomic Write Pattern

All saves to `.seshat.json` files MUST use the atomic write pattern:

1. **Write to temp file** in the same directory as target: `.<filename>.tmp.<pid>`
2. **fsync the temp file** to ensure data reaches disk platter
3. **Atomic rename** temp file to target path
4. **fsync the parent directory** to ensure rename is durable

This ensures:
- Original file untouched if process crashes during write
- File is either fully written or not written at all
- No partial/corrupted files left behind

### Implementation Location

```
diagram_tool/src/cli_persistence/write.rs  → Already implemented
diagram_tool/src/ui/toolbar/persistence/save.rs  → Needs atomic upgrade
diagram_models/src/physical_io.rs  → Base implementation (non-atomic)
```

### File Format

Save files are **canonical JSON** (per `diagram_models/src/canonical_json.rs`):
- Deterministic key ordering
- Pretty-printed for human readability and version control
- No trailing newline differences

### Schema Validation Before Save

Before any save:
1. Validate all floats are finite (no NaN/Infinity)
2. Validate serialization depth < 100
3. Run `validate_schema()` from `diagram_models/src/schema/mod.rs`

### Error Handling

| Error | User Message | Recovery |
|-------|-------------|----------|
| Disk full | "Save failed: disk full" | Offer Save As to different location |
| Read-only file | "Cannot save to read-only file" | Offer Save As |
| Network disconnect | "Cannot reach save location. Changes saved to local cache" | Auto-retry when reconnected |
| Validation failure | "Save failed: document invalid" | Log error, keep dirty=true |
| Temp file creation fails | "Save failed: cannot create temp file" | Offer Save As |

### Save Triggers

| Trigger | Path |
|---------|------|
| Cmd+S / Ctrl+S | `ui/toolbar/persistence/save.rs::save_workspace` |
| Menu → Save | Same as above |
| Menu → Save As | `ui/toolbar/persistence/save.rs::save_workspace_as` |
| CLI `seshat save` | `cli_persistence/write.rs::save_workspace_atomic` |
| Auto-save timer | Writes to SQLite only, NOT to file |

### Contracts

**Preconditions:**
- Parent directory exists and is writable
- Document has no NaN/Infinity floats
- Document passes structural validation

**Postconditions:**
- File written atomically
- Dirty flag cleared (DocumentSession.mark_saved() called)
- SQLite updated to match file (per ADR-016)

## Consequences

### Positive
- Crash-proof saves - never lose user data
- Git-friendly canonical JSON for meaningful diffs
- Same atomic pattern across CLI and UI

### Negative
- fsync on every save may be slow on some drives
- Temp file left behind if atomic rename fails (handled by cleanup)

### Risks
- NFS and some network mounts don't support atomic rename properly - mitigated by detecting and warning
- SSD wear from frequent fsync - acceptable tradeoff for data safety

## Machine-Readable Spec (JSONL)

```jsonl
{"type":"adr","id":"adr-017","title":"Save File Implementation","status":"proposed","date":"2026-04-03"}
{"type":"contract","domain":"save","preconditions":[{"field":"parent_dir","constraint":"exists_and_writable"},{"field":"floats","constraint":"all_finite"},{"field":"schema","constraint":"valid"}],"postconditions":[{"field":"file","constraint":"written_atomically"},{"field":"dirty_flag","constraint":"cleared"},{"field":"sqlite","constraint":"updated"}]}
{"type":"implementation","component":"save","locations":[{"path":"diagram_tool/src/cli_persistence/write.rs","function":"save_workspace_atomic","status":"implemented"},{"path":"diagram_tool/src/ui/toolbar/persistence/save.rs","function":"save_workspace","status":"needs_atomic_upgrade"},{"path":"diagram_models/src/physical_io.rs","function":"save_document","status":"base_implementation"}],"atomic_pattern":["temp_file_creation","fsync_temp","atomic_rename","fsync_parent_dir"]}
```
