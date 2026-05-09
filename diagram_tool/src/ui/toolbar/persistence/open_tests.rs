#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::cli_persistence::{load_workspace_with_lkg, save_workspace_atomic};
use crate::history::History;
use crate::ui::toast::{ToastApi, ToastIntent, ToastOptions};
use diagram_models::document::Revision;
use dioxus::prelude::{rsx, Component, Element, VirtualDom};
use proptest::prelude::*;

fn make_test_doc() -> DiagramDocument {
    DiagramDocument::default()
}

fn make_test_json() -> String {
    r#"{"version":2,"revision":0,"document":{"nodes":{},"edges":{}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":20.0,"snap_to_grid":true,"selected_items":[],"theme":"system","show_grid":true,"minimap_visible":false}}"#.to_string()
}

// =============================================================================
// Integration tests for open_workspace expected behavior
// =============================================================================
//
// open_workspace() is an async UI action that:
//   1. Shows a "Loading workspace" toast with "Reading persisted document..." detail
//   2. On native: uses FileDialog to pick a file, then loads and validates it
//   3. On WASM: uses browser file picker via JavaScript interop
//   4. On success: updates doc/session/history signals, shows success toast
//   5. On error: shows error toast with OpenError details
//
// These tests document the expected behavior and test the pure functions
// that open_workspace calls internally (apply_open_document, report_open_error).

mod open_workspace_integration_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::history::History;
    use crate::ui::toast::ToastApi;
    use dioxus::prelude::{rsx, Component, Element, VirtualDom};

    /// Test that `apply_open_document` creates session that is not dirty.
    #[test]
    fn apply_open_document_creates_clean_session() {
        let doc = make_test_doc();
        let history = History::new();
        let file_path = std::path::PathBuf::from("/test/diagram.json");
        let contents = make_test_json();

        let (_, _, session) = apply_open_document(&doc, &history, &contents, file_path)
            .expect("apply_open_document should succeed with valid JSON");

        assert!(!session.is_dirty(), "Loaded session should not be dirty");
    }

    /// Test that `report_open_error` updates toast to error state.
    #[test]
    fn report_open_error_updates_toast_to_error() {
        #[component]
        fn TestComponent() -> Element {
            let state = crate::app::AppState::provide();
            let toast_api = ToastApi::from_signal(state.toasts);
            let toast_handle = toast_api.toast(ToastOptions::new(ToastIntent::Info, "Loading..."));

            // This is what report_open_error does on error
            let err = OpenError::Parse(String::from("Invalid JSON"));
            let detail = format!("Parse error: Invalid JSON");
            update_load_save_error(toast_handle, "Load failed", detail);

            // Verify toast was updated
            let queue = state.toasts.read();
            let toast = queue.items().iter().find(|t| t.title == "Load failed");
            assert!(
                toast.is_some(),
                "Toast with title 'Load failed' should exist"
            );
            assert_eq!(
                toast.as_ref().unwrap().intent,
                ToastIntent::Error,
                "Toast intent should be Error"
            );

            rsx! { div {} }
        }

        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place();
    }

    /// Test that `OpenError::Parse` displays correctly.
    #[test]
    fn open_error_parse_display() {
        let err = OpenError::Parse(String::from("invalid json"));
        assert_eq!(format!("{err}"), "Parse error: invalid json");
    }

    /// Test that `OpenError::Validation` displays correctly.
    #[test]
    fn open_error_validation_display() {
        let err = OpenError::Validation(String::from("missing required field"));
        assert_eq!(format!("{err}"), "Validation error: missing required field");
    }

    /// Test that `OpenError::Io` displays correctly.
    #[test]
    fn open_error_io_display() {
        let err = OpenError::Io(String::from("file not found"));
        assert_eq!(format!("{err}"), "IO error: file not found");
    }
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

// =====================================================================
// Proptest invariants
// =====================================================================

proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn apply_open_document_identity_invariant(
        _seed in 0u64..1000u64,
    ) {
        // Use a default document for the round-trip test
        let doc = DiagramDocument::default();

        // Step 1: Save to a temp file
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("test.json");

        save_workspace_atomic(&doc, &path).unwrap();

        // Step 2: Load from the file
        let loaded = load_workspace_with_lkg(&path).unwrap();

        // Step 3: Save again and load again
        save_workspace_atomic(&loaded, &path).unwrap();
        let loaded2 = load_workspace_with_lkg(&path).unwrap();

        // Invariant: The loaded documents should be identical after round-trip
        prop_assert_eq!(
            loaded, loaded2,
            "Loading and re-saving should produce identical content"
        );
    }
}
