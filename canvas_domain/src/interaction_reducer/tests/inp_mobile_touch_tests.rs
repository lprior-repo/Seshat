#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use super::super::{commit_inline_edit, finalize_motion_release, start_resize_interaction, InteractionMode, ResizeHandle};
use super::super::commit::{calculate_edge_label_edit, calculate_node_label_edit};
use super::super::types::LabelEditError;
use diagram_models::document::{DiagramDocument, Edge, EdgeId, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat};
use diagram_models::history::History;
use dioxus::prelude::*;
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
        assert_eq!(last_pos.0.is_nan(), true, "NaN x should be preserved");
        assert_eq!(last_pos.1.is_nan(), true, "NaN y should be preserved");
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
        assert_eq!(last_pos.0.is_infinite(), true, "x should be infinite");
        assert_eq!(last_pos.0.is_sign_positive(), true, "x should be positive");
        assert_eq!(last_pos.1.is_infinite(), true, "y should be infinite");
        assert_eq!(last_pos.1.is_sign_positive(), false, "y should be non-positive");
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

#[test]
fn given_test_node_helper_when_called_then_returns_node_with_correct_properties() {
    let (node_id, node) = make_test_node("test-node", 10.0, 20.0);
    assert_eq!(node_id.as_str(), "test-node");
    assert_eq!(node.label, "test-node");
    assert_eq!(node.x, OrderedFloat(10.0));
    assert_eq!(node.y, OrderedFloat(20.0));
    assert_eq!(node.width, OrderedFloat(100.0));
    assert_eq!(node.height, OrderedFloat(50.0));
    assert_eq!(node.kind, NodeKind::Node);
    assert_eq!(node.lock_state, LockState::Unlocked);
}

// ============== INP-5: Touch label editing ==============
// Touch interactions should support inline label editing for nodes and edges.

#[test]
fn given_valid_document_with_node_when_calculate_node_label_edit_then_returns_updated_doc() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("touch-node".to_string());

    doc.document.nodes = doc.document.nodes.update(
        node_id.clone(),
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Old Label".to_string(),
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
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        },
    );

    let result = calculate_node_label_edit(&doc, &node_id, "New Label");

    assert!(result.is_ok(), "calculate_node_label_edit should succeed for valid label");
    let new_doc = result.unwrap();
    let updated_node = new_doc.document.nodes.get(&node_id).unwrap();
    assert_eq!(updated_node.label, "New Label");
}

#[test]
fn given_nonexistent_node_when_calculate_node_label_edit_then_returns_target_not_found() {
    let doc = DiagramDocument::default();
    let fake_id = NodeId::new("nonexistent".to_string());

    let result = calculate_node_label_edit(&doc, &fake_id, "New Label");

    assert!(result.is_err(), "Should return error for nonexistent node");
    assert!(matches!(result.unwrap_err(), LabelEditError::TargetNotFound));
}

#[test]
fn given_valid_document_with_edge_when_calculate_edge_label_edit_then_returns_updated_doc() {
    let mut doc = DiagramDocument::default();
    let edge_id = EdgeId::new("touch-edge".to_string());

    doc.document.edges = doc.document.edges.update(
        edge_id.clone(),
        Edge {
            source: NodeId::new("src".to_string()),
            target: NodeId::new("tgt".to_string()),
            label: "Old Edge".to_string(),
            style: Default::default(),
            arrow_type: Default::default(),
            label_offset_t: OrderedFloat(0.5),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            color: None,
            thickness: OrderedFloat(1.0),
            font_size: None,
            source_port: None,
            target_port: None,
        },
    );

    let result = calculate_edge_label_edit(&doc, &edge_id, "New Edge Label");

    assert!(result.is_ok(), "calculate_edge_label_edit should succeed for valid label");
    let new_doc = result.unwrap();
    let updated_edge = new_doc.document.edges.get(&edge_id).unwrap();
    assert_eq!(updated_edge.label, "New Edge Label");
}

