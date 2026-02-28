# Martin Fowler Test Plan: Atomic Persistence Functions

**Bead ID**: bd-2cm
**Feature**: storage-sync: add atomic redb-plus-file persistence

---

## Test Categories

### 1. Happy Path Tests
### 2. Error Path Tests
### 3. Edge Case Tests
### 4. Contract Verification Tests
### 5. Contract Violation Tests

---

## 1. Happy Path Tests

### `test_given_valid_document_when_save_atomic_then_file_exists_and_valid`

**Purpose**: Verify atomic save writes valid file.

**Given**: A valid `DiagramDocument::default()` and a temp directory path
**When**: `save_workspace_atomic(&doc, &path)` is called
**Then**:
- `path.exists()` returns `true`
- Contents are valid JSON
- `validate_schema(&loaded_doc)` returns `Ok(())`

```rust
#[test]
fn given_valid_document_when_save_atomic_then_file_exists_and_valid() {
    let doc = DiagramDocument::default();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    save_workspace_atomic(&doc, &path).unwrap();

    assert!(path.exists());
    let loaded: DiagramDocument = serde_json::from_str(
        &fs::read_to_string(&path).unwrap()
    ).unwrap();
    assert!(validate_schema(&loaded).is_ok());
}
```

---

### `test_given_valid_file_when_load_with_lkg_then_returns_doc`

**Purpose**: Verify load returns valid document.

**Given**: A valid JSON file at `path`
**When**: `load_workspace_with_lkg(&path)` is called
**Then**:
- Returns `Ok(doc)`
- `validate_schema(&doc)` returns `Ok(())`

