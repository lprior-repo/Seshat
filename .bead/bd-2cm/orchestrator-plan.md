# Orchestrator Plan: bd-2cm

## storage-sync: add atomic redb-plus-file persistence

**Bead ID**: bd-2cm  
**Effort**: 2hr  
**Priority**: 2 (High)  
**Type**: feature

---

## 1. Clarifications

### Problem Statement
The CLI (`cli.rs`) uses basic file I/O (`save_doc`, `load_doc`) without:
- Atomic write guarantees (write-to-temp, then rename)
- Last Known Good (LKG) fallback on corrupt files
- Structured JSONL event emission with stage information

The UI toolbar persistence (`persistence.rs`) has atomic transition patterns but is tightly coupled to Dioxus signals and toast notifications, making it unsuitable for CLI reuse.

### Scope Boundaries
| In Scope | Out of Scope |
|----------|--------------|
| CLI atomic persistence functions | UI/toolbar persistence changes |
| JSONL stage events (validating, persisted, error) | WASM backend persistence |
| LKG fallback for load operations | Backup file rotation |
| Integration with existing `validate_schema` | Schema version migration |

### Success Criteria
1. `save_workspace_atomic()` writes to temp file, validates, then atomic renames
2. `load_workspace_with_lkg()` attempts load, falls back to `.lkg` on validation failure
3. CLI emits `{"stage": "validating"}`, `{"stage": "persisted"}`, `{"stage": "error", ...}`
4. All existing CLI tests pass
5. New functions have unit tests for happy path, error path, and edge cases

---

## 2. EARS Requirements

### Ubiquitous
- **THE SYSTEM SHALL** provide `save_workspace_atomic(doc, path)` that writes to a temp file, validates the written content, and atomically renames to the target path
- **THE SYSTEM SHALL** provide `load_workspace_with_lkg(path)` that loads a document, validates it, and falls back to `<path>.lkg` if validation fails
- **THE SYSTEM SHALL** emit JSONL events to stdout for each persistence operation stage

### Event-Driven
| Trigger | Shall |
|---------|-------|
| WHEN `save_workspace_atomic` is called | THE SYSTEM SHALL emit `{"event": "stage", "name": "validating", ...}` before validation |
| WHEN `save_workspace_atomic` succeeds | THE SYSTEM SHALL emit `{"event": "stage", "name": "persisted", "path": "..."}` |
| WHEN `save_workspace_atomic` fails | THE SYSTEM SHALL emit `{"event": "stage", "name": "error", "code": N, "message": "..."}` |
| WHEN `load_workspace_with_lkg` falls back to LKG | THE SYSTEM SHALL emit `{"event": "stage", "name": "lkg_fallback", "reason": "..."}` |

### Unwanted
| Condition | Shall Not |
|-----------|-----------|
| IF the temp file write fails | THE SYSTEM SHALL NOT leave partial/corrupt data at the target path |
| IF validation fails on load | THE SYSTEM SHALL NOT return an invalid document without attempting LKG fallback |
| IF JSONL serialization fails | THE SYSTEM SHALL NOT panic; it SHALL emit a fallback error JSON |

---

## 3. KIRK Contracts

### Preconditions
```rust
// save_workspace_atomic
precondition: path.parent().exists() || path == PathBuf::from("-")
precondition: doc.version == 2

// load_workspace_with_lkg  
precondition: path.exists() || path.with_extension("json.lkg").exists()
```

### Postconditions
```rust
// save_workspace_atomic (success)
postcondition: target_path.exists()
postcondition: target_path contains valid JSON matching doc
postcondition: no temp files remain in target directory

// save_workspace_atomic (failure)
postcondition: target_path unchanged from before call
postcondition: temp file deleted

// load_workspace_with_lkg (success)
postcondition: returned Result is Ok(valid_document)
postcondition: returned document passes validate_schema()

// load_workspace_with_lkg (fallback)
postcondition: if primary fails, lkg_path was attempted
postcondition: fallback reason logged via JSONL
```

