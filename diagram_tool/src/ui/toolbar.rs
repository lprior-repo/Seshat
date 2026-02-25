#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

#[cfg(not(target_arch = "wasm32"))]
use crate::export::png::export_png;
use crate::export::svg::generate_svg_string;
use crate::history::History;
use crate::layout::dag::{dag_layout, DagLayoutSettings};
use crate::models::document::{ArrowType, DiagramDocument, EdgeStyle, NodeId};
use crate::mutation::error::MutationError;
use crate::mutation::pipeline::{run_mutation, run_mutation_with_policy, RevisionPolicy};
#[cfg(not(target_arch = "wasm32"))]
use crate::models::document::Revision;
#[cfg(target_arch = "wasm32")]
use crate::backend::{
    backend_health, load_workspace_from_backend, save_workspace_to_backend, PersistedWorkspace,
};
use crate::ui::commands::{apply_redo, apply_undo};
use crate::ui::editor::ToolMode;
use crate::ui::panels::PanelVisibility;
use crate::ui::theme::{
    ThemeMode, ACCENT, ACCENT_SOFT, BG_BASE, BG_ELEVATED, BG_SURFACE, BORDER, BORDER_SUBTLE,
    ERROR, TEXT_MAIN, TEXT_MUTED,
};
use crate::ui::toast::{use_toast, ToastIntent, ToastQueue, ToastUpdate};
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use base64::{engine::general_purpose, Engine as _};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

fn zoom_to_center(doc: &mut DiagramDocument, factor: f64, viewport_size: (f64, f64)) {
    let old_zoom = doc.editor_state.zoom.0;
    let new_zoom = (old_zoom * factor).clamp(0.1, 4.0);
    if (new_zoom - old_zoom).abs() < f64::EPSILON {
        return;
    }

    let viewport_w = viewport_size.0.max(1.0);
    let viewport_h = viewport_size.1.max(1.0);
    let center_world_x = ((viewport_w / 2.0) - doc.editor_state.camera_x.0) / old_zoom;
    let center_world_y = ((viewport_h / 2.0) - doc.editor_state.camera_y.0) / old_zoom;

    doc.editor_state.camera_x.0 = (viewport_w / 2.0) - (center_world_x * new_zoom);
    doc.editor_state.camera_y.0 = (viewport_h / 2.0) - (center_world_y * new_zoom);
    doc.editor_state.zoom.0 = new_zoom;
}

fn delete_selected_items(doc: &mut DiagramDocument) {
    let selected = doc.editor_state.selected_items.clone();
    if selected.is_empty() {
        return;
    }

    doc.document.nodes = doc
        .document
        .nodes
        .iter()
        .filter(|(id, _)| !selected.contains(&id.to_string()))
        .map(|(id, node)| (id.clone(), node.clone()))
        .collect();

    let node_ids: im::HashSet<NodeId> = doc.document.nodes.keys().cloned().collect();
    doc.document.edges = doc
        .document
        .edges
        .iter()
        .filter(|(id, edge)| {
            node_ids.contains(&edge.source)
                && node_ids.contains(&edge.target)
                && !selected.contains(&id.to_string())
        })
        .map(|(id, edge)| (id.clone(), edge.clone()))
        .collect();

    doc.editor_state.selected_items.clear();
    doc.revision = doc.revision.increment();
}

#[cfg(target_arch = "wasm32")]
fn wasm_download_bytes(filename: &str, mime: &str, bytes: &[u8]) {
    let b64 = general_purpose::STANDARD.encode(bytes);
    let script = format!(
        "(function() {{ const raw = atob('{b64}'); const array = new Uint8Array(raw.length); for (let i = 0; i < raw.length; i++) array[i] = raw.charCodeAt(i); const blob = new Blob([array], {{ type: '{mime}' }}); const url = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = url; a.download = '{filename}'; a.click(); setTimeout(() => URL.revokeObjectURL(url), 0); dioxus.send({{ ok: true }}); }})();"
    );
    let mut eval = document::eval(&script);
    spawn(async move {
        let _ = eval.recv::<serde_json::Value>().await;
    });
}

