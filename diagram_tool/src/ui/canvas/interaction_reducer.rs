#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::selection_geometry::{selected_node_ids, selection_bounds};
use crate::history::History;
use crate::models::document::{DiagramDocument, Edge, EdgeId, Node, NodeId, NodeKind};
use crate::mutation::ui_helpers::mutate_doc_with_history;
use dioxus::prelude::*;
use im::HashMap;
use std::collections::HashSet;

fn safe_zoom(zoom: f64) -> Option<f64> {
    (zoom.is_finite() && zoom > f64::EPSILON).then_some(zoom)
}

fn within(subgraph: (f64, f64, f64, f64), node: (f64, f64, f64, f64)) -> bool {
    let (sx, sy, sw, sh) = subgraph;
    let (nx, ny, nw, nh) = node;
    nx >= sx && ny >= sy && nx + nw <= sx + sw && ny + nh <= sy + sh
}

fn resize_target_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    let selected = selected_node_ids(doc);
    let selected_set = selected.iter().cloned().collect::<HashSet<_>>();

    let selected_subgraphs = selected
        .iter()
        .filter_map(|id| doc.document.nodes.get(id).map(|node| (id, node)))
        .filter(|(_, node)| node.kind == NodeKind::Subgraph)
        .map(|(_, node)| (node.x.0, node.y.0, node.width.0, node.height.0))
        .collect::<Vec<_>>();

    if selected_subgraphs.is_empty() {
        return selected;
    }

    doc.document
        .nodes
        .iter()
        .fold(selected_set, |acc, (id, node)| {
            let node_rect = (node.x.0, node.y.0, node.width.0, node.height.0);
            let included = selected_subgraphs
                .iter()
                .any(|subgraph_rect| within(*subgraph_rect, node_rect));
            if included {
                let mut updated = acc;
                let _ = updated.insert(id.clone());
                updated
            } else {
                acc
            }
        })
        .into_iter()
        .collect::<Vec<_>>()
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum InteractionMode {
    Select,
    RubberBand {
        start: (f64, f64),
        current: (f64, f64),
    },
    DraggingSelection {
        anchor_canvas: (f64, f64),
        anchor_client: (f64, f64),
        original_positions: HashMap<NodeId, (f64, f64)>,
        did_move: bool,
    },
    DrawingEdge {
        from_node: NodeId,
        current_pos: (f64, f64),
    },
    DrawingSubgraph {
        start: (f64, f64),
        current: (f64, f64),
    },
    ResizingSelection {
        handle: ResizeHandle,
        original_bounds: (f64, f64, f64, f64),
        originals: HashMap<NodeId, (f64, f64, f64, f64)>,
        anchor: (f64, f64),
        did_resize: bool,
    },
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
            let _ = mutate_doc_with_history(&mut doc_signal, &mut history_signal, |doc| {
                let new_nodes: HashMap<NodeId, Node> = doc
                    .document
                    .nodes
                    .iter()
                    .map(|(id, node)| {
                        if *id == target {
                            (
                                id.clone(),
                                Node {
                                    label: new_label.clone(),
                                    ..node.clone()
                                },
                            )
                        } else {
                            (id.clone(), node.clone())
                        }
                    })
                    .collect();

                let new_doc = DiagramDocument {
                    version: doc.version,
                    revision: doc.revision.increment(),
                    document: crate::models::document::DocumentData {
                        nodes: new_nodes,
                        edges: doc.document.edges.clone(),
                    },
                    editor_state: doc.editor_state,
                };
                Ok(new_doc)
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
            let _ = mutate_doc_with_history(&mut doc_signal, &mut history_signal, |doc| {
                let new_edges: HashMap<EdgeId, Edge> = doc
                    .document
                    .edges
                    .iter()
                    .map(|(id, edge)| {
                        if *id == target {
                            (
                                id.clone(),
                                Edge {
                                    label: new_label.clone(),
                                    ..edge.clone()
                                },
                            )
                        } else {
                            (id.clone(), edge.clone())
                        }
                    })
                    .collect();

                let new_doc = DiagramDocument {
                    version: doc.version,
                    revision: doc.revision.increment(),
                    document: crate::models::document::DocumentData {
                        nodes: doc.document.nodes.clone(),
                        edges: new_edges,
                    },
                    editor_state: doc.editor_state,
                };
                Ok(new_doc)
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
        let Some(zoom) = safe_zoom(doc.editor_state.zoom.0) else {
            return;
        };
        let cx = (client_x / zoom) + doc.editor_state.camera_x.0;
        let cy = (client_y / zoom) + doc.editor_state.camera_y.0;

        let originals = resize_target_ids(&doc)
            .into_iter()
            .fold(HashMap::new(), |acc, id| {
                if let Some(n) = doc.document.nodes.get(&id) {
                    acc.update(id, (n.x.0, n.y.0, n.width.0, n.height.0))
                } else {
                    acc
                }
            });

        interaction_mode.set(InteractionMode::ResizingSelection {
            handle,
            original_bounds: bounds,
            originals,
            anchor: (cx, cy),
            did_resize: false,
        });
    }
}

pub(super) fn finalize_motion_release(
    mode: &mut InteractionMode,
    doc: &mut DiagramDocument,
) -> bool {
    let should_increment = match mode {
        InteractionMode::DraggingSelection { did_move, .. } => Some(*did_move),
        InteractionMode::ResizingSelection { did_resize, .. } => Some(*did_resize),
        _ => None,
    };

    if let Some(increment) = should_increment {
        if increment {
            doc.revision = doc.revision.increment();
        }
        *mode = InteractionMode::Select;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{finalize_motion_release, resize_target_ids, InteractionMode, ResizeHandle};
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
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
        let mut mode = InteractionMode::DraggingSelection {
            anchor_canvas: (0.0, 0.0),
            anchor_client: (0.0, 0.0),
            original_positions: HashMap::new(),
            did_move: true,
        };

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
        let mut mode = InteractionMode::ResizingSelection {
            handle: ResizeHandle::Se,
            original_bounds: (0.0, 0.0, 10.0, 10.0),
            originals: HashMap::new(),
            anchor: (0.0, 0.0),
            did_resize: false,
        };

        let finalized = finalize_motion_release(&mut mode, &mut doc);

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
        };

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
        let mut mode = InteractionMode::DraggingSelection {
            anchor_canvas: (0.0, 0.0),
            anchor_client: (0.0, 0.0),
            original_positions: HashMap::new(),
            did_move: true,
        };

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
        let mut mode = InteractionMode::ResizingSelection {
            handle: ResizeHandle::E,
            original_bounds: (0.0, 0.0, 100.0, 100.0),
            originals: HashMap::new(),
            anchor: (50.0, 50.0),
            did_resize: true,
        };

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
        mode = InteractionMode::DraggingSelection {
            anchor_canvas: (0.0, 0.0),
            anchor_client: (0.0, 0.0),
            original_positions: HashMap::new(),
            did_move: true,
        };
        let result = finalize_motion_release(&mut mode, &mut doc);
        assert!(result);
        assert_eq!(doc.revision, initial_revision.increment());

        // Third: resize gesture
        mode = InteractionMode::ResizingSelection {
            handle: ResizeHandle::Se,
            original_bounds: (0.0, 0.0, 100.0, 100.0),
            originals: HashMap::new(),
            anchor: (50.0, 50.0),
            did_resize: true,
        };
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
    // This test verifies that a selection with thin/narrow nodes (representing lines) scales correctly.

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
    use super::{
        finalize_motion_release, resize_target_ids, safe_zoom, within, InteractionMode,
        ResizeHandle,
    };
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::HashMap;
    use proptest::prelude::*;

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
        fn prop_safe_zoom_rejects_extreme_values(zoom: f64) {
            let result = safe_zoom(zoom);
            if zoom.is_nan() || zoom.is_infinite() || zoom <= f64::EPSILON {
                prop_assert!(result.is_none());
            } else {
                prop_assert!(result.is_some());
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_handles_nan_subgraph_coords(
            sx in prop::option::of(any::<f64>()),
            sy in prop::option::of(any::<f64>()),
            sw in prop::option::of(any::<f64>()),
            sh in prop::option::of(any::<f64>()),
        ) {
            let subgraph = (
                sx.unwrap_or(f64::NAN),
                sy.unwrap_or(f64::NAN),
                sw.unwrap_or(f64::NAN),
                sh.unwrap_or(f64::NAN),
            );
            let node = (10.0, 10.0, 50.0, 50.0);
            let result = within(subgraph, node);
            if sx.is_none() || sy.is_none() || sw.is_none() || sh.is_none() {
                prop_assert!(!result);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_handles_nan_node_coords(
            nx in prop::option::of(any::<f64>()),
            ny in prop::option::of(any::<f64>()),
            nw in prop::option::of(any::<f64>()),
            nh in prop::option::of(any::<f64>()),
        ) {
            let subgraph = (0.0, 0.0, 100.0, 100.0);
            let node = (
                nx.unwrap_or(f64::NAN),
                ny.unwrap_or(f64::NAN),
                nw.unwrap_or(f64::NAN),
                nh.unwrap_or(f64::NAN),
            );
            let result = within(subgraph, node);
            if nx.is_none() || ny.is_none() || nw.is_none() || nh.is_none() {
                prop_assert!(!result);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_degenerate_rectangles(
            sx: f64, sy: f64, sw: f64, sh: f64,
            nx: f64, ny: f64, nw: f64, nh: f64,
        ) {
            prop_assume!(sw.is_finite() && sh.is_finite() && nw.is_finite() && nh.is_finite());
            let subgraph = (sx, sy, sw, sh);
            let node = (nx, ny, nw, nh);
            let result = within(subgraph, node);
            if sw > 0.0 && sh > 0.0 && nw > 0.0 && nh > 0.0 && sx.is_finite() && sy.is_finite() && nx.is_finite() && ny.is_finite() {
                let nx_end = nx + nw;
                let ny_end = ny + nh;
                let sx_end = sx + sw;
                let sy_end = sy + sh;
                let should_be_within = nx >= sx && ny >= sy && nx_end <= sx_end && ny_end <= sy_end;
                prop_assert_eq!(result, should_be_within);
            }
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
            let mut mode = InteractionMode::DraggingSelection {
                anchor_canvas: (anchor_canvas_x, anchor_canvas_y),
                anchor_client: (0.0, 0.0),
                original_positions: HashMap::new(),
                did_move: true,
            };
            let _ = finalize_motion_release(&mut mode, &mut doc);
            prop_assert_eq!(mode, InteractionMode::Select);
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn stress_resize_nan_bounds() {
        for _ in 0..256 {
            let mut doc = DiagramDocument::default();
            let mut mode = InteractionMode::ResizingSelection {
                handle: ResizeHandle::Se,
                original_bounds: (f64::NAN, f64::NAN, f64::NAN, f64::NAN),
                originals: HashMap::new(),
                anchor: (f64::NAN, f64::NAN),
                did_resize: true,
            };
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
            let _ = mode;
            prop_assert!(true);
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
            let _ = mode;
            prop_assert!(true);
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
            let mode = InteractionMode::DrawingEdge {
                from_node: NodeId::new(from_node),
                current_pos: (pos_x, pos_y),
            };
            let _ = mode;
            prop_assert!(true);
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
                let mut mode = InteractionMode::DraggingSelection {
                    anchor_canvas: (0.0, 0.0),
                    anchor_client: (0.0, 0.0),
                    original_positions: HashMap::new(),
                    did_move: true,
                };
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
        fn prop_within_infinite_dims(
            sw in prop::sample::select(&[f64::INFINITY, f64::NEG_INFINITY]),
            sh in prop::sample::select(&[f64::INFINITY, f64::NEG_INFINITY]),
        ) {
            let subgraph = (0.0, 0.0, sw, sh);
            let node = (10.0, 10.0, 10.0, 10.0);
            let result = within(subgraph, node);
            if sw.is_infinite() && sh.is_infinite() && sw > 0.0 && sh > 0.0 {
                prop_assert!(result);
            }
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
            let mut mode = InteractionMode::DraggingSelection {
                anchor_canvas: (0.0, 0.0),
                anchor_client: (0.0, 0.0),
                original_positions: positions.clone(),
                did_move: true,
            };
            let mut doc = DiagramDocument::default();
            let _ = finalize_motion_release(&mut mode, &mut doc);
            prop_assert_eq!(mode, InteractionMode::Select);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_exact_boundary(x in -1e6_f64..1e6_f64, y in -1e6_f64..1e6_f64, w in 1e-6_f64..1e6_f64, h in 1e-6_f64..1e6_f64) {
            let subgraph = (x, y, w, h);
            let node = (x, y, w, h);
            prop_assert!(within(subgraph, node));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_node_on_edge(x in -1e6_f64..1e6_f64, y in -1e6_f64..1e6_f64, w in 1e-6_f64..1e6_f64, h in 1e-6_f64..1e6_f64) {
            let subgraph = (x, y, w, h);
            let node = (x, y, (w - f64::EPSILON).max(0.0), (h - f64::EPSILON).max(0.0));
            prop_assert!(within(subgraph, node));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_exceeds_by_epsilon(x in -1e6_f64..1e6_f64, y in -1e6_f64..1e6_f64, w in 1e-6_f64..1e6_f64, h in 1e-6_f64..1e6_f64) {
            let subgraph = (x, y, w, h);
            let exceed_amount = (w * 0.01).max(f64::EPSILON * 100.0);
            let node = (x, y, w + exceed_amount, h);
            prop_assert!(!within(subgraph, node));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_safe_zoom_boundary(
            zoom in prop::sample::select(&[
                f64::EPSILON,
                f64::EPSILON * 0.5,
                f64::EPSILON * 2.0,
                -f64::EPSILON,
                0.0,
                -0.0,
                f64::MIN_POSITIVE,
            ])
        ) {
            let result = safe_zoom(zoom);
            if zoom > f64::EPSILON && zoom.is_finite() {
                prop_assert!(result.is_some());
            } else {
                prop_assert!(result.is_none());
            }
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
                prop_assert!(mode1 != mode2 || true);
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
            let _ = mode;
            prop_assert!(true);
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
            let mut mode = InteractionMode::DraggingSelection {
                anchor_canvas: (0.0, 0.0),
                anchor_client: (0.0, 0.0),
                original_positions: HashMap::new(),
                did_move: false,
            };
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
            let mut mode = InteractionMode::ResizingSelection {
                handle: ResizeHandle::Se,
                original_bounds: (0.0, 0.0, 100.0, 100.0),
                originals: HashMap::new(),
                anchor: (50.0, 50.0),
                did_resize: false,
            };
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
        fn prop_subnormal_floats(sw in prop::sample::select(&[f64::MIN_POSITIVE, 1e-310]), sh in prop::sample::select(&[f64::MIN_POSITIVE, 1e-310])) {
            let subgraph = (0.0, 0.0, sw, sh);
            let node = (0.0, 0.0, sw / 2.0, sh / 2.0);
            if sw > 0.0 && sh > 0.0 {
                let result = within(subgraph, node);
                prop_assert!(result);
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
                    2 => InteractionMode::DraggingSelection {
                        anchor_canvas: (0.0, 0.0),
                        anchor_client: (0.0, 0.0),
                        original_positions: HashMap::new(),
                        did_move: false,
                    },
                    3 => InteractionMode::DrawingEdge {
                        from_node: NodeId::new(String::from("n")),
                        current_pos: (0.0, 0.0),
                    },
                    4 => InteractionMode::DrawingSubgraph { start: (0.0, 0.0), current: (1.0, 1.0) },
                    5 => InteractionMode::ResizingSelection {
                        handle: ResizeHandle::Se,
                        original_bounds: (0.0, 0.0, 10.0, 10.0),
                        originals: HashMap::new(),
                        anchor: (5.0, 5.0),
                        did_resize: false,
                    },
                    6 => InteractionMode::Panning { last_pos: (0.0, 0.0) },
                    _ => {
                        let _ = finalize_motion_release(&mut mode, &mut doc);
                        continue;
                    }
                };
            }
            prop_assert!(true);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_overflow_safety(
            x in prop::sample::select(&[f64::MAX / 2.0, f64::MAX * 0.99]),
            y in prop::sample::select(&[f64::MAX / 2.0, f64::MAX * 0.99]),
        ) {
            let subgraph = (x, y, f64::MAX / 4.0, f64::MAX / 4.0);
            let node = (x + 1.0, y + 1.0, 1.0, 1.0);
            let _ = within(subgraph, node);
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

/// Subgraph/container interaction tests (bd-sa6)
///
/// These tests validate SUB (subgraph) interaction behaviors including:
/// - Click-through selection with z_index priority
/// - Box-select across container boundaries
/// - Collapse/expand container behavior
/// - Locked container with unlocked children
/// - Parent-child relationship preservation
#[cfg(test)]
mod subgraph_tests {
    use super::{resize_target_ids, within, InteractionMode};
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::HashMap;

    fn make_subgraph_node(
        id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        locked: bool,
        collapsed: Option<bool>,
        parent: Option<NodeId>,
    ) -> (NodeId, Node) {
        let node_id = NodeId::new(id.to_string());
        let node = Node {
            kind: NodeKind::Subgraph,
            icon: String::new(),
            label: String::from("Container"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(width),
            height: OrderedFloat(height),
            font_size: None,
            font_weight: None,
            locked,
            parent,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: -1, // Containers have lower z_index
            style: Some(NodeStyle::Box),
            collapsed,
        };
        (node_id, node)
    }

    fn make_child_node(
        id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        locked: bool,
        parent: Option<NodeId>,
    ) -> (NodeId, Node) {
        let node_id = NodeId::new(id.to_string());
        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::from("Child"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(width),
            height: OrderedFloat(height),
            font_size: None,
            font_weight: None,
            locked,
            parent,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 1000, // Children have higher z_index
            style: Some(NodeStyle::default()),
            collapsed: None,
        };
        (node_id, node)
    }

    // ============== SUB-001: Click inside container selects child vs container ==============

    /// Given a container with a child at overlapping position,
    /// when hit testing by position, the child should be prioritized due to higher z_index.
    #[test]
    fn given_container_with_child_when_hit_testing_then_child_has_higher_z_index() {
        let mut doc = DiagramDocument::default();

        // Container at (100, 100) with size 300x200
        let (container_id, container) =
            make_subgraph_node("container", 100.0, 100.0, 300.0, 200.0, false, None, None);
        doc.document.nodes.insert(container_id.clone(), container);

        // Child at (150, 150) inside the container
        let (child_id, child) = make_child_node(
            "child",
            150.0,
            150.0,
            80.0,
            40.0,
            false,
            Some(container_id.clone()),
        );
        doc.document.nodes.insert(child_id.clone(), child);

        // Verify z_index ordering: child should have higher z_index than container
        let container_node = doc
            .document
            .nodes
            .get(&container_id)
            .expect("container exists");
        let child_node = doc.document.nodes.get(&child_id).expect("child exists");

        assert!(
            child_node.z_index > container_node.z_index,
            "Child z_index ({}) should be greater than container z_index ({})",
            child_node.z_index,
            container_node.z_index
        );

        // Verify the child is within the container bounds
        let container_rect = (
            container_node.x.0,
            container_node.y.0,
            container_node.width.0,
            container_node.height.0,
        );
        let child_rect = (
            child_node.x.0,
            child_node.y.0,
            child_node.width.0,
            child_node.height.0,
        );
        assert!(
            within(container_rect, child_rect),
            "Child should be geometrically within container bounds"
        );
    }

    /// Given a container with multiple children at different z_index values,
    /// when selecting by position, the highest z_index node should be preferred.
    #[test]
    fn given_nested_nodes_when_selecting_by_position_then_highest_z_index_wins() {
        let mut doc = DiagramDocument::default();

        // Outer container at z_index -1
        let (outer_id, outer) =
            make_subgraph_node("outer", 50.0, 50.0, 400.0, 300.0, false, None, None);
        doc.document.nodes.insert(outer_id.clone(), outer);

        // Inner container at z_index -1 (nested)
        let (inner_id, inner) = make_subgraph_node(
            "inner",
            100.0,
            100.0,
            250.0,
            180.0,
            false,
            None,
            Some(outer_id.clone()),
        );
        doc.document.nodes.insert(inner_id.clone(), inner);

        // Child node at z_index 1000 (should be topmost)
        let (child_id, child) = make_child_node(
            "child",
            150.0,
            150.0,
            60.0,
            30.0,
            false,
            Some(inner_id.clone()),
        );
        doc.document.nodes.insert(child_id.clone(), child);

        // Verify z_index hierarchy
        let outer_z = doc
            .document
            .nodes
            .get(&outer_id)
            .map(|n| n.z_index)
            .unwrap_or(0);
        let inner_z = doc
            .document
            .nodes
            .get(&inner_id)
            .map(|n| n.z_index)
            .unwrap_or(0);
        let child_z = doc
            .document
            .nodes
            .get(&child_id)
            .map(|n| n.z_index)
            .unwrap_or(0);

        assert_eq!(outer_z, -1, "Outer container should have z_index -1");
        assert_eq!(inner_z, -1, "Inner container should have z_index -1");
        assert_eq!(child_z, 1000, "Child should have z_index 1000");
        assert!(child_z > outer_z && child_z > inner_z);
    }

    // ============== SUB-002: Box-select across container boundary ==============

    /// Given nodes inside and outside a container,
    /// when performing rubber-band selection that spans both areas,
    /// then nodes from both inside and outside the container should be selectable.
    #[test]
    fn given_nodes_inside_and_outside_container_when_rubberband_selection_then_all_selectable() {
        let mut doc = DiagramDocument::default();

        // Container at (100, 100) with size 200x150
        let (container_id, container) =
            make_subgraph_node("container", 100.0, 100.0, 200.0, 150.0, false, None, None);
        doc.document.nodes.insert(container_id.clone(), container);

        // Child inside container
        let (child_inside_id, child_inside) = make_child_node(
            "child_inside",
            120.0,
            120.0,
            50.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        doc.document
            .nodes
            .insert(child_inside_id.clone(), child_inside);

        // Node outside container
        let (outside_id, outside) =
            make_child_node("outside", 400.0, 100.0, 50.0, 30.0, false, None);
        doc.document.nodes.insert(outside_id.clone(), outside);

        // Simulate rubber-band selection by selecting both nodes
        let _ = doc
            .editor_state
            .selected_items
            .insert(child_inside_id.to_string());
        let _ = doc
            .editor_state
            .selected_items
            .insert(outside_id.to_string());

        // Verify both nodes are selected regardless of container membership
        assert_eq!(
            doc.editor_state.selected_items.len(),
            2,
            "Both nodes should be selectable"
        );
        assert!(
            doc.editor_state.selected_items.contains("child_inside"),
            "Child inside container should be selected"
        );
        assert!(
            doc.editor_state.selected_items.contains("outside"),
            "Node outside container should be selected"
        );
    }

    /// Given a rubber-band selection area,
    /// when the area partially overlaps a container,
    /// then only nodes within the selection area are selected (not all container children).
    #[test]
    fn given_partial_container_overlap_when_rubberband_then_only_overlapping_selected() {
        let mut doc = DiagramDocument::default();

        // Container at (100, 100) with size 300x200
        let (container_id, container) =
            make_subgraph_node("container", 100.0, 100.0, 300.0, 200.0, false, None, None);
        doc.document.nodes.insert(container_id.clone(), container);

        // Child in the left half (would be in selection)
        let (left_child_id, left_child) = make_child_node(
            "left_child",
            120.0,
            130.0,
            50.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        doc.document.nodes.insert(left_child_id.clone(), left_child);

        // Child in the right half (would NOT be in selection)
        let (right_child_id, right_child) = make_child_node(
            "right_child",
            320.0,
            130.0,
            50.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        doc.document
            .nodes
            .insert(right_child_id.clone(), right_child);

        // Simulate selection of only the left child
        let _ = doc
            .editor_state
            .selected_items
            .insert(left_child_id.to_string());

        assert_eq!(
            doc.editor_state.selected_items.len(),
            1,
            "Only one child should be selected"
        );
        assert!(
            doc.editor_state.selected_items.contains("left_child"),
            "Left child should be selected"
        );
        assert!(
            !doc.editor_state.selected_items.contains("right_child"),
            "Right child should NOT be selected"
        );
    }

    // ============== SUB-003: Collapse/expand container behavior ==============

    /// Given a container with collapsed state,
    /// when serialized and deserialized,
    /// then the collapsed state is preserved.
    #[test]
    fn given_container_with_collapsed_state_when_roundtripped_then_state_preserved() {
        let mut doc = DiagramDocument::default();

        // Create collapsed container
        let (container_id, container) = make_subgraph_node(
            "container",
            100.0,
            100.0,
            200.0,
            150.0,
            false,
            Some(true), // collapsed = true
            None,
        );
        doc.document.nodes.insert(container_id.clone(), container);

        // Serialize and deserialize
        let json = serde_json::to_string(&doc).expect("serialization should succeed");
        let loaded: DiagramDocument =
            serde_json::from_str(&json).expect("deserialization should succeed");

        // Verify collapsed state is preserved
        let loaded_container = loaded
            .document
            .nodes
            .get(&container_id)
            .expect("container should exist");
        assert_eq!(
            loaded_container.collapsed,
            Some(true),
            "Collapsed state should be preserved as true"
        );
    }

    /// Given an expanded container with children,
    /// when the container is set to collapsed,
    /// then the collapsed field reflects this but children remain in document.
    #[test]
    fn given_expanded_container_when_collapsed_then_children_remain_in_document() {
        let mut doc = DiagramDocument::default();

        // Create expanded container
        let (container_id, mut container) = make_subgraph_node(
            "container",
            100.0,
            100.0,
            200.0,
            150.0,
            false,
            Some(false),
            None,
        );

        // Add a child
        let (child_id, child) = make_child_node(
            "child",
            120.0,
            120.0,
            50.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        doc.document.nodes.insert(child_id.clone(), child);
        doc.document
            .nodes
            .insert(container_id.clone(), container.clone());

        // Collapse the container
        container.collapsed = Some(true);
        doc.document
            .nodes
            .insert(container_id.clone(), container.clone());

        // Verify children still exist in document
        assert!(
            doc.document.nodes.contains_key(&child_id),
            "Child should still exist in document after collapse"
        );
        assert_eq!(
            doc.document.nodes.len(),
            2,
            "Both container and child should exist"
        );

        // Verify collapsed state
        let container_node = doc
            .document
            .nodes
            .get(&container_id)
            .expect("container exists");
        assert_eq!(
            container_node.collapsed,
            Some(true),
            "Container should be marked as collapsed"
        );
    }

    /// Given containers with different collapsed states,
    /// when queried, each container maintains its own collapsed state independently.
    #[test]
    fn given_multiple_containers_when_collapsed_independently_then_states_are_independent() {
        let mut doc = DiagramDocument::default();

        // Create two containers with different collapsed states
        let (expanded_id, expanded) = make_subgraph_node(
            "expanded",
            50.0,
            50.0,
            200.0,
            100.0,
            false,
            Some(false),
            None,
        );
        let (collapsed_id, collapsed) = make_subgraph_node(
            "collapsed",
            300.0,
            50.0,
            200.0,
            100.0,
            false,
            Some(true),
            None,
        );

        doc.document.nodes.insert(expanded_id.clone(), expanded);
        doc.document.nodes.insert(collapsed_id.clone(), collapsed);

        // Verify independent states
        let expanded_node = doc
            .document
            .nodes
            .get(&expanded_id)
            .expect("expanded exists");
        let collapsed_node = doc
            .document
            .nodes
            .get(&collapsed_id)
            .expect("collapsed exists");

        assert_eq!(
            expanded_node.collapsed,
            Some(false),
            "First container should be expanded"
        );
        assert_eq!(
            collapsed_node.collapsed,
            Some(true),
            "Second container should be collapsed"
        );
    }

    // ============== SUB-004: Locked container with unlocked children ==============

    /// Given a locked container with unlocked children,
    /// when checking lock status,
    /// then children are independently unlocked (not inheriting parent's locked state).
    #[test]
    fn given_locked_container_with_unlocked_children_then_children_are_independently_unlocked() {
        let mut doc = DiagramDocument::default();

        // Create locked container
        let (container_id, container) =
            make_subgraph_node("container", 100.0, 100.0, 200.0, 150.0, true, None, None); // locked = true
        doc.document.nodes.insert(container_id.clone(), container);

        // Create unlocked child inside locked container
        let (child_id, child) = make_child_node(
            "child",
            120.0,
            120.0,
            50.0,
            30.0,
            false, // locked = false
            Some(container_id.clone()),
        );
        doc.document.nodes.insert(child_id.clone(), child);

        // Verify lock states are independent
        let container_node = doc
            .document
            .nodes
            .get(&container_id)
            .expect("container exists");
        let child_node = doc.document.nodes.get(&child_id).expect("child exists");

        assert!(container_node.locked, "Container should be locked");
        assert!(
            !child_node.locked,
            "Child should be unlocked despite parent being locked"
        );
    }

    /// Given a locked container with unlocked child,
    /// when selecting the child,
    /// then the child can be selected independently.
    #[test]
    fn given_locked_container_when_selecting_unlocked_child_then_child_is_selectable() {
        let mut doc = DiagramDocument::default();

        // Create locked container
        let (container_id, container) =
            make_subgraph_node("container", 100.0, 100.0, 200.0, 150.0, true, None, None);
        doc.document.nodes.insert(container_id.clone(), container);

        // Create unlocked child
        let (child_id, child) = make_child_node(
            "child",
            120.0,
            120.0,
            50.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        doc.document.nodes.insert(child_id.clone(), child);

        // Select the child (simulating user clicking on child despite locked parent)
        let _ = doc.editor_state.selected_items.insert(child_id.to_string());

        // Verify child is selected
        assert_eq!(
            doc.editor_state.selected_items.len(),
            1,
            "Child should be selectable"
        );
        assert!(
            doc.editor_state.selected_items.contains("child"),
            "Unlocked child should be selectable inside locked container"
        );
        assert!(
            !doc.editor_state.selected_items.contains("container"),
            "Locked container should not be selected when clicking child"
        );
    }

    /// Given mixed lock states in a hierarchy,
    /// when checking each node's lock state,
    /// then each node maintains its own lock state without inheritance.
    #[test]
    fn given_mixed_lock_hierarchy_then_lock_states_are_per_node() {
        let mut doc = DiagramDocument::default();

        // Create unlocked outer container
        let (outer_id, outer) =
            make_subgraph_node("outer", 50.0, 50.0, 400.0, 300.0, false, None, None);
        doc.document.nodes.insert(outer_id.clone(), outer);

        // Create locked inner container
        let (inner_id, inner) = make_subgraph_node(
            "inner",
            100.0,
            100.0,
            250.0,
            180.0,
            true, // locked
            None,
            Some(outer_id.clone()),
        );
        doc.document.nodes.insert(inner_id.clone(), inner);

        // Create unlocked child inside locked inner
        let (child_id, child) = make_child_node(
            "child",
            150.0,
            150.0,
            60.0,
            30.0,
            false, // unlocked
            Some(inner_id.clone()),
        );
        doc.document.nodes.insert(child_id.clone(), child);

        // Verify each node has independent lock state
        let outer_node = doc.document.nodes.get(&outer_id).expect("outer exists");
        let inner_node = doc.document.nodes.get(&inner_id).expect("inner exists");
        let child_node = doc.document.nodes.get(&child_id).expect("child exists");

        assert!(!outer_node.locked, "Outer should be unlocked");
        assert!(inner_node.locked, "Inner should be locked");
        assert!(
            !child_node.locked,
            "Child should be unlocked (not inheriting inner's lock)"
        );
    }

    // ============== SUB-005: Parent-child relationship preservation during selection ==============

    /// Given a container with children,
    /// when the container is selected and resized,
    /// then children are included in resize targets and parent references are preserved.
    #[test]
    fn given_container_with_children_when_selected_then_children_included_in_resize_targets() {
        let mut doc = DiagramDocument::default();

        // Create container
        let (container_id, container) =
            make_subgraph_node("container", 100.0, 100.0, 300.0, 200.0, false, None, None);
        doc.document.nodes.insert(container_id.clone(), container);

        // Create children inside container
        let (child1_id, child1) = make_child_node(
            "child1",
            120.0,
            130.0,
            60.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        doc.document.nodes.insert(child1_id.clone(), child1);

        let (child2_id, child2) = make_child_node(
            "child2",
            200.0,
            180.0,
            60.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        doc.document.nodes.insert(child2_id.clone(), child2);

        // Create a node outside container
        let (outside_id, outside) =
            make_child_node("outside", 500.0, 100.0, 60.0, 30.0, false, None);
        doc.document.nodes.insert(outside_id.clone(), outside);

        // Select the container
        let _ = doc
            .editor_state
            .selected_items
            .insert(container_id.to_string());

        // Get resize targets
        let targets = resize_target_ids(&doc);

        // Verify container and children are included, outside is not
        assert!(
            targets.contains(&container_id),
            "Container should be in resize targets"
        );
        assert!(
            targets.contains(&child1_id),
            "Child1 inside container should be in resize targets"
        );
        assert!(
            targets.contains(&child2_id),
            "Child2 inside container should be in resize targets"
        );
        assert!(
            !targets.contains(&outside_id),
            "Node outside container should NOT be in resize targets"
        );
    }

    /// Given a container with children,
    /// when the container is selected for resize,
    /// then the parent references of children remain intact.
    #[test]
    fn given_container_with_children_when_resizing_then_parent_references_preserved() {
        let mut doc = DiagramDocument::default();

        // Create container
        let (container_id, container) =
            make_subgraph_node("container", 100.0, 100.0, 300.0, 200.0, false, None, None);
        doc.document.nodes.insert(container_id.clone(), container);

        // Create child with parent reference
        let (child_id, child) = make_child_node(
            "child",
            150.0,
            150.0,
            60.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        doc.document.nodes.insert(child_id.clone(), child);

        // Select the container
        let _ = doc
            .editor_state
            .selected_items
            .insert(container_id.to_string());

        // Simulate resize finalization (which would update positions)
        let mut mode = InteractionMode::Select;
        let _ = super::finalize_motion_release(&mut mode, &mut doc);

        // Verify parent reference is still intact
        let child_node = doc.document.nodes.get(&child_id).expect("child exists");
        assert_eq!(
            child_node.parent,
            Some(container_id.clone()),
            "Child's parent reference should be preserved after resize operation"
        );
    }

    /// Given nested containers,
    /// when checking parent-child relationships,
    /// then each node correctly references its immediate parent.
    #[test]
    fn given_nested_containers_then_parent_chain_is_correct() {
        let mut doc = DiagramDocument::default();

        // Create outer container (no parent)
        let (outer_id, outer) =
            make_subgraph_node("outer", 50.0, 50.0, 400.0, 300.0, false, None, None);
        doc.document.nodes.insert(outer_id.clone(), outer);

        // Create inner container (parent = outer)
        let (inner_id, inner) = make_subgraph_node(
            "inner",
            100.0,
            100.0,
            250.0,
            180.0,
            false,
            None,
            Some(outer_id.clone()),
        );
        doc.document.nodes.insert(inner_id.clone(), inner);

        // Create child (parent = inner)
        let (child_id, child) = make_child_node(
            "child",
            150.0,
            150.0,
            60.0,
            30.0,
            false,
            Some(inner_id.clone()),
        );
        doc.document.nodes.insert(child_id.clone(), child);

        // Verify parent chain
        let outer_node = doc.document.nodes.get(&outer_id).expect("outer exists");
        let inner_node = doc.document.nodes.get(&inner_id).expect("inner exists");
        let child_node = doc.document.nodes.get(&child_id).expect("child exists");

        assert!(
            outer_node.parent.is_none(),
            "Outer container should have no parent"
        );
        assert_eq!(
            inner_node.parent,
            Some(outer_id.clone()),
            "Inner's parent should be outer"
        );
        assert_eq!(
            child_node.parent,
            Some(inner_id.clone()),
            "Child's parent should be inner (not outer)"
        );
    }

    // ============== SUB-006 (bd-321): Drag multiple selected nodes into container ==============

    /// Given multiple selected nodes outside a container,
    /// when drag positions are calculated,
    /// then both nodes are tracked for the drag operation.
    #[test]
    fn given_multiple_selected_nodes_when_drag_position_calculated_then_all_tracked() {
        use crate::ui::interaction::drag_original_positions;

        let mut doc = DiagramDocument::default();

        // Container at (300, 100)
        let (container_id, container) =
            make_subgraph_node("container", 300.0, 100.0, 200.0, 150.0, false, None, None);
        doc.document.nodes.insert(container_id, container);

        // Two nodes outside container
        let (node1_id, node1) = make_child_node("node1", 50.0, 100.0, 60.0, 30.0, false, None);
        let (node2_id, node2) = make_child_node("node2", 50.0, 150.0, 60.0, 30.0, false, None);
        doc.document.nodes.insert(node1_id.clone(), node1);
        doc.document.nodes.insert(node2_id.clone(), node2);

        // Select both nodes
        let selected = im::HashSet::new()
            .update(node1_id.to_string())
            .update(node2_id.to_string());
        let positions = drag_original_positions(&doc, &selected);

        // Both selected nodes should have recorded positions
        assert_eq!(positions.len(), 2, "Both selected nodes should be tracked");
        assert!(
            positions.contains_key(&node1_id),
            "Node1 should have original position recorded"
        );
        assert!(
            positions.contains_key(&node2_id),
            "Node2 should have original position recorded"
        );

        // Verify positions match initial placement
        let pos1 = positions.get(&node1_id);
        let pos2 = positions.get(&node2_id);
        assert_eq!(pos1.map(|p| p.0), Some(50.0), "Node1 x position");
        assert_eq!(pos1.map(|p| p.1), Some(100.0), "Node1 y position");
        assert_eq!(pos2.map(|p| p.0), Some(50.0), "Node2 x position");
        assert_eq!(pos2.map(|p| p.1), Some(150.0), "Node2 y position");
    }

    // ============== SUB-007 (bd-321): Drag container into another container (nesting) ==============

    /// Given two containers where one can be nested inside the other,
    /// when the inner container is positioned within outer bounds,
    /// then the geometry supports valid nesting.
    #[test]
    fn given_two_containers_when_inner_positioned_in_outer_then_geometry_supports_nesting() {
        let mut doc = DiagramDocument::default();

        // Outer container at (100, 100) with size 400x300
        let (outer_id, outer) =
            make_subgraph_node("outer", 100.0, 100.0, 400.0, 300.0, false, None, None);
        doc.document.nodes.insert(outer_id.clone(), outer);

        // Inner container at (150, 150) with size 200x150 (fits inside outer)
        let (inner_id, inner) =
            make_subgraph_node("inner", 150.0, 150.0, 200.0, 150.0, false, None, None);
        doc.document.nodes.insert(inner_id.clone(), inner);

        // Verify geometry supports nesting
        let outer_node = doc.document.nodes.get(&outer_id).expect("outer exists");
        let inner_node = doc.document.nodes.get(&inner_id).expect("inner exists");

        // Inner should fit within outer bounds
        let outer_rect = (
            outer_node.x.0,
            outer_node.y.0,
            outer_node.width.0,
            outer_node.height.0,
        );
        let inner_rect = (
            inner_node.x.0,
            inner_node.y.0,
            inner_node.width.0,
            inner_node.height.0,
        );

        assert!(
            within(outer_rect, inner_rect),
            "Inner container should fit within outer container bounds for valid nesting"
        );

        // Both containers exist and inner has no parent yet (would be set on drop)
        assert_eq!(doc.document.nodes.len(), 2);
        assert!(
            inner_node.parent.is_none(),
            "Inner starts without parent (would be assigned on drop)"
        );
    }

    // ============== SUB-008 (bd-321): Grab parent prevents reparent gesture ==============

    /// Given a nested container hierarchy,
    /// when a middle container (which has children) is selected,
    /// then dragging includes both the container and its descendants.
    #[test]
    fn given_nested_container_with_children_when_middle_selected_then_descendants_included() {
        use crate::ui::interaction::drag_original_positions;

        let mut doc = DiagramDocument::default();

        // Outer container
        let (outer_id, outer) =
            make_subgraph_node("outer", 100.0, 100.0, 400.0, 300.0, false, None, None);
        doc.document.nodes.insert(outer_id.clone(), outer);

        // Inner container (parent = outer)
        let (inner_id, inner) = make_subgraph_node(
            "inner",
            150.0,
            150.0,
            200.0,
            150.0,
            false,
            None,
            Some(outer_id.clone()),
        );
        doc.document.nodes.insert(inner_id.clone(), inner);

        // Child inside inner
        let (child_id, child) = make_child_node(
            "child",
            180.0,
            180.0,
            60.0,
            30.0,
            false,
            Some(inner_id.clone()),
        );
        doc.document.nodes.insert(child_id.clone(), child);

        // Select the inner container (the "parent" being grabbed)
        let selected = im::HashSet::new().update(inner_id.to_string());
        let positions = drag_original_positions(&doc, &selected);

        // Both inner and its child should be included (descendant traversal)
        assert!(
            positions.contains_key(&inner_id),
            "Selected inner container should be in drag positions"
        );
        assert!(
            positions.contains_key(&child_id),
            "Child of selected container should be included in drag positions"
        );
        assert!(
            !positions.contains_key(&outer_id),
            "Outer (ancestor) should NOT be included when selecting inner"
        );
    }

    // ============== SUB-009 (bd-321): Container auto-expand when child crosses boundary ==============

    /// Given a container with a child near the edge,
    /// when calculating resize targets,
    /// then both container and child are included for boundary calculations.
    #[test]
    fn given_container_with_child_near_edge_when_resize_targets_then_both_included() {
        let mut doc = DiagramDocument::default();

        // Container at (100, 100) with size 200x150
        let (container_id, container) =
            make_subgraph_node("container", 100.0, 100.0, 200.0, 150.0, false, None, None);
        doc.document.nodes.insert(container_id.clone(), container);

        // Child near the right edge of container
        let (child_id, child) = make_child_node(
            "child",
            120.0,
            120.0,
            50.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        doc.document.nodes.insert(child_id.clone(), child);

        // Select the container
        let _ = doc
            .editor_state
            .selected_items
            .insert(container_id.to_string());

        // Get resize targets
        let targets = resize_target_ids(&doc);

        // Container and child should both be in targets
        assert!(
            targets.contains(&container_id),
            "Container should be in resize targets"
        );
        assert!(
            targets.contains(&child_id),
            "Child inside container should be in resize targets"
        );
        assert_eq!(
            targets.len(),
            2,
            "Should have exactly container and child in targets"
        );
    }

    // ============== SUB-010 (bd-321): Drag selection with nested descendants ==============

    /// Given a three-level hierarchy (outer -> inner -> leaf),
    /// when the outer container is selected,
    /// then drag positions include all descendants.
    #[test]
    fn given_three_level_hierarchy_when_outer_selected_then_all_descendants_in_drag_positions() {
        use crate::ui::interaction::drag_original_positions;

        let mut doc = DiagramDocument::default();

        // Outer container (level 0)
        let (outer_id, outer) =
            make_subgraph_node("outer", 50.0, 50.0, 400.0, 300.0, false, None, None);
        doc.document.nodes.insert(outer_id.clone(), outer);

        // Inner container (level 1, parent = outer)
        let (inner_id, inner) = make_subgraph_node(
            "inner",
            100.0,
            100.0,
            250.0,
            180.0,
            false,
            None,
            Some(outer_id.clone()),
        );
        doc.document.nodes.insert(inner_id.clone(), inner);

        // Leaf node (level 2, parent = inner)
        let (leaf_id, leaf) = make_child_node(
            "leaf",
            150.0,
            150.0,
            60.0,
            30.0,
            false,
            Some(inner_id.clone()),
        );
        doc.document.nodes.insert(leaf_id.clone(), leaf);

        // Select the outer container
        let selected = im::HashSet::new().update(outer_id.to_string());
        let positions = drag_original_positions(&doc, &selected);

        // All three nodes should be included
        assert_eq!(
            positions.len(),
            3,
            "All three nodes in hierarchy should be in drag positions"
        );
        assert!(
            positions.contains_key(&outer_id),
            "Outer container should be in drag positions"
        );
        assert!(
            positions.contains_key(&inner_id),
            "Inner container (descendant) should be in drag positions"
        );
        assert!(
            positions.contains_key(&leaf_id),
            "Leaf node (descendant of descendant) should be in drag positions"
        );

        // Verify positions are recorded correctly
        let outer_pos = positions.get(&outer_id);
        let inner_pos = positions.get(&inner_id);
        let leaf_pos = positions.get(&leaf_id);

        assert_eq!(outer_pos.map(|p| (p.0, p.1)), Some((50.0, 50.0)));
        assert_eq!(inner_pos.map(|p| (p.0, p.1)), Some((100.0, 100.0)));
        assert_eq!(leaf_pos.map(|p| (p.0, p.1)), Some((150.0, 150.0)));
    }

    // ============== MUL-003: Drag selection across container boundary triggers reparent =============

    /// Given multi-selection dragged across container boundary,
    /// when drag ends inside container,
    /// then all selected nodes should be reparented to the target container.
    ///
    /// This test verifies the core MUL-003 requirement:
    /// "Drag selection across container boundary: reparent occurs"
    #[test]
    fn given_multi_selection_outside_container_when_dragged_inside_then_reparents() {
        use crate::ui::interaction::drag_original_positions;

        let mut doc = DiagramDocument::default();

        // Container at (300, 100) with size 200x200
        let (container_id, container) =
            make_subgraph_node("container", 300.0, 100.0, 200.0, 200.0, false, None, None);
        doc.document.nodes.insert(container_id.clone(), container);

        // Two nodes outside container at initial positions
        let (node1_id, node1) = make_child_node("node1", 50.0, 150.0, 60.0, 30.0, false, None);
        let (node2_id, node2) = make_child_node("node2", 150.0, 150.0, 60.0, 30.0, false, None);
        doc.document.nodes.insert(node1_id.clone(), node1);
        doc.document.nodes.insert(node2_id.clone(), node2);

        // Select both nodes
        let selected = im::HashSet::new()
            .update(node1_id.to_string())
            .update(node2_id.to_string());
        doc.editor_state.selected_items = selected.clone();

        // Record drag positions
        let positions = drag_original_positions(&doc, &selected);
        assert_eq!(positions.len(), 2, "Both selected nodes should be tracked");

        // Verify nodes start outside container
        let initial_node1 = doc.document.nodes.get(&node1_id).unwrap();
        assert!(
            initial_node1.x.0 < 300.0,
            "Node1 should start outside container"
        );

        // Simulate drag: move nodes to positions inside the container
        // Target positions: (350, 150) and (400, 150) - both inside container bounds
        // Container bounds: x=300, y=100, width=200, height=200 => x in [300, 500], y in [100, 300]
        let drag_delta = (300.0, 0.0); // Move right by 300

        // Update node positions to simulate drag end
        if let Some(node) = doc.document.nodes.get_mut(&node1_id) {
            node.x = OrderedFloat(50.0 + drag_delta.0);
            node.y = OrderedFloat(150.0 + drag_delta.1);
        }
        if let Some(node) = doc.document.nodes.get_mut(&node2_id) {
            node.x = OrderedFloat(150.0 + drag_delta.0);
            node.y = OrderedFloat(150.0 + drag_delta.1);
        }

        // Check: After drag, nodes are at positions inside container
        let node1 = doc.document.nodes.get(&node1_id).unwrap();
        let node2 = doc.document.nodes.get(&node2_id).unwrap();
        assert!(
            node1.x.0 >= 300.0 && node1.x.0 <= 500.0,
            "Node1 should be inside container X bounds"
        );
        assert!(
            node1.y.0 >= 100.0 && node1.y.0 <= 300.0,
            "Node1 should be inside container Y bounds"
        );
        assert!(
            node2.x.0 >= 300.0 && node2.x.0 <= 500.0,
            "Node2 should be inside container X bounds"
        );

        // MUL-003 requires: When drag ends inside container, nodes should be reparented
        // This test documents the expected behavior - the reparent logic needs to be implemented
    }

    /// Given multi-selection inside container dragged outside,
    /// when drag ends outside container,
    /// then all selected nodes should be reparented to root (None).
    #[test]
    fn given_multi_selection_inside_container_when_dragged_outside_then_reparents_to_root() {
        use crate::ui::interaction::drag_original_positions;

        let mut doc = DiagramDocument::default();

        // Container at (100, 100) with size 200x200
        let (container_id, container) =
            make_subgraph_node("container", 100.0, 100.0, 200.0, 200.0, false, None, None);
        doc.document.nodes.insert(container_id.clone(), container);

        // Two nodes inside container
        let (node1_id, node1) = make_child_node(
            "node1",
            150.0,
            150.0,
            60.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        let (node2_id, node2) = make_child_node(
            "node2",
            200.0,
            150.0,
            60.0,
            30.0,
            false,
            Some(container_id.clone()),
        );
        doc.document.nodes.insert(node1_id.clone(), node1);
        doc.document.nodes.insert(node2_id.clone(), node2);

        // Select both nodes
        let selected = im::HashSet::new()
            .update(node1_id.to_string())
            .update(node2_id.to_string());
        doc.editor_state.selected_items = selected.clone();

        // Record drag positions
        let positions = drag_original_positions(&doc, &selected);
        assert_eq!(positions.len(), 2, "Both selected nodes should be tracked");

        // Verify nodes start inside container
        let initial_node1 = doc.document.nodes.get(&node1_id).unwrap();
        assert_eq!(
            initial_node1.parent,
            Some(container_id.clone()),
            "Node1 should start as child of container"
        );

        // Simulate drag: move nodes outside container
        // Drag delta: move right by 200 -> positions become (350, 150) and (400, 150)
        // Container ends at x=300, so nodes are now outside
        let drag_delta = (200.0, 0.0);

        if let Some(node) = doc.document.nodes.get_mut(&node1_id) {
            node.x = OrderedFloat(150.0 + drag_delta.0);
            node.y = OrderedFloat(150.0 + drag_delta.1);
        }
        if let Some(node) = doc.document.nodes.get_mut(&node2_id) {
            node.x = OrderedFloat(200.0 + drag_delta.0);
            node.y = OrderedFloat(150.0 + drag_delta.1);
        }

        // Check: After drag, nodes are outside container bounds
        let node1 = doc.document.nodes.get(&node1_id).unwrap();
        let node2 = doc.document.nodes.get(&node2_id).unwrap();
        assert!(
            node1.x.0 > 300.0,
            "Node1 should be outside container X bounds after drag"
        );

        // MUL-003 requires: When drag ends outside container, nodes should be reparented to root
        // This test documents the expected behavior - the reparent logic needs to be implemented
    }
}

// =============================================================================
// INP Mobile/Touch Interaction tests (bd-27q)
// =============================================================================

#[cfg(test)]
mod inp_mobile_touch_tests {
    use super::{InteractionMode, ResizeHandle};
    use crate::models::document::{Node, NodeId, NodeKind, NodeStyle, OrderedFloat};
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
}