### Invariants
- Atomic rename is used (not copy) for final persistence
- JSONL events are always single-line (no embedded newlines)
- LKG file has `.lkg` suffix before extension: `diagram.json.lkg`

---

## 4. Research Requirements

### Already Known
- `validate_schema` exists in `models/schema.rs` and works
- JSONL pattern established in `cli.rs` lines 87-165
- File operations use `std::fs` with `anyhow` error context

### Needs Investigation
- [ ] Determine temp file naming convention (`.tmp` suffix vs random)
- [ ] Confirm atomic rename works across filesystems on target platform
- [ ] Check if `.lkg` files already exist in any test fixtures

---

## 5. Inversions (What Could Go Wrong)

| Risk | Mitigation |
|------|------------|
| Temp file permissions differ from target | Use `NamedTempFile` from `tempfile` crate which preserves umask |
| Atomic rename fails across filesystems | Write temp to same directory as target |
| LKG file is also corrupt | Return original error, log LKG failure |
| JSONL event serialization panics | Use `serde_json::to_string` with fallback static JSON |
| Race condition on concurrent saves | Filesystem advisory locks not required for CLI (single process) |

---

## 6. ATDD Tests (Unit)

### Happy Path
```rust
#[test]
fn given_valid_doc_when_save_atomic_then_file_exists_and_valid() {
    let doc = DiagramDocument::default();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");
    
    save_workspace_atomic(&doc, &path).unwrap();
    
    assert!(path.exists());
    let loaded: DiagramDocument = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert!(validate_schema(&loaded).is_ok());
}

#[test]
fn given_valid_file_when_load_with_lkg_then_returns_doc() {
    let doc = DiagramDocument::default();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");
    fs::write(&path, serde_json::to_string(&doc).unwrap()).unwrap();
    
    let result = load_workspace_with_lkg(&path).unwrap();
    
    assert!(validate_schema(&result).is_ok());
}
```

### Error Path
```rust
#[test]
fn given_invalid_doc_when_save_atomic_then_target_unchanged() {
    let mut doc = DiagramDocument::default();
    doc.version = 99; // Invalid version
    
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");
    fs::write(&path, r#"{"existing": true}"#).unwrap();
    let before = fs::read_to_string(&path).unwrap();
    
    let result = save_workspace_atomic(&doc, &path);
    
    assert!(result.is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn given_corrupt_file_when_load_with_lkg_then_falls_back() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");
    let lkg_path = temp_dir.path().join("test.json.lkg");
    
    fs::write(&path, "{not valid json").unwrap();
    let valid_doc = DiagramDocument::default();
    fs::write(&lkg_path, serde_json::to_string(&valid_doc).unwrap()).unwrap();
    
    let result = load_workspace_with_lkg(&path).unwrap();
    assert!(validate_schema(&result).is_ok());
}
```

### Edge Cases
```rust
#[test]
fn given_no_lkg_when_corrupt_load_then_returns_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");
    fs::write(&path, "{corrupt").unwrap();
    
    let result = load_workspace_with_lkg(&path);
    assert!(result.is_err());
}

#[test]
fn given_missing_parent_dir_when_save_then_returns_error() {
    let doc = DiagramDocument::default();
    let path = PathBuf::from("/nonexistent/dir/test.json");
    
    let result = save_workspace_atomic(&doc, &path);
    assert!(result.is_err());
}
```

---

## 7. E2E Tests (CLI Integration)

### Test 1: Full Save-Load Cycle
```bash
# Create valid diagram
echo '{"version":2,"revision":0,"document":{"nodes":{},"edges":{}},"editor_state":{...}}' > input.json

# Save via CLI
./diagram_tool validate --input input.json
# Expected: {"event":"start",...} {"event":"stage","name":"validating",...} {"event":"finish",...}

# Load via CLI  
./diagram_tool render --input input.json --output output.svg
# Expected: SVG file created, stage events emitted
```

### Test 2: LKG Fallback
```bash
# Create corrupt file with LKG backup
echo "{corrupt" > diagram.json
echo '{"version":2,...}' > diagram.json.lkg

# Load should succeed with fallback
./diagram_tool validate --input diagram.json
# Expected: {"event":"stage","name":"lkg_fallback",...} {"event":"finish",...}
```

