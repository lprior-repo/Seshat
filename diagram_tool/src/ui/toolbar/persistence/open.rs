#![allow(dead_code)]
use crate::history::History;
use crate::ui::editor::ToolMode;
use crate::ui::toast::{ToastApi, ToastIntent, ToastOptions, ToastQueue};
use diagram_models::document::{ArrowType, DiagramDocument, DocumentSession, EdgeStyle};
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

use super::common::{
    apply_import_contents, update_load_save_error, update_load_save_success, ImportTransitionError,
};

#[derive(Debug)]
pub enum OpenError {
    Parse(String),
    Validation(String),
    Io(String),
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(s) => write!(f, "Parse error: {s}"),
            Self::Validation(s) => write!(f, "Validation error: {s}"),
            Self::Io(s) => write!(f, "IO error: {s}"),
        }
    }
}

pub fn apply_open_document(
    current_doc: &DiagramDocument,
    current_history: &History,
    contents: &str,
    file_path: std::path::PathBuf,
) -> Result<(DiagramDocument, History, DocumentSession), OpenError> {
    let mut next_doc = current_doc.clone();
    let mut next_history = current_history.clone();

    apply_import_contents(&mut next_doc, &mut next_history, contents).map_err(|e| match e {
        ImportTransitionError::Parse(s) => OpenError::Parse(s),
        ImportTransitionError::Validation(s) => OpenError::Validation(s),
    })?;

    let session = DocumentSession::from_file(next_doc.clone(), file_path);

    Ok((next_doc, next_history, session))
}

#[allow(clippy::too_many_lines)]
pub fn open_workspace(
    mut doc_signal: Signal<DiagramDocument>,
    mut session_signal: Signal<DocumentSession>,
    mut history_signal: Signal<History>,
    tool_signal: Signal<ToolMode>,
    edge_style_signal: Signal<EdgeStyle>,
    arrow_type_signal: Signal<ArrowType>,
    toasts: Signal<ToastQueue>,
    store_bridge: Option<std::sync::Arc<crate::store_bridge::StoreBridge>>,
) {
    let toast_api = ToastApi::from_signal(toasts);
    let toast_handle = toast_api.toast(
        ToastOptions::new(ToastIntent::Info, "Loading workspace")
            .with_detail("Reading persisted document..."),
    );
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (
            &tool_signal,
            &edge_style_signal,
            &arrow_type_signal,
            &store_bridge,
        );
        spawn(async move {
            let mut eval = document::eval(
                r#"
                (function() {
                    if (window.__SESHAT_E2E_IMPORT_JSON) {
                        const contents = window.__SESHAT_E2E_IMPORT_JSON;
                        delete window.__SESHAT_E2E_IMPORT_JSON;
                        dioxus.send({ ok: true, contents, filename: "imported.json" });
                        return;
                    }
                    const input = document.createElement('input');
                    input.type = 'file';
                    input.accept = '.json,application/json';
                    input.style.display = 'none';
                    let settled = false;
                    const finish = (payload) => {
                        if (settled) return;
                        settled = true;
                        window.removeEventListener('focus', onFocus, true);
                        dioxus.send(payload);
                    };
                    const onFocus = () => {
                        setTimeout(() => {
                            finish({ ok: false, cancelled: true });
                        }, 150);
                    };
                    window.addEventListener('focus', onFocus, true);
                    input.addEventListener('change', () => {
                        const file = input.files && input.files[0];
                        if (!file) {
                            finish({ ok: false, cancelled: true });
                            return;
                        }

                        const reader = new FileReader();
                        reader.onload = () => {
                            finish({ ok: true, contents: String(reader.result || ''), filename: file.name });
                        };
                        reader.onerror = () => {
                            finish({ ok: false, cancelled: false, error: 'read-failed' });
                        };
                        reader.readAsText(file);
                    });
                    input.click();
                })();
                "#,
            );

            match eval.recv::<serde_json::Value>().await {
                Ok(msg) => {
                    if msg["cancelled"].as_bool().is_some_and(|v| v) {
                        let _ = toast_handle.dismiss();
                        return;
                    }

                    if msg["ok"].as_bool() != Some(true) {
                        let detail = msg["error"].as_str().map_or_else(
                            || String::from("Browser file import failed"),
                            String::from,
                        );
                        update_load_save_error(toast_handle, "Load failed", detail);
                        return;
                    }

                    let contents = msg["contents"].as_str().map_or("", |v| v);
                    let filename = msg["filename"].as_str().map_or("imported.json", |v| v);
                    let file_path = std::path::PathBuf::from(filename);

                    let current_doc = doc_signal.read().clone();
                    let current_history = history_signal.read().clone();

                    match apply_open_document(&current_doc, &current_history, contents, file_path) {
                        Ok((next_doc, next_history, session)) => {
                            *doc_signal.write() = next_doc;
                            *session_signal.write() = session;
                            *history_signal.write() = next_history;
                            update_load_save_success(
                                toast_handle,
                                "Workspace loaded",
                                String::from("Loaded from local JSON"),
                            );
                        }
                        Err(OpenError::Parse(err)) => {
                            update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Parse error: {err}"),
                            );
                        }
                        Err(OpenError::Validation(code)) => {
                            update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Load validation error: {code}"),
                            );
                        }
                        Err(OpenError::Io(err)) => {
                            update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("IO error: {err}"),
                            );
                        }
                    }
                }
                Err(err) => update_load_save_error(
                    toast_handle,
                    "Load failed",
                    format!("Import bridge error: {err}"),
                ),
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (&tool_signal, &edge_style_signal, &arrow_type_signal);
        spawn(async move {
            let path = FileDialog::new()
                .add_filter("Seshat Diagram", &["json"])
                .pick_file();
            match path {
                None => {
                    let _ = toast_handle.dismiss();
                }
                Some(p) => match fs::read_to_string(&p) {
                    Err(e) => update_load_save_error(
                        toast_handle,
                        "Load failed",
                        format!("Read error: {e}"),
                    ),
                    Ok(contents) => {
                        let current_doc = doc_signal.read().clone();
                        let current_history = history_signal.read().clone();

                        match apply_open_document(
                            &current_doc,
                            &current_history,
                            &contents,
                            p.clone(),
                        ) {
                            Ok((next_doc, next_history, session)) => {
                                if let Some(bridge) = &store_bridge {
                                    if let Err(e) = bridge.reset_store_sync() {
                                        update_load_save_error(
                                            toast_handle,
                                            "Load failed",
                                            format!("Failed to reset store: {e}"),
                                        );
                                        return;
                                    }
                                }
                                *doc_signal.write() = next_doc;
                                *session_signal.write() = session;
                                *history_signal.write() = next_history;
                                update_load_save_success(
                                    toast_handle,
                                    "Workspace loaded",
                                    format!("Loaded from {}", p.display()),
                                );
                            }
                            Err(OpenError::Parse(e)) => update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Parse error: {e}"),
                            ),
                            Err(OpenError::Validation(code)) => update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Load validation error: {code}"),
                            ),
                            Err(OpenError::Io(e)) => update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("IO error: {e}"),
                            ),
                        }
                    }
                },
            }
        });
    }
}

#[cfg(test)]
mod tests {
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

        let result = apply_open_document(&doc, &history, &contents, file_path.clone());

        assert!(result.is_ok());
        let (_, _, session) = result.unwrap();
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

        let result = apply_open_document(&doc, &history, &contents, file_path);

        assert!(result.is_ok());
        let (next_doc, _, _) = result.unwrap();
        assert_eq!(next_doc.revision, Revision::INITIAL);
    }
}
