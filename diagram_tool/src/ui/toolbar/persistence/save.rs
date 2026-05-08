#![allow(dead_code)]
use crate::ui::toast::{ToastApi, ToastHandle, ToastIntent, ToastOptions, ToastQueue};
use diagram_models::document::{DiagramDocument, DocumentSession};
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use std::path::PathBuf;

use super::common::{update_load_save_error, update_load_save_success};
#[cfg(not(target_arch = "wasm32"))]
use crate::cli_persistence::{save_workspace_atomic, validate_safe_path, CliPersistenceError};

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
    // Validate path before saving - prevents path traversal attacks
    // Use parent directory as base (or cwd if no parent) since we write to parent
    let base_dir = file_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    validate_safe_path(file_path, base_dir).map_err(|e| match e {
        CliPersistenceError::PathTraversalDenied { path } => {
            SaveError::Io(format!("Path traversal denied: {path}"))
        }
        CliPersistenceError::IoError(e) => SaveError::Io(e.to_string()),
        _ => SaveError::Io(String::from("Path validation failed")),
    })?;
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
    doc: &DiagramDocument,
    _session: &DocumentSession,
    _file_path: &PathBuf,
) -> Result<DiagramDocument, SaveError> {
    use diagram_models::canonical_json::to_canonical_pretty_json;
    to_canonical_pretty_json(doc).map_err(|e| SaveError::Serialize(e.to_string()))?;
    Ok(doc.clone())
}

#[cfg(target_arch = "wasm32")]
fn commit_save_wasm(session_signal: &mut Signal<DocumentSession>, new_session: DocumentSession) {
    *session_signal.write() = new_session;
}

#[cfg(target_arch = "wasm32")]
fn save_workspace_wasm(
    doc: DiagramDocument,
    session: DocumentSession,
    toasts: Signal<ToastQueue>,
    toast_handle: ToastHandle,
    mut session_signal: Signal<DocumentSession>,
) {
    spawn(async move {
        use diagram_models::canonical_json::to_canonical_pretty_json;

        let json = match to_canonical_pretty_json(&doc) {
            Ok(j) => j,
            Err(e) => {
                update_load_save_error(toast_handle, "Save failed", format!("Serialize error: {e}"));
                return;
            }
        };

        let session_snapshot = session.clone();
        let suggested_name = session_snapshot
            .file_path()
            .map(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("diagram")
                    .to_string()
            })
            .unwrap_or_else(|| "diagram".to_string());

        let js = format!(
            r#"
            (async function() {{
                const filename = "{name}.json";
                const content = `{json}`;
                let saved = false;

                // Try File System Access API first (Chrome/Edge)
                if (window.showSaveFilePicker) {{
                    try {{
                        const handle = await window.showSaveFilePicker({{
                            suggestedName: filename,
                            types: [{{
                                description: "Seshat Diagram",
                                accept: {{ "application/json": [".json"] }}
                            }}]
                        }});
                        const writable = await handle.createWritable();
                        await writable.write(content);
                        await writable.close();
                        saved = true;
                        dioxus.send({{ ok: true, filename: handle.name }});
                        return;
                    }} catch (e) {{
                        if (e.name !== 'AbortError') {{
                            console.warn('showSaveFilePicker failed:', e);
                        }}
                    }}
                }}

                // Fallback: download via blob + link click
                try {{
                    const blob = new Blob([content], {{ type: "application/json" }});
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement("a");
                    a.href = url;
                    a.download = filename;
                    a.style.display = "none";
                    document.body.appendChild(a);
                    a.click();
                    document.body.removeChild(a);
                    URL.revokeObjectURL(url);
                    saved = true;
                    dioxus.send({{ ok: true, filename: filename }});
                }} catch (e2) {{
                    dioxus.send({{ ok: false, error: String(e2) }});
                }}
            }})()
            "#,
            name = suggested_name,
            json = json.replace("`", "\\`").replace("${", "\\${")
        );

        let mut eval = dioxus::document::eval(&js);

        match eval.recv::<serde_json::Value>().await {
            Ok(msg) => {
                if msg.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                    let saved_filename = msg
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .unwrap_or("diagram.json");
                    let new_session = session.mark_saved();
                    commit_save_wasm(&mut session_signal, new_session);
                    update_load_save_success(
                        toast_handle,
                        "Workspace saved",
                        format!("Saved as {}", saved_filename),
                    );
                } else {
                    let error = msg
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Save was cancelled or failed");
                    update_load_save_error(toast_handle, "Save failed", error.to_string());
                }
            }
            Err(e) => {
                update_load_save_error(
                    toast_handle,
                    "Save failed",
                    format!("JS bridge error: {e}"),
                );
            }
        }
    });
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
        let doc = doc_signal.read().clone();
        let session = session_signal.read().clone();
        save_workspace_wasm(doc, session, toasts, toast_handle, session_signal);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let doc_snapshot = doc_signal.read().clone();
        let session_snapshot = session_signal.read().clone();
        spawn(async move {
            let path_opt = session_snapshot.file_path().map(PathBuf::from);
            let chosen_path = path_opt.or_else(|| {
                FileDialog::new()
                    .add_filter("Seshat Diagram", &["json"])
                    .set_file_name("diagram.json")
                    .save_file()
            });
            match chosen_path {
                None => {
                    let _ = toast_handle.dismiss();
                }
                Some(p) => {
                    let result = apply_save_document(&doc_snapshot, &session_snapshot, &p);
                    handle_save_result(result, &p, &mut session_signal, toast_handle);
                }
            }
        });
    }
}

