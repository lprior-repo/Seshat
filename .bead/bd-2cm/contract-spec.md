# Contract Specification: Atomic Persistence Functions

**Bead ID**: bd-2cm
**Feature**: storage-sync: add atomic redb-plus-file persistence
**Status**: Draft for Implementation

---

## Context

### Feature
Add atomic persistence functions to CLI (`cli_persistence.rs`) with:
- Temp file + fsync + atomic rename semantics
- Last Known Good (LKG) fallback on load failure
- Structured JSONL event emission with stage information

### Domain Terms
| Term | Definition |
|------|------------|
| **Atomic Write** | Write to temp file, validate, then `fs::rename` to target (same filesystem guarantee) |
| **LKG (Last Known Good)** | Backup file at `<path>.lkg` (e.g., `diagram.json.lkg`) used as fallback |
| **JSONL** | Newline-delimited JSON - each event is single-line JSON to stdout |
| **Stage Event** | `{"event":"stage","name":"<stage>",...}` emitted during persistence operations |

### Assumptions
- CLI is single-process; no concurrent save contention
- Target filesystem supports atomic `rename` within same directory
- `tempfile` crate will be added to dependencies (or manual temp file with UUID)
- `uuid` crate may be needed for unique temp file names

### Open Questions
1. ~~Temp file naming convention~~ → Use UUID suffix: `<name>.<ext>.<uuid>.tmp`
2. ~~LKG file naming~~ → `<path>.lkg` where path is `diagram.json.lkg` (suffix before extension)

---

## Function Contracts

### 1. `save_workspace_atomic`

**Signature**
```rust
pub fn save_workspace_atomic(doc: &DiagramDocument, path: &Path) -> Result<(), CliPersistenceError>
```

**Purpose**: Persist document to disk with atomic write semantics.

**Preconditions**
| ID | Precondition | Enforcement Level |
|----|--------------|-------------------|
| P1 | `doc.version == 2` | Runtime error variant `InvalidVersion` |
| P2 | `path.parent().exists()` OR `path == PathBuf::from("-")` (stdout) | Runtime error variant `ParentDirectoryNotFound` |
| P3 | Document passes `validate_schema(&doc)` | Runtime error variant `SchemaValidationFailed` |

**Postconditions (Success)**
| ID | Postcondition |
|----|---------------|
| Q1 | `path.exists()` returns `true` |
| Q2 | Contents at `path` are valid JSON matching `doc` |
| Q3 | No `.tmp` files remain in `path.parent()` |
| Q4 | `{"event":"stage","name":"persisted",...}` emitted to stdout |

**Postconditions (Failure)**
| ID | Postcondition |
|----|---------------|
| Q5 | Original `path` unchanged (if existed before) |
| Q6 | Temp file deleted |
| Q7 | `{"event":"stage","name":"error",...}` emitted to stdout |

**Invariants**
- Temp file written to same directory as target (ensures same filesystem for atomic rename)
- JSONL events are single-line (no embedded newlines in JSON)
- Atomic rename via `fs::rename`, not copy

**Side Effects**
- Writes to filesystem
- Emits JSONL to stdout

---

### 2. `load_workspace_with_lkg`

**Signature**
```rust
pub fn load_workspace_with_lkg(path: &Path) -> Result<DiagramDocument, CliPersistenceError>
```

**Purpose**: Load document from disk with LKG fallback on validation failure.

**Preconditions**
| ID | Precondition | Enforcement Level |
|----|--------------|-------------------|
| P4 | `path.exists()` OR `lkg_path.exists()` | Runtime error variant `FileNotFound` |

**Postconditions (Success - Primary)**
| ID | Postcondition |
|----|---------------|
| Q8 | Returns `Ok(valid_document)` |
| Q9 | Document passes `validate_schema(&doc)` |

**Postconditions (Success - LKG Fallback)**
| ID | Postcondition |
|----|---------------|
| Q10 | Primary load attempted and failed |
| Q11 | LKG file exists and was loaded |
| Q12 | `{"event":"stage","name":"lkg_fallback",...}` emitted |
| Q13 | Returned document is valid per `validate_schema` |

