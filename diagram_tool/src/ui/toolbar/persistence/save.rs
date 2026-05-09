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
                update_load_save_error(
                    toast_handle,
                    "Save failed",
                    format!("Serialize error: {e}"),
                );
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
#[path = "save_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "save_workspace_tests.rs"]
mod save_workspace_integration_tests;
