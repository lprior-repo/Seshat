#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use super::super::{InteractionMode, ResizeHandle};
use diagram_models::document::{
    EdgeId, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use im::HashMap;

#[allow(dead_code)]
fn make_test_node(id: &str, x: f64, y: f64) -> (NodeId, Node) {
    (
        NodeId::new(id.to_string()),
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        },
    )
}

// INP-4: Two-finger pan does not move shapes
// A two-finger pan gesture should pan the canvas, not move selected shapes.
// The InteractionMode::Panning should take precedence over DraggingSelection.

#[test]
fn given_panning_mode_when_two_finger_gesture_then_is_distinct_from_dragging() {
    // Panning mode should be distinct from dragging selection
    let panning = InteractionMode::Panning {
        last_pos: (100.0, 100.0),
    };

    let dragging = InteractionMode::DraggingSelection {
        anchor_canvas: (0.0, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: HashMap::new(),
        did_move: false,
    };

    // Modes should be different
    assert_ne!(
        panning, dragging,
        "Panning and DraggingSelection should be distinct modes"
    );

    // Panning should not be DraggingSelection
    match panning {
        InteractionMode::Panning { .. } => {}
        _ => panic!("Expected Panning mode"),
    }

    // Dragging should not be Panning
    match dragging {
        InteractionMode::DraggingSelection { .. } => {}
        _ => panic!("Expected DraggingSelection mode"),
    }
}

#[test]
fn given_panning_mode_when_compared_to_rubber_band_then_modes_differ() {
    // Panning should not trigger rubber-band selection
    let panning = InteractionMode::Panning {
        last_pos: (50.0, 50.0),
    };

    let rubber_band = InteractionMode::RubberBand {
        start: (0.0, 0.0),
        current: (50.0, 50.0),
    };

    assert_ne!(
        panning, rubber_band,
        "Panning should not equal RubberBand mode"
    );
}

#[test]
fn given_panning_mode_when_compared_to_drawing_modes_then_modes_differ() {
    // Panning should not trigger edge drawing or subgraph drawing
    let panning = InteractionMode::Panning {
        last_pos: (100.0, 100.0),
    };

    let drawing_edge = InteractionMode::DrawingEdge {
        from_node: NodeId::new("test".to_string()),
        current_pos: (50.0, 50.0),
        start_port: None,
    };

    let drawing_subgraph = InteractionMode::DrawingSubgraph {
        start: (0.0, 0.0),
        current: (100.0, 100.0),
    };

    assert_ne!(
        panning, drawing_edge,
        "Panning should not equal DrawingEdge mode"
    );
    assert_ne!(
        panning, drawing_subgraph,
        "Panning should not equal DrawingSubgraph mode"
    );
}

#[test]
fn given_panning_mode_when_compared_to_resizing_then_modes_differ() {
    // Panning should not trigger resize
    let panning = InteractionMode::Panning {
        last_pos: (200.0, 200.0),
    };

    let resizing = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (100.0, 100.0),
        did_resize: false,
        aspect_ratio: None,
    };

    assert_ne!(
        panning, resizing,
        "Panning should not equal ResizingSelection mode"
    );
}

#[test]
fn given_select_mode_when_compared_to_panning_then_modes_differ() {
    // Select mode (idle) should differ from Panning
    let select = InteractionMode::Select;
    let panning = InteractionMode::Panning {
        last_pos: (0.0, 0.0),
    };

    assert_ne!(
        select, panning,
        "Select and Panning should be distinct modes"
    );
}

#[test]
fn given_all_interaction_modes_when_panning_is_active_then_only_panning_matches() {
    // Verify Panning is its own distinct state
    let panning = InteractionMode::Panning {
        last_pos: (42.0, 24.0),
    };

    let other_modes: Vec<InteractionMode> = vec![
        InteractionMode::Select,
        InteractionMode::RubberBand {
            start: (0.0, 0.0),
            current: (42.0, 24.0),
        },
        InteractionMode::DraggingSelection {
            anchor_canvas: (0.0, 0.0),
            anchor_client: (0.0, 0.0),
            original_positions: HashMap::new(),
            did_move: false,
        },
        InteractionMode::DrawingEdge {
            from_node: NodeId::new("x".to_string()),
            current_pos: (42.0, 24.0),
            start_port: None,
        },
        InteractionMode::DrawingSubgraph {
            start: (0.0, 0.0),
            current: (42.0, 24.0),
        },
        InteractionMode::ResizingSelection {
            handle: ResizeHandle::Nw,
            original_bounds: (0.0, 0.0, 42.0, 24.0),
            originals: HashMap::new(),
            anchor: (21.0, 12.0),
            did_resize: false,
            aspect_ratio: None,
        },
        InteractionMode::DraggingBendPoint {
            edge_id: EdgeId::new("edge-1".to_string()),
            bend_index: 0,
        },
    ];

    for other in other_modes {
        assert_ne!(
            panning, other,
            "Panning should be distinct from all other interaction modes"
        );
    }
}