**Postconditions (Failure - Both Failed)**
| ID | Postcondition |
|----|---------------|
| Q14 | Returns `Err(CliPersistenceError::...)` with primary error |
| Q15 | `{"event":"stage","name":"error",...}` emitted for primary failure |
| Q16 | If LKG also fails, LKG failure logged but primary error returned |

**Invariants**
- LKG path is always `<path>.lkg` (suffix after full filename)
- Validation via `validate_schema` is mandatory
- JSONL events are single-line

**Side Effects**
- Reads from filesystem
- Emits JSONL to stdout

---

### 3. `emit_stage_event`

**Signature**
```rust
pub fn emit_stage_event(name: &str, details: Option<&serde_json::Value>)
```

**Purpose**: Emit a single-line JSONL stage event to stdout.

**Preconditions**
| ID | Precondition | Enforcement Level |
|----|--------------|-------------------|
| P5 | `name` is non-empty | Debug assert (debug_assert!) |
| P6 | `details` (if provided) serializes to single-line JSON | Fallback to static error JSON |

**Postconditions**
| ID | Postcondition |
|----|---------------|
| Q17 | Single line written to stdout (ends with `\n`) |
| Q18 | If serialization fails, fallback JSON emitted |

**Invariants**
- Output is always valid JSON
- Output is always single-line (no embedded `\n` in values)

---

## Error Taxonomy

```rust
#[derive(Debug, Error)]
pub enum CliPersistenceError {
    // Input validation errors
    #[error("document version must be 2, got {actual}")]
    InvalidVersion { actual: u32 },

    #[error("parent directory does not exist: {path}")]
    ParentDirectoryNotFound { path: PathBuf },

    #[error("schema validation failed: {reason}")]
    SchemaValidationFailed { reason: String },

    // File I/O errors
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("failed to read file: {path}")]
    ReadFailed { path: PathBuf, #[source] source: std::io::Error },

    #[error("failed to write temp file: {path}")]
    TempWriteFailed { path: PathBuf, #[source] source: std::io::Error },

    #[error("failed to rename temp file to target: {from} -> {to}")]
    RenameFailed { from: PathBuf, to: PathBuf, #[source] source: std::io::Error },

    // Parse errors
    #[error("failed to parse JSON: {path}")]
    ParseFailed { path: PathBuf, #[source] source: serde_json::Error },

    #[error("failed to serialize document")]
    SerializeFailed { #[source] source: serde_json::Error },

    // LKG-specific
    #[error("LKG fallback failed after primary error: {primary_error}")]
    LkgFallbackFailed { primary_error: String, #[source] source: Box<CliPersistenceError> },
}
```

---

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| `doc.version == 2` | Runtime error variant | `Result<T, CliPersistenceError::InvalidVersion>` |
| `path.parent().exists()` | Runtime error variant | `Result<T, CliPersistenceError::ParentDirectoryNotFound>` |
| Document valid via `validate_schema` | Runtime error variant | `Result<T, CliPersistenceError::SchemaValidationFailed>` |
| `path.exists()` | Runtime error variant | `Result<T, CliPersistenceError::FileNotFound>` |
| `name` non-empty (emit_stage_event) | Debug-only | `debug_assert!(!name.is_empty())` |
| JSONL single-line | Implementation invariant | Use `serde_json::to_string` which escapes newlines |

---

## Violation Examples (REQUIRED)

### `save_workspace_atomic` Violations

```
VIOLATES P1: save_workspace_atomic(&DiagramDocument{version: 99, ..}, &path)
  → Err(CliPersistenceError::InvalidVersion { actual: 99 })

VIOLATES P2: save_workspace_atomic(&doc, &PathBuf::from("/nonexistent/dir/test.json"))
  → Err(CliPersistenceError::ParentDirectoryNotFound { path: "/nonexistent/dir" })

VIOLATES P3: save_workspace_atomic(&doc_with_invalid_edge, &path)
  → Err(CliPersistenceError::SchemaValidationFailed { reason: "Edge e1 references non-existent source n_missing" })
```

### `load_workspace_with_lkg` Violations

```
VIOLATES P4: load_workspace_with_lkg(&PathBuf::from("/nonexistent/file.json"))
  → Err(CliPersistenceError::FileNotFound { path: "/nonexistent/file.json" })

VIOLATES P4 (no LKG either): load_workspace_with_lkg(&path) where path doesn't exist AND path.lkg doesn't exist
  → Err(CliPersistenceError::FileNotFound { path: "/path/to/file.json" })
```

