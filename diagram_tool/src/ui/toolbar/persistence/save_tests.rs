#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use diagram_models::document::Revision;
use proptest::prelude::*;

fn make_test_doc() -> DiagramDocument {
    DiagramDocument::default()
}

fn make_dirty_session_with_doc() -> (DocumentSession, DiagramDocument) {
    let mut doc = make_test_doc();
    doc.revision = Revision::new(5);
    let session = DocumentSession::new(doc.clone());
    doc.revision = Revision::new(10);
    (session.with_document(doc.clone()), doc)
}

#[cfg(not(target_arch = "wasm32"))]
fn make_temp_file() -> tempfile::NamedTempFile {
    tempfile::NamedTempFile::new().expect("temp file should be created")
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn apply_save_document_writes_file_and_clears_dirty_flag() {
    let (session, doc) = make_dirty_session_with_doc();
    assert!(session.is_dirty(), "session should start dirty");
    let temp = make_temp_file();
    let path = temp.path().to_path_buf();

    let result = apply_save_document(&doc, &session, &path);

    assert!(result.is_ok(), "save should succeed");
    let saved_session = result.unwrap();
    assert!(!saved_session.is_dirty(), "dirty flag should be cleared");
    assert!(path.exists(), "file should exist");
    let contents = std::fs::read_to_string(&path).expect("file should be readable");
    assert!(contents.contains("\"version\""), "file should contain JSON");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn apply_save_document_returns_io_error_for_invalid_path() {
    let (session, doc) = make_dirty_session_with_doc();
    let invalid_path = PathBuf::from("/nonexistent/directory/that/does/not/exist/file.json");

    let result = apply_save_document(&doc, &session, &invalid_path);

    assert!(matches!(result, Err(SaveError::Io(_))));
}

#[test]
fn save_error_no_file_path_display() {
    let err = SaveError::NoFilePath;
    assert_eq!(format!("{err}"), "No file path set - use Save As");
}

#[test]
fn save_error_serialize_display() {
    let err = SaveError::Serialize(String::from("test error"));
    assert_eq!(format!("{err}"), "Serialize error: test error");
}

#[test]
fn save_error_io_display() {
    let err = SaveError::Io(String::from("permission denied"));
    assert_eq!(format!("{err}"), "IO error: permission denied");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn apply_save_document_syncs_revision_from_saved_document() {
    let mut doc = make_test_doc();
    doc.revision = Revision::new(5);
    let session = DocumentSession::new(doc.clone());
    doc.revision = Revision::new(10);

    let temp = make_temp_file();
    let path = temp.path().to_path_buf();
    let result = apply_save_document(&doc, &session, &path);

    assert!(result.is_ok());
    let saved_session = result.unwrap();
    assert_eq!(
        saved_session.last_saved_revision(),
        Revision::new(10),
        "last_saved_revision should match the document that was saved"
    );
    assert!(
        !saved_session.is_dirty(),
        "session should not be dirty after saving the current document"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn apply_save_document_multiple_cycles_syncs_revisions() {
    let mut doc = make_test_doc();
    doc.revision = Revision::new(1);
    let session = DocumentSession::new(doc.clone());
    let temp = make_temp_file();
    let path = temp.path().to_path_buf();

    let result1 = apply_save_document(&doc, &session, &path);
    assert!(result1.is_ok());
    let session1 = result1.unwrap();
    assert_eq!(session1.last_saved_revision(), Revision::new(1));
    assert!(!session1.is_dirty());

    doc.revision = Revision::new(2);
    let result2 = apply_save_document(&doc, &session1, &path);
    assert!(result2.is_ok());
    let session2 = result2.unwrap();
    assert_eq!(session2.last_saved_revision(), Revision::new(2));
    assert!(!session2.is_dirty());

    doc.revision = Revision::new(100);
    let result3 = apply_save_document(&doc, &session2, &path);
    assert!(result3.is_ok());
    let session3 = result3.unwrap();
    assert_eq!(session3.last_saved_revision(), Revision::new(100));
    assert!(!session3.is_dirty());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn apply_save_document_preserves_file_path() {
    let mut doc = make_test_doc();
    doc.revision = Revision::new(5);
    let session = DocumentSession::from_file(doc.clone(), PathBuf::from("/original.json"));
    doc.revision = Revision::new(10);

    let temp = make_temp_file();
    let save_path = temp.path().to_path_buf();
    let result = apply_save_document(&doc, &session, &save_path);

    assert!(result.is_ok());
    let saved_session = result.unwrap();
    assert_eq!(
        saved_session.file_path(),
        Some(&PathBuf::from("/original.json")),
        "session should preserve original file path, not the save path"
    );
}

proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn apply_save_document_revision_sync_invariant(doc_rev in 0u64..1000u64) {
        let mut doc = make_test_doc();
        doc.revision = Revision::new(doc_rev);
        let session = DocumentSession::new(doc.clone());
        let temp = tempfile::NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();

        let result = apply_save_document(&doc, &session, &path);

        prop_assert!(result.is_ok(), "apply_save_document should succeed for valid doc");
        let saved_session = result.unwrap();
        prop_assert_eq!(
            saved_session.last_saved_revision(),
            doc.revision,
            "last_saved_revision should match the document's revision after save"
        );
        prop_assert!(
            !saved_session.is_dirty(),
            "session should not be dirty after saving"
        );
    }
}