---

## 8. Verification Checkpoints

| Checkpoint | Command | Expected |
|------------|---------|----------|
| Code compiles | `moon run diagram_tool:check` | Success |
| Clippy clean | `moon run diagram_tool:clippy` | No warnings |
| Unit tests pass | `moon run diagram_tool:test -- persistence` | All pass |
| Integration test | `moon run diagram_tool:test -- cli` | All pass |
| Format check | `moon run diagram_tool:fmt -- --check` | No changes |

---

## 9. Implementation Tasks

### Task 1: Create `cli_persistence.rs` Module (30min)
```
Location: diagram_tool/src/cli_persistence.rs
Contents:
  - save_workspace_atomic(doc: &DiagramDocument, path: &Path) -> Result<()>
  - load_workspace_with_lkg(path: &Path) -> Result<DiagramDocument>
  - emit_stage_event(name: &str, details: Option<serde_json::Value>)
  - CliPersistenceError enum
```

### Task 2: Implement Atomic Save (30min)
```rust
pub fn save_workspace_atomic(doc: &DiagramDocument, path: &Path) -> Result<()> {
    emit_stage_event("validating", None);
    
    // Validate before any I/O
    validate_schema(doc).context("Document failed schema validation")?;
    
    // Serialize to string first (catches serialization errors early)
    let json = serde_json::to_string_pretty(doc).context("Failed to serialize document")?;
    
    // Write to temp file in same directory (ensures same filesystem for rename)
    let temp_path = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    fs::write(&temp_path, &json).context("Failed to write temp file")?;
    
    // Atomic rename
    fs::rename(&temp_path, path).context("Failed to rename temp file to target")?;
    
    emit_stage_event("persisted", Some(json!({"path": path.display().to_string()})));
    Ok(())
}
```

### Task 3: Implement LKG Load (30min)
```rust
pub fn load_workspace_with_lkg(path: &Path) -> Result<DiagramDocument> {
    // Try primary load
    match load_and_validate(path) {
        Ok(doc) => return Ok(doc),
        Err(primary_err) => {
            emit_stage_event("error", Some(json!({
                "code": "validation_failed",
                "message": primary_err.to_string()
            })));
            
            // Try LKG fallback
            let lkg_path = path.with_extension(format!("{}.lkg", 
                path.extension().map(|e| e.to_string_lossy()).unwrap_or_default()
            ));
            
            if lkg_path.exists() {
                emit_stage_event("lkg_fallback", Some(json!({"path": lkg_path.display().to_string()})));
                return load_and_validate(&lkg_path)
                    .context(format!("LKG load failed after primary failure: {}", primary_err));
            }
            
            Err(primary_err)
        }
    }
}

fn load_and_validate(path: &Path) -> Result<DiagramDocument> {
    emit_stage_event("validating", Some(json!({"path": path.display().to_string()})));
    let contents = fs::read_to_string(path).context("Failed to read file")?;
    let doc: DiagramDocument = serde_json::from_str(&contents).context("Failed to parse JSON")?;
    validate_schema(&doc).context("Schema validation failed")?;
    Ok(doc)
}
```

### Task 4: Wire CLI Commands (15min)
```rust
// In cli.rs, replace load_doc and save_doc:

use crate::cli_persistence::{load_workspace_with_lkg, save_workspace_atomic};

fn execute_command(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Render { input, output } => {
            let doc = load_workspace_with_lkg(Path::new(input))?;
            // ... render logic ...
        }
        Commands::Patch { input, patch, output } => {
            let doc = load_workspace_with_lkg(Path::new(input))?;
            // ... patch logic ...
            save_workspace_atomic(&patched_doc, Path::new(output))?;
        }
        // ... etc
    }
}
```

### Task 5: Update Module Exports (5min)
```rust
// In diagram_tool/src/lib.rs or main.rs:
mod cli_persistence;
pub use cli_persistence::*;
```

