use canvas_domain::interaction_reducer::{finalize_motion_release, InteractionMode};
use canvas_domain::perf::to_canvas_coords;
use canvas_domain::{CanvasCoord, ScreenCoord};
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle, NodeId};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::history::History;
use crate::ui::canvas::document_ops::{
    dispatch_drag_move_batch, flush_pending_pointer_update, snap_edge_port_toward,
    snapped_edge_ports, sync_canvas_origin,
};
use crate::ui::editor::ToolMode;
use crate::ui::interaction::{
    drag_original_positions, select_single, toggle_selection, with_auto_selected_edges,
};

use super::state::{apply_event, CanvasError, CanvasEvent, CanvasState};

/// Fast path for node selection that avoids cloning the entire document.
/// Only modifies `selected_items` and `interaction_mode` in place.
fn handle_node_selected_fast(
    id: NodeId,
    additive: bool,
    canvas_pos: CanvasCoord,
    client_pos: ScreenCoord,
    mut doc_signal: Signal<DiagramDocument>,
    mut interaction_mode: Signal<InteractionMode>,
) {
    // Phase 1: Read only what we need — borrow, not clone
    let (auto_selected, original_positions) = {
        let doc = doc_signal.read();
        let was_selected = doc.editor_state.selected_items.contains(id.as_str());
        let new_selected = if additive {
            toggle_selection(&doc.editor_state.selected_items, &id.to_string())
        } else if !was_selected {
            select_single(id.to_string())
        } else {
            doc.editor_state.selected_items.clone()
        };
        let auto_selected = with_auto_selected_edges(&doc, &new_selected);
        let original_positions = drag_original_positions(&doc, &auto_selected);
        (auto_selected, original_positions)
    };
    // Read lock is dropped here

    // Phase 2: Write only selected_items — this triggers reactive update
    // but does NOT clone the full document (nodes, edges stay in place)
    doc_signal.write().editor_state.selected_items = auto_selected;

    interaction_mode.set(InteractionMode::DraggingSelection {
        anchor_canvas: (canvas_pos.0, canvas_pos.1),
        anchor_client: (client_pos.0, client_pos.1),
        original_positions,
        did_move: false,
    });
}

#[allow(clippy::too_many_arguments)]
pub fn handle_mousedown(
    evt: Event<dioxus::prelude::MouseData>,
    id: NodeId,
    multi_touch_active: bool,
    tool: ToolMode,
    camera: (f64, f64, f64),
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
        CanvasCoord(camera.0, camera.1),
        camera.2,
    );

    if is_primary && tool == ToolMode::Edge {
        // Edge drawing — still uses full clone (uncommon path)
        let event = {
            let doc_now = doc_signal.read();
            let start_port = doc_now
                .document
                .nodes
                .get(&id)
                .map(|src| snap_edge_port_toward(src, pos.0, pos.1));
            CanvasEvent::EdgeDrawingStarted {
                from_node: id,
                current_pos: CanvasCoord(pos.0, pos.1),
                start_port,
            }
        };
        let initial_state = CanvasState {
            document: doc_signal.read().clone(),
            interaction_mode: interaction_mode.read().clone(),
        };
        if let Ok(new_state) = apply_event(initial_state, event) {
            doc_signal.set(new_state.document);
            interaction_mode.set(new_state.interaction_mode);
        }
    } else if is_primary {
        // Node selection — FAST PATH: avoids cloning entire document
        handle_node_selected_fast(
            id,
            additive,
            CanvasCoord(pos.0, pos.1),
            ScreenCoord(local_x, local_y),
            doc_signal,
            interaction_mode,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_mouseup(
    evt: Event<dioxus::prelude::MouseData>,
    id: NodeId,
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut interaction_mode: Signal<InteractionMode>,
    mut pending_pointer_sample: Signal<Option<(f64, f64)>>,
    mut geometry_render_tick: Signal<u64>,
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

    if matches!(mode, InteractionMode::DrawingEdge { .. })
        && evt.data.trigger_button() != Some(MouseButton::Primary)
    {
        if evt.data.trigger_button() == Some(MouseButton::Secondary) {
            evt.prevent_default();
        }
        evt.stop_propagation();
        pending_pointer_sample.set(None);
        interaction_mode.set(InteractionMode::Select);
        tool_signal.set(ToolMode::Select);
        return;
    }

    evt.stop_propagation();
    flush_pending_pointer_update(
        doc_signal,
        history_signal,
        interaction_mode,
        pending_pointer_sample,
        geometry_render_tick,
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

            let (start_port, end_port) = doc_now
                .document
                .nodes
                .get(&from_node)
                .zip(doc_now.document.nodes.get(&id))
                .map_or((start_port, None), |(source, target)| {
                    let (snapped_start, snapped_end) = snapped_edge_ports(source, target);
                    (Some(snapped_start), Some(snapped_end))
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
                let original_positions = match mode_mut {
                    InteractionMode::DraggingSelection {
                        original_positions, ..
                    } => Some(original_positions.clone()),
                    _ => None,
                };
                let did_change = finalize_motion_release(mode_mut, &mut doc_clone, &db_tx);
                if did_change {
                    if let Some(original_positions) = original_positions.as_ref() {
                        dispatch_drag_move_batch(original_positions, &doc_clone, &db_tx);
                    }
                    doc_signal.set(doc_clone);
                }
            });
            None
        }
        _ => None,
    };

    if let Some(event) = event {
        let edge_finish_attempt = matches!(&event, CanvasEvent::EdgeDrawingFinished { .. });
        let initial_state = CanvasState {
            document: doc_signal.read().clone(),
            interaction_mode: interaction_mode.read().clone(),
        };
        let previous_revision = initial_state.document.revision;
        match apply_event(initial_state, event) {
            Ok(new_state) => {
                let edge_created = new_state.document.revision != previous_revision;
                if edge_created {
                    let history = history_signal.read().clone();
                    *history_signal.write() = history.push(doc_signal.read().clone());
                    geometry_render_tick.with_mut(|tick| *tick = (*tick).saturating_add(1));
                }
                doc_signal.set(new_state.document);
                interaction_mode.set(new_state.interaction_mode);
                if edge_finish_attempt && !edge_created {
                    tool_signal.set(ToolMode::Select);
                }
            }
            Err(CanvasError::CircularConnectionRejected) => {
                interaction_mode.set(InteractionMode::Select);
                tool_signal.set(ToolMode::Select);
                let _ = toast.show(
                    crate::ui::toast::ToastIntent::Warning,
                    "Cannot create circular connection",
                    None,
                );
            }
            Err(_) => {
                if edge_finish_attempt {
                    interaction_mode.set(InteractionMode::Select);
                    tool_signal.set(ToolMode::Select);
                }
            }
        }
    }

    if *tool_signal.read() != ToolMode::Edge {
        tool_signal.set(ToolMode::Select);
    }
}

#[cfg(test)]
#[path = "handlers_tests.rs"]
mod handlers_tests;