#[cfg(target_arch = "wasm32")]
fn wasm_export_png_from_svg(svg: String, filename: &str, mut toasts: Signal<ToastQueue>) {
    let svg_b64 = general_purpose::STANDARD.encode(svg.as_bytes());
    let script = format!(
        "(function() {{ try {{ const svgText = atob('{svg_b64}'); const svgBlob = new Blob([svgText], {{ type: 'image/svg+xml' }}); const svgUrl = URL.createObjectURL(svgBlob); const img = new Image(); img.onload = () => {{ const canvas = document.createElement('canvas'); canvas.width = Math.max(1, img.naturalWidth || img.width || 1); canvas.height = Math.max(1, img.naturalHeight || img.height || 1); const ctx = canvas.getContext('2d'); if (!ctx) {{ URL.revokeObjectURL(svgUrl); dioxus.send({{ ok: false, reason: 'no-canvas-context' }}); return; }} ctx.drawImage(img, 0, 0); canvas.toBlob((blob) => {{ URL.revokeObjectURL(svgUrl); if (!blob) {{ dioxus.send({{ ok: false, reason: 'png-encode-failed' }}); return; }} const out = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = out; a.download = '{filename}'; a.click(); setTimeout(() => URL.revokeObjectURL(out), 0); dioxus.send({{ ok: true }}); }}, 'image/png'); }}; img.onerror = () => {{ URL.revokeObjectURL(svgUrl); dioxus.send({{ ok: false, reason: 'svg-image-load-failed' }}); }}; img.src = svgUrl; }} catch (error) {{ dioxus.send({{ ok: false, reason: String(error) }}); }} }})();"
    );
    let mut eval = document::eval(&script);
    spawn(async move {
        if let Ok(msg) = eval.recv::<serde_json::Value>().await {
            if msg["ok"].as_bool() != Some(true) {
                let reason = msg["reason"].as_str().map_or("unknown", |v| v);
                toasts.with_mut(|queue| {
                    let _ = queue.add(
                        ToastIntent::Error,
                        "PNG export failed",
                        Some(format!("Browser export error: {reason}")),
                    );
                });
            }
        }
    });
}