### `emit_stage_event` Violations

```
VIOLATES P5: emit_stage_event("", None)
  → Triggers debug_assert! failure in debug builds (undefined behavior in release)

VIOLATES P6: emit_stage_event("test", Some(&json!({"nested": "has\nnewline"})))
  → Emits fallback: {"event":"stage","name":"test","error":"jsonl_encode_error"}
```

### Postcondition Violations

```
VIOLATES Q1 (after save success): path.exists() returns false
  → Should never happen; indicates filesystem or OS bug

VIOLATES Q5 (after save failure): Original path contents changed
  → Should never happen; indicates temp file written directly to target

VIOLATES Q6 (after save failure): .tmp file remains
  → Temp file cleanup missing in error path

VIOLATES Q9 (after load success): validate_schema(&doc) fails
  → Indicates validation skipped or corrupted after validation
```

---

## Ownership Contracts

### `save_workspace_atomic(doc: &DiagramDocument, path: &Path)`

| Parameter | Mode | Contract |
|-----------|------|----------|
| `doc` | `&DiagramDocument` | Shared borrow; read-only; document not modified |
| `path` | `&Path` | Shared borrow; path not modified |

**Clone Policy**: No cloning of `doc` except for serialization buffer (owned `String`).

### `load_workspace_with_lkg(path: &Path) -> Result<DiagramDocument, ...>`

| Parameter | Mode | Contract |
|-----------|------|----------|
| `path` | `&Path` | Shared borrow; path not modified |

**Ownership Transfer**: Returns owned `DiagramDocument` on success.

### `emit_stage_event(name: &str, details: Option<&serde_json::Value>)`

| Parameter | Mode | Contract |
|-----------|------|----------|
| `name` | `&str` | Shared borrow; string not modified |
| `details` | `Option<&serde_json::Value>` | Shared borrow; value not modified |

---

## JSONL Event Schema

### Stage Event
```json
{"event":"stage","name":"<stage_name>","path":"<optional_path>","details":{...}}
```

### Stage Names
| Name | When Emitted | Additional Fields |
|------|--------------|-------------------|
| `validating` | Before schema validation | `path` |
| `persisted` | After successful atomic write | `path` |
| `error` | On any failure | `code`, `message` |
| `lkg_fallback` | When falling back to LKG | `reason`, `path` |

### Error Codes
| Code | Meaning |
|------|---------|
| `invalid_version` | Document version != 2 |
| `parent_not_found` | Parent directory doesn't exist |
| `schema_validation_failed` | Document failed schema validation |
| `file_not_found` | Neither primary nor LKG file exists |
| `parse_error` | JSON parsing failed |
| `write_error` | File write failed |
| `rename_error` | Atomic rename failed |

---

## Non-goals

- [ ] Backup file rotation (keep N backups)
- [ ] Compression of persisted files
- [ ] Encryption at rest
- [ ] Concurrent write locking (CLI is single-process)
- [ ] Schema version migration (only v2 supported)
- [ ] WASM backend persistence (that's `backend.rs`)
- [ ] UI toast integration (that's `persistence.rs`)

---

## Implementation Notes

### Temp File Naming
```
<filename>.<ext>.<uuid>.tmp
```
Example: `diagram.json.a1b2c3d4-e5f6-7890-abcd-ef1234567890.tmp`

### LKG File Naming
```
<full_filename>.lkg
```
Example: `diagram.json` → `diagram.json.lkg`

### Atomic Rename Guarantee
- Write temp to `path.parent()` (same directory)
- Use `fs::rename` which is atomic on same filesystem
- On error, attempt `fs::remove_file` on temp (best effort)

### Cleanup on Error
```rust
// Pseudocode
let temp_path = compute_temp_path(path);
match fs::write(&temp_path, &json) {
    Ok(()) => {
        match fs::rename(&temp_path, path) {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = fs::remove_file(&temp_path); // best effort cleanup
                Err(e.into())
            }
        }
    }
    Err(e) => {
        let _ = fs::remove_file(&temp_path); // best effort cleanup
        Err(e.into())
    }
}
```
