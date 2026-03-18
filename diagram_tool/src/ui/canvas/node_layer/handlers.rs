use canvas_domain::interaction_reducer::{finalize_motion_release, InteractionMode};
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{DiagramDocument, EdgeId, NodeId, OrderedFloat};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use im::HashMap;
use uuid::Uuid;

use crate::history::History;
use crate::ui::canvas::document_ops::{
    edge_preserves_dag, flush_pending_pointer_update, sync_canvas_origin,
};
use crate::ui::editor::ToolMode;
use crate::ui::interaction::{
    drag_original_positions, select_single, toggle_selection, with_auto_selected_edges,
};

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
    mut space_pan_active: Signal<bool>,
    space_pressed: bool,
) {
    if multi_touch_active {
        return;
    }
    evt.stop_propagation();
    let is_middle = evt.data.trigger_button() == Some(MouseButton::Auxiliary);
    let is_right = evt.data.trigger_button() == Some(MouseButton::Secondary);
    let is_primary = evt.data.trigger_button() == Some(MouseButton::Primary);
    let coords = evt.data.coordinates().client();
    let origin = sync_canvas_origin().unwrap_or(canvas_origin);
    let local_x = coords.x - origin.0;
    let local_y = coords.y - origin.1;
    let pos = to_canvas_coords(
        canvas_domain::ScreenCoord(local_x, local_y),
        canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
        doc.editor_state.zoom.0,
    );

    if space_pressed || is_middle || is_right || tool == ToolMode::Pan {
        space_pan_active.set(space_pressed && !is_middle && !is_right && tool != ToolMode::Pan);
        interaction_mode.set(InteractionMode::Panning {
            last_pos: (local_x, local_y),
        });
        return;
    }

    if !is_primary {
        return;
    }

    if tool == ToolMode::Edge {
        let mode_now = interaction_mode.read().clone();
        if !matches!(mode_now, InteractionMode::DrawingEdge { .. }) {
            interaction_mode.set(InteractionMode::DrawingEdge {
                from_node: id.clone(),
                current_pos: (pos.0, pos.1),
            });
        }
    } else {
        let was_selected = doc.editor_state.selected_items.contains(id.as_str());

        doc_signal.with_mut(|d| {
            let selected = if additive {
                toggle_selection(&d.editor_state.selected_items, &id.to_string())
            } else if !was_selected {
                select_single(id.to_string())
            } else {
                d.editor_state.selected_items.clone()
            };
            d.editor_state.selected_items = with_auto_selected_edges(d, &selected);
        });

        let current_doc = doc_signal.read().clone();
        let original_positions =
            drag_original_positions(&current_doc, &current_doc.editor_state.selected_items);
        interaction_mode.set(InteractionMode::DraggingSelection {
            anchor_canvas: (pos.0, pos.1),
            anchor_client: (local_x, local_y),
            original_positions,
            did_move: false,
        });
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
    edge_style_default: diagram_models::document::EdgeStyle,
    arrow_type_default: diagram_models::document::ArrowType,
    canvas_origin: (f64, f64),
    toast: crate::ui::toast::ToastApi,
) {
    evt.stop_propagation();
    flush_pending_pointer_update(
        doc_signal,
        history_signal,
        interaction_mode,
        pending_pointer_sample,
        db_tx,
    );
    let mode = interaction_mode.read().clone();
    match mode {
        InteractionMode::DrawingEdge { from_node, .. } => {
            if from_node != id {
                let doc_now = doc_signal.read().clone();
                let candidate_edge = diagram_models::document::Edge {
                    source: from_node,
                    target: id.clone(),
                    label: String::new(),
                    style: edge_style_default,
                    arrow_type: arrow_type_default,
                    label_offset_t: OrderedFloat(0.5),
                    color: None,
                    thickness: OrderedFloat(1.5),
                    directed: true,
                    bend_points: im::Vector::new(),
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    font_size: None,
                    source_port: None,
                    target_port: None,
                };

                if edge_preserves_dag(&doc_now, &candidate_edge) {
                    let history = history_signal.read().clone();
                    *history_signal.write() = history.push(doc_now);
                    doc_signal.with_mut(|doc| {
                        doc.document.edges = doc
                            .document
                            .edges
                            .update(EdgeId::new(Uuid::new_v4().to_string()), candidate_edge);
                        doc.revision = doc.revision.increment();
                    });
                } else {
                    let _ = toast.show(
                        crate::ui::toast::ToastIntent::Warning,
                        "Cannot create circular connection",
                        None,
                    );
                }
            }
            if *tool_signal.read() == ToolMode::Edge {
                let doc_now = doc_signal.read().clone();
                let coords = evt.data.coordinates().client();
                let origin = sync_canvas_origin().unwrap_or(canvas_origin);
                let local_x = coords.x - origin.0;
                let local_y = coords.y - origin.1;
                let pos = to_canvas_coords(
                    canvas_domain::ScreenCoord(local_x, local_y),
                    canvas_domain::CanvasCoord(
                        doc_now.editor_state.camera_x.0,
                        doc_now.editor_state.camera_y.0,
                    ),
                    doc_now.editor_state.zoom.0,
                );
                interaction_mode.set(InteractionMode::DrawingEdge {
                    from_node: id.clone(),
                    current_pos: (pos.0, pos.1),
                });
            } else {
                interaction_mode.set(InteractionMode::Select);
            }
        }
        InteractionMode::DraggingSelection { .. } | InteractionMode::ResizingSelection { .. } => {
            let mut doc_clone = doc_signal.read().clone();
            interaction_mode.with_mut(|mode_mut| {
                let did_change = finalize_motion_release(mode_mut, &mut doc_clone, &db_tx);
                if did_change {
                    doc_signal.set(doc_clone);
                }
            });
        }
        _ => {}
    }

    if *tool_signal.read() != ToolMode::Edge {
        tool_signal.set(ToolMode::Select);
    }
}