#[component]
pub fn Toolbar() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history_signal = use_context::<Signal<History>>();
    let mut tool_signal = use_context::<Signal<ToolMode>>();
    let viewport_size_signal = use_context::<Signal<(f64, f64)>>();
    let mut theme_mode_signal = use_context::<Signal<ThemeMode>>();
    let mut panel_visibility = use_context::<Signal<PanelVisibility>>();
    let mut toasts = use_context::<Signal<ToastQueue>>();
    let toast = use_toast();
    let edge_style_signal = use_context::<Signal<EdgeStyle>>();
    let arrow_type_signal = use_context::<Signal<ArrowType>>();
    let mut validate_trigger = use_context::<Signal<u64>>();
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (&edge_style_signal, &arrow_type_signal);
    }
    let save_label = if cfg!(target_arch = "wasm32") {
        "Save to Server"
    } else {
        "Save"
    };
    let open_label = if cfg!(target_arch = "wasm32") {
        "Load from Server"
    } else {
        "Open"
    };

    #[cfg(target_arch = "wasm32")]
    let _backend_status = use_resource(move || async move {
        backend_health()
            .await
            .map_or_else(|_| String::from("Backend unavailable"), |ok| ok)
    });

    let handle_auto_layout = move |_| {
        let current_doc = doc_signal.read().clone();
        match run_mutation(&current_doc, |doc| {
            Ok(dag_layout(doc, &DagLayoutSettings::default()))
        }) {
            Ok(next_doc) => {
                let history = history_signal.read().clone();
                *history_signal.write() = history.push(current_doc);
                *doc_signal.write() = next_doc;
            }
            Err(err) => {
                let _ = toast.error(
                    "Auto-arrange failed",
                    Some(format!("Code: {}", mutation_error_code(&err))),
                );
            }
        }
    };

    let handle_undo = move |_| {
        apply_undo(doc_signal, history_signal);
    };

    let handle_redo = move |_| {
        apply_redo(doc_signal, history_signal);
    };

    let handle_zoom_in = move |_| {
        let viewport_size = *viewport_size_signal.read();
        let current = doc_signal.read().clone();
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(current);
        doc_signal.with_mut(|doc| {
            zoom_to_center(doc, 1.25, viewport_size);
            doc.revision = doc.revision.increment();
        });
    };

    let handle_zoom_out = move |_| {
        let viewport_size = *viewport_size_signal.read();
        let current = doc_signal.read().clone();
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(current);
        doc_signal.with_mut(|doc| {
            zoom_to_center(doc, 0.8, viewport_size);
            doc.revision = doc.revision.increment();
        });
    };

    let handle_zoom_reset = move |_| {
        let current = doc_signal.read().clone();
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(current);
        doc_signal.with_mut(|doc| {
            doc.editor_state.zoom.0 = 1.0;
            doc.revision = doc.revision.increment();
        });
    };

    let handle_delete = move |_| {
        let current = doc_signal.read().clone();
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(current);
        doc_signal.with_mut(delete_selected_items);
    };

    let selected_count = doc_signal.read().editor_state.selected_items.len();
    let node_count = doc_signal.read().document.nodes.len();
    let edge_count = doc_signal.read().document.edges.len();
    let zoom_pct = (doc_signal.read().editor_state.zoom.0 * 100.0).round();
    let delete_color = if selected_count > 0 {
        ERROR
    } else {
        TEXT_MAIN
    };
    let delete_opacity = if selected_count > 0 { "1" } else { "0.6" };

    let handle_save = move |_| {
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
            let mut toast_queue = toasts;
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
                        toast_queue.with_mut(|queue| {
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
                        toast_queue.with_mut(|queue| {
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
        let doc_snapshot = doc_signal.read().clone();
        #[cfg(not(target_arch = "wasm32"))]
        let mut toast_queue = toasts;
        #[cfg(not(target_arch = "wasm32"))]
        spawn(async move {
            let path = FileDialog::new()
                .add_filter("Seshat Diagram", &["json"])
                .set_file_name("diagram.json")
                .save_file();
            match path {
                None => {
                    if let Some(id) = toast_id {
                        toast_queue.with_mut(|queue| {
                            let _ = queue.dismiss(id);
                        });
                    }
                }
                Some(p) => match serde_json::to_string_pretty(&doc_snapshot) {
                    Ok(json_str) => match fs::write(&p, json_str.as_bytes()) {
                        Ok(()) => {
                            toast_queue.with_mut(|queue| {
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
                        Err(e) => {
                            toast_queue.with_mut(|queue| {
                                if let Some(id) = toast_id {
                                    let _ = queue.update(
                                        id,
                                        ToastUpdate {
                                            title: Some(String::from("Save failed")),
                                            detail: Some(Some(format!("Save error: {e}"))),
                                            intent: Some(ToastIntent::Error),
                                            action: None,
                                        },
                                    );
                                }
                            });
                        }
                    },
                    Err(e) => {
                        toast_queue.with_mut(|queue| {
                            if let Some(id) = toast_id {
                                let _ = queue.update(
                                    id,
                                    ToastUpdate {
                                        title: Some(String::from("Save failed")),
                                        detail: Some(Some(format!("Serialize error: {e}"))),
                                        intent: Some(ToastIntent::Error),
                                        action: None,
                                    },
                                );
                            }
                        });
                    }
                },
            }
        });
    };

    let handle_open = move |_| {
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
            let mut doc_sig = doc_signal;
            let mut hist_sig = history_signal;
            let mut tool_sig = tool_signal;
            let mut edge_style_sig = edge_style_signal;
            let mut arrow_type_sig = arrow_type_signal;
            let mut toast_queue = toasts;
            spawn(async move {
                match load_workspace_from_backend().await {
                    Ok(loaded_workspace) => {
                        let current = doc_sig.read().clone();
                        match run_mutation_with_policy(
                            &current,
                            RevisionPolicy::Preserve,
                            |_| Ok(loaded_workspace.document.clone()),
                        ) {
                            Ok(next_doc) => {
                                *doc_sig.write() = next_doc;
                                *hist_sig.write() = History::new();
                                if let Some(mode) =
                                    ToolMode::from_persisted_key(&loaded_workspace.tool_mode)
                                {
                                    tool_sig.set(mode);
                                }
                                edge_style_sig.set(loaded_workspace.edge_style);
                                arrow_type_sig.set(loaded_workspace.arrow_type);
                                toast_queue.with_mut(|queue| {
                                    if let Some(id) = toast_id {
                                        let _ = queue.update(
                                            id,
                                            ToastUpdate {
                                                title: Some(String::from("Workspace loaded")),
                                                detail: Some(Some(String::from("Loaded diagram from backend"))),
                                                intent: Some(ToastIntent::Success),
                                                action: None,
                                            },
                                        );
                                    }
                                });
                            }
                            Err(err) => {
                                toast_queue.with_mut(|queue| {
                                    if let Some(id) = toast_id {
                                        let _ = queue.update(
                                            id,
                                            ToastUpdate {
                                                title: Some(String::from("Load failed")),
                                                detail: Some(Some(format!(
                                                    "Backend load validation error: {}",
                                                    mutation_error_code(&err)
                                                ))),
                                                intent: Some(ToastIntent::Error),
                                                action: None,
                                            },
                                        );
                                    }
                                });
                            }
                        }
                    }
                    Err(err) => {
                        toast_queue.with_mut(|queue| {
                            if let Some(id) = toast_id {
                                let _ = queue.update(
                                    id,
                                    ToastUpdate {
                                        title: Some(String::from("Load failed")),
                                        detail: Some(Some(format!("Backend load error: {err}"))),
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
        let mut doc_sig = doc_signal;
        #[cfg(not(target_arch = "wasm32"))]
        let mut hist_sig = history_signal;
        #[cfg(not(target_arch = "wasm32"))]
        let mut toast_queue = toasts;
        #[cfg(not(target_arch = "wasm32"))]
        spawn(async move {
            let path = FileDialog::new()
                .add_filter("Seshat Diagram", &["json"])
                .pick_file();
            match path {
                None => {
                    if let Some(id) = toast_id {
                        toast_queue.with_mut(|queue| {
                            let _ = queue.dismiss(id);
                        });
                    }
                }
                Some(p) => match fs::read_to_string(&p) {
                    Err(e) => {
                        toast_queue.with_mut(|queue| {
                            if let Some(id) = toast_id {
                                let _ = queue.update(
                                    id,
                                    ToastUpdate {
                                        title: Some(String::from("Load failed")),
                                        detail: Some(Some(format!("Read error: {e}"))),
                                        intent: Some(ToastIntent::Error),
                                        action: None,
                                    },
                                );
                            }
                        });
                    }
                    Ok(contents) => match parse_diagram_document_with_compat(&contents) {
                        Err(e) => {
                            toast_queue.with_mut(|queue| {
                                if let Some(id) = toast_id {
                                    let _ = queue.update(
                                        id,
                                        ToastUpdate {
                                            title: Some(String::from("Load failed")),
                                            detail: Some(Some(format!("Parse error: {e}"))),
                                            intent: Some(ToastIntent::Error),
                                            action: None,
                                        },
                                    );
                                }
                            });
                        }
                        Ok(mut loaded_doc) => {
                            loaded_doc.revision = Revision::INITIAL;
                            let current = doc_sig.read().clone();
                            match run_mutation_with_policy(
                                &current,
                                RevisionPolicy::Preserve,
                                |_| Ok(loaded_doc),
                            ) {
                                Ok(next_doc) => {
                                    *doc_sig.write() = next_doc;
                                    *hist_sig.write() = History::new();
                                    toast_queue.with_mut(|queue| {
                                        if let Some(id) = toast_id {
                                            let _ = queue.update(
                                                id,
                                                ToastUpdate {
                                                    title: Some(String::from("Workspace loaded")),
                                                    detail: Some(Some(format!("Loaded from {}", p.display()))),
                                                    intent: Some(ToastIntent::Success),
                                                    action: None,
                                                },
                                            );
                                        }
                                    });
                                }
                                Err(err) => {
                                    toast_queue.with_mut(|queue| {
                                        if let Some(id) = toast_id {
                                            let _ = queue.update(
                                                id,
                                                ToastUpdate {
                                                    title: Some(String::from("Load failed")),
                                                    detail: Some(Some(format!(
                                                        "Load validation error: {}",
                                                        mutation_error_code(&err)
                                                    ))),
                                                    intent: Some(ToastIntent::Error),
                                                    action: None,
                                                },
                                            );
                                        }
                                    });
                                }
                            }
                        }
                    },
                },
            }
        });
    };

    rsx! {
        div {
            class: "toolbar",
            style: "height: 56px; background: linear-gradient(180deg, {BG_SURFACE} 0%, {BG_ELEVATED} 100%); color: {TEXT_MAIN}; display: flex; align-items: center; padding: 0 12px; gap: 8px; border-bottom: 1px solid {BORDER_SUBTLE}; box-shadow: 0 4px 16px color-mix(in oklch, black 22%, transparent); overflow-x: auto;",

            for mode in [ToolMode::Select, ToolMode::Pan, ToolMode::Edge, ToolMode::Subgraph, ToolMode::Text] {
                {
                    let active = *tool_signal.read() == mode;
                    let bg = if active { ACCENT_SOFT } else { "transparent" };
                    let border = if active { format!("1px solid {ACCENT}") } else { format!("1px solid {BORDER}") };
                    rsx! {
                        button {
                            style: "padding: 6px 10px; border-radius: 6px; border: {border}; background: {bg}; color: {TEXT_MAIN}; cursor: pointer; font-size: 12px;",
                            onclick: move |_| tool_signal.set(mode),
                            "{mode.label()}"
                        }
                    }
                }
            }

            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: handle_auto_layout,
                "Auto-Arrange"
            }

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: handle_undo,
                "Undo"
            }
            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: handle_redo,
                "Redo"
            }

            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: handle_zoom_in,
                "+"
            }
            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: handle_zoom_reset,
                "{zoom_pct}%"
            }
            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: handle_zoom_out,
                "-"
            }

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {delete_color}; opacity: {delete_opacity};",
                onclick: handle_delete,
                disabled: selected_count == 0,
                "Delete"
            }

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            button {
                style: "padding: 5px 10px; cursor: pointer; background: {ACCENT}; border: none; border-radius: 4px; color: {BG_BASE};",
                onclick: move |_| {
                    validate_trigger.with_mut(|t| *t = t.saturating_add(1));
                },
                "Validate"
            }

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: handle_save,
                "{save_label}"
            }
            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: handle_open,
                "{open_label}"
            }

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            select {
                style: "padding: 6px 8px; min-width: 110px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN}; cursor: pointer; font-size: 12px;",
                value: "{theme_mode_signal.read().persisted_key()}",
                onchange: move |evt| {
                    if let Some(next) = ThemeMode::from_persisted_key(&evt.value()) {
                        theme_mode_signal.set(next);
                    }
                },
                for mode in [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark] {
                    option { value: "{mode.persisted_key()}", "{mode.label()} theme" }
                }
            }

            for (label, enabled, setter) in [
                ("Icons", panel_visibility.read().sidebar, 0_u8),
                ("Props", panel_visibility.read().properties, 1_u8),
                ("Mini", panel_visibility.read().minimap, 2_u8),
                ("Valid", panel_visibility.read().validation, 3_u8),
            ] {
                {
                    let bg = if enabled { ACCENT_SOFT } else { BG_BASE };
                    let border = if enabled { format!("1px solid {ACCENT}") } else { format!("1px solid {BORDER}") };
                    rsx! {
                        button {
                            style: "padding: 6px 8px; cursor: pointer; border-radius: 6px; border: {border}; background: {bg}; color: {TEXT_MAIN}; font-size: 11px;",
                            onclick: move |_| {
                                panel_visibility.with_mut(|panels| {
                                    match setter {
                                        0 => panels.sidebar = !panels.sidebar,
                                        1 => panels.properties = !panels.properties,
                                        2 => panels.minimap = !panels.minimap,
                                        _ => panels.validation = !panels.validation,
                                    }
                                });
                            },
                            "{label}"
                        }
                    }
                }
            }

            div { style: "flex: 1;" }

            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let doc = doc_signal.read().clone();
                        let svg = generate_svg_string(&doc);
                        wasm_export_png_from_svg(svg, "diagram.png", toasts);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    let doc = doc_signal.read();
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = export_png(&doc, "diagram.png");
                },
                "Export PNG"
            }
            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let doc = doc_signal.read().clone();
                        let svg = generate_svg_string(&doc);
                        wasm_download_bytes("diagram.svg", "image/svg+xml", svg.as_bytes());
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    let doc = doc_signal.read();
                    #[cfg(not(target_arch = "wasm32"))]
                    let svg = generate_svg_string(&doc);
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Ok(mut file) = File::create("diagram.svg") {
                        #[cfg(not(target_arch = "wasm32"))]
                        let _ = file.write_all(svg.as_bytes());
                    }
                },
                "Export SVG"
            }
            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| {
                    let doc = doc_signal.read().clone();
                    if let Ok(json) = serde_json::to_vec_pretty(&doc) {
                        #[cfg(target_arch = "wasm32")]
                        {
                            wasm_download_bytes("diagram.json", "application/json", &json);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Ok(mut file) = File::create("diagram.json") {
                                let _ = file.write_all(&json);
                            }
                        }
                    }
                },
                "Export JSON"
            }

            span {
                style: "font-size: 11px; color: {TEXT_MUTED}; margin-left: 8px;",
                "{node_count} nodes"
            }
            span {
                style: "font-size: 11px; color: {TEXT_MUTED};",
                "{edge_count} edges"
            }
            span {
                style: "font-size: 11px; color: {TEXT_MUTED};",
                "{selected_count} selected"
            }
            span {
                style: "font-size: 11px; color: {TEXT_MUTED};",
                "Rev {doc_signal.read().revision:?}"
            }
        }
    }
}

const fn mutation_error_code(err: &MutationError) -> &'static str {
    match err {
        MutationError::Transform(_) => "transform_error",
        MutationError::Schema(_) => "schema_error",
        MutationError::Semantic(_) => "semantic_error",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn remap_key(obj: &mut serde_json::Map<String, serde_json::Value>, from: &str, to: &str) {
    if obj.contains_key(to) {
        let _ = obj.remove(from);
    } else if let Some(value) = obj.remove(from) {
        let _ = obj.insert(to.to_string(), value);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_compat_shape(root: &mut serde_json::Value) {
    let Some(document) = root
        .as_object_mut()
        .and_then(|obj| obj.get_mut("document"))
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };

    if let Some(nodes) = document
        .get_mut("nodes")
        .and_then(serde_json::Value::as_object_mut)
    {
        for node in nodes.values_mut() {
            if let Some(node_obj) = node.as_object_mut() {
                let _ = node_obj.remove("id");
                remap_key(node_obj, "fontSize", "font_size");
                remap_key(node_obj, "fontWeight", "font_weight");
                remap_key(node_obj, "dagRank", "dag_rank");
            }
        }
    }

    if let Some(edges) = document
        .get_mut("edges")
        .and_then(serde_json::Value::as_object_mut)
    {
        for edge in edges.values_mut() {
            if let Some(edge_obj) = edge.as_object_mut() {
                let _ = edge_obj.remove("id");
                remap_key(edge_obj, "arrowType", "arrowhead");
                remap_key(edge_obj, "arrow_type", "arrowhead");
                remap_key(edge_obj, "fontSize", "font_size");
                remap_key(edge_obj, "bendPoints", "bend_points");
                remap_key(edge_obj, "labelOffsetT", "label_offset_t");
                if let Some(arrowhead) = edge_obj.get_mut("arrowhead") {
                    let normalized = arrowhead
                        .as_str()
                        .map(|value| match value {
                            "default" => "arrow",
                            "straight" => "open",
                            "step" => "diamond",
                            "curved" => "circle",
                            "sharp" => "none",
                            _ => value,
                        })
                        .map(ToString::to_string);
                    if let Some(value) = normalized {
                        *arrowhead = serde_json::Value::String(value);
                    }
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_diagram_document_with_compat(contents: &str) -> Result<DiagramDocument, String> {
    let mut value =
        serde_json::from_str::<serde_json::Value>(contents).map_err(|err| err.to_string())?;
    normalize_compat_shape(&mut value);
    serde_json::from_value::<DiagramDocument>(value).map_err(|err| err.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use crate::models::document::DiagramDocument;

    #[test]
    fn given_document_when_serialized_then_round_trips() {
        let doc = DiagramDocument::default();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let loaded: DiagramDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc.revision, loaded.revision);
    }

    #[test]
    fn given_ts_style_json_when_parsed_then_document_loads() {
        let json = r#"{
          "version": 2,
          "revision": 1,
          "document": {
            "nodes": {
              "n1": {
                "id": "n1",
                "kind": "node",
                "icon": "aws/compute/ec2",
                "label": "EC2",
                "x": 10,
                "y": 20,
                "width": 64,
                "height": 64,
                "locked": true,
                "parent": null,
                "tags": ["aws", "compute"],
                "metadata": {}
              }
            },
            "edges": {
              "e1": {
                "id": "e1",
                "source": "n1",
                "target": "n1",
                "label": "",
                "style": "solid",
                "arrowType": "curved",
                "directed": true,
                "bend_points": []
              }
            }
          },
          "editor_state": {
            "camera_x": 0,
            "camera_y": 0,
            "zoom": 1,
            "grid_size": 20,
            "snap_to_grid": true,
            "selected_items": []
          }
        }"#;

        let loaded = super::parse_diagram_document_with_compat(json);
        assert!(loaded.is_ok(), "{:?}", loaded.err());
    }
}
