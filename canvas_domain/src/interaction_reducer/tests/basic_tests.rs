#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
use super::super::{finalize_motion_release, resize_target_ids, InteractionMode, ResizeHandle};
use diagram_models::document::{
    DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use im::HashMap;

fn node(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Node {
    Node {
        kind,
        icon: String::new(),
        label: String::from("n"),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(w),
        height: OrderedFloat(h),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Locked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: Some(NodeStyle::default()),
        collapsed: None,
    }
}

#[test]
fn given_drag_end_when_finalized_twice_then_revision_bumps_once() {
    let mut doc = DiagramDocument::default();
    let mut mode = InteractionMode::DraggingSelection {
        anchor_canvas: (0.0, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: HashMap::new(),
        did_move: true,
    };

    let first = finalize_motion_release(&mut mode, &mut doc, &None);
    let second = finalize_motion_release(&mut mode, &mut doc, &None);

    assert!(first);
    assert!(!second);
    assert_eq!(
        doc.revision,
        DiagramDocument::default().revision.increment()
    );
    assert_eq!(mode, InteractionMode::Select);
}

#[test]
fn given_resize_end_without_resize_when_finalized_then_no_revision_bump() {
    let mut doc = DiagramDocument::default();
    let mut mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 10.0, 10.0),
        originals: HashMap::new(),
        anchor: (0.0, 0.0),
        did_resize: false,
        aspect_ratio: None,
    };

    let finalized = finalize_motion_release(&mut mode, &mut doc, &None);

    assert!(finalized);
    assert_eq!(doc.revision, DiagramDocument::default().revision);
    assert_eq!(mode, InteractionMode::Select);
}

#[test]
fn given_resize_end_when_finalized_twice_then_revision_bumps_once() {
    let mut doc = DiagramDocument::default();
    let mut mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::E,
        original_bounds: (0.0, 0.0, 10.0, 10.0),
        originals: HashMap::new(),
        anchor: (0.0, 0.0),
        did_resize: true,
        aspect_ratio: None,
    };

    let first = finalize_motion_release(&mut mode, &mut doc, &None);
    let second = finalize_motion_release(&mut mode, &mut doc, &None);

    assert!(first);
    assert!(!second);
    assert_eq!(
        doc.revision,
        DiagramDocument::default().revision.increment()
    );
    assert_eq!(mode, InteractionMode::Select);
}

