#![allow(dead_code)]
use crate::history::History;
use crate::mutation::pipeline::{run_mutation_with_policy, RevisionPolicy, ValidationPolicy};
use crate::ui::toast::{ToastApi, ToastIntent, ToastOptions, ToastQueue};
use diagram_models::document::{DiagramDocument, DocumentSession, Revision};
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

#[cfg(not(target_arch = "wasm32"))]
use crate::cli_persistence::{load_workspace_with_lkg, CliPersistenceError};

use super::common::{
    apply_import_contents, update_load_save_error, update_load_save_success, ImportTransitionError,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

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

/// Bundles the three mutable document-state signals needed when opening a
/// workspace. Passed as a single parameter to keep `open_workspace` within
/// the 7-argument clippy limit.
#[derive(Clone, Copy)]
pub struct WorkspaceSignals {
    pub doc: Signal<DiagramDocument>,
    pub session: Signal<DocumentSession>,
    pub history: Signal<History>,
}

// ---------------------------------------------------------------------------
// Pure calculation
// ---------------------------------------------------------------------------

/// Parse and validate `contents`, returning the new document state.
///
/// # Errors
/// Returns `OpenError::Parse` or `OpenError::Validation` on failure.
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

// ---------------------------------------------------------------------------
// Private helpers — apply result to signals
// ---------------------------------------------------------------------------

fn commit_open_result(
    mut signals: WorkspaceSignals,
    next_doc: DiagramDocument,
    next_history: History,
    session: DocumentSession,
) {
    *signals.doc.write() = next_doc;
    *signals.session.write() = session;
    *signals.history.write() = next_history;
}

fn report_open_error(
    toast_handle: crate::ui::toast::ToastHandle,
    label: &'static str,
    err: OpenError,
) {
    let detail = match err {
        OpenError::Parse(s) => format!("Parse error: {s}"),
        OpenError::Validation(s) => format!("Load validation error: {s}"),
        OpenError::Io(s) => format!("IO error: {s}"),
    };
    update_load_save_error(toast_handle, label, detail);
}

// ---------------------------------------------------------------------------
// WASM action
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
fn open_workspace_wasm(mut signals: WorkspaceSignals, toast_handle: crate::ui::toast::ToastHandle) {
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
                    let detail = msg["error"]
                        .as_str()
                        .map_or_else(|| String::from("Browser file import failed"), String::from);
                    update_load_save_error(toast_handle, "Load failed", detail);
                    return;
                }

                let contents = msg["contents"].as_str().map_or("", |v| v);
                let filename = msg["filename"].as_str().map_or("imported.json", |v| v);
                let file_path = std::path::PathBuf::from(filename);
                let current_doc = signals.doc.read().clone();
                let current_history = signals.history.read().clone();

                match apply_open_document(&current_doc, &current_history, contents, file_path) {
                    Ok((next_doc, next_history, session)) => {
                        commit_open_result(signals, next_doc, next_history, session);
                        update_load_save_success(
                            toast_handle,
                            "Workspace loaded",
                            String::from("Loaded from local JSON"),
                        );
                    }
                    Err(err) => report_open_error(toast_handle, "Load failed", err),
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

// ---------------------------------------------------------------------------
// Native (non-WASM) action
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn open_workspace_native(
    mut signals: WorkspaceSignals,
    toast_handle: crate::ui::toast::ToastHandle,
) {
    spawn(async move {
        let path = FileDialog::new()
            .add_filter("Seshat Diagram", &["json"])
            .pick_file();

        match path {
            None => {
                let _ = toast_handle.dismiss();
            }
            Some(p) => {
                let current_doc = signals.doc.read().clone();
                let current_history = signals.history.read().clone();

                match load_workspace_with_lkg(&p) {
                    Ok(mut loaded_doc) => {
                        loaded_doc.revision = Revision::INITIAL;

                        let transform = |_: &DiagramDocument| Ok(loaded_doc);
                        match run_mutation_with_policy(
                            &current_doc,
                            RevisionPolicy::Preserve,
                            ValidationPolicy::default(),
                            transform,
                        ) {
                            Ok(next_doc) => {
                                let next_history = current_history.push(current_doc.clone());
                                let session =
                                    DocumentSession::from_file(next_doc.clone(), p.clone());
                                commit_open_result(signals, next_doc, next_history, session);
                                update_load_save_success(
                                    toast_handle,
                                    "Workspace loaded",
                                    format!("Loaded from {}", p.display()),
                                );
                            }
                            Err(e) => {
                                report_open_error(
                                    toast_handle,
                                    "Load failed",
                                    OpenError::Validation(e.to_string()),
                                );
                            }
                        }
                    }
                    Err(CliPersistenceError::NoValidDocument(primary_err)) => {
                        report_open_error(
                            toast_handle,
                            "Load failed",
                            OpenError::Io(format!(
                                "Cannot open file: {primary_err}. Backup also unavailable."
                            )),
                        );
                    }
                    Err(e) => {
                        report_open_error(
                            toast_handle,
                            "Load failed",
                            OpenError::Io(e.to_string()),
                        );
                    }
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Open a workspace document from disk (native) or a file picker (WASM).
///
/// `signals` bundles the three mutable document-state signals.
/// `toasts` drives the loading/error notifications.
pub fn open_workspace(signals: WorkspaceSignals, toasts: Signal<ToastQueue>) {
    let toast_api = ToastApi::from_signal(toasts);
    let toast_handle = toast_api.toast(
        ToastOptions::new(ToastIntent::Info, "Loading workspace")
            .with_detail("Reading persisted document..."),
    );

    #[cfg(target_arch = "wasm32")]
    {
        open_workspace_wasm(signals, toast_handle);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        open_workspace_native(signals, toast_handle);
    }
}

#[cfg(test)]
#[path = "open_tests.rs"]
mod open_tests;
