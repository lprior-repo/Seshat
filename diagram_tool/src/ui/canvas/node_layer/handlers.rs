use canvas_domain::interaction_reducer::{finalize_motion_release, InteractionMode};
use canvas_domain::perf::to_canvas_coords;
use canvas_domain::{CanvasCoord, ScreenCoord};
use diagram_models::document::{
    ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, NodeId, OrderedFloat,
};
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

#[derive(Clone, Debug, PartialEq)]
pub enum CanvasEvent {
    NodeSelected {
        id: NodeId,
        additive: bool,
        canvas_pos: CanvasCoord,
        client_pos: ScreenCoord,
    },
    EdgeDrawingStarted {
        from_node: NodeId,
        current_pos: CanvasCoord,
    },
    EdgeDrawingFinished {
        from_node: NodeId,
        to_node: NodeId,
        current_pos: CanvasCoord,
        continue_drawing: bool,
        edge_style: EdgeStyle,
        arrow_type: ArrowType,
    },
    PanStarted {
        last_pos: ScreenCoord,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum CanvasError {
    CircularConnectionRejected,
    InvalidStateTransition,
    NodeNotFound(NodeId),
}

pub struct CanvasState {
    pub document: DiagramDocument,
    pub interaction_mode: InteractionMode,
}

pub fn apply_event(mut state: CanvasState, event: CanvasEvent) -> Result<CanvasState, CanvasError> {
    match event {
        CanvasEvent::NodeSelected {
            id,
            additive,
            canvas_pos,
            client_pos,
        } => {
            let was_selected = state
                .document
                .editor_state
                .selected_items
                .contains(id.as_str());
            let selected = if additive {
                toggle_selection(&state.document.editor_state.selected_items, &id.to_string())
            } else if !was_selected {
                select_single(id.to_string())
            } else {
                state.document.editor_state.selected_items.clone()
            };
            state.document.editor_state.selected_items =
                with_auto_selected_edges(&state.document, &selected);
            let original_positions = drag_original_positions(
                &state.document,
                &state.document.editor_state.selected_items,
            );
            state.interaction_mode = InteractionMode::DraggingSelection {
                anchor_canvas: (canvas_pos.0, canvas_pos.1),
                anchor_client: (client_pos.0, client_pos.1),
                original_positions,
                did_move: false,
            };
            Ok(state)
        }
        CanvasEvent::EdgeDrawingStarted {
            from_node,
            current_pos,
        } => {
            if !matches!(state.interaction_mode, InteractionMode::DrawingEdge { .. }) {
                state.interaction_mode = InteractionMode::DrawingEdge {
                    from_node,
                    current_pos: (current_pos.0, current_pos.1),
                };
            }
            Ok(state)
        }
        CanvasEvent::PanStarted { last_pos } => {
            state.interaction_mode = InteractionMode::Panning {
                last_pos: (last_pos.0, last_pos.1),
            };
            Ok(state)
        }
        CanvasEvent::EdgeDrawingFinished {
            from_node,
            to_node,
            current_pos,
            continue_drawing,
            edge_style,
            arrow_type,
        } => {
            if from_node != to_node {
                let candidate_edge = Edge {
                    source: from_node,
                    target: to_node.clone(),
                    label: String::new(),
                    style: edge_style,
                    arrow_type,
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
                if edge_preserves_dag(&state.document, &candidate_edge) {
                    state.document.document.edges = state
                        .document
                        .document
                        .edges
                        .update(EdgeId::new(Uuid::new_v4().to_string()), candidate_edge);
                    state.document.revision = state.document.revision.increment();
                } else {
                    return Err(CanvasError::CircularConnectionRejected);
                }
            }
            if continue_drawing {
                state.interaction_mode = InteractionMode::DrawingEdge {
                    from_node: to_node,
                    current_pos: (current_pos.0, current_pos.1),
                };
            } else {
                state.interaction_mode = InteractionMode::Select;
            }
            Ok(state)
        }
    }
}

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
        ScreenCoord(local_x, local_y),
        CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
        doc.editor_state.zoom.0,
    );

    let event = if space_pressed || is_middle || is_right || tool == ToolMode::Pan {
        space_pan_active.set(space_pressed && !is_middle && !is_right && tool != ToolMode::Pan);
        Some(CanvasEvent::PanStarted {
            last_pos: ScreenCoord(local_x, local_y),
        })
    } else if is_primary {
        if tool == ToolMode::Edge {
            Some(CanvasEvent::EdgeDrawingStarted {
                from_node: id.clone(),
                current_pos: CanvasCoord(pos.0, pos.1),
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
        InteractionMode::DrawingEdge { from_node, .. } => {
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
            Some(CanvasEvent::EdgeDrawingFinished {
                from_node,
                to_node: id,
                current_pos: CanvasCoord(pos.0, pos.1),
                continue_drawing: *tool_signal.read() == ToolMode::Edge,
                edge_style: edge_style_default,
                arrow_type: arrow_type_default,
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
