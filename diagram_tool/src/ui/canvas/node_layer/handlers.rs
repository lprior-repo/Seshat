use canvas_domain::interaction_reducer::{finalize_motion_release, InteractionMode};
use canvas_domain::perf::to_canvas_coords;
use canvas_domain::{CanvasCoord, ScreenCoord};
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle, NodeId};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::history::History;
use crate::ui::canvas::document_ops::{flush_pending_pointer_update, sync_canvas_origin};
use crate::ui::editor::ToolMode;

use super::state::{apply_event, CanvasError, CanvasEvent, CanvasState};

#[allow(clippy::too_many_arguments)]
pub fn handle_mousedown(
    evt: Event<dioxus::prelude::MouseData>,
    id: NodeId,
    multi_touch_active: bool,
    tool: ToolMode,
    doc: DiagramDocument,
    additive: bool,
    canvas_origin: (f64, f64),
    mut interaction_mode: Signal<InteractionMode>,
    mut doc_signal: Signal<DiagramDocument>,
    _space_pan_active: Signal<bool>,
    space_pressed: bool,
) {
    if multi_touch_active {
        return;
    }

    let is_middle = evt.data.trigger_button() == Some(MouseButton::Auxiliary);
    let is_right = evt.data.trigger_button() == Some(MouseButton::Secondary);

    if space_pressed || is_middle || is_right || tool == ToolMode::Pan {
        // Let panning events bubble up to the root container
        return;
    }

    evt.stop_propagation();
    let is_primary = evt.data.trigger_button() == Some(MouseButton::Primary);
    let coords = evt.data.coordinates().client();
    let origin = sync_canvas_origin().unwrap_or(canvas_origin);
    let local_x = coords.x - origin.0;
    let local_y = coords.y - origin.1;
    let pos = to_canvas_coords(
        ScreenCoord(local_x, local_y),
        CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
        doc.editor_state.zoom.0,
    );

    let event = if is_primary {
        if tool == ToolMode::Edge {
            let doc_now = doc_signal.read().clone();
            let start_port = doc_now.document.nodes.get(&id).and_then(|src| {
                let dx = if src.width.0 > 0.0 {
                    (pos.0 - src.x.0) / src.width.0
                } else {
                    0.5
                };
                let dy = if src.height.0 > 0.0 {
                    (pos.1 - src.y.0) / src.height.0
                } else {
                    0.5
                };
                diagram_models::port::NormalizedOffset::new(
                    diagram_models::document::OrderedFloat::new_unchecked(dx.clamp(0.0, 1.0)),
                    diagram_models::document::OrderedFloat::new_unchecked(dy.clamp(0.0, 1.0)),
                )
                .ok()
                .map(diagram_models::port::PortAnchor::Custom)
            });
            Some(CanvasEvent::EdgeDrawingStarted {
                from_node: id,
                current_pos: CanvasCoord(pos.0, pos.1),
                start_port,
            })
        } else {
            Some(CanvasEvent::NodeSelected {
                id,
                additive,
                canvas_pos: CanvasCoord(pos.0, pos.1),
                client_pos: ScreenCoord(local_x, local_y),
            })
        }
    } else {
        None
    };

    if let Some(event) = event {
        let initial_state = CanvasState {
            document: doc_signal.read().clone(),
            interaction_mode: interaction_mode.read().clone(),
        };
        if let Ok(new_state) = apply_event(initial_state, event) {
            doc_signal.set(new_state.document);
            interaction_mode.set(new_state.interaction_mode);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_mouseup(
    evt: Event<dioxus::prelude::MouseData>,
    id: NodeId,
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut interaction_mode: Signal<InteractionMode>,
    pending_pointer_sample: Signal<Option<(f64, f64)>>,
    db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
    mut tool_signal: Signal<ToolMode>,
    edge_style_default: EdgeStyle,
    arrow_type_default: ArrowType,
    canvas_origin: (f64, f64),
    toast: crate::ui::toast::ToastApi,
) {
    let mode = interaction_mode.read().clone();

    // Let panning release events bubble up to the root container
    if matches!(mode, InteractionMode::Panning { .. }) {
        return;
    }

    evt.stop_propagation();
    flush_pending_pointer_update(
        doc_signal,
        history_signal,
        interaction_mode,
        pending_pointer_sample,
        db_tx,
    );
    let mode = interaction_mode.read().clone();

    let event = match mode {
        InteractionMode::DrawingEdge {
            from_node,
            start_port,
            ..
        } => {
            let doc_now = doc_signal.read().clone();
            let coords = evt.data.coordinates().client();
            let origin = sync_canvas_origin().unwrap_or(canvas_origin);
            let local_x = coords.x - origin.0;
            let local_y = coords.y - origin.1;
            let pos = to_canvas_coords(
                ScreenCoord(local_x, local_y),
                CanvasCoord(
                    doc_now.editor_state.camera_x.0,
                    doc_now.editor_state.camera_y.0,
                ),
                doc_now.editor_state.zoom.0,
            );

            let end_port = doc_now.document.nodes.get(&id).and_then(|tgt| {
                let dx = if tgt.width.0 > 0.0 {
                    (pos.0 - tgt.x.0) / tgt.width.0
                } else {
                    0.5
                };
                let dy = if tgt.height.0 > 0.0 {
                    (pos.1 - tgt.y.0) / tgt.height.0
                } else {
                    0.5
                };
                diagram_models::port::NormalizedOffset::new(
                    diagram_models::document::OrderedFloat::new_unchecked(dx.clamp(0.0, 1.0)),
                    diagram_models::document::OrderedFloat::new_unchecked(dy.clamp(0.0, 1.0)),
                )
                .ok()
                .map(diagram_models::port::PortAnchor::Custom)
            });

            Some(CanvasEvent::EdgeDrawingFinished {
                from_node,
                to_node: id,
                current_pos: CanvasCoord(pos.0, pos.1),
                continue_drawing: *tool_signal.read() == ToolMode::Edge,
                edge_style: edge_style_default,
                arrow_type: arrow_type_default,
                start_port,
                end_port,
            })
        }
        InteractionMode::DraggingSelection { .. } | InteractionMode::ResizingSelection { .. } => {
            let mut doc_clone = doc_signal.read().clone();
            interaction_mode.with_mut(|mode_mut| {
                let did_change = finalize_motion_release(mode_mut, &mut doc_clone, &db_tx);
                if did_change {
                    doc_signal.set(doc_clone);
                }
            });
            None
        }
        _ => None,
    };

    if let Some(event) = event {
        let initial_state = CanvasState {
            document: doc_signal.read().clone(),
            interaction_mode: interaction_mode.read().clone(),
        };
        match apply_event(initial_state, event) {
            Ok(new_state) => {
                let history = history_signal.read().clone();
                *history_signal.write() = history.push(doc_signal.read().clone());
                doc_signal.set(new_state.document);
                interaction_mode.set(new_state.interaction_mode);
            }
            Err(CanvasError::CircularConnectionRejected) => {
                let _ = toast.show(
                    crate::ui::toast::ToastIntent::Warning,
                    "Cannot create circular connection",
                    None,
                );
            }
            Err(_) => {}
        }
    }

    if *tool_signal.read() != ToolMode::Edge {
        tool_signal.set(ToolMode::Select);
    }
}

#[cfg(test)]
#[path = "handlers_tests.rs"]
mod handlers_tests;
