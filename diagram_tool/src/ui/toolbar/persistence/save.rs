#![allow(dead_code)]
use crate::ui::toast::{ToastApi, ToastIntent, ToastOptions, ToastQueue};
use diagram_models::document::{DiagramDocument, DocumentSession};
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
use super::common::{update_load_save_error, update_load_save_success};
#[cfg(not(target_arch = "wasm32"))]
use crate::cli_persistence::{save_workspace_atomic, CliPersistenceError};

#[derive(Debug)]
pub enum SaveError {
    NoFilePath,
    Serialize(String),
    Io(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFilePath => write!(f, "No file path set - use Save As"),
            Self::Serialize(s) => write!(f, "Serialize error: {s}"),
            Self::Io(s) => write!(f, "IO error: {s}"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn apply_save_document(
    doc: &DiagramDocument,
    session: &DocumentSession,
    file_path: &PathBuf,
) -> Result<DocumentSession, SaveError> {
    save_workspace_atomic(doc, file_path).map_err(|e| match e {
        CliPersistenceError::IoError(e) => SaveError::Io(e.to_string()),
        CliPersistenceError::ValidationError(e) => SaveError::Serialize(e),
        CliPersistenceError::TempFileError(e) => SaveError::Io(e),
        CliPersistenceError::AtomicRenameError { from: _, to: _ } => {
            SaveError::Io(String::from("Atomic rename failed"))
        }
        CliPersistenceError::NoValidDocument(e) => SaveError::Io(e),
        CliPersistenceError::PathTraversalDenied { path } => {
            SaveError::Io(format!("Path traversal denied: {path}"))
        }
        CliPersistenceError::ParseError(e) => SaveError::Serialize(e.to_string()),
    })?;
    Ok(session.with_document(doc.clone()).mark_saved())
}

#[cfg(target_arch = "wasm32")]
pub fn apply_save_document(
    _doc: &DiagramDocument,
    _session: &DocumentSession,
    _file_path: &PathBuf,
) -> Result<DocumentSession, SaveError> {
    Err(SaveError::Io(String::from("Save not available in WASM")))
}

pub fn save_workspace(
    doc_signal: Signal<DiagramDocument>,
    mut session_signal: Signal<DocumentSession>,
    toasts: Signal<ToastQueue>,
) {
    let toast_api = ToastApi::from_signal(toasts);
    let toast_handle = toast_api.toast(
        ToastOptions::new(ToastIntent::Info, "Saving workspace").with_detail("Preparing data..."),
    );
    #[cfg(target_arch = "wasm32")]
    {
        let _ = doc_signal;
        let _ = session_signal;
        let _ = toast_handle.dismiss();
        let toast_api = ToastApi::from_signal(toasts);
        let _ = toast_api.toast(
            ToastOptions::new(ToastIntent::Error, "Save not available")
                .with_detail("Backend has been decommissioned"),
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let doc_snapshot = doc_signal.read().clone();
        let session_snapshot = session_signal.read().clone();
        spawn(async move {
            match session_snapshot.file_path() {
                None => {
                    let _ = toast_handle.dismiss();
                    let path = FileDialog::new()
                        .add_filter("Seshat Diagram", &["json"])
                        .set_file_name("diagram.json")
                        .save_file();
                    match path {
                        None => {}
                        Some(p) => {
                            match apply_save_document(&doc_snapshot, &session_snapshot, &p) {
                                Ok(saved_session) => {
                                    *session_signal.write() = saved_session;
                                    update_load_save_success(
                                        toast_handle,
                                        "Workspace saved",
                                        format!("Saved to {}", p.display()),
                                    );
                                }
                                Err(SaveError::Io(e)) => update_load_save_error(
                                    toast_handle,
                                    "Save failed",
                                    format!("Save error: {e}"),
                                ),
                                Err(SaveError::Serialize(e)) => update_load_save_error(
                                    toast_handle,
                                    "Save failed",
                                    format!("Serialize error: {e}"),
                                ),
                                Err(SaveError::NoFilePath) => {
                                    let _ = toast_handle.dismiss();
                                }
                            }
                        }
                    }
                }
                Some(path) => match apply_save_document(&doc_snapshot, &session_snapshot, path) {
                    Ok(saved_session) => {
                        *session_signal.write() = saved_session;
                        update_load_save_success(
                            toast_handle,
                            "Workspace saved",
                            format!("Saved to {}", path.display()),
                        );
                    }
                    Err(SaveError::Io(e)) => update_load_save_error(
                        toast_handle,
                        "Save failed",
                        format!("Save error: {e}"),
                    ),
                    Err(SaveError::Serialize(e)) => update_load_save_error(
                        toast_handle,
                        "Save failed",
                        format!("Serialize error: {e}"),
                    ),
                    Err(SaveError::NoFilePath) => {
                        let _ = toast_handle.dismiss();
                    }
                },
            }
        });
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use diagram_models::document::Revision;

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
}
