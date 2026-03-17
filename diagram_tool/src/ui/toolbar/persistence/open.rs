use crate::history::History;
use crate::ui::editor::ToolMode;
use crate::ui::toast::{ToastApi, ToastIntent, ToastOptions, ToastQueue};
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle};
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

use super::common::{
    apply_import_contents, update_load_save_error, update_load_save_success, ImportTransitionError,
};

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
                    if (window.__SESHAT_E2E_IMPORT_JSON) {
                        const contents = window.__SESHAT_E2E_IMPORT_JSON;
                        delete window.__SESHAT_E2E_IMPORT_JSON;
                        dioxus.send({ ok: true, contents });
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
                    let mut next_doc = doc_signal.read().clone();
                    let mut next_history = history_signal.read().clone();
                    match apply_import_contents(&mut next_doc, &mut next_history, contents) {
                        Ok(()) => {
                            *doc_signal.write() = next_doc;
                            *history_signal.write() = next_history;
                            update_load_save_success(
                                toast_handle,
                                "Workspace loaded",
                                String::from("Loaded from local JSON"),
                            );
                        }
                        Err(ImportTransitionError::Parse(err)) => {
                            update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Parse error: {err}"),
                            );
                        }
                        Err(ImportTransitionError::Validation(code)) => {
                            update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Load validation error: {code}"),
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
                        let mut next_doc = doc_signal.read().clone();
                        let mut next_history = history_signal.read().clone();
                        match apply_import_contents(&mut next_doc, &mut next_history, &contents) {
                            Ok(()) => {
                                *doc_signal.write() = next_doc;
                                *history_signal.write() = next_history;
                                update_load_save_success(
                                    toast_handle,
                                    "Workspace loaded",
                                    format!("Loaded from {}", p.display()),
                                );
                            }
                            Err(ImportTransitionError::Parse(e)) => update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Parse error: {e}"),
                            ),
                            Err(ImportTransitionError::Validation(code)) => {
                                update_load_save_error(
                                    toast_handle,
                                    "Load failed",
                                    format!("Load validation error: {code}"),
                                );
                            }
                        }
                    }
                },
            }
        });
    }
}
