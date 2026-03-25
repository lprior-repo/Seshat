#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use diagram_models::document::Revision;

fn make_test_doc() -> DiagramDocument {
    DiagramDocument::default()
}

fn make_test_json() -> String {
    r#"{"version":2,"revision":0,"document":{"nodes":{},"edges":{}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":20.0,"snap_to_grid":true,"selected_items":[],"theme":"system","show_grid":true,"minimap_visible":false}}"#.to_string()
}

#[test]
fn apply_open_document_creates_session_with_file_path() {
    let doc = make_test_doc();
    let history = History::new();
    let file_path = std::path::PathBuf::from("/test/diagram.json");
    let contents = make_test_json();

    let (_, _, session) = apply_open_document(&doc, &history, &contents, file_path.clone())
        .expect("apply_open_document should succeed with valid JSON");

    assert_eq!(session.file_path(), Some(&file_path));
    assert!(!session.is_dirty());
}

#[test]
fn apply_open_document_returns_parse_error_for_invalid_json() {
    let doc = make_test_doc();
    let history = History::new();
    let file_path = std::path::PathBuf::from("/test/invalid.json");
    let contents = "not valid json".to_string();

    let result = apply_open_document(&doc, &history, &contents, file_path);

    assert!(matches!(result, Err(OpenError::Parse(_))));
}

#[test]
fn apply_open_document_returns_error_for_missing_version() {
    let doc = make_test_doc();
    let history = History::new();
    let file_path = std::path::PathBuf::from("/test/no-version.json");
    let contents = r#"{"document":{"nodes":{},"edges":{}}}"#.to_string();

    let result = apply_open_document(&doc, &history, &contents, file_path);

    assert!(result.is_err());
}

#[test]
fn apply_open_document_resets_revision_to_initial() {
    let doc = make_test_doc();
    let history = History::new();
    let file_path = std::path::PathBuf::from("/test/diagram.json");
    let contents = make_test_json();

    let (next_doc, _, _) = apply_open_document(&doc, &history, &contents, file_path)
        .expect("apply_open_document should succeed with valid JSON");

    assert_eq!(next_doc.revision, Revision::INITIAL);
}