### Task 6: Add Unit Tests (30min)
- Add tests to `cli_persistence.rs` under `#[cfg(test)]` module
- Cover happy path, error path, edge cases as defined in ATDD section

---

## 10. Failure Modes

| Failure | Detection | Recovery |
|---------|-----------|----------|
| Temp file write fails | `fs::write` returns Err | Return error, target unchanged |
| Rename fails | `fs::rename` returns Err | Return error, try to clean up temp file |
| Primary file corrupt | `serde_json::from_str` fails | Attempt LKG fallback |
| LKG also corrupt | Same as primary | Return original error |
| Validation fails | `validate_schema` returns Err | Do not persist / do not return doc |

---

## 11. Anti-Hallucination

### DO NOT
- Invent functions that don't exist in codebase
- Assume `tempfile` crate is already in dependencies (need to add)
- Create WASM-specific code paths
- Modify `persistence.rs` toolbar functions

### DO
- Use existing `validate_schema` from `models/schema.rs`
- Follow existing error handling patterns in `cli.rs`
- Use `anyhow` for error handling (already in use)
- Match existing JSONL event structure from `cli.rs`

### Verification
After implementation, run:
```bash
grep -r "save_workspace_atomic\|load_workspace_with_lkg" diagram_tool/src/
```
Should find exactly 2 function definitions and N call sites.

---

## 12. Context Survival

### If Context Is Lost

