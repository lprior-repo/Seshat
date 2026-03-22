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

#[cfg(test)]
mod tests {
    use super::*;
    use diagram_models::document::OrderedFloat;
    use diagram_models::document::{ArrowType, Edge, EdgeStyle, LockState, Node, NodeId, NodeKind};

    fn create_test_node(id: &str) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn create_test_edge(source: NodeId, target: NodeId) -> Edge {
        Edge {
            source,
            target,
            label: String::new(),
            style: EdgeStyle::Solid,
            arrow_type: ArrowType::Default,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    fn create_test_doc() -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        let node1 = NodeId::new("node1".to_string());
        let node2 = NodeId::new("node2".to_string());

        let n1 = create_test_node("Node 1");
        let n2 = create_test_node("Node 2");

        doc.document.nodes.insert(node1.clone(), n1);
        doc.document.nodes.insert(node2.clone(), n2);
        doc
    }

    #[test]
    fn test_node_selected_additive() {
        let mut doc = create_test_doc();
        doc.editor_state.selected_items.insert("node1".to_string());

        let state = CanvasState {
            document: doc,
            interaction_mode: InteractionMode::Select,
        };

        let result = apply_event(
            state,
            CanvasEvent::NodeSelected {
                id: NodeId::new("node2".to_string()),
                additive: true,
                canvas_pos: CanvasCoord(0.0, 0.0),
                client_pos: ScreenCoord(0.0, 0.0),
            },
        )
        .unwrap();

        assert!(result
            .document
            .editor_state
            .selected_items
            .contains("node1"));
        assert!(result
            .document
            .editor_state
            .selected_items
            .contains("node2"));
        assert!(matches!(
            result.interaction_mode,
            InteractionMode::DraggingSelection { .. }
        ));
    }

    #[test]
    fn test_edge_drawing_started() {
        let state = CanvasState {
            document: create_test_doc(),
            interaction_mode: InteractionMode::Select,
        };

        let result = apply_event(
            state,
            CanvasEvent::EdgeDrawingStarted {
                from_node: NodeId::new("node1".to_string()),
                current_pos: CanvasCoord(10.0, 10.0),
            },
        )
        .unwrap();

        match result.interaction_mode {
            InteractionMode::DrawingEdge {
                from_node,
                current_pos,
            } => {
                assert_eq!(from_node.as_str(), "node1");
                assert_eq!(current_pos, (10.0, 10.0));
            }
            _ => panic!("Expected DrawingEdge mode"),
        }
    }

    #[test]
    fn test_pan_started() {
        let state = CanvasState {
            document: create_test_doc(),
            interaction_mode: InteractionMode::Select,
        };

        let result = apply_event(
            state,
            CanvasEvent::PanStarted {
                last_pos: ScreenCoord(100.0, 200.0),
            },
        )
        .unwrap();

        match result.interaction_mode {
            InteractionMode::Panning { last_pos } => {
                assert_eq!(last_pos, (100.0, 200.0));
            }
            _ => panic!("Expected Panning mode"),
        }
    }

    #[test]
    fn test_edge_drawing_finished() {
        let state = CanvasState {
            document: create_test_doc(),
            interaction_mode: InteractionMode::DrawingEdge {
                from_node: NodeId::new("node1".to_string()),
                current_pos: (0.0, 0.0),
            },
        };

        let result = apply_event(
            state,
            CanvasEvent::EdgeDrawingFinished {
                from_node: NodeId::new("node1".to_string()),
                to_node: NodeId::new("node2".to_string()),
                current_pos: CanvasCoord(200.0, 200.0),
                continue_drawing: false,
                edge_style: EdgeStyle::Solid,
                arrow_type: ArrowType::Default,
            },
        )
        .unwrap();

        assert_eq!(result.document.document.edges.len(), 1);
        assert!(matches!(result.interaction_mode, InteractionMode::Select));
        let edge = result.document.document.edges.values().next().unwrap();
        assert_eq!(edge.source.as_str(), "node1");
        assert_eq!(edge.target.as_str(), "node2");
    }

    #[test]
    fn test_edge_drawing_finished_circular_rejected() {
        let mut doc = create_test_doc();
        let edge = create_test_edge(
            NodeId::new("node2".to_string()),
            NodeId::new("node1".to_string()),
        );
        doc.document.edges.insert(
            diagram_models::document::EdgeId::new("edge1".to_string()),
            edge,
        );

        let state = CanvasState {
            document: doc,
            interaction_mode: InteractionMode::DrawingEdge {
                from_node: NodeId::new("node1".to_string()),
                current_pos: (0.0, 0.0),
            },
        };

        let result = apply_event(
            state,
            CanvasEvent::EdgeDrawingFinished {
                from_node: NodeId::new("node1".to_string()),
                to_node: NodeId::new("node2".to_string()),
                current_pos: CanvasCoord(200.0, 200.0),
                continue_drawing: false,
                edge_style: EdgeStyle::Solid,
                arrow_type: ArrowType::Default,
            },
        );

        assert_eq!(result.unwrap_err(), CanvasError::CircularConnectionRejected);
    }
}