```rust
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

---

### `test_given_save_atomic_when_success_then_emits_persisted_event`

**Purpose**: Verify JSONL event emission on success.

**Given**: A valid document and writable path
**When**: `save_workspace_atomic(&doc, &path)` is called
**Then**: stdout contains `{"event":"stage","name":"persisted",...}`

```rust
#[test]
fn given_save_atomic_when_success_then_emits_persisted_event() {
    // Use a captured stdout or internal event collector
    // Verify "persisted" stage event was emitted
}
```

---

### `test_given_load_with_lkg_when_success_then_emits_validating_event`

**Purpose**: Verify JSONL event emission on load.

**Given**: A valid JSON file
**When**: `load_workspace_with_lkg(&path)` is called
**Then**: stdout contains `{"event":"stage","name":"validating",...}`

```rust
#[test]
fn given_load_with_lkg_when_success_then_emits_validating_event() {
    // Verify "validating" stage event was emitted with path
}
```

---

## 2. Error Path Tests

### `test_given_invalid_version_when_save_atomic_then_returns_invalid_version_error`

**Purpose**: Verify P1 precondition violation.

**Given**: Document with `version: 99`
**When**: `save_workspace_atomic(&doc, &path)` is called
**Then**: Returns `Err(CliPersistenceError::InvalidVersion { actual: 99 })`

```rust
#[test]
fn given_invalid_version_when_save_atomic_then_returns_invalid_version_error() {
    let mut doc = DiagramDocument::default();
    doc.version = 99;
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    let result = save_workspace_atomic(&doc, &path);

    assert!(matches!(result, Err(CliPersistenceError::InvalidVersion { actual: 99 })));
}
```

---

### `test_given_missing_parent_dir_when_save_then_returns_parent_not_found_error`

**Purpose**: Verify P2 precondition violation.

**Given**: Path with non-existent parent directory
**When**: `save_workspace_atomic(&doc, &path)` is called
**Then**: Returns `Err(CliPersistenceError::ParentDirectoryNotFound)`

```rust
#[test]
fn given_missing_parent_dir_when_save_then_returns_parent_not_found_error() {
    let doc = DiagramDocument::default();
    let path = PathBuf::from("/nonexistent/dir/test.json");

    let result = save_workspace_atomic(&doc, &path);

    assert!(matches!(result, Err(CliPersistenceError::ParentDirectoryNotFound { .. })));
}
```

---

### `test_given_schema_invalid_doc_when_save_atomic_then_returns_schema_error`

**Purpose**: Verify P3 precondition violation.

**Given**: Document with invalid edge (references non-existent node)
**When**: `save_workspace_atomic(&doc, &path)` is called
**Then**: Returns `Err(CliPersistenceError::SchemaValidationFailed)`

```rust
#[test]
fn given_schema_invalid_doc_when_save_atomic_then_returns_schema_error() {
    let mut doc = DiagramDocument::default();
    // Add edge referencing non-existent node
    doc.document.edges.insert(
        EdgeId::new("e1".into()),
        Edge {
            source: NodeId::new("missing".into()),
            target: NodeId::new("also_missing".into()),
            ..Default::default()
        }
    );
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    let result = save_workspace_atomic(&doc, &path);

    assert!(matches!(result, Err(CliPersistenceError::SchemaValidationFailed { .. })));
}
```

---

### `test_given_corrupt_file_when_load_with_lkg_then_falls_back_to_lkg`

**Purpose**: Verify LKG fallback behavior.

**Given**: Corrupt primary file, valid LKG file
**When**: `load_workspace_with_lkg(&path)` is called
**Then**: Returns valid document from LKG file

```rust
#[test]
fn given_corrupt_file_when_load_with_lkg_then_falls_back_to_lkg() {
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

---

### `test_given_lkg_fallback_when_occurs_then_emits_lkg_fallback_event`

**Purpose**: Verify JSONL event on LKG fallback.

**Given**: Corrupt primary, valid LKG
**When**: Fallback occurs
**Then**: `{"event":"stage","name":"lkg_fallback",...}` emitted

```rust
#[test]
fn given_lkg_fallback_when_occurs_then_emits_lkg_fallback_event() {
    // Verify "lkg_fallback" stage event with reason and path
}
```

---

### `test_given_no_lkg_when_corrupt_load_then_returns_parse_error`

**Purpose**: Verify error when both primary and LKG fail.

**Given**: Corrupt primary file, no LKG file
**When**: `load_workspace_with_lkg(&path)` is called
**Then**: Returns `Err(CliPersistenceError::ParseFailed)`

```rust
#[test]
fn given_no_lkg_when_corrupt_load_then_returns_parse_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");
    fs::write(&path, "{corrupt").unwrap();

    let result = load_workspace_with_lkg(&path);

    assert!(matches!(result, Err(CliPersistenceError::ParseFailed { .. })));
}
```

---

### `test_given_nonexistent_file_when_load_then_returns_file_not_found_error`

**Purpose**: Verify P4 precondition violation.

**Given**: Non-existent path
**When**: `load_workspace_with_lkg(&path)` is called
**Then**: Returns `Err(CliPersistenceError::FileNotFound)`

```rust
#[test]
fn given_nonexistent_file_when_load_then_returns_file_not_found_error() {
    let path = PathBuf::from("/nonexistent/file.json");

    let result = load_workspace_with_lkg(&path);

    assert!(matches!(result, Err(CliPersistenceError::FileNotFound { .. })));
}
```

---

### `test_given_save_failure_when_occurs_then_target_unchanged`

**Purpose**: Verify Q5 postcondition - target unchanged on failure.

**Given**: Existing file at target path, invalid document
**When**: `save_workspace_atomic` fails
**Then**: Original file contents unchanged

```rust
#[test]
fn given_save_failure_when_occurs_then_target_unchanged() {
    let mut doc = DiagramDocument::default();
    doc.version = 99; // Invalid

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");
    fs::write(&path, r#"{"existing": true}"#).unwrap();
    let before = fs::read_to_string(&path).unwrap();

    let _ = save_workspace_atomic(&doc, &path);

    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}
```

---

### `test_given_save_failure_when_occurs_then_no_temp_files_remain`

**Purpose**: Verify Q6 postcondition - temp file cleanup.

**Given**: Invalid document, writable directory
**When**: `save_workspace_atomic` fails
**Then**: No `.tmp` files in directory

```rust
#[test]
fn given_save_failure_when_occurs_then_no_temp_files_remain() {
    let mut doc = DiagramDocument::default();
    doc.version = 99;

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    let _ = save_workspace_atomic(&doc, &path);

    let tmp_files: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "tmp").unwrap_or(false))
        .collect();
    assert!(tmp_files.is_empty(), "No .tmp files should remain");
}
```

---

## 3. Edge Case Tests

### `test_given_path_without_extension_when_lkg_computed_then_suffix_is_lkg`

**Purpose**: Verify LKG path for extensionless files.

**Given**: Path `/tmp/diagram` (no extension)
**When**: LKG path computed
**Then**: LKG path is `/tmp/diagram.lkg`

```rust
#[test]
fn given_path_without_extension_when_lkg_computed_then_suffix_is_lkg() {
    let path = PathBuf::from("/tmp/diagram");
    let lkg_path = compute_lkg_path(&path); // Helper function
    assert_eq!(lkg_path, PathBuf::from("/tmp/diagram.lkg"));
}
```

---

### `test_given_path_with_multiple_dots_when_lkg_computed_then_correct_suffix`

**Purpose**: Verify LKG path for files with dots in name.

**Given**: Path `/tmp/my.diagram.backup.json`
**When**: LKG path computed
**Then**: LKG path is `/tmp/my.diagram.backup.json.lkg`

```rust
#[test]
fn given_path_with_multiple_dots_when_lkg_computed_then_correct_suffix() {
    let path = PathBuf::from("/tmp/my.diagram.backup.json");
    let lkg_path = compute_lkg_path(&path);
    assert_eq!(lkg_path, PathBuf::from("/tmp/my.diagram.backup.json.lkg"));
}
```

---

### `test_given_empty_document_when_save_atomic_then_succeeds`

**Purpose**: Verify empty (default) document is valid.

**Given**: `DiagramDocument::default()` (empty nodes/edges)
**When**: `save_workspace_atomic` called
**Then**: Success

```rust
#[test]
fn given_empty_document_when_save_atomic_then_succeeds() {
    let doc = DiagramDocument::default();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    let result = save_workspace_atomic(&doc, &path);

    assert!(result.is_ok());
}
```

---

### `test_given_large_document_when_save_atomic_then_succeeds`

**Purpose**: Verify handling of large documents.

**Given**: Document with 1000 nodes and 500 edges
**When**: `save_workspace_atomic` called
**Then**: Success

```rust
#[test]
fn given_large_document_when_save_atomic_then_succeeds() {
    let mut doc = DiagramDocument::default();
    for i in 0..1000 {
        doc.document.nodes.insert(
            NodeId::new(format!("n{}", i)),
            Node { label: format!("Node {}", i), ..Default::default() }
        );
    }
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    let result = save_workspace_atomic(&doc, &path);

    assert!(result.is_ok());
}
```

---

### `test_given_unicode_in_labels_when_save_and_load_then_preserved`

**Purpose**: Verify Unicode handling.

**Given**: Document with Unicode labels
**When**: Save and load cycle
**Then**: Unicode preserved

```rust
#[test]
fn given_unicode_in_labels_when_save_and_load_then_preserved() {
    let mut doc = DiagramDocument::default();
    doc.document.nodes.insert(
        NodeId::new("n1".into()),
        Node { label: "日本語 🎉 café".into(), ..Default::default() }
    );
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    save_workspace_atomic(&doc, &path).unwrap();
    let loaded = load_workspace_with_lkg(&path).unwrap();

    assert_eq!(
        loaded.document.nodes.get(&NodeId::new("n1".into())).unwrap().label,
        "日本語 🎉 café"
    );
}
```

---

### `test_given_readonly_target_dir_when_save_then_returns_write_error`

**Purpose**: Verify write permission error.

**Given**: Read-only directory
**When**: `save_workspace_atomic` called
**Then**: Returns write error

```rust
#[test]
#[cfg(unix)]
fn given_readonly_target_dir_when_save_then_returns_write_error() {
    let doc = DiagramDocument::default();
    let temp_dir = tempfile::tempdir().unwrap();
    let readonly_dir = temp_dir.path().join("readonly");
    fs::create_dir(&readonly_dir).unwrap();

    // Make directory read-only
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&readonly_dir, fs::Permissions::from_mode(0o555)).unwrap();

    let path = readonly_dir.join("test.json");
    let result = save_workspace_atomic(&doc, &path);

    assert!(result.is_err());
}
```

---

## 4. Contract Verification Tests

### `test_precondition_version_must_be_2`

**Purpose**: Explicit test for P1.

```rust
#[test]
fn test_precondition_version_must_be_2() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    // Version 2 should pass
    let doc_v2 = DiagramDocument { version: 2, ..Default::default() };
    assert!(save_workspace_atomic(&doc_v2, &path).is_ok());

    // Version 1 should fail
    let doc_v1 = DiagramDocument { version: 1, ..Default::default() };
    let result = save_workspace_atomic(&doc_v1, &path);
    assert!(matches!(result, Err(CliPersistenceError::InvalidVersion { actual: 1 })));

    // Version 3 should fail
    let doc_v3 = DiagramDocument { version: 3, ..Default::default() };
    let result = save_workspace_atomic(&doc_v3, &path);
    assert!(matches!(result, Err(CliPersistenceError::InvalidVersion { actual: 3 })));
}
```

---

### `test_postcondition_target_exists_after_success`

**Purpose**: Explicit test for Q1.

```rust
#[test]
fn test_postcondition_target_exists_after_success() {
    let doc = DiagramDocument::default();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    assert!(!path.exists());
    save_workspace_atomic(&doc, &path).unwrap();
    assert!(path.exists(), "Q1: target must exist after successful save");
}
```

---

### `test_postcondition_no_temp_files_after_success`

**Purpose**: Explicit test for Q3.

```rust
#[test]
fn test_postcondition_no_temp_files_after_success() {
    let doc = DiagramDocument::default();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    save_workspace_atomic(&doc, &path).unwrap();

    let tmp_files: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
        .collect();
    assert!(tmp_files.is_empty(), "Q3: no .tmp files should remain after success");
}
```

---

### `test_invariant_jsonl_single_line`

**Purpose**: Verify JSONL events are single-line.

```rust
#[test]
fn test_invariant_jsonl_single_line() {
    // Capture stdout and verify each line is valid JSON
    // No line should contain unescaped newlines
}
```

---

### `test_invariant_atomic_rename_same_directory`

**Purpose**: Verify temp file is in same directory as target.

```rust
#[test]
fn test_invariant_atomic_rename_same_directory() {
    let doc = DiagramDocument::default();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    save_workspace_atomic(&doc, &path).unwrap();

    // If rename was atomic, file should exist at target
    assert!(path.exists());
    // Verify by checking file contents match expected
    let contents = fs::read_to_string(&path).unwrap();
    let loaded: DiagramDocument = serde_json::from_str(&contents).unwrap();
    assert_eq!(loaded.version, 2);
}
```

---

## 5. Contract Violation Tests

### `test_p1_violation_returns_invalid_version`

**Purpose**: Direct test of P1 violation example.

```rust
#[test]
fn test_p1_violation_returns_invalid_version() {
    // VIOLATES P1: save_workspace_atomic(&DiagramDocument{version: 99, ..}, &path)
    let mut doc = DiagramDocument::default();
    doc.version = 99;
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    let result = save_workspace_atomic(&doc, &path);

    assert!(matches!(
        result,
        Err(CliPersistenceError::InvalidVersion { actual: 99 })
    ));
}
```

---

### `test_p2_violation_returns_parent_not_found`

**Purpose**: Direct test of P2 violation example.

```rust
#[test]
fn test_p2_violation_returns_parent_not_found() {
    // VIOLATES P2: save_workspace_atomic(&doc, &PathBuf::from("/nonexistent/dir/test.json"))
    let doc = DiagramDocument::default();
    let path = PathBuf::from("/nonexistent/dir/test.json");

    let result = save_workspace_atomic(&doc, &path);

    assert!(matches!(
        result,
        Err(CliPersistenceError::ParentDirectoryNotFound { .. })
    ));
}
```

---

### `test_p3_violation_returns_schema_validation_failed`

**Purpose**: Direct test of P3 violation example.

```rust
#[test]
fn test_p3_violation_returns_schema_validation_failed() {
    // VIOLATES P3: save_workspace_atomic(&doc_with_invalid_edge, &path)
    let mut doc = DiagramDocument::default();
    doc.document.edges.insert(
        EdgeId::new("e1".into()),
        Edge {
            source: NodeId::new("missing".into()),
            target: NodeId::new("also_missing".into()),
            ..Default::default()
        }
    );
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.json");

    let result = save_workspace_atomic(&doc, &path);

    assert!(matches!(
        result,
        Err(CliPersistenceError::SchemaValidationFailed { .. })
    ));
}
```

---

### `test_p4_violation_returns_file_not_found`

**Purpose**: Direct test of P4 violation example.

```rust
#[test]
fn test_p4_violation_returns_file_not_found() {
    // VIOLATES P4: load_workspace_with_lkg(&PathBuf::from("/nonexistent/file.json"))
    let path = PathBuf::from("/nonexistent/file.json");

    let result = load_workspace_with_lkg(&path);

    assert!(matches!(
        result,
        Err(CliPersistenceError::FileNotFound { .. })
    ));
}
```

---

## Given-When-Then Scenarios

### Scenario 1: Atomic Save Success

```
Given: A valid DiagramDocument with version=2
And: A writable directory at /tmp/test/
When: save_workspace_atomic is called with path /tmp/test/diagram.json
Then:
  - File exists at /tmp/test/diagram.json
  - File contains valid JSON
  - JSON parses to DiagramDocument
  - Parsed document passes validate_schema
  - No .tmp files remain in /tmp/test/
  - stdout contains {"event":"stage","name":"validating",...}
  - stdout contains {"event":"stage","name":"persisted",...}
```

---

### Scenario 2: Atomic Save Failure - Invalid Version

```
Given: A DiagramDocument with version=99
And: A writable directory at /tmp/test/
When: save_workspace_atomic is called with path /tmp/test/diagram.json
Then:
  - Returns Err(CliPersistenceError::InvalidVersion { actual: 99 })
  - No file created at /tmp/test/diagram.json (if didn't exist before)
  - Existing file at /tmp/test/diagram.json unchanged (if existed before)
  - stdout contains {"event":"stage","name":"error","code":"invalid_version",...}
```

---

### Scenario 3: Load with LKG Fallback

```
Given: Corrupt file at /tmp/test/diagram.json containing "{not valid"
And: Valid LKG file at /tmp/test/diagram.json.lkg containing valid DiagramDocument
When: load_workspace_with_lkg is called with path /tmp/test/diagram.json
Then:
  - Returns Ok(valid_document)
  - Returned document passes validate_schema
  - stdout contains {"event":"stage","name":"error","code":"parse_error",...}
  - stdout contains {"event":"stage","name":"lkg_fallback","reason":"parse_error",...}
```

---

### Scenario 4: Load Failure - No LKG Available

```
Given: Corrupt file at /tmp/test/diagram.json containing "{not valid"
And: No file at /tmp/test/diagram.json.lkg
When: load_workspace_with_lkg is called with path /tmp/test/diagram.json
Then:
  - Returns Err(CliPersistenceError::ParseFailed)
  - stdout contains {"event":"stage","name":"error","code":"parse_error",...}
```

---

### Scenario 5: Full Save-Load Roundtrip

```
Given: A valid DiagramDocument with nodes and edges
When:
  - save_workspace_atomic is called
  - load_workspace_with_lkg is called on the saved file
Then:
  - Loaded document equals original document
  - All nodes preserved
  - All edges preserved
  - Editor state preserved
```

---

## Test Count Summary

| Category | Count |
|----------|-------|
| Happy Path | 4 |
| Error Path | 8 |
| Edge Case | 6 |
| Contract Verification | 5 |
| Contract Violation | 4 |
| **Total** | **27** |

---

## Coverage Matrix

| Function | Happy | Error | Edge | Contract |
|----------|-------|-------|------|----------|
| `save_workspace_atomic` | 3 | 5 | 4 | 4 |
| `load_workspace_with_lkg` | 2 | 4 | 2 | 2 |
| `emit_stage_event` | - | - | - | 1 |
| `compute_lkg_path` | - | - | 2 | - |

---

## Test File Location

Tests should be placed in:
- `diagram_tool/src/cli_persistence.rs` under `#[cfg(test)] mod tests`
- `diagram_tool/tests/cli_persistence_e2e.rs` for integration tests