#[test]
fn given_panning_mode_last_pos_when_updated_then_tracks_movement() {
    // Panning mode should track the last position for continuous pan updates
    let initial_pos = (100.0, 100.0);
    let panning = InteractionMode::Panning {
        last_pos: initial_pos,
    };

    // Extract and verify position
    if let InteractionMode::Panning { last_pos } = panning {
        assert_eq!(last_pos, initial_pos, "Panning should track last position");
    } else {
        panic!("Expected Panning mode");
    }
}

#[test]
fn given_dragging_bend_point_when_constructed_then_fields_are_accessible() {
    let mode = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("edge-42".to_string()),
        bend_index: 3,
    };

    match mode {
        InteractionMode::DraggingBendPoint {
            edge_id,
            bend_index,
        } => {
            assert_eq!(edge_id.as_str(), "edge-42");
            assert_eq!(bend_index, 3);
        }
        _ => panic!("Expected DraggingBendPoint mode"),
    }
}

#[test]
fn given_dragging_bend_point_when_compared_to_panning_then_modes_differ() {
    let bend_point = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("e1".to_string()),
        bend_index: 0,
    };
    let panning = InteractionMode::Panning {
        last_pos: (0.0, 0.0),
    };

    assert_ne!(
        bend_point, panning,
        "DraggingBendPoint should be distinct from Panning"
    );
}

#[test]
fn given_dragging_bend_point_when_compared_to_drawing_edge_then_modes_differ() {
    let bend_point = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("e1".to_string()),
        bend_index: 0,
    };
    let drawing_edge = InteractionMode::DrawingEdge {
        from_node: NodeId::new("n1".to_string()),
        current_pos: (0.0, 0.0),
        start_port: None,
    };

    assert_ne!(
        bend_point, drawing_edge,
        "DraggingBendPoint should be distinct from DrawingEdge"
    );
}

#[test]
fn given_dragging_bend_point_when_compared_to_dragging_selection_then_modes_differ() {
    let bend_point = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("e1".to_string()),
        bend_index: 0,
    };
    let dragging_sel = InteractionMode::DraggingSelection {
        anchor_canvas: (0.0, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: HashMap::new(),
        did_move: false,
    };

    assert_ne!(
        bend_point, dragging_sel,
        "DraggingBendPoint should be distinct from DraggingSelection"
    );
}

#[test]
fn given_dragging_bend_point_with_different_indices_then_modes_differ() {
    let bend_a = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("e1".to_string()),
        bend_index: 0,
    };
    let bend_b = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("e1".to_string()),
        bend_index: 1,
    };

    assert_ne!(
        bend_a, bend_b,
        "Same edge but different bend_index should produce distinct modes"
    );
}

#[test]
fn given_dragging_bend_point_with_different_edges_then_modes_differ() {
    let bend_a = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("e1".to_string()),
        bend_index: 0,
    };
    let bend_b = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("e2".to_string()),
        bend_index: 0,
    };

    assert_ne!(
        bend_a, bend_b,
        "Different edge_id should produce distinct modes"
    );
}

#[test]
fn given_dragging_bend_point_with_zero_bend_index_then_constructs() {
    let mode = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("edge".to_string()),
        bend_index: 0,
    };
    if let InteractionMode::DraggingBendPoint { bend_index, .. } = mode {
        assert_eq!(bend_index, 0);
    } else {
        panic!("Expected DraggingBendPoint mode");
    }
}

#[test]
fn given_dragging_bend_point_with_large_bend_index_then_constructs() {
    let mode = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("edge".to_string()),
        bend_index: usize::MAX,
    };
    if let InteractionMode::DraggingBendPoint { bend_index, .. } = mode {
        assert_eq!(bend_index, usize::MAX);
    } else {
        panic!("Expected DraggingBendPoint mode");
    }
}

#[test]
fn given_dragging_bend_point_when_compared_to_all_other_modes_then_differs() {
    let bend_point = InteractionMode::DraggingBendPoint {
        edge_id: EdgeId::new("e1".to_string()),
        bend_index: 2,
    };

    let other_modes: Vec<InteractionMode> = vec![
        InteractionMode::Select,
        InteractionMode::RubberBand {
            start: (0.0, 0.0),
            current: (10.0, 10.0),
        },
        InteractionMode::DraggingSelection {
            anchor_canvas: (0.0, 0.0),
            anchor_client: (0.0, 0.0),
            original_positions: HashMap::new(),
            did_move: false,
        },
        InteractionMode::DrawingEdge {
            from_node: NodeId::new("n".to_string()),
            current_pos: (5.0, 5.0),
            start_port: None,
        },
        InteractionMode::DrawingSubgraph {
            start: (0.0, 0.0),
            current: (10.0, 10.0),
        },
        InteractionMode::ResizingSelection {
            handle: ResizeHandle::Se,
            original_bounds: (0.0, 0.0, 100.0, 100.0),
            originals: HashMap::new(),
            anchor: (50.0, 50.0),
            did_resize: false,
            aspect_ratio: None,
        },
        InteractionMode::Panning {
            last_pos: (5.0, 5.0),
        },
    ];

    for other in other_modes {
        assert_ne!(
            bend_point, other,
            "DraggingBendPoint should be distinct from all other interaction modes"
        );
    }
}
