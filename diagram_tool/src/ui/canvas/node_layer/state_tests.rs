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
fn test_node_selected_additive() -> Result<(), CanvasError> {
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
    )?;

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
    Ok(())
}

#[test]
fn test_edge_drawing_started() -> Result<(), CanvasError> {
    let state = CanvasState {
        document: create_test_doc(),
        interaction_mode: InteractionMode::Select,
    };

    let result = apply_event(
        state,
        CanvasEvent::EdgeDrawingStarted {
            from_node: NodeId::new("node1".to_string()),
            current_pos: CanvasCoord(10.0, 10.0),
            start_port: None,
        },
    )?;

    match result.interaction_mode {
        InteractionMode::DrawingEdge {
            from_node,
            current_pos,
            ..
        } => {
            assert_eq!(from_node.as_str(), "node1");
            assert_eq!(current_pos, (10.0, 10.0));
        }
        _ => assert!(false, "Expected DrawingEdge mode"),
    }
    Ok(())
}

#[test]
fn test_pan_started() -> Result<(), CanvasError> {
    let state = CanvasState {
        document: create_test_doc(),
        interaction_mode: InteractionMode::Select,
    };

    let result = apply_event(
        state,
        CanvasEvent::PanStarted {
            last_pos: ScreenCoord(100.0, 200.0),
        },
    )?;

    match result.interaction_mode {
        InteractionMode::Panning { last_pos } => {
            assert_eq!(last_pos, (100.0, 200.0));
        }
        _ => assert!(false, "Expected Panning mode"),
    }
    Ok(())
}

#[test]
fn test_edge_drawing_finished() -> Result<(), CanvasError> {
    let state = CanvasState {
        document: create_test_doc(),
        interaction_mode: InteractionMode::DrawingEdge {
            from_node: NodeId::new("node1".to_string()),
            current_pos: (0.0, 0.0),
            start_port: None,
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
            start_port: None,
            end_port: None,
        },
    )?;

    assert_eq!(result.document.document.edges.len(), 1);
    assert!(matches!(result.interaction_mode, InteractionMode::Select));
    if let Some(edge) = result.document.document.edges.values().next() {
        assert_eq!(edge.source.as_str(), "node1");
        assert_eq!(edge.target.as_str(), "node2");
    }
    Ok(())
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
            start_port: None,
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
            start_port: None,
            end_port: None,
        },
    );

    assert!(matches!(
        result,
        Err(CanvasError::CircularConnectionRejected)
    ));
}
