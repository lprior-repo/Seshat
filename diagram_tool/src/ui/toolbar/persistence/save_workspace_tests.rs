#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::ui::toast::{ToastApi, ToastIntent, ToastOptions};
use diagram_models::document::Revision;
use dioxus::prelude::{rsx, Component, Element, VirtualDom};

fn make_test_doc() -> DiagramDocument {
    DiagramDocument::default()
}

/// Verifies `apply_save_document` writes file and clears dirty flag.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn apply_save_document_clears_dirty_flag_on_success() {
    let mut doc = make_test_doc();
    doc.revision = Revision::new(5);
    let session = DocumentSession::new(doc.clone());
    let temp = tempfile::NamedTempFile::new().expect("temp file should be created");
    let path = temp.path().to_path_buf();

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

/// Test that `apply_save_document` correctly rejects path traversal attempts.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn apply_save_document_rejects_path_traversal_attack() {
    let mut doc = make_test_doc();
    doc.revision = Revision::new(5);
    let session = DocumentSession::new(doc.clone());

    let malicious_path = PathBuf::from("/tmp/../../../etc/passwd");
    let result = apply_save_document(&doc, &session, &malicious_path);

    assert!(
        matches!(result, Err(SaveError::Io(ref msg)) if msg.contains("traversal") || msg.contains("denied")),
        "Path traversal should be rejected with appropriate error, got: {result:?}"
    );
}

/// Test that `update_load_save_success` updates toast to success state.
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn update_load_save_success_updates_toast_to_success() {
    #[component]
    fn TestComponent() -> Element {
        let state = crate::app::AppState::provide();
        let toast_api = ToastApi::from_signal(state.toasts);
        let toast_handle = toast_api.toast(ToastOptions::new(ToastIntent::Info, "Saving..."));

        let path = PathBuf::from("/tmp/test.json");

        update_load_save_success(
            toast_handle,
            "Workspace saved",
            format!("Saved to {}", path.display()),
        );

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

/// Test that `update_load_save_error` updates toast to error state.
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn update_load_save_error_updates_toast_to_error() {
    #[component]
    fn TestComponent() -> Element {
        let state = crate::app::AppState::provide();
        let toast_api = ToastApi::from_signal(state.toasts);
        let toast_handle = toast_api.toast(ToastOptions::new(ToastIntent::Info, "Saving..."));

        update_load_save_error(
            toast_handle,
            "Save failed",
            String::from("Save error: Permission denied"),
        );

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
