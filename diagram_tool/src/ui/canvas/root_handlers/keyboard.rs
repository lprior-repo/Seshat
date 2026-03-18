use crate::history::History;
use crate::ui::commands::{
    apply_clear_selection, apply_delete_selected, apply_nudge_selection, apply_zoom_in,
    apply_zoom_out, apply_zoom_reset,
};
use crate::ui::dispatch::dispatch_node_delete_batch;
use crate::ui::editor::ToolMode;
use canvas_domain::interaction_reducer::{finalize_motion_release, InteractionMode};
use canvas_domain::selection_geometry::selected_node_ids;
use diagram_models::document::DiagramDocument;
use diagram_models::document::EdgeId;
use diagram_models::document::NodeId;
use dioxus::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn use_keyboard_handler(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    mut interaction_mode: Signal<InteractionMode>,
    mut tool_signal: Signal<ToolMode>,
    mut space_pressed: Signal<bool>,
    mut shift_pressed: Signal<bool>,
    mut ctrl_pressed: Signal<bool>,
    mut meta_pressed: Signal<bool>,
    mut nudge_batch_active: Signal<bool>,
    mut space_pan_active: Signal<bool>,
    mut editing_node: Signal<Option<NodeId>>,
    mut editing_edge: Signal<Option<EdgeId>>,
    mut edit_value: Signal<String>,
    viewport_size: Signal<(f64, f64)>,
    db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
) {
    use_effect(move || {
        let mut eval = document::eval(
            r"
                if (window.__seshat_canvas_keyboard_cleanup) {
                    window.__seshat_canvas_keyboard_cleanup();
                }

                const onKeyDown = (e) => {
                    const active = document.activeElement;
                    const editing = active && (
                        active.tagName === 'INPUT' ||
                        active.tagName === 'TEXTAREA' ||
                        active.isContentEditable
                    );
                    if (editing) return;
                    const key = e.key;
                    const isArrow = key === 'ArrowUp' || key === 'ArrowDown' || key === 'ArrowLeft' || key === 'ArrowRight';
                    const isZoom = key === '+' || key === '=' || key === '-' || key === '_' || key === '0';
                    const isDelete = key === 'Delete' || key === 'Backspace';
                    const handled = key === ' ' || key === 'Escape' || isArrow || isZoom || isDelete;
                    if (handled) {
                        e.preventDefault();
                    }
                    dioxus.send({ type: 'keydown', key: key, ctrl: e.ctrlKey, shift: e.shiftKey, meta: e.metaKey, repeat: e.repeat });
                };

                const onKeyUp = (e) => {
                    const active = document.activeElement;
                    const editing = active && (
                        active.tagName === 'INPUT' ||
                        active.tagName === 'TEXTAREA' ||
                        active.isContentEditable
                    );
                    if (editing) return;
                    dioxus.send({ type: 'keyup', key: e.key, ctrl: e.ctrlKey, shift: e.shiftKey, meta: e.metaKey, repeat: false });
                };

                const onWindowBlur = () => {
                    dioxus.send({ type: 'blur', key: '', ctrl: false, shift: false, meta: false, repeat: false });
                };

                window.addEventListener('keydown', onKeyDown);
                window.addEventListener('keyup', onKeyUp);
                window.addEventListener('blur', onWindowBlur);
                window.__seshat_canvas_keyboard_cleanup = () => {
                    window.removeEventListener('keydown', onKeyDown);
                    window.removeEventListener('keyup', onKeyUp);
                    window.removeEventListener('blur', onWindowBlur);
                };
            ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                let event_type = json["type"].as_str().map_or("", |s| s);
                let key = json["key"].as_str().map_or("", |s| s);
                let ctrl = json["ctrl"].as_bool().is_some_and(|v| v);
                let meta = json["meta"].as_bool().is_some_and(|v| v);
                let shift = json["shift"].as_bool().is_some_and(|v| v);
                let modifier = ctrl || meta;
                let is_arrow_key =
                    matches!(key, "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight");

                if event_type == "blur" {
                    space_pressed.set(false);
                    shift_pressed.set(false);
                    ctrl_pressed.set(false);
                    meta_pressed.set(false);
                    nudge_batch_active.set(false);
                    space_pan_active.set(false);
                    continue;
                }

                if key == " " {
                    space_pressed.set(event_type == "keydown");
                    if event_type == "keyup" {
                        let should_cancel_space_pan = *space_pan_active.read()
                            && matches!(*interaction_mode.read(), InteractionMode::Panning { .. })
                            && *tool_signal.read() != ToolMode::Pan;
                        if should_cancel_space_pan {
                            interaction_mode.set(InteractionMode::Select);
                        }
                        space_pan_active.set(false);
                    }
                }
                if key == "Shift" {
                    shift_pressed.set(event_type == "keydown");
                }
                if key == "Control" {
                    ctrl_pressed.set(event_type == "keydown");
                }
                if key == "Meta" {
                    meta_pressed.set(event_type == "keydown");
                }

                if event_type == "keydown" {
                    if !is_arrow_key {
                        nudge_batch_active.set(false);
                    }
                    match key {
                        "Delete" | "Backspace" => {
                            let node_ids: Vec<String> = {
                                let doc = doc_signal.read();
                                selected_node_ids(&doc)
                                    .into_iter()
                                    .map(|id| id.to_string())
                                    .collect()
                            };

                            let dispatch_result = dispatch_node_delete_batch(&db_tx, &node_ids);

                            match dispatch_result {
                                Ok(_) => apply_clear_selection(doc_signal),
                                Err(_) => {
                                    let _ = apply_delete_selected(doc_signal, history_signal);
                                }
                            }
                        }
                        "Escape" => {
                            if editing_node.read().is_some() || editing_edge.read().is_some() {
                                editing_node.set(None);
                                editing_edge.set(None);
                                edit_value.set(String::new());
                                apply_clear_selection(doc_signal);
                            } else {
                                let mode = interaction_mode.read().clone();
                                match mode {
                                    InteractionMode::DraggingSelection { .. }
                                    | InteractionMode::ResizingSelection { .. } => {
                                        let db_tx = db_tx;
                                        let mut doc_clone = doc_signal.read().clone();
                                        interaction_mode.with_mut(|mode_mut| {
                                            let did_change = finalize_motion_release(
                                                mode_mut,
                                                &mut doc_clone,
                                                &db_tx,
                                            );
                                            if did_change {
                                                doc_signal.set(doc_clone);
                                            }
                                        });
                                    }
                                    InteractionMode::Select => {
                                        apply_clear_selection(doc_signal);
                                    }
                                    _ => {
                                        interaction_mode.set(InteractionMode::Select);
                                    }
                                }
                            }
                        }
                        "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" if !modifier => {
                            let step = if shift { 10.0 } else { 1.0 };
                            let (dx, dy) = match key {
                                "ArrowUp" => (0.0, -step),
                                "ArrowDown" => (0.0, step),
                                "ArrowLeft" => (-step, 0.0),
                                _ => (step, 0.0),
                            };
                            let push_undo = !*nudge_batch_active.read();
                            let nudged = apply_nudge_selection(
                                doc_signal,
                                history_signal,
                                dx,
                                dy,
                                push_undo,
                            );
                            if nudged {
                                nudge_batch_active.set(true);
                            }
                        }
                        "+" | "=" if !modifier => {
                            let viewport_size_now = *viewport_size.read();
                            let _ = apply_zoom_in(doc_signal, history_signal, viewport_size_now);
                        }
                        "-" | "_" if !modifier => {
                            let viewport_size_now = *viewport_size.read();
                            let _ = apply_zoom_out(doc_signal, history_signal, viewport_size_now);
                        }
                        "0" if !modifier => {
                            let viewport_size_now = *viewport_size.read();
                            let _ = apply_zoom_reset(doc_signal, history_signal, viewport_size_now);
                        }
                        "v" | "V" if !modifier => tool_signal.set(ToolMode::Select),
                        "h" | "H" if !modifier => tool_signal.set(ToolMode::Pan),
                        "l" | "L" if !modifier => tool_signal.set(ToolMode::Edge),
                        "r" | "R" if !modifier => tool_signal.set(ToolMode::Subgraph),
                        "t" | "T" if !modifier => tool_signal.set(ToolMode::Text),
                        _ => {}
                    }
                }
            }
        });
    });
}