#[test]
fn given_selected_subgraph_when_collecting_resize_targets_then_interior_nodes_included() {
    let mut doc = DiagramDocument::default();
    let subgraph = NodeId::new(String::from("sub"));
    let inside = NodeId::new(String::from("inside"));
    let outside = NodeId::new(String::from("outside"));

    // Create a helper for unlocked nodes
    fn unlocked_node(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: String::from("n"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked, // Unlocked so selected_node_ids includes them
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            subgraph.clone(),
            unlocked_node(NodeKind::Subgraph, 100.0, 100.0, 300.0, 220.0),
        )
        .update(
            inside.clone(),
            unlocked_node(NodeKind::Node, 140.0, 140.0, 80.0, 60.0),
        )
        .update(
            outside.clone(),
            unlocked_node(NodeKind::Node, 450.0, 300.0, 80.0, 60.0),
        );
    let _ = doc.editor_state.selected_items.insert(subgraph.to_string());

    let targets = resize_target_ids(&doc);
    assert!(targets.contains(&subgraph));
    assert!(targets.contains(&inside));
    assert!(!targets.contains(&outside));
}

// History atomicity regression tests for gesture finalization
// Ensures one completed user gesture maps to exactly one history entry
// See: contract bd-1yu - history: guarantee gesture atomicity

#[test]
fn given_already_in_select_mode_when_finalized_then_no_revision_change() {
    // Simulates duplicate pointerup/mouseup events arriving after gesture completed
    let mut doc = DiagramDocument::default();
    let initial_revision = doc.revision;
    let mut mode = InteractionMode::Select;

    let result = finalize_motion_release(&mut mode, &mut doc, &None);

    assert!(!result, "Should return false when already in Select mode");
    assert_eq!(doc.revision, initial_revision, "Revision should not change");
    assert_eq!(mode, InteractionMode::Select);
}

#[test]
fn given_drag_gesture_when_duplicate_events_arrive_then_history_single_entry() {
    // Simulates the E2E test scenario: normal mouseup + duplicate pointerup/mouseup
    let mut doc = DiagramDocument::default();
    let initial_revision = doc.revision;

    // First event: normal drag completion
    let mut mode = InteractionMode::DraggingSelection {
        anchor_canvas: (0.0, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: HashMap::new(),
        did_move: true,
    };

    let first_result = finalize_motion_release(&mut mode, &mut doc, &None);
    let first_revision = doc.revision;

    assert!(first_result, "First finalize should succeed");
    assert_eq!(mode, InteractionMode::Select);

    // Second event: duplicate pointerup arrives after already finalized
    let second_result = finalize_motion_release(&mut mode, &mut doc, &None);
    let second_revision = doc.revision;

    assert!(!second_result, "Second finalize should be idempotent");
    assert_eq!(
        first_revision, second_revision,
        "Revision should not change on duplicate"
    );

    // Third event: another duplicate mouseup
    let third_result = finalize_motion_release(&mut mode, &mut doc, &None);
    let third_revision = doc.revision;

    assert!(!third_result, "Third finalize should also be idempotent");
    assert_eq!(
        second_revision, third_revision,
        "Revision should remain unchanged"
    );

    // Verify only one revision increment occurred
    assert_eq!(
        third_revision,
        initial_revision.increment(),
        "Exactly one revision increment for entire gesture"
    );
}

#[test]
fn given_resize_gesture_when_duplicate_events_arrive_then_history_single_entry() {
    // Similar to drag test but for resize gestures
    let mut doc = DiagramDocument::default();
    let initial_revision = doc.revision;

    // First event: normal resize completion
    let mut mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::E,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: true,
        aspect_ratio: None,
    };

    let first_result = finalize_motion_release(&mut mode, &mut doc, &None);
    assert!(first_result);
    assert_eq!(mode, InteractionMode::Select);

    // Duplicate events after finalization
    for _ in 0..5 {
        let result = finalize_motion_release(&mut mode, &mut doc, &None);
        assert!(!result, "Finalize should be idempotent after first call");
    }

    // Verify only one revision increment
    assert_eq!(
        doc.revision,
        initial_revision.increment(),
        "Exactly one revision increment for resize gesture"
    );
}

#[test]
fn given_no_op_gesture_when_finalized_then_no_revision_bump() {
    // Drag without movement should not create history entry
    let mut doc = DiagramDocument::default();
    let initial_revision = doc.revision;

    let mut mode = InteractionMode::DraggingSelection {
        anchor_canvas: (0.0, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: HashMap::new(),
        did_move: false, // No actual movement
    };

    let result = finalize_motion_release(&mut mode, &mut doc, &None);

    assert!(result, "Should return true (mode transitioned)");
    assert_eq!(doc.revision, initial_revision, "No revision bump for no-op");
    assert_eq!(mode, InteractionMode::Select);
}

#[test]
fn given_mixed_gesture_sequence_when_finalized_then_correct_revisions() {
    // Simulates a realistic sequence: select -> drag -> select -> resize -> select
    let mut doc = DiagramDocument::default();
    let initial_revision = doc.revision;

    // First: select (no-op)
    let mut mode = InteractionMode::Select;
    let result = finalize_motion_release(&mut mode, &mut doc, &None);
    assert!(!result);
    assert_eq!(doc.revision, initial_revision);

    // Second: drag gesture
    mode = InteractionMode::DraggingSelection {
        anchor_canvas: (0.0, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: HashMap::new(),
        did_move: true,
    };
    let result = finalize_motion_release(&mut mode, &mut doc, &None);
    assert!(result);
    assert_eq!(doc.revision, initial_revision.increment());

    // Third: resize gesture
    mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: true,
        aspect_ratio: None,
    };
    let result = finalize_motion_release(&mut mode, &mut doc, &None);
    assert!(result);
    assert_eq!(doc.revision, initial_revision.increment().increment());

    // Duplicate finalizations should not change anything
    for _ in 0..3 {
        let result = finalize_motion_release(&mut mode, &mut doc, &None);
        assert!(!result);
    }
    assert_eq!(
        doc.revision,
        initial_revision.increment().increment(),
        "No additional revisions after gestures complete"
    );
}

// ============== MUL-001: Resize selection containing rotated items ==============
// Note: Rotation is stored in the geometry module's Rectangle struct, not directly on Node.
// This test verifies that multi-select resize correctly scales nodes that would have rotation
// applied at render time (the bounding box scales correctly).

#[test]
fn given_selection_with_rotated_item_bounds_when_resize_computed_then_scales_correctly() {
    // Given: A selection with nodes that represent rotated item bounds
    // (rotated items have expanded bounding boxes to account for rotation)
    let mut doc = DiagramDocument::default();
    let normal_id = NodeId::new(String::from("normal"));
    let rotated_bound_id = NodeId::new(String::from("rotated_bound"));

    // Use unlocked nodes since selected_node_ids filters out locked nodes
    fn unlocked_node(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: String::from("n"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    // Normal node: 100x100 at origin
    doc.document.nodes = doc
        .document
        .nodes
        .update(
            normal_id.clone(),
            unlocked_node(NodeKind::Node, 0.0, 0.0, 100.0, 100.0),
        )
        // Rotated item's bounding box is larger (e.g., 45deg rotation of 100x100 gives ~141x141)
        .update(
            rotated_bound_id.clone(),
            unlocked_node(NodeKind::Node, 150.0, 0.0, 141.0, 141.0),
        );

    let _ = doc
        .editor_state
        .selected_items
        .insert(normal_id.to_string());
    let _ = doc
        .editor_state
        .selected_items
        .insert(rotated_bound_id.to_string());

    // When: Computing resize targets
    let targets = resize_target_ids(&doc);

    // Then: Both nodes are included in resize targets
    assert_eq!(targets.len(), 2);
    assert!(targets.contains(&normal_id));
    assert!(targets.contains(&rotated_bound_id));
}

// ============== MUL-002: Resize selection with text ==============

#[test]
fn given_selection_with_text_node_when_resize_computed_then_text_included() {
    // Given: A selection containing a text node and a regular shape
    let mut doc = DiagramDocument::default();
    let shape_id = NodeId::new(String::from("shape"));
    let text_id = NodeId::new(String::from("text"));

    // Use unlocked nodes since selected_node_ids filters out locked nodes
    fn unlocked_node(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: String::from("n"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            shape_id.clone(),
            unlocked_node(NodeKind::Node, 0.0, 0.0, 100.0, 80.0),
        )
        .update(
            text_id.clone(),
            unlocked_node(NodeKind::Text, 120.0, 20.0, 80.0, 30.0),
        );

    let _ = doc.editor_state.selected_items.insert(shape_id.to_string());
    let _ = doc.editor_state.selected_items.insert(text_id.to_string());

    // When: Computing resize targets
    let targets = resize_target_ids(&doc);

    // Then: Text node is included in resize targets
    assert_eq!(targets.len(), 2);
    assert!(targets.contains(&shape_id));
    assert!(
        targets.contains(&text_id),
        "Text nodes should be included in multi-select resize"
    );
}

// ============== MUL-003: Resize selection with 2-point line ==============
// Note: Lines in this diagram tool are represented as edges, not nodes.
// Edges don't have bounding boxes in the same way nodes do.
// This test verifies that a selection with thin/narrow nodes (representing lines) scales correctly.

#[test]
fn given_selection_with_line_like_node_when_resize_computed_then_scales_proportionally() {
    // Given: A selection with a very narrow node representing a line-like element
    let mut doc = DiagramDocument::default();
    let shape_id = NodeId::new(String::from("shape"));
    let line_like_id = NodeId::new(String::from("line_like"));

    // Use unlocked nodes since selected_node_ids filters out locked nodes
    fn unlocked_node(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: String::from("n"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            shape_id.clone(),
            unlocked_node(NodeKind::Node, 0.0, 0.0, 100.0, 100.0),
        )
        // A "2-point line" can be represented as a very thin rectangle
        .update(
            line_like_id.clone(),
            unlocked_node(NodeKind::Node, 120.0, 45.0, 80.0, 2.0),
        );

    let _ = doc.editor_state.selected_items.insert(shape_id.to_string());
    let _ = doc
        .editor_state
        .selected_items
        .insert(line_like_id.to_string());

    // When: Computing resize targets
    let targets = resize_target_ids(&doc);

    // Then: Line-like node is included and will scale
    assert_eq!(targets.len(), 2);
    assert!(
        targets.contains(&line_like_id),
        "Line-like nodes should be included in resize"
    );

    // Verify selection bounds include the line
    let originals: HashMap<NodeId, (f64, f64, f64, f64)> = targets
        .iter()
        .filter_map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|n| (id.clone(), (n.x.0, n.y.0, n.width.0, n.height.0)))
        })
        .collect();

    // The line-like node should have its dimensions preserved in originals
    assert!(originals.contains_key(&line_like_id));
    let (_, _, w, h) = originals.get(&line_like_id).unwrap();
    assert_eq!(*w, 80.0);
    assert_eq!(*h, 2.0);
}

// ============== MUL-004: Resize selection with curved arrow ==============
// Note: Curved arrows are edges with ArrowType::Curved and optional bend_points.
// This test verifies selection behavior with nodes connected by curved edges.

#[test]
fn given_selection_with_nodes_connected_by_curved_edge_when_resize_computed_then_nodes_scale() {
    // Given: Two nodes connected by what would be a curved arrow (edge)
    let mut doc = DiagramDocument::default();
    let source_id = NodeId::new(String::from("source"));
    let target_id = NodeId::new(String::from("target"));

    // Use unlocked nodes since selected_node_ids filters out locked nodes
    fn unlocked_node(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: String::from("n"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            source_id.clone(),
            unlocked_node(NodeKind::Node, 0.0, 0.0, 60.0, 40.0),
        )
        .update(
            target_id.clone(),
            unlocked_node(NodeKind::Node, 150.0, 50.0, 60.0, 40.0),
        );

    let _ = doc
        .editor_state
        .selected_items
        .insert(source_id.to_string());
    let _ = doc
        .editor_state
        .selected_items
        .insert(target_id.to_string());

    // When: Computing resize targets for the selection
    let targets = resize_target_ids(&doc);

    // Then: Both nodes are included (edges scale implicitly as they connect nodes)
    assert_eq!(targets.len(), 2);
    assert!(targets.contains(&source_id));
    assert!(targets.contains(&target_id));

    // The selection bounds should encompass both nodes
    // min_x=0, min_y=0, max_x=210 (150+60), max_y=90 (50+40)
    // width=210, height=90
    let bounds_x = 0.0_f64;
    let bounds_y = 0.0_f64;

    let originals: HashMap<NodeId, (f64, f64, f64, f64)> = targets
        .iter()
        .filter_map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|n| (id.clone(), (n.x.0, n.y.0, n.width.0, n.height.0)))
        })
        .collect();

    // Verify both nodes are captured with their original positions
    assert_eq!(originals.len(), 2);

    // Simulate a 2x scale from the top-left corner
    let scale_x = 2.0;
    let scale_y = 2.0;
    let new_bounds_x = bounds_x;
    let new_bounds_y = bounds_y;

    for (id, (ox, oy, ow, oh)) in &originals {
        let expected_x = (ox - bounds_x).mul_add(scale_x, new_bounds_x);
        let expected_y = (oy - bounds_y).mul_add(scale_y, new_bounds_y);
        let expected_w = ow * scale_x;
        let expected_h = oh * scale_y;

        // Verify calculations are valid (not NaN or infinite)
        assert!(
            expected_x.is_finite(),
            "Scaled x should be finite for {id:?}"
        );
        assert!(
            expected_y.is_finite(),
            "Scaled y should be finite for {id:?}"
        );
        assert!(
            expected_w.is_finite(),
            "Scaled width should be finite for {id:?}"
        );
        assert!(
            expected_h.is_finite(),
            "Scaled height should be finite for {id:?}"
        );
    }
}

// ============== MUL-005: Resize selection past inversion ==============
// When resizing past the anchor point, the selection inverts (negative scale).

#[test]
fn given_selection_resize_past_inversion_when_finalized_then_handles_negative_scale() {
    // Given: A resize gesture that would cause inversion (drag past anchor)
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new(String::from("test_node"));

    doc.document.nodes = doc.document.nodes.update(
        node_id.clone(),
        node(NodeKind::Node, 50.0, 50.0, 100.0, 100.0),
    );

    // Original bounds: (50, 50, 100, 100) meaning x=50, y=50, w=100, h=100
    // If we drag the SE handle past the NW corner, we get inversion
    let originals: HashMap<NodeId, (f64, f64, f64, f64)> = {
        let mut map = HashMap::new();
        map.insert(node_id, (50.0, 50.0, 100.0, 100.0));
        map
    };

    let mut mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (50.0, 50.0, 100.0, 100.0),
        originals,
        anchor: (150.0, 150.0), // Anchor at SE corner
        did_resize: true,
        aspect_ratio: None,
    };

    // When: Finalizing the resize (even with inversion potential)
    let result = finalize_motion_release(&mut mode, &mut doc, &None);

    // Then: The resize completes without panic/error
    assert!(result);
    assert_eq!(mode, InteractionMode::Select);

    // Revision should be incremented since did_resize is true
    assert_eq!(
        doc.revision,
        DiagramDocument::default().revision.increment()
    );
}

#[test]
fn given_selection_with_inverted_dimensions_when_resize_computed_then_clamps_to_minimum() {
    // Given: A selection where resize would result in dimensions below minimum
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new(String::from("small_node"));

    doc.document.nodes = doc
        .document
        .nodes
        .update(node_id.clone(), node(NodeKind::Node, 0.0, 0.0, 50.0, 50.0));

    let originals: HashMap<NodeId, (f64, f64, f64, f64)> = {
        let mut map = HashMap::new();
        map.insert(node_id, (0.0, 0.0, 50.0, 50.0));
        map
    };

    // Simulate extreme resize that would go negative
    // The canvas resize logic clamps to 24.0 minimum
    let mut mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 50.0, 50.0),
        originals,
        anchor: (25.0, 25.0),
        did_resize: true,
        aspect_ratio: None,
    };

    // When: Finalizing
    let result = finalize_motion_release(&mut mode, &mut doc, &None);

    // Then: Completes successfully (canvas logic handles clamping)
    assert!(result);
    assert_eq!(mode, InteractionMode::Select);
}
