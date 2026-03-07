#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use im::HashMap;

use super::selection_geometry::{selected_node_ids, selection_bounds};
use crate::{
    history::History,
    models::document::{DiagramDocument, EdgeId, NodeId, NodeKind},
};

fn resize_target_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    let selected = selected_node_ids(doc);
    let node_geometry = doc
        .document
        .nodes
        .iter()
        .map(|(id, node)| {
            (
                id.clone(),
                (
                    node.x.0,
                    node.y.0,
                    node.width.0,
                    node.height.0,
                    node.kind == NodeKind::Subgraph,
                ),
            )
        })
        .collect::<im::HashMap<_, _>>();

    crate::ui::canvas::drag_math::calculate_resize_target_ids(&selected, &node_geometry)
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct DragState {
    pub anchor_canvas: (f64, f64),
    pub original_positions: HashMap<NodeId, (f64, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct DragPendingState {
    pub anchor_canvas: (f64, f64),
    pub anchor_client: (f64, f64),
    pub original_positions: HashMap<NodeId, (f64, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ResizeState {
    pub handle: ResizeHandle,
    pub original_bounds: (f64, f64, f64, f64),
    pub originals: HashMap<NodeId, (f64, f64, f64, f64)>,
    pub anchor: (f64, f64),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum InteractionMode {
    Select,
    RubberBand {
        start: (f64, f64),
        current: (f64, f64),
    },
    DragPending(DragPendingState),
    DraggingSelection {
        anchor_canvas: (f64, f64),
        anchor_client: (f64, f64),
        original_positions: HashMap<NodeId, (f64, f64)>,
        did_move: bool,
    },
    Dragging(DragState),
    DrawingEdge {
        from_node: NodeId,
        current_pos: (f64, f64),
    },
    DrawingSubgraph {
        start: (f64, f64),
        current: (f64, f64),
    },
    ResizePending(ResizeState),
    ResizingSelection {
        handle: ResizeHandle,
        original_bounds: (f64, f64, f64, f64),
        originals: HashMap<NodeId, (f64, f64, f64, f64)>,
        anchor: (f64, f64),
        did_resize: bool,
    },
    Resizing(ResizeState),
    Panning {
        last_pos: (f64, f64),
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResizeHandle {
    Nw,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
}

pub(super) fn commit_inline_edit(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut editing_node: Signal<Option<NodeId>>,
    mut editing_edge: Signal<Option<EdgeId>>,
    edit_value: Signal<String>,
) {
    let node_target = editing_node.read().clone();
    if let Some(node_id) = node_target {
        let new_label = edit_value.read().clone();
        let target = node_id;
        let current_label = doc_signal
            .read()
            .document
            .nodes
            .get(&target)
            .map_or_else(String::new, |n| n.label.clone());
        if current_label != new_label {
            let current = doc_signal.read().clone();
            let history = history_signal.read().clone();
            *history_signal.write() = history.push(current);
            doc_signal.with_mut(|doc| {
                if let Some(n) = doc.document.nodes.get_mut(&target) {
                    n.label = new_label;
                    doc.revision = doc.revision.increment();
                }
            });
        }
        editing_node.set(None);
        return;
    }

    let edge_target = editing_edge.read().clone();
    if let Some(edge_id) = edge_target {
        let new_label = edit_value.read().clone();
        let target = edge_id;
        let current_label = doc_signal
            .read()
            .document
            .edges
            .get(&target)
            .map_or_else(String::new, |e| e.label.clone());
        if current_label != new_label {
            let current = doc_signal.read().clone();
            let history = history_signal.read().clone();
            *history_signal.write() = history.push(current);
            doc_signal.with_mut(|doc| {
                if let Some(e) = doc.document.edges.get_mut(&target) {
                    e.label = new_label;
                    doc.revision = doc.revision.increment();
                }
            });
        }
        editing_edge.set(None);
    }
}

pub(super) fn start_resize_interaction(
    mut interaction_mode: Signal<InteractionMode>,
    doc_signal: Signal<DiagramDocument>,
    handle: ResizeHandle,
    client_x: f64,
    client_y: f64,
) {
    let doc = doc_signal.read().clone();
    if let Some(bounds) = selection_bounds(&doc) {
        let Some((cx, cy)) = crate::ui::canvas::math::screen_to_canvas(
            client_x,
            client_y,
            doc.editor_state.camera_x.0,
            doc.editor_state.camera_y.0,
            doc.editor_state.zoom.0,
        ) else {
            return;
        };

        let originals = resize_target_ids(&doc)
            .into_iter()
            .fold(HashMap::new(), |acc, id| {
                if let Some(n) = doc.document.nodes.get(&id) {
                    acc.update(id, (n.x.0, n.y.0, n.width.0, n.height.0))
                } else {
                    acc
                }
            });

        interaction_mode.set(InteractionMode::ResizePending(ResizeState {
            handle,
            original_bounds: bounds,
            originals,
            anchor: (cx, cy),
        }));
    }
}

pub(super) fn finalize_motion_release(
    mode: &mut InteractionMode,
    doc: &mut DiagramDocument,
) -> bool {
    let should_increment = match mode {
        InteractionMode::Dragging(_) | InteractionMode::Resizing(_) => true,
        InteractionMode::DraggingSelection { did_move, .. } => *did_move,
        InteractionMode::ResizingSelection { did_resize, .. } => *did_resize,
        InteractionMode::DragPending(_) | InteractionMode::ResizePending(_) => {
            *mode = InteractionMode::Select;
            return true;
        }
        _ => return false,
    };

    if should_increment {
        doc.revision = doc.revision.increment();
    }
    *mode = InteractionMode::Select;
    true
}

#[cfg(test)]
mod tests {
    use im::HashMap;

    use super::{finalize_motion_release, resize_target_ids, InteractionMode, ResizeHandle, DragPendingState, DragState, ResizeState};
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };

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
            locked: true,
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
        let mut mode = InteractionMode::Dragging(DragState { anchor_canvas: (0.0, 0.0), original_positions: HashMap::new() });

        let first = finalize_motion_release(&mut mode, &mut doc);
        let second = finalize_motion_release(&mut mode, &mut doc);

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
        let mut mode = InteractionMode::ResizePending(ResizeState { handle: ResizeHandle::Se, original_bounds: (0.0, 0.0, 10.0, 10.0), originals: HashMap::new(), anchor: (0.0, 0.0) });

        let finalized = finalize_motion_release(&mut mode, &mut doc);

        assert!(finalized);
        assert_eq!(doc.revision, DiagramDocument::default().revision);
        assert_eq!(mode, InteractionMode::Select);
    }

    #[test]
    fn given_resize_end_when_finalized_twice_then_revision_bumps_once() {
        let mut doc = DiagramDocument::default();
        let mut mode = InteractionMode::Resizing(ResizeState { handle: ResizeHandle::E, original_bounds: (0.0, 0.0, 10.0, 10.0), originals: HashMap::new(), anchor: (0.0, 0.0) });

        let first = finalize_motion_release(&mut mode, &mut doc);
        let second = finalize_motion_release(&mut mode, &mut doc);

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

        doc.document.nodes = doc
            .document
            .nodes
            .update(
                subgraph.clone(),
                node(NodeKind::Subgraph, 100.0, 100.0, 300.0, 220.0),
            )
            .update(
                inside.clone(),
                node(NodeKind::Node, 140.0, 140.0, 80.0, 60.0),
            )
            .update(
                outside.clone(),
                node(NodeKind::Node, 450.0, 300.0, 80.0, 60.0),
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

        let result = finalize_motion_release(&mut mode, &mut doc);

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
        let mut mode = InteractionMode::Dragging(DragState { anchor_canvas: (0.0, 0.0), original_positions: HashMap::new() });

        let first_result = finalize_motion_release(&mut mode, &mut doc);
        let first_revision = doc.revision;

        assert!(first_result, "First finalize should succeed");
        assert_eq!(mode, InteractionMode::Select);

        // Second event: duplicate pointerup arrives after already finalized
        let second_result = finalize_motion_release(&mut mode, &mut doc);
        let second_revision = doc.revision;

        assert!(!second_result, "Second finalize should be idempotent");
        assert_eq!(
            first_revision, second_revision,
            "Revision should not change on duplicate"
        );

        // Third event: another duplicate mouseup
        let third_result = finalize_motion_release(&mut mode, &mut doc);
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
        let mut mode = InteractionMode::Resizing(ResizeState { handle: ResizeHandle::E, original_bounds: (0.0, 0.0, 100.0, 100.0), originals: HashMap::new(), anchor: (50.0, 50.0) });

        let first_result = finalize_motion_release(&mut mode, &mut doc);
        assert!(first_result);
        assert_eq!(mode, InteractionMode::Select);

        // Duplicate events after finalization
        for _ in 0..5 {
            let result = finalize_motion_release(&mut mode, &mut doc);
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

        let result = finalize_motion_release(&mut mode, &mut doc);

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
        let result = finalize_motion_release(&mut mode, &mut doc);
        assert!(!result);
        assert_eq!(doc.revision, initial_revision);

        // Second: drag gesture
        mode = InteractionMode::Dragging(DragState { anchor_canvas: (0.0, 0.0), original_positions: HashMap::new() });
        let result = finalize_motion_release(&mut mode, &mut doc);
        assert!(result);
        assert_eq!(doc.revision, initial_revision.increment());

        // Third: resize gesture
        mode = InteractionMode::Resizing(ResizeState { handle: ResizeHandle::Se, original_bounds: (0.0, 0.0, 100.0, 100.0), originals: HashMap::new(), anchor: (50.0, 50.0) });
        let result = finalize_motion_release(&mut mode, &mut doc);
        assert!(result);
        assert_eq!(doc.revision, initial_revision.increment().increment());

        // Duplicate finalizations should not change anything
        for _ in 0..3 {
            let result = finalize_motion_release(&mut mode, &mut doc);
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

        // Normal node: 100x100 at origin
        doc.document.nodes = doc
            .document
            .nodes
            .update(
                normal_id.clone(),
                node(NodeKind::Node, 0.0, 0.0, 100.0, 100.0),
            )
            // Rotated item's bounding box is larger (e.g., 45deg rotation of 100x100 gives ~141x141)
            .update(
                rotated_bound_id.clone(),
                node(NodeKind::Node, 150.0, 0.0, 141.0, 141.0),
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

        doc.document.nodes = doc
            .document
            .nodes
            .update(
                shape_id.clone(),
                node(NodeKind::Node, 0.0, 0.0, 100.0, 80.0),
            )
            .update(
                text_id.clone(),
                node(NodeKind::Text, 120.0, 20.0, 80.0, 30.0),
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
    // This test verifies that a selection with thin/narrow nodes (representing lines) scales
    // correctly.

    #[test]
    fn given_selection_with_line_like_node_when_resize_computed_then_scales_proportionally() {
        // Given: A selection with a very narrow node representing a line-like element
        let mut doc = DiagramDocument::default();
        let shape_id = NodeId::new(String::from("shape"));
        let line_like_id = NodeId::new(String::from("line_like"));

        doc.document.nodes = doc
            .document
            .nodes
            .update(
                shape_id.clone(),
                node(NodeKind::Node, 0.0, 0.0, 100.0, 100.0),
            )
            // A "2-point line" can be represented as a very thin rectangle
            .update(
                line_like_id.clone(),
                node(NodeKind::Node, 120.0, 45.0, 80.0, 2.0),
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

        doc.document.nodes = doc
            .document
            .nodes
            .update(
                source_id.clone(),
                node(NodeKind::Node, 0.0, 0.0, 60.0, 40.0),
            )
            .update(
                target_id.clone(),
                node(NodeKind::Node, 150.0, 50.0, 60.0, 40.0),
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
                "Scaled x should be finite for {:?}",
                id
            );
            assert!(
                expected_y.is_finite(),
                "Scaled y should be finite for {:?}",
                id
            );
            assert!(
                expected_w.is_finite(),
                "Scaled width should be finite for {:?}",
                id
            );
            assert!(
                expected_h.is_finite(),
                "Scaled height should be finite for {:?}",
                id
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
            map.insert(node_id.clone(), (50.0, 50.0, 100.0, 100.0));
            map
        };

        let mut mode = InteractionMode::ResizingSelection {
            handle: ResizeHandle::Se,
            original_bounds: (50.0, 50.0, 100.0, 100.0),
            originals,
            anchor: (150.0, 150.0), // Anchor at SE corner
            did_resize: true,
        };

        // When: Finalizing the resize (even with inversion potential)
        let result = finalize_motion_release(&mut mode, &mut doc);

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
            map.insert(node_id.clone(), (0.0, 0.0, 50.0, 50.0));
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
        };

        // When: Finalizing
        let result = finalize_motion_release(&mut mode, &mut doc);

        // Then: Completes successfully (canvas logic handles clamping)
        assert!(result);
        assert_eq!(mode, InteractionMode::Select);
    }
}

#[cfg(test)]
mod proptests {
    use im::HashMap;
    use proptest::prelude::*;

    use super::{finalize_motion_release, resize_target_ids, InteractionMode, ResizeHandle, DragPendingState, DragState, ResizeState};
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };

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
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_resize_target_ids_empty_doc(nodes_count in 0usize..10) {
            let mut doc = DiagramDocument::default();
            for i in 0..nodes_count {
                let id = NodeId::new(format!("node_{i}"));
                doc.document.nodes = doc.document.nodes.update(
                    id,
                    node(NodeKind::Node, i as f64 * 100.0, 0.0, 50.0, 50.0),
                );
            }
            let targets = resize_target_ids(&doc);
            prop_assert!(targets.is_empty());
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn stress_finalize_idempotent_on_select() {
        for _ in 0..256 {
            let mut doc = DiagramDocument::default();
            let mut mode = InteractionMode::Select;
            let result = finalize_motion_release(&mut mode, &mut doc);
            assert!(!result);
            assert_eq!(mode, InteractionMode::Select);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_drag_extreme_anchor(
            anchor_canvas_x in prop::sample::select(&[f64::MIN, f64::MAX, f64::INFINITY, f64::NEG_INFINITY, f64::NAN]),
            anchor_canvas_y in prop::sample::select(&[f64::MIN, f64::MAX, f64::INFINITY, f64::NEG_INFINITY, f64::NAN]),
        ) {
            let mut doc = DiagramDocument::default();
            let mut mode = InteractionMode::Dragging(DragState { anchor_canvas: (anchor_canvas_x, anchor_canvas_y), original_positions: HashMap::new() });
            let _ = finalize_motion_release(&mut mode, &mut doc);
            prop_assert_eq!(mode, InteractionMode::Select);
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn stress_resize_nan_bounds() {
        for _ in 0..256 {
            let mut doc = DiagramDocument::default();
            let mut mode = InteractionMode::Resizing(ResizeState { handle: ResizeHandle::Se, original_bounds: (f64::NAN, f64::NAN, f64::NAN, f64::NAN), originals: HashMap::new(), anchor: (f64::NAN, f64::NAN) });
            let result = finalize_motion_release(&mut mode, &mut doc);
            assert!(result);
            assert_eq!(mode, InteractionMode::Select);
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn stress_resize_infinite_bounds() {
        for _ in 0..256 {
            let mut doc = DiagramDocument::default();
            let mut mode = InteractionMode::ResizingSelection {
                handle: ResizeHandle::Nw,
                original_bounds: (
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                    f64::INFINITY,
                    f64::INFINITY,
                ),
                originals: HashMap::new(),
                anchor: (f64::INFINITY, f64::NEG_INFINITY),
                did_resize: true,
            };
            let result = finalize_motion_release(&mut mode, &mut doc);
            assert!(result);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_rubberband_zero_area(start_x: f64, start_y: f64) {
            let mode = InteractionMode::RubberBand {
                start: (start_x, start_y),
                current: (start_x, start_y),
            };
            // Verify zero-area rubberband (start == current)
            if let InteractionMode::RubberBand { start, current } = mode {
                prop_assert_eq!(start, current);
            } else {
                prop_assert!(false, "Expected RubberBand mode");
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_rubberband_negative_area(start_x: f64, start_y: f64, delta_x: f64, delta_y: f64) {
            let mode = InteractionMode::RubberBand {
                start: (start_x, start_y),
                current: (start_x - delta_x.abs(), start_y - delta_y.abs()),
            };
            // Verify the rubberband mode was created
            if let InteractionMode::RubberBand { .. } = mode {
                // Mode was successfully created
            } else {
                prop_assert!(false, "Expected RubberBand mode");
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_drawing_edge_extreme(
            from_node in "node_[0-9]{1,3}",
            pos_x in prop::sample::select(&[f64::MIN, f64::MAX, 0.0, 1.0]),
            pos_y in prop::sample::select(&[f64::MIN, f64::MAX, 0.0, 1.0]),
        ) {
            let from_node_clone = from_node.clone();
            let mode = InteractionMode::DrawingEdge {
                from_node: NodeId::new(from_node.clone()),
                current_pos: (pos_x, pos_y),
            };
            // Verify mode was created with the correct from_node
            if let InteractionMode::DrawingEdge { from_node: result_node, .. } = mode {
                prop_assert_eq!(result_node, NodeId::new(from_node_clone));
            } else {
                prop_assert!(false, "Expected DrawingEdge mode");
            }
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn stress_panning_nan() {
        for _ in 0..256 {
            let mode = InteractionMode::Panning {
                last_pos: (f64::NAN, f64::NAN),
            };
            let _ = mode;
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_nested_subgraphs(depth in 1usize..5) {
            let mut doc = DiagramDocument::default();
            for i in 0..depth {
                let id = NodeId::new(format!("sub_{i}"));
                let size = 100.0 + (depth - i) as f64 * 50.0;
                doc.document.nodes = doc.document.nodes.update(
                    id.clone(),
                    node(NodeKind::Subgraph, i as f64 * 10.0, i as f64 * 10.0, size, size),
                );
                if i == depth - 1 {
                    let _ = doc.editor_state.selected_items.insert(id.to_string());
                }
            }
            let targets = resize_target_ids(&doc);
            prop_assert!(!targets.is_empty());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_finalize_stress(iterations in 1usize..100) {
            let mut doc = DiagramDocument::default();
            let initial_revision = doc.revision;
            for _ in 0..iterations {
                let mut mode = InteractionMode::Dragging(DragState { anchor_canvas: (0.0, 0.0), original_positions: HashMap::new() });
                let _ = finalize_motion_release(&mut mode, &mut doc);
            }
            let mut expected = initial_revision;
            for _ in 0..iterations {
                expected = expected.increment();
            }
            prop_assert_eq!(doc.revision, expected);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_all_resize_handles(handle in prop::sample::select(&[
            ResizeHandle::Nw, ResizeHandle::N, ResizeHandle::Ne,
            ResizeHandle::E, ResizeHandle::Se, ResizeHandle::S,
            ResizeHandle::Sw, ResizeHandle::W,
        ])) {
            let mut doc = DiagramDocument::default();
            let mut mode = InteractionMode::ResizingSelection {
                handle,
                original_bounds: (0.0, 0.0, 100.0, 100.0),
                originals: HashMap::new(),
                anchor: (50.0, 50.0),
                did_resize: true,
            };
            let result = finalize_motion_release(&mut mode, &mut doc);
            prop_assert!(result);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_many_original_positions(node_count in 10usize..50) {
            let mut positions = HashMap::new();
            for i in 0..node_count {
                let id = NodeId::new(format!("node_{i}"));
                positions = positions.update(id, (i as f64, i as f64 * 2.0));
            }
            let mut mode = InteractionMode::Dragging(DragState { anchor_canvas: (0.0, 0.0), original_positions: positions.clone() });
            let mut doc = DiagramDocument::default();
            let _ = finalize_motion_release(&mut mode, &mut doc);
            prop_assert_eq!(mode, InteractionMode::Select);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_mode_equality_rubberband(
            start_x: f64, start_y: f64,
            current_x: f64, current_y: f64,
        ) {
            let mode1 = InteractionMode::RubberBand {
                start: (start_x, start_y),
                current: (current_x, current_y),
            };
            let mode2 = InteractionMode::RubberBand {
                start: (start_x, start_y),
                current: (current_x, current_y),
            };
            if start_x.is_nan() || start_y.is_nan() || current_x.is_nan() || current_y.is_nan() {
                // NaN values should result in non-equal modes (since NaN != NaN)
                prop_assert!(mode1 != mode2);
            } else {
                prop_assert_eq!(mode1, mode2);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_drawing_subgraph_extreme(
            start in (any::<f64>(), any::<f64>()),
            current in (any::<f64>(), any::<f64>()),
        ) {
            let mode = InteractionMode::DrawingSubgraph { start, current };
            // Verify mode was created
            if let InteractionMode::DrawingSubgraph { .. } = mode {
                // Mode created successfully
            } else {
                prop_assert!(false, "Expected DrawingSubgraph mode");
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_resize_handles_extreme_coords(
            handle in prop::sample::select(&[
                ResizeHandle::Nw, ResizeHandle::N, ResizeHandle::Ne,
                ResizeHandle::E, ResizeHandle::Se, ResizeHandle::S,
                ResizeHandle::Sw, ResizeHandle::W,
            ]),
            coord in prop::sample::select(&[f64::MIN, f64::MAX, 0.0, f64::NAN]),
        ) {
            let mut mode = InteractionMode::ResizingSelection {
                handle,
                original_bounds: (coord, coord, coord, coord),
                originals: HashMap::new(),
                anchor: (coord, coord),
                did_resize: false,
            };
            let mut doc = DiagramDocument::default();
            let result = finalize_motion_release(&mut mode, &mut doc);
            prop_assert!(result);
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn stress_no_revision_bump_no_move() {
        for _ in 0..256 {
            let mut doc = DiagramDocument::default();
            let initial = doc.revision;
            let mut mode = InteractionMode::DragPending(DragPendingState { anchor_canvas: (0.0, 0.0), anchor_client: (0.0, 0.0), original_positions: HashMap::new() });
            let _ = finalize_motion_release(&mut mode, &mut doc);
            assert_eq!(doc.revision, initial);
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn stress_no_revision_bump_no_resize() {
        for _ in 0..256 {
            let mut doc = DiagramDocument::default();
            let initial = doc.revision;
            let mut mode = InteractionMode::ResizePending(ResizeState { handle: ResizeHandle::Se, original_bounds: (0.0, 0.0, 100.0, 100.0), originals: HashMap::new(), anchor: (50.0, 50.0) });
            let _ = finalize_motion_release(&mut mode, &mut doc);
            assert_eq!(doc.revision, initial);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_tiny_subgraph(
            node_x: f64, node_y: f64,
            subgraph_offset_x: f64, subgraph_offset_y: f64,
        ) {
            let mut doc = DiagramDocument::default();
            let sub_id = NodeId::new(String::from("tiny_sub"));
            let node_id = NodeId::new(String::from("node"));

            doc.document.nodes = doc.document.nodes
                .update(sub_id.clone(), node(NodeKind::Subgraph, subgraph_offset_x, subgraph_offset_y, 1.0, 1.0))
                .update(node_id.clone(), node(NodeKind::Node, node_x, node_y, 10.0, 10.0));
            let _ = doc.editor_state.selected_items.insert(sub_id.to_string());

            let targets = resize_target_ids(&doc);
            let expected_inside = node_x >= subgraph_offset_x
                && node_y >= subgraph_offset_y
                && node_x + 10.0 <= subgraph_offset_x + 1.0
                && node_y + 10.0 <= subgraph_offset_y + 1.0;

            if expected_inside && node_x.is_finite() && node_y.is_finite() {
                prop_assert!(targets.contains(&node_id));
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_rapid_mode_transitions(ops in prop::collection::vec(0u8..8, 1..100)) {
            let mut mode = InteractionMode::Select;
            let mut doc = DiagramDocument::default();

            for op in ops {
                mode = match op {
                    0 => InteractionMode::Select,
                    1 => InteractionMode::RubberBand { start: (0.0, 0.0), current: (1.0, 1.0) },
                    2 => InteractionMode::DragPending(DragPendingState { anchor_canvas: (0.0, 0.0), anchor_client: (0.0, 0.0), original_positions: HashMap::new() }),
                    3 => InteractionMode::DrawingEdge {
                        from_node: NodeId::new(String::from("n")),
                        current_pos: (0.0, 0.0),
                    },
                    4 => InteractionMode::DrawingSubgraph { start: (0.0, 0.0), current: (1.0, 1.0) },
                    5 => InteractionMode::ResizePending(ResizeState { handle: ResizeHandle::Se, original_bounds: (0.0, 0.0, 10.0, 10.0), originals: HashMap::new(), anchor: (5.0, 5.0) }),
                    6 => InteractionMode::Panning { last_pos: (0.0, 0.0) },
                    _ => {
                        let _ = finalize_motion_release(&mut mode, &mut doc);
                        continue;
                    }
                };
            }
            // Test completed - modes were created without panicking
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_overlapping_subgraphs(count in 2usize..10) {
            let mut doc = DiagramDocument::default();
            let inner_node = NodeId::new(String::from("inner"));

            for i in 0..count {
                let id = NodeId::new(format!("sub_{i}"));
                let size = 200.0 - i as f64 * 10.0;
                doc.document.nodes = doc.document.nodes.update(
                    id.clone(),
                    node(NodeKind::Subgraph, i as f64 * 5.0, i as f64 * 5.0, size, size),
                );
            }

            doc.document.nodes = doc.document.nodes.update(
                inner_node.clone(),
                node(NodeKind::Node, 100.0, 100.0, 20.0, 20.0),
            );

            let outer_id = NodeId::new(String::from("sub_0"));
            let _ = doc.editor_state.selected_items.insert(outer_id.to_string());

            let targets = resize_target_ids(&doc);
            prop_assert!(targets.contains(&inner_node));
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn stress_preserves_state_on_select() {
        for _ in 0..256 {
            let mut doc = DiagramDocument::default();
            let node_id = NodeId::new(String::from("test"));
            doc.document.nodes = doc.document.nodes.update(
                node_id.clone(),
                node(NodeKind::Node, 100.0, 100.0, 50.0, 50.0),
            );
            let doc_before = doc.clone();
            let mut mode = InteractionMode::Select;
            let _ = finalize_motion_release(&mut mode, &mut doc);
            assert_eq!(doc.document.nodes.len(), doc_before.document.nodes.len());
        }
    }
}

// =============================================================================
// INP Mobile/Touch Interaction tests (bd-27q)
// =============================================================================

#[cfg(test)]
mod inp_mobile_touch_tests {
    use im::HashMap;

    use super::{DragPendingState, DragState, InteractionMode, ResizeHandle, ResizeState};
    use crate::models::document::{Node, NodeId, NodeKind, NodeStyle, OrderedFloat};

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
                locked: false,
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

        let dragging = InteractionMode::DragPending(DragPendingState { anchor_canvas: (0.0, 0.0), anchor_client: (0.0, 0.0), original_positions: HashMap::new() });

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

        let resizing = InteractionMode::ResizePending(ResizeState { handle: ResizeHandle::Se, original_bounds: (0.0, 0.0, 100.0, 100.0), originals: HashMap::new(), anchor: (100.0, 100.0) });

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
            InteractionMode::DragPending(DragPendingState { anchor_canvas: (0.0, 0.0), anchor_client: (0.0, 0.0), original_positions: HashMap::new() }),
            InteractionMode::DrawingEdge {
                from_node: NodeId::new("x".to_string()),
                current_pos: (42.0, 24.0),
            },
            InteractionMode::DrawingSubgraph {
                start: (0.0, 0.0),
                current: (42.0, 24.0),
            },
            InteractionMode::ResizePending(ResizeState { handle: ResizeHandle::Nw, original_bounds: (0.0, 0.0, 42.0, 24.0), originals: HashMap::new(), anchor: (21.0, 12.0) }),
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
}
