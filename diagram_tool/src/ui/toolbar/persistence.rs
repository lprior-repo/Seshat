use crate::history::History;
use crate::models::document::{ArrowType, DiagramDocument, EdgeStyle};
#[cfg(not(target_arch = "wasm32"))]
use crate::models::document::Revision;
use crate::mutation::pipeline::{run_mutation_with_policy, RevisionPolicy};
use crate::ui::editor::ToolMode;
use crate::ui::toast::{ToastIntent, ToastQueue, ToastUpdate};
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(target_arch = "wasm32")]
use crate::backend::{load_workspace_from_backend, save_workspace_to_backend, PersistedWorkspace};

pub fn save_workspace(
    doc_signal: Signal<DiagramDocument>,
    tool_signal: Signal<ToolMode>,
    edge_style_signal: Signal<EdgeStyle>,
    arrow_type_signal: Signal<ArrowType>,
    mut toasts: Signal<ToastQueue>,
) {
    let toast_id = {
        let mut id = None;
        toasts.with_mut(|queue| {
            id = Some(queue.add(
                ToastIntent::Info,
                "Saving workspace",
                Some(String::from("Preparing data...")),
            ));
        });
        id
    };
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
                    toasts.with_mut(|queue| {
                        if let Some(id) = toast_id {
                            let _ = queue.update(
                                id,
                                ToastUpdate {
                                    title: Some(String::from("Workspace saved")),
                                    detail: Some(Some(saved)),
                                    intent: Some(ToastIntent::Success),
                                    action: None,
                                },
                            );
                        }
                    });
                }
                Err(err) => {
                    toasts.with_mut(|queue| {
                        if let Some(id) = toast_id {
                            let _ = queue.update(
                                id,
                                ToastUpdate {
                                    title: Some(String::from("Save failed")),
                                    detail: Some(Some(format!("Backend save error: {err}"))),
                                    intent: Some(ToastIntent::Error),
                                    action: None,
                                },
                            );
                        }
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
                    if let Some(id) = toast_id {
                        toasts.with_mut(|queue| {
                            let _ = queue.dismiss(id);
                        });
                    }
                }
                Some(p) => match serde_json::to_string_pretty(&doc_snapshot) {
                    Ok(json_str) => match fs::write(&p, json_str.as_bytes()) {
                        Ok(()) => {
                            toasts.with_mut(|queue| {
                                if let Some(id) = toast_id {
                                    let _ = queue.update(
                                        id,
                                        ToastUpdate {
                                            title: Some(String::from("Workspace saved")),
                                            detail: Some(Some(format!("Saved to {}", p.display()))),
                                            intent: Some(ToastIntent::Success),
                                            action: None,
                                        },
                                    );
                                }
                            });
                        }
                        Err(e) => update_load_save_error(
                            &mut toasts,
                            toast_id,
                            "Save failed",
                            format!("Save error: {e}"),
                        ),
                    },
                    Err(e) => update_load_save_error(
                        &mut toasts,
                        toast_id,
                        "Save failed",
                        format!("Serialize error: {e}"),
                    ),
                },
            }
        });
    }
}
pub fn open_workspace(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    tool_signal: Signal<ToolMode>,
    edge_style_signal: Signal<EdgeStyle>,
    arrow_type_signal: Signal<ArrowType>,
    mut toasts: Signal<ToastQueue>,
) {
    let toast_id = {
        let mut id = None;
        toasts.with_mut(|queue| {
            id = Some(queue.add(
                ToastIntent::Info,
                "Loading workspace",
                Some(String::from("Reading persisted document...")),
            ));
        });
        id
    };
    #[cfg(target_arch = "wasm32")]
    {
        let mut tool_signal = tool_signal;
        let mut edge_style_signal = edge_style_signal;
        let mut arrow_type_signal = arrow_type_signal;
        spawn(async move {
            match load_workspace_from_backend().await {
                Ok(loaded_workspace) => {
                    let current = doc_signal.read().clone();
                    match run_mutation_with_policy(
                        &current,
                        RevisionPolicy::Preserve,
                        |_| Ok(loaded_workspace.document.clone()),
                    ) {
                        Ok(next_doc) => {
                            *doc_signal.write() = next_doc;
                            *history_signal.write() = History::new();
                            if let Some(mode) = ToolMode::from_persisted_key(&loaded_workspace.tool_mode)
                            {
                                tool_signal.set(mode);
                            }
                            edge_style_signal.set(loaded_workspace.edge_style);
                            arrow_type_signal.set(loaded_workspace.arrow_type);
                            update_load_save_success(
                                &mut toasts,
                                toast_id,
                                "Workspace loaded",
                                String::from("Loaded diagram from backend"),
                            );
                        }
                        Err(err) => update_load_save_error(
                            &mut toasts,
                            toast_id,
                            "Load failed",
                            format!(
                                "Backend load validation error: {}",
                                super::mutation_error_code(&err)
                            ),
                        ),
                    }
                }
                Err(err) => update_load_save_error(
                    &mut toasts,
                    toast_id,
                    "Load failed",
                    format!("Backend load error: {err}"),
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
                    if let Some(id) = toast_id {
                        toasts.with_mut(|queue| {
                            let _ = queue.dismiss(id);
                        });
                    }
                }
                Some(p) => match fs::read_to_string(&p) {
                    Err(e) => update_load_save_error(
                        &mut toasts,
                        toast_id,
                        "Load failed",
                        format!("Read error: {e}"),
                    ),
                    Ok(contents) => match super::persistence_compat::parse_diagram_document_with_compat(&contents) {
                        Err(e) => update_load_save_error(
                            &mut toasts,
                            toast_id,
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
                                    *history_signal.write() = History::new();
                                    update_load_save_success(
                                        &mut toasts,
                                        toast_id,
                                        "Workspace loaded",
                                        format!("Loaded from {}", p.display()),
                                    );
                                }
                                Err(err) => update_load_save_error(
                                    &mut toasts,
                                    toast_id,
                                    "Load failed",
                                    format!("Load validation error: {}", super::mutation_error_code(&err)),
                                ),
                            }
                        }
                    },
                },
            }
        });
    }
}
fn update_load_save_success(toasts: &mut Signal<ToastQueue>, toast_id: Option<crate::ui::toast::ToastId>, title: &str, detail: String) {
    toasts.with_mut(|queue| {
        if let Some(id) = toast_id {
            let _ = queue.update(
                id,
                ToastUpdate {
                    title: Some(title.to_string()),
                    detail: Some(Some(detail)),
                    intent: Some(ToastIntent::Success),
                    action: None,
                },
            );
        }
    });
}
fn update_load_save_error(toasts: &mut Signal<ToastQueue>, toast_id: Option<crate::ui::toast::ToastId>, title: &str, detail: String) {
    toasts.with_mut(|queue| {
        if let Some(id) = toast_id {
            let _ = queue.update(
                id,
                ToastUpdate {
                    title: Some(title.to_string()),
                    detail: Some(Some(detail)),
                    intent: Some(ToastIntent::Error),
                    action: None,
                },
            );
        }
    });
}