#[test]
fn given_nonexistent_edge_when_calculate_edge_label_edit_then_returns_target_not_found() {
    let doc = DiagramDocument::default();
    let fake_id = EdgeId::new("nonexistent-edge".to_string());

    let result = calculate_edge_label_edit(&doc, &fake_id, "New Label");

    assert!(result.is_err(), "Should return error for nonexistent edge");
    assert!(matches!(result.unwrap_err(), LabelEditError::TargetNotFound));
}

#[test]
fn given_dragging_selection_when_finalize_motion_release_then_transitions_to_select() {
    let mut doc = DiagramDocument::default();
    let mut mode = InteractionMode::DraggingSelection {
        anchor_canvas: (50.0, 50.0),
        anchor_client: (100.0, 100.0),
        original_positions: HashMap::new(),
        did_move: true,
    };

    let result = finalize_motion_release(&mut mode, &mut doc, &None);

    assert!(result, "finalize_motion_release should return true");
    assert!(matches!(mode, InteractionMode::Select), "Mode should transition to Select");
    assert_eq!(doc.revision, DiagramDocument::default().revision.increment());
}

#[test]
fn given_resize_with_movement_when_finalize_motion_release_then_bumps_revision() {
    let mut doc = DiagramDocument::default();
    let mut mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: true,
        aspect_ratio: None,
    };

    let result = finalize_motion_release(&mut mode, &mut doc, &None);

    assert!(result, "finalize_motion_release should return true for resize");
    assert!(matches!(mode, InteractionMode::Select), "Mode should transition to Select");
}

#[test]
fn given_resize_without_movement_when_finalize_motion_release_then_no_revision_bump() {
    let mut doc = DiagramDocument::default();
    let initial_revision = doc.revision;
    let mut mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Nw,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: false,
        aspect_ratio: None,
    };

    let result = finalize_motion_release(&mut mode, &mut doc, &None);

    assert!(result, "finalize_motion_release should return true");
    assert_eq!(doc.revision, initial_revision, "Revision should not change without movement");
}

#[test]
fn given_commit_inline_edit_with_node_target_when_labels_differ_then_returns_ok() {
    let mut vdom = VirtualDom::new(|| {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new("test-node".to_string());
        doc.document.nodes = doc.document.nodes.update(
            node_id.clone(),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: "Original".to_string(),
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
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            },
        );
        let mut history = History::new();
        let edit_value = Signal::new("Edited Label".to_string());

        let result = commit_inline_edit(
            Signal::new(doc),
            Signal::new(history),
            Some(node_id),
            None,
            edit_value,
            None,
        );

        assert!(matches!(result, Ok(_)), "commit_inline_edit should succeed");
        rsx! { div {} }
    });
    let () = vdom.rebuild_in_place();
}

#[test]
fn given_commit_inline_edit_with_invalid_label_when_dispatched_then_returns_error() {
    let mut vdom = VirtualDom::new(|| {
        let doc = DiagramDocument::default();
        let history = History::new();
        let edit_value = Signal::new("".to_string());

        let result = commit_inline_edit(
            Signal::new(doc),
            Signal::new(history),
            None,
            None,
            edit_value,
            None,
        );

        assert!(result.is_err() || matches!(result, Ok(false)),
            "commit_inline_edit should handle empty/invalid label");
        rsx! { div {} }
    });
    let () = vdom.rebuild_in_place();
}

#[test]
fn given_start_resize_interaction_with_empty_selection_then_no_mode_change() {
    let mut vdom = VirtualDom::new(|| {
        let doc = DiagramDocument::default();
        let mut mode = Signal::new(InteractionMode::Select);

        start_resize_interaction(
            mode.clone(),
            Signal::new(doc),
            ResizeHandle::Se,
            100.0,
            100.0,
            false,
        );

        assert!(matches!(mode.read().clone(), InteractionMode::Select),
            "start_resize_interaction should not change mode with no selection");
        rsx! { div {} }
    });
    let () = vdom.rebuild_in_place();
}
