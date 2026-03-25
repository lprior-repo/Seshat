use canvas_domain::interaction_reducer::InteractionMode;
use canvas_domain::{CanvasCoord, ScreenCoord};
use diagram_models::document::{
    ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, NodeId, OrderedFloat,
};
use im::HashMap;
use uuid::Uuid;

use crate::ui::canvas::document_ops::edge_preserves_dag;
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
        start_port: Option<diagram_models::port::PortAnchor>,
    },
    EdgeDrawingFinished {
        from_node: NodeId,
        to_node: NodeId,
        current_pos: CanvasCoord,
        continue_drawing: bool,
        edge_style: EdgeStyle,
        arrow_type: ArrowType,
        start_port: Option<diagram_models::port::PortAnchor>,
        end_port: Option<diagram_models::port::PortAnchor>,
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

#[derive(Debug)]
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
            start_port,
        } => {
            if !matches!(state.interaction_mode, InteractionMode::DrawingEdge { .. }) {
                state.interaction_mode = InteractionMode::DrawingEdge {
                    from_node,
                    current_pos: (current_pos.0, current_pos.1),
                    start_port,
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
            start_port,
            end_port,
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
                    source_port: start_port,
                    target_port: end_port,
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
                    start_port: end_port,
                };
            } else {
                state.interaction_mode = InteractionMode::Select;
            }
            Ok(state)
        }
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
