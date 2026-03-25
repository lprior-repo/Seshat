#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use diagram_models::document::{DiagramDocument, DocumentData, EditorState, Revision};
use im::HashMap;
use std::path::Path;
use tempfile::TempDir;

fn create_test_document() -> DiagramDocument {
    DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        },
        editor_state: EditorState::default(),
    }
}

#[test]
fn given_valid_document_when_saved_atomically_then_file_exists() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.json");
    let doc = create_test_document();

    let result = save_workspace_atomic(&doc, &path);

    assert!(result.is_ok());
    assert!(path.exists());
}

#[test]
fn given_saved_document_when_loaded_with_lkg_then_returns_same_document() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.json");
    let doc = create_test_document();

    save_workspace_atomic(&doc, &path).unwrap();
    let loaded = load_workspace_with_lkg(&path);

    assert!(loaded.is_ok());
    let loaded_doc = loaded.unwrap();
    assert_eq!(loaded_doc.version, doc.version);
    assert_eq!(loaded_doc.revision, doc.revision);
}

#[test]
fn given_missing_file_when_loaded_with_lkg_then_fails() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("nonexistent.json");

    let result = load_workspace_with_lkg(&path);

    assert!(result.is_err());
    assert!(matches!(
        result.err(),
        Some(CliPersistenceError::NoValidDocument(_))
    ));
}

#[test]
fn given_invalid_json_when_loaded_with_lkg_then_fails() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("invalid.json");

    std::fs::write(&path, b"not valid json").unwrap();

    let result = load_workspace_with_lkg(&path);

    assert!(result.is_err());
}

#[test]
fn given_lkg_fallback_file_when_primary_fails_then_uses_lkg() {
    let temp_dir = TempDir::new().unwrap();
    let primary_path = temp_dir.path().join("doc.json");
    let lkg_dir = temp_dir.path().join(".lkg");
    let lkg_path = lkg_dir.join("doc.json.lkg");

    // Write invalid primary
    std::fs::write(&primary_path, b"invalid").unwrap();

    // Create LKG directory and write valid LKG file
    std::fs::create_dir_all(&lkg_dir).unwrap();
    let doc = create_test_document();
    let json = serde_json::to_string_pretty(&doc).unwrap();
    std::fs::write(&lkg_path, &json).unwrap();

    let result = load_workspace_with_lkg(&primary_path);

    assert!(result.is_ok());
}

#[test]
fn given_stage_details_when_serialized_then_contains_expected_fields() {
    let details = StageDetails::new()
        .with_path(Path::new("/test/path.json"))
        .with_code("test_code")
        .with_message("test message");

    let json = serde_json::to_string(&details).unwrap();

    assert!(json.contains("test_code"));
    assert!(json.contains("test message"));
    assert!(json.contains("/test/path.json"));
}

#[test]
fn given_atomic_save_when_complete_then_no_temp_files_remain() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.json");
    let doc = create_test_document();

    save_workspace_atomic(&doc, &path).unwrap();

    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .expect("Failed to read temp directory")
        .filter_map(|r| r.ok())
        .collect();

    let has_temp_files = entries
        .iter()
        .any(|e| e.file_name().to_string_lossy().contains(".tmp."));

    assert!(
        !has_temp_files,
        "Temp files should be cleaned up after atomic save"
    );
}

// === Path Traversal Prevention Tests ===

#[test]
fn given_simple_filename_when_validated_then_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    let path = Path::new("diagram.json");

    let result = validate_safe_path(path, base_dir);

    assert!(
        result.is_ok(),
        "Simple filename should be allowed: {:?}",
        result
    );
}

#[test]
fn given_path_traversal_when_validated_then_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    let path = Path::new("../../etc/passwd");

    let result = validate_safe_path(path, base_dir);

    assert!(result.is_err());
    assert!(matches!(
        result.err(),
        Some(CliPersistenceError::PathTraversalDenied { .. })
    ));
}

#[test]
fn given_absolute_path_outside_cwd_when_validated_then_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    let path = Path::new("/etc/shadow");

    let result = validate_safe_path(path, base_dir);

    assert!(result.is_err());
    assert!(matches!(
        result.err(),
        Some(CliPersistenceError::PathTraversalDenied { .. })
    ));
}

#[test]
fn given_sibling_escape_when_validated_then_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    let path = Path::new("diagram/../sibling.json");

    let result = validate_safe_path(path, base_dir);

    assert!(result.is_err());
    assert!(matches!(
        result.err(),
        Some(CliPersistenceError::PathTraversalDenied { .. })
    ));
}

#[test]
fn given_valid_subpath_when_validated_then_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    let path = Path::new("subdir/diagram.json");

    let result = validate_safe_path(path, base_dir);

    assert!(
        result.is_ok(),
        "Valid subdirectory path should be allowed: {:?}",
        result
    );
}

#[test]
fn given_relative_path_with_dot_prefix_when_validated_then_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    let path = Path::new("./diagram.json");

    let result = validate_safe_path(path, base_dir);

    assert!(result.is_ok(), "Path with ./ prefix should be allowed");
}
