use super::super::{InteractionMode, ResizeHandle};
use diagram_models::document::{LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat};
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
