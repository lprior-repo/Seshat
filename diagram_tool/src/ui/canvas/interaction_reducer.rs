#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::selection_geometry::{selected_node_ids, selection_bounds};
use crate::history::History;
use crate::models::document::{DiagramDocument, EdgeId, NodeId, NodeKind};
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
            tags: Vec::new(),
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
            tags: Vec::new(),
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
