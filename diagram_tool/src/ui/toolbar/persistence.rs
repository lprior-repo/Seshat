#[cfg(target_arch = "wasm32")]
use crate::backend::{save_workspace_to_backend, PersistedWorkspace};
use crate::history::History;
use crate::models::document::{ArrowType, DiagramDocument, EdgeStyle, Revision};
use crate::mutation::pipeline::{run_mutation_with_policy, RevisionPolicy};
use crate::ui::editor::ToolMode;
use crate::ui::toast::{ToastApi, ToastHandle, ToastIntent, ToastOptions, ToastQueue, ToastUpdate};
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

pub fn save_workspace(
    doc_signal: Signal<DiagramDocument>,
    tool_signal: Signal<ToolMode>,
    edge_style_signal: Signal<EdgeStyle>,
    arrow_type_signal: Signal<ArrowType>,
    toasts: Signal<ToastQueue>,
) {
    let toast_api = ToastApi::from_signal(toasts);
    let toast_handle = toast_api.toast(
        ToastOptions::new(ToastIntent::Info, "Saving workspace").with_detail("Preparing data..."),
    );
    #[cfg(target_arch = "wasm32")]
    {
        let doc_snapshot = doc_signal.read().clone();
        let tool_mode = tool_signal.read().persisted_key().to_string();
        let edge_style = *edge_style_signal.read();
        let arrow_type = *arrow_type_signal.read();
        spawn(async move {
            let workspace = PersistedWorkspace {
                schema_version: PersistedWorkspace::SCHEMA_VERSION,
                document: doc_snapshot,
                tool_mode,
                edge_style,
                arrow_type,
            };
            match save_workspace_to_backend(workspace).await {
                Ok(saved) => {
                    let _ = toast_handle.update(ToastUpdate {
                        title: Some(String::from("Workspace saved")),
                        detail: Some(Some(saved)),
                        intent: Some(ToastIntent::Success),
                        action: None,
                    });
                }
                Err(err) => {
                    let _ = toast_handle.update(ToastUpdate {
                        title: Some(String::from("Save failed")),
                        detail: Some(Some(format!("Backend save error: {err}"))),
                        intent: Some(ToastIntent::Error),
                        action: None,
                    });
                }
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (&tool_signal, &edge_style_signal, &arrow_type_signal);
        let doc_snapshot = doc_signal.read().clone();
        spawn(async move {
            let path = FileDialog::new()
                .add_filter("Seshat Diagram", &["json"])
                .set_file_name("diagram.json")
                .save_file();
            match path {
                None => {
                    let _ = toast_handle.dismiss();
                }
                Some(p) => match serde_json::to_string_pretty(&doc_snapshot) {
                    Ok(json_str) => match fs::write(&p, json_str.as_bytes()) {
                        Ok(()) => {
                            update_load_save_success(
                                toast_handle,
                                "Workspace saved",
                                format!("Saved to {}", p.display()),
                            );
                        }
                        Err(e) => update_load_save_error(
                            toast_handle,
                            "Save failed",
                            format!("Save error: {e}"),
                        ),
                    },
                    Err(e) => update_load_save_error(
                        toast_handle,
                        "Save failed",
                        format!("Serialize error: {e}"),
                    ),
                },
            }
        });
    }
}
#[allow(clippy::too_many_lines)]
pub fn open_workspace(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    tool_signal: Signal<ToolMode>,
    edge_style_signal: Signal<EdgeStyle>,
    arrow_type_signal: Signal<ArrowType>,
    toasts: Signal<ToastQueue>,
) {
    let toast_api = ToastApi::from_signal(toasts);
    let toast_handle = toast_api.toast(
        ToastOptions::new(ToastIntent::Info, "Loading workspace")
            .with_detail("Reading persisted document..."),
    );
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (&tool_signal, &edge_style_signal, &arrow_type_signal);
        spawn(async move {
            let mut eval = document::eval(
                r#"
                (function() {
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
                            finish({ ok: true, contents: String(reader.result || '') });
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
                    match super::persistence_compat::parse_diagram_document_with_compat(contents) {
                        Err(err) => {
                            update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Parse error: {err}"),
                            );
                        }
                        Ok(mut loaded_doc) => {
                            loaded_doc.revision = Revision::INITIAL;
                            let current = doc_signal.read().clone();
                            match run_mutation_with_policy(
                                &current,
                                RevisionPolicy::Preserve,
                                |_| Ok(loaded_doc),
                            ) {
                                Ok(next_doc) => {
                                    *doc_signal.write() = next_doc;
                                    *history_signal.write() = History::new().push(current);
                                    update_load_save_success(
                                        toast_handle,
                                        "Workspace loaded",
                                        String::from("Loaded from local JSON"),
                                    );
                                }
                                Err(err) => update_load_save_error(
                                    toast_handle,
                                    "Load failed",
                                    format!(
                                        "Load validation error: {}",
                                        super::mutation_error_code(&err)
                                    ),
                                ),
                            }
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
                        match super::persistence_compat::parse_diagram_document_with_compat(
                            &contents,
                        ) {
                            Err(e) => update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Parse error: {e}"),
                            ),
                            Ok(mut loaded_doc) => {
                                loaded_doc.revision = Revision::INITIAL;
                                let current = doc_signal.read().clone();
                                match run_mutation_with_policy(
                                    &current,
                                    RevisionPolicy::Preserve,
                                    |_| Ok(loaded_doc),
                                ) {
                                    Ok(next_doc) => {
                                        *doc_signal.write() = next_doc;
                                        *history_signal.write() = History::new().push(current);
                                        update_load_save_success(
                                            toast_handle,
                                            "Workspace loaded",
                                            format!("Loaded from {}", p.display()),
                                        );
                                    }
                                    Err(err) => update_load_save_error(
                                        toast_handle,
                                        "Load failed",
                                        format!(
                                            "Load validation error: {}",
                                            super::mutation_error_code(&err)
                                        ),
                                    ),
                                }
                            }
                        }
                    }
                },
            }
        });
    }
}
fn update_load_save_success(toast_handle: ToastHandle, title: &str, detail: String) {
    let _ = toast_handle.update(ToastUpdate {
        title: Some(title.to_string()),
        detail: Some(Some(detail)),
        intent: Some(ToastIntent::Success),
        action: None,
    });
}
fn update_load_save_error(toast_handle: ToastHandle, title: &str, detail: String) {
    let _ = toast_handle.update(ToastUpdate {
        title: Some(title.to_string()),
        detail: Some(Some(detail)),
        intent: Some(ToastIntent::Error),
        action: None,
    });
}