/// Handles the result of a save operation, updating signals and toasts.
#[cfg(not(target_arch = "wasm32"))]
fn handle_save_result(
    result: Result<DocumentSession, SaveError>,
    path: &PathBuf,
    session_signal: &mut Signal<DocumentSession>,
    toast_handle: ToastHandle,
) {
    match result {
        Ok(saved_session) => {
            *session_signal.write() = saved_session;
            update_load_save_success(
                toast_handle,
                "Workspace saved",
                format!("Saved to {}", path.display()),
            );
        }
        Err(SaveError::Io(e)) => {
            update_load_save_error(toast_handle, "Save failed", format!("Save error: {e}"));
        }
        Err(SaveError::Serialize(e)) => {
            update_load_save_error(toast_handle, "Save failed", format!("Serialize error: {e}"));
        }
        Err(SaveError::NoFilePath) => {
            let _ = toast_handle.dismiss();
        }
    }
}

#[cfg(test)]
mod tests {
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

    // =====================================================================
    // Proptest invariants
    // =====================================================================

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
            // Invariant: last_saved_revision must equal the saved document's revision
            prop_assert_eq!(
                saved_session.last_saved_revision(),
                doc.revision,
                "last_saved_revision should match the document's revision after save"
            );
            // Invariant: dirty flag must be cleared after saving current document
            prop_assert!(
                !saved_session.is_dirty(),
                "session should not be dirty after saving"
            );
        }
    }
}

// =============================================================================
// Integration tests for save_workspace expected behavior
// =============================================================================
//
// save_workspace() is an async UI action that:
//   1. Shows a "Saving workspace" toast with "Preparing data..." detail
//   2. On native: uses FileDialog to pick a save location (or uses existing path)
//   3. On WASM: shows "Save not available" error toast, dismisses loading toast
//   4. On success: updates session signal, dismisses saving toast, shows success toast
//   5. On error: shows error toast with details from SaveError variant
//
// These tests document the expected behavior and test the pure functions
// that save_workspace calls internally (apply_save_document, handle_save_result).
//
// Note: Testing save_workspace directly requires async infrastructure and
// FileDialog mocking. The tests below verify the core save logic.

#[cfg(test)]
mod save_workspace_integration_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::ui::toast::{ToastApi, ToastIntent, ToastOptions};
    use diagram_models::document::Revision;
    use dioxus::prelude::{rsx, Component, Element, VirtualDom};

    fn make_test_doc() -> DiagramDocument {
        DiagramDocument::default()
    }

    /// Verifies apply_save_document writes file and clears dirty flag.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn apply_save_document_clears_dirty_flag_on_success() {
        let mut doc = make_test_doc();
        doc.revision = Revision::new(5);
        let session = DocumentSession::new(doc.clone());
        let temp = tempfile::NamedTempFile::new().expect("temp file should be created");
        let path = temp.path().to_path_buf();

        // Perform save
        let result = apply_save_document(&doc, &session, &path);
        assert!(result.is_ok(), "save should succeed");

        let saved_session = result.unwrap();
        assert!(
            !saved_session.is_dirty(),
            "session should not be dirty after save"
        );
        assert_eq!(
            saved_session.last_saved_revision(),
            doc.revision,
            "last saved revision should match document revision"
        );
    }

    /// Test that apply_save_document correctly rejects path traversal attempts.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn apply_save_document_rejects_path_traversal_attack() {
        let mut doc = make_test_doc();
        doc.revision = Revision::new(5);
        let session = DocumentSession::new(doc.clone());

        // Attempt path traversal - should be rejected by validate_safe_path
        let malicious_path = PathBuf::from("/tmp/../../../etc/passwd");
        let result = apply_save_document(&doc, &session, &malicious_path);

        assert!(
            matches!(result, Err(SaveError::Io(ref msg)) if msg.contains("traversal") || msg.contains("denied")),
            "Path traversal should be rejected with appropriate error, got: {result:?}"
        );
    }

    /// Test that update_load_save_success updates toast to success state.
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn update_load_save_success_updates_toast_to_success() {
        #[component]
        fn TestComponent() -> Element {
            let state = crate::app::AppState::provide();
            let toast_api = ToastApi::from_signal(state.toasts);
            let toast_handle = toast_api.toast(ToastOptions::new(ToastIntent::Info, "Saving..."));

            let path = PathBuf::from("/tmp/test.json");

            // This would be called by handle_save_result on success
            update_load_save_success(
                toast_handle,
                "Workspace saved",
                format!("Saved to {}", path.display()),
            );

            // Verify toast was updated
            let queue = state.toasts.read();
            let toast = queue.items().iter().find(|t| t.title == "Workspace saved");
            assert!(
                toast.is_some(),
                "Toast with title 'Workspace saved' should exist"
            );
            assert_eq!(
                toast.as_ref().unwrap().intent,
                ToastIntent::Success,
                "Toast intent should be Success"
            );

            rsx! { "test" }
        }

        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place();
    }

    /// Test that update_load_save_error updates toast to error state.
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn update_load_save_error_updates_toast_to_error() {
        #[component]
        fn TestComponent() -> Element {
            let state = crate::app::AppState::provide();
            let toast_api = ToastApi::from_signal(state.toasts);
            let toast_handle = toast_api.toast(ToastOptions::new(ToastIntent::Info, "Saving..."));

            // This would be called by handle_save_result on IO error
            update_load_save_error(
                toast_handle,
                "Save failed",
                String::from("Save error: Permission denied"),
            );

            // Verify toast was updated
            let queue = state.toasts.read();
            let toast = queue.items().iter().find(|t| t.title == "Save failed");
            assert!(
                toast.is_some(),
                "Toast with title 'Save failed' should exist"
            );
            assert_eq!(
                toast.as_ref().unwrap().intent,
                ToastIntent::Error,
                "Toast intent should be Error"
            );

            rsx! { "test" }
        }

        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place();
    }
}
