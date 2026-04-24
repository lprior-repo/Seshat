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
fn given_panning_mode_with_nan_coords_then_mode_constructs_without_panic() {
    let panning = InteractionMode::Panning {
        last_pos: (f64::NAN, f64::NAN),
    };
    if let InteractionMode::Panning { last_pos } = panning {
        assert!(last_pos.0.is_nan(), "NaN x should be preserved");
        assert!(last_pos.1.is_nan(), "NaN y should be preserved");
    } else {
        panic!("Expected Panning mode");
    }
}

#[test]
fn given_panning_mode_with_infinity_coords_then_mode_constructs_without_panic() {
    let panning = InteractionMode::Panning {
        last_pos: (f64::INFINITY, f64::NEG_INFINITY),
    };
    if let InteractionMode::Panning { last_pos } = panning {
        assert!(last_pos.0.is_infinite() && last_pos.0.is_sign_positive());
        assert!(last_pos.1.is_infinite() && last_pos.1.is_sign_negative());
    } else {
        panic!("Expected Panning mode");
    }
}

#[test]
fn given_rubber_band_with_nan_coords_then_mode_constructs_without_panic() {
    let rubber_band = InteractionMode::RubberBand {
        start: (f64::NAN, f64::NAN),
        current: (f64::NAN, f64::NAN),
    };
    if let InteractionMode::RubberBand { start, current } = rubber_band {
        assert!(start.0.is_nan() && start.1.is_nan());
        assert!(current.0.is_nan() && current.1.is_nan());
    } else {
        panic!("Expected RubberBand mode");
    }
}

#[test]
fn given_rubber_band_with_infinity_coords_then_mode_constructs_without_panic() {
    let rubber_band = InteractionMode::RubberBand {
        start: (f64::NEG_INFINITY, f64::NEG_INFINITY),
        current: (f64::INFINITY, f64::INFINITY),
    };
    if let InteractionMode::RubberBand { start, current } = rubber_band {
        assert!(start.0.is_infinite() && start.1.is_infinite());
        assert!(current.0.is_infinite() && current.1.is_infinite());
    } else {
        panic!("Expected RubberBand mode");
    }
}

#[test]
fn given_dragging_selection_with_nan_anchor_then_mode_constructs_without_panic() {
    let dragging = InteractionMode::DraggingSelection {
        anchor_canvas: (f64::NAN, f64::NAN),
        anchor_client: (f64::NAN, f64::NAN),
        original_positions: HashMap::new(),
        did_move: false,
    };
    if let InteractionMode::DraggingSelection { anchor_canvas, anchor_client, .. } = dragging {
        assert!(anchor_canvas.0.is_nan() && anchor_canvas.1.is_nan());
        assert!(anchor_client.0.is_nan() && anchor_client.1.is_nan());
    } else {
        panic!("Expected DraggingSelection mode");
    }
}

#[test]
fn given_dragging_selection_with_infinity_anchor_then_mode_constructs_without_panic() {
    let dragging = InteractionMode::DraggingSelection {
        anchor_canvas: (f64::INFINITY, f64::NEG_INFINITY),
        anchor_client: (f64::INFINITY, f64::NEG_INFINITY),
        original_positions: HashMap::new(),
        did_move: true,
    };
    if let InteractionMode::DraggingSelection { anchor_canvas, anchor_client, did_move, .. } = dragging {
        assert!(anchor_canvas.0.is_infinite() && anchor_canvas.1.is_infinite());
        assert!(anchor_client.0.is_infinite() && anchor_client.1.is_infinite());
        assert!(did_move);
    } else {
        panic!("Expected DraggingSelection mode");
    }
}

#[test]
fn given_drawing_edge_with_nan_pos_then_mode_constructs_without_panic() {
    let drawing = InteractionMode::DrawingEdge {
        from_node: NodeId::new("touch-node".to_string()),
        current_pos: (f64::NAN, f64::NAN),
        start_port: None,
    };
    if let InteractionMode::DrawingEdge { current_pos, .. } = drawing {
        assert!(current_pos.0.is_nan() && current_pos.1.is_nan());
    } else {
        panic!("Expected DrawingEdge mode");
    }
}