1. **Read this file first**: `.bead/bd-2cm/orchestrator-plan.md`
2. **Key files to read**:
   - `diagram_tool/src/cli.rs` (lines 1-231) - existing CLI structure
   - `diagram_tool/src/models/schema.rs` - validation function
   - `diagram_tool/src/ui/toolbar/persistence.rs` - reference patterns (don't modify)
3. **Create new file**: `diagram_tool/src/cli_persistence.rs`
4. **Modify existing**: `diagram_tool/src/cli.rs` to import and use new functions

### State Recovery Questions
- [ ] Has `cli_persistence.rs` been created?
- [ ] Are `save_workspace_atomic` and `load_workspace_with_lkg` defined?
- [ ] Does `cli.rs` import from `cli_persistence`?
- [ ] Do tests pass?

---

## 13. Completion Checklist

- [ ] `cli_persistence.rs` created with all functions
- [ ] `save_workspace_atomic` implemented with temp file + atomic rename
- [ ] `load_workspace_with_lkg` implemented with fallback logic
- [ ] `emit_stage_event` helper function created
- [ ] CLI commands wired to use new persistence functions
- [ ] Unit tests for happy path added
- [ ] Unit tests for error path added
- [ ] Unit tests for edge cases added
- [ ] `moon run diagram_tool:check` passes
- [ ] `moon run diagram_tool:clippy` passes
- [ ] `moon run diagram_tool:test` passes
- [ ] Manual test: save/load cycle works
- [ ] Manual test: corrupt file triggers LKG fallback

---

## 14. Context

### Dependencies
```toml
# Already in Cargo.toml
anyhow = "1.0"
serde_json = "1.0"

# May need to add (check first)
tempfile = "3"  # For NamedTempFile if preferred over manual temp
uuid = "1"      # For unique temp file names (if using manual approach)
```

### Related Files
```
diagram_tool/src/
  cli.rs                    # MODIFY - wire new functions
  cli_persistence.rs        # CREATE - new module
  models/schema.rs          # REFERENCE - validate_schema
  ui/toolbar/persistence.rs # REFERENCE ONLY - don't modify
```

### Git Context
- Branch: main (or current feature branch)
- No uncommitted changes before starting

---

## 15. AI Hints

### Implementation Order
1. Create empty `cli_persistence.rs` with function signatures
2. Add module declaration in `lib.rs`/`main.rs`
3. Implement `emit_stage_event` first (simplest)
4. Implement `save_workspace_atomic` (self-contained)
5. Implement `load_workspace_with_lkg` (depends on understanding of save)
6. Wire CLI commands
7. Add tests

### Code Style Notes
- Use `context()` for all error handling (already established in cli.rs)
- Use `Path` and `PathBuf` for all file paths
- Keep functions pure where possible (no global state)
- Emit JSONL to stdout only (no logging crate)

### Common Pitfalls
- Forgetting to clean up temp file on error
- Not handling the case where path has no extension for LKG naming
- Emitting multi-line JSON (breaks JSONL parsers)

---

## 16. Execution Summary

| Phase | Task | Duration | Status |
|-------|------|----------|--------|
| 1 | Create module structure | 5min | NOT STARTED |
| 2 | Implement `emit_stage_event` | 10min | NOT STARTED |
| 3 | Implement `save_workspace_atomic` | 30min | NOT STARTED |
| 4 | Implement `load_workspace_with_lkg` | 30min | NOT STARTED |
| 5 | Wire CLI commands | 15min | NOT STARTED |
| 6 | Add unit tests | 30min | NOT STARTED |
| 7 | Verify and clean up | 10min | NOT STARTED |
| **TOTAL** | | **~2hr** | |

---

## Receipt

### Objective
Add atomic persistence functions (`save_workspace_atomic`, `load_workspace_with_lkg`) to CLI with structured JSONL event emission for validation stages, persistence success, and error handling.

### Allowed Scope
- Create: `diagram_tool/src/cli_persistence.rs`
- Modify: `diagram_tool/src/cli.rs` (import and call new functions)
- Modify: `diagram_tool/src/lib.rs` or `main.rs` (add module declaration)
- Add tests to new module

### Files Touched
| File | Action | Lines Changed (Est.) |
|------|--------|---------------------|
| `diagram_tool/src/cli_persistence.rs` | CREATE | ~150 |
| `diagram_tool/src/cli.rs` | MODIFY | ~20 |
| `diagram_tool/src/lib.rs` | MODIFY | ~2 |
| `diagram_tool/Cargo.toml` | MODIFY | ~2 (if deps needed) |

### Commands
```bash
# Verification commands
moon run diagram_tool:check
moon run diagram_tool:clippy
moon run diagram_tool:test -- persistence
moon run diagram_tool:test -- cli
```

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Validation/schema error |
| 2 | Parse/I/O error |

### Key stdout/stderr
```jsonl
{"event":"start","command":"validate","ok":true,"code":"start"}
{"event":"stage","name":"validating","path":"diagram.json"}
{"event":"stage","name":"persisted","path":"diagram.json"}
{"event":"finish","command":"validate","ok":true,"code":"ok"}
{"event":"stage","name":"error","code":"validation_failed","message":"Document version must be 2"}
{"event":"stage","name":"lkg_fallback","reason":"validation_failed","path":"diagram.json.lkg"}
```

### Diff Summary
```diff
+ diagram_tool/src/cli_persistence.rs (new file)
  - save_workspace_atomic()
  - load_workspace_with_lkg()
  - emit_stage_event()
  - load_and_validate()
  - Unit tests

~ diagram_tool/src/cli.rs
  + use crate::cli_persistence::{...};
  ~ load_doc -> load_workspace_with_lkg
  ~ save_doc -> save_workspace_atomic

~ diagram_tool/src/lib.rs
  + mod cli_persistence;
```

### Risks & Unknowns
1. **Temp file cleanup on crash**: If process crashes during write, temp file may remain. Mitigation: Use predictable temp naming for manual cleanup if needed.
2. **Cross-filesystem rename**: Atomic rename only works on same filesystem. Mitigation: Write temp to same directory as target.
3. **LKG file naming**: Edge case when path has no extension. Mitigation: Handle in `with_extension` logic.

### Pass/Fail Recommendation
**PASS** when:
- All unit tests pass
- `moon run diagram_tool:clippy` shows no warnings
- Manual save/load cycle works
- JSONL events are valid single-line JSON
- Corrupt file triggers LKG fallback (if LKG exists)

**FAIL** if:
- Temp files leak on error
- Atomic rename not actually atomic (copies instead)
- JSONL events have embedded newlines
- LKG fallback not attempted on validation failure