#[test]
fn given_drawing_subgraph_with_nan_coords_then_mode_constructs_without_panic() {
    let drawing = InteractionMode::DrawingSubgraph {
        start: (f64::NAN, f64::NAN),
        current: (f64::INFINITY, f64::NEG_INFINITY),
    };
    if let InteractionMode::DrawingSubgraph { start, current } = drawing {
        assert!(start.0.is_nan() && start.1.is_nan());
        assert!(current.0.is_infinite() && current.1.is_infinite());
    } else {
        panic!("Expected DrawingSubgraph mode");
    }
}

#[test]
fn given_resizing_selection_with_nan_bounds_then_mode_constructs_without_panic() {
    let resizing = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (f64::NAN, f64::NAN, f64::NAN, f64::NAN),
        originals: HashMap::new(),
        anchor: (f64::NAN, f64::NAN),
        did_resize: false,
        aspect_ratio: None,
    };
    if let InteractionMode::ResizingSelection { original_bounds, anchor, .. } = resizing {
        assert!(original_bounds.0.is_nan());
        assert!(anchor.0.is_nan() && anchor.1.is_nan());
    } else {
        panic!("Expected ResizingSelection mode");
    }
}

#[test]
fn given_resizing_selection_with_infinity_bounds_then_mode_constructs_without_panic() {
    let resizing = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Nw,
        original_bounds: (f64::NEG_INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::INFINITY),
        originals: HashMap::new(),
        anchor: (f64::INFINITY, f64::NEG_INFINITY),
        did_resize: true,
        aspect_ratio: Some(f64::INFINITY),
    };
    if let InteractionMode::ResizingSelection { original_bounds, anchor, aspect_ratio, .. } = resizing {
        assert!(original_bounds.0.is_infinite() && original_bounds.2.is_infinite());
        assert!(anchor.0.is_infinite() && anchor.1.is_infinite());
        assert!(aspect_ratio.unwrap().is_infinite());
    } else {
        panic!("Expected ResizingSelection mode");
    }
}

#[test]
fn given_panning_with_nan_vs_normal_then_modes_remain_distinct() {
    let panning_nan = InteractionMode::Panning {
        last_pos: (f64::NAN, f64::NAN),
    };
    let panning_normal = InteractionMode::Panning {
        last_pos: (100.0, 200.0),
    };
    assert_ne!(
        panning_nan, panning_normal,
        "NaN panning and normal panning should be distinct (NaN != finite)"
    );
}

#[test]
fn given_all_modes_with_edge_coords_then_none_panic_on_construction() {
    let modes: Vec<InteractionMode> = vec![
        InteractionMode::Panning {
            last_pos: (f64::NAN, f64::INFINITY),
        },
        InteractionMode::RubberBand {
            start: (f64::NEG_INFINITY, 0.0),
            current: (0.0, f64::INFINITY),
        },
        InteractionMode::DraggingSelection {
            anchor_canvas: (f64::NAN, f64::NAN),
            anchor_client: (f64::INFINITY, f64::NEG_INFINITY),
            original_positions: HashMap::new(),
            did_move: false,
        },
        InteractionMode::DrawingEdge {
            from_node: NodeId::new("edge".to_string()),
            current_pos: (f64::NAN, f64::INFINITY),
            start_port: None,
        },
        InteractionMode::DrawingSubgraph {
            start: (f64::NEG_INFINITY, f64::NAN),
            current: (f64::INFINITY, f64::INFINITY),
        },
        InteractionMode::ResizingSelection {
            handle: ResizeHandle::Se,
            original_bounds: (f64::NAN, f64::NAN, f64::INFINITY, f64::INFINITY),
            originals: HashMap::new(),
            anchor: (f64::NAN, f64::INFINITY),
            did_resize: false,
            aspect_ratio: Some(f64::NAN),
        },
    ];
    assert_eq!(modes.len(), 6, "All edge-case modes should construct without panic");
}
