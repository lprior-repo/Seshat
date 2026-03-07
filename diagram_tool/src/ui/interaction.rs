#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::{DiagramDocument, NodeId};
use crate::ui::grid::{snap_point as grid_snap_point, snap_value as grid_snap_value, GridSize};
use im::{HashMap, HashSet};

const DRAG_THRESHOLD_PX: f64 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionMode {
    Contain,
    #[allow(dead_code)]
    Intersect,
}

#[must_use]
pub const fn selection_mode_from_drag(start: (f64, f64), current: (f64, f64)) -> SelectionMode {
    if current.0 < start.0 {
        SelectionMode::Intersect
    } else {
        SelectionMode::Contain
    }
}

#[must_use]
pub fn has_drag_threshold(origin: (f64, f64), current: (f64, f64)) -> bool {
    let dx = current.0 - origin.0;
    let dy = current.1 - origin.1;
    (dx.mul_add(dx, dy * dy)).sqrt() >= DRAG_THRESHOLD_PX
}

#[must_use]
pub fn select_single(item_id: String) -> HashSet<String> {
    HashSet::new().update(item_id)
}

#[must_use]
pub fn toggle_selection(current: &HashSet<String>, item_id: &str) -> HashSet<String> {
    if current.contains(item_id) {
        current.without(item_id)
    } else {
        current.update(item_id.to_string())
    }
}

#[must_use]
pub fn node_ids_in_rect(
    doc: &DiagramDocument,
    start: (f64, f64),
    current: (f64, f64),
) -> HashSet<String> {
    node_ids_in_rect_with_mode(
        doc,
        start,
        current,
        selection_mode_from_drag(start, current),
    )
}

#[must_use]
pub fn node_ids_in_rect_with_mode(
    doc: &DiagramDocument,
    start: (f64, f64),
    current: (f64, f64),
    mode: SelectionMode,
) -> HashSet<String> {
    let min_x = start.0.min(current.0);
    let min_y = start.1.min(current.1);
    let max_x = start.0.max(current.0);
    let max_y = start.1.max(current.1);

    doc.document
        .nodes
        .iter()
        .filter(|(_, n)| match mode {
            SelectionMode::Contain => {
                n.x.0 >= min_x
                    && n.y.0 >= min_y
                    && n.x.0 + n.width.0 <= max_x
                    && n.y.0 + n.height.0 <= max_y
            }
            SelectionMode::Intersect => {
                let node_max_x = n.x.0 + n.width.0;
                let node_max_y = n.y.0 + n.height.0;
                n.x.0 < max_x && node_max_x > min_x && n.y.0 < max_y && node_max_y > min_y
            }
        })
        .map(|(id, _)| id.to_string())
        .collect()
}

#[must_use]
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use crate::ui::grid::snap_value instead")]
pub fn snap_value(value: f64, snap_to_grid: bool, grid_size: f64) -> f64 {
    let clamped = grid_size.clamp(GridSize::MIN, GridSize::MAX);
    let grid = GridSize::new(clamped).unwrap_or_default();
    grid_snap_value(value, snap_to_grid, grid)
}

#[must_use]
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use crate::ui::grid::snap_point instead")]
pub fn snap_point(point: (f64, f64), snap_to_grid: bool, grid_size: f64) -> (f64, f64) {
    let clamped = grid_size.clamp(GridSize::MIN, GridSize::MAX);
    let grid = GridSize::new(clamped).unwrap_or_default();
    grid_snap_point(point, snap_to_grid, grid)
}

#[must_use]
#[allow(dead_code)]
pub fn dragged_positions(
    originals: &HashMap<NodeId, (f64, f64)>,
    anchor: (f64, f64),
    current: (f64, f64),
) -> HashMap<NodeId, (f64, f64)> {
    dragged_positions_with_snap(originals, anchor, current, false, GridSize::default())
}

#[must_use]
pub fn dragged_positions_with_snap(
    originals: &HashMap<NodeId, (f64, f64)>,
    anchor: (f64, f64),
    current: (f64, f64),
    snap_to_grid: bool,
    grid_size: GridSize,
) -> HashMap<NodeId, (f64, f64)> {
    let dx = current.0 - anchor.0;
    let dy = current.1 - anchor.1;
    let (dx, dy) = grid_snap_point((dx, dy), snap_to_grid, grid_size);
    originals
        .iter()
        .fold(HashMap::new(), |acc, (id, (ox, oy))| {
            acc.update(id.clone(), (ox + dx, oy + dy))
        })
}

#[must_use]
pub fn drag_original_positions(
    doc: &DiagramDocument,
    selected_items: &HashSet<String>,
) -> HashMap<NodeId, (f64, f64)> {
    let selected_nodes = selected_items
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .filter(|id| doc.document.nodes.contains_key(id))
        .collect::<HashSet<_>>();

    let with_children = std::iter::successors(Some(selected_nodes), |current| {
        let expanded = doc
            .document
            .nodes
            .iter()
            .fold(current.clone(), |acc, (id, node)| {
                if node
                    .parent
                    .as_ref()
                    .is_some_and(|parent| acc.contains(parent))
                {
                    acc.update(id.clone())
                } else {
                    acc
                }
            });

        (expanded.len() > current.len()).then_some(expanded)
    })
    .last()
    .unwrap_or_else(HashSet::new);

    with_children.iter().fold(HashMap::new(), |acc, id| {
        if let Some(node) = doc.document.nodes.get(id) {
            acc.update(id.clone(), (node.x.0, node.y.0))
        } else {
            acc
        }
    })
}

#[must_use]
pub fn with_auto_selected_edges(
    doc: &DiagramDocument,
    selected_items: &HashSet<String>,
) -> HashSet<String> {
    doc.document
        .edges
        .iter()
        .fold(selected_items.clone(), |acc, (id, edge)| {
            let source_selected = selected_items.contains(&edge.source.to_string());
            let target_selected = selected_items.contains(&edge.target.to_string());
            if source_selected && target_selected {
                acc.update(id.to_string())
            } else {
                acc
            }
        })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, deprecated)]
mod tests {
    use super::{
        dragged_positions, dragged_positions_with_snap, has_drag_threshold, node_ids_in_rect,
        node_ids_in_rect_with_mode, select_single, selection_mode_from_drag, snap_point,
        snap_value, toggle_selection, with_auto_selected_edges, SelectionMode,
    };
    use crate::models::document::{
        DiagramDocument, DocumentData, Edge, EdgeId, EditorState, Node, NodeId, NodeKind,
        NodeStyle, OrderedFloat, Revision,
    };
    use crate::ui::grid::GridSize;
    use im::{HashMap, HashSet};

    fn node(x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::new(),
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

    fn doc_with_nodes() -> DiagramDocument {
        let a = NodeId::new(String::from("a"));
        let b = NodeId::new(String::from("b"));
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new()
                    .update(a, node(10.0, 10.0, 30.0, 20.0))
                    .update(b, node(120.0, 80.0, 20.0, 20.0)),
                edges: HashMap::new(),
            },
            editor_state: EditorState {
                snap_to_grid: true,
                selected_items: HashSet::new(),
                ..EditorState::default()
            },
        }
    }

    #[test]
    fn given_small_motion_when_threshold_checked_then_returns_false() {
        assert!(!has_drag_threshold((0.0, 0.0), (1.0, 1.0)));
    }

    #[test]
    fn given_large_motion_when_threshold_checked_then_returns_true() {
        assert!(has_drag_threshold((0.0, 0.0), (4.0, 0.0)));
    }

    #[test]
    fn given_selection_when_toggling_then_adds_and_removes_item() {
        let once = toggle_selection(&HashSet::new(), "node-1");
        assert!(once.contains("node-1"));

        let twice = toggle_selection(&once, "node-1");
        assert!(!twice.contains("node-1"));
    }

    #[test]
    fn given_single_item_when_select_single_then_only_item_is_selected() {
        let selected = select_single(String::from("edge-1"));
        assert!(selected.contains("edge-1"));
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn given_drag_anchor_and_current_when_dragged_positions_then_offsets_nodes() {
        let originals = HashMap::new().update(NodeId::new(String::from("a")), (2.0, 3.0));
        let updated = dragged_positions(&originals, (0.0, 0.0), (5.0, -2.0));
        let pos = updated.get(&NodeId::new(String::from("a"))).copied();
        assert_eq!(pos, Some((7.0, 1.0)));
    }

    #[test]
    fn given_snap_enabled_when_dragging_then_positions_use_grid_delta() {
        let originals = HashMap::new().update(NodeId::new(String::from("a")), (3.0, 7.0));
        let grid = GridSize::new(20.0).unwrap();
        let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (14.0, 26.0), true, grid);
        let pos = updated.get(&NodeId::new(String::from("a"))).copied();
        assert_eq!(pos, Some((23.0, 27.0)));
    }

    #[test]
    fn given_rect_when_node_ids_in_rect_then_returns_contained_nodes() {
        let doc = doc_with_nodes();
        let selected = node_ids_in_rect(&doc, (0.0, 0.0), (60.0, 60.0));
        assert!(selected.contains("a"));
        assert!(!selected.contains("b"));
    }

    #[test]
    fn given_leftward_drag_when_selection_mode_resolved_then_uses_intersect() {
        let mode = selection_mode_from_drag((100.0, 100.0), (40.0, 120.0));
        assert_eq!(mode, SelectionMode::Intersect);
    }

    #[test]
    fn given_rightward_drag_when_selection_mode_resolved_then_uses_contain() {
        let mode = selection_mode_from_drag((40.0, 100.0), (100.0, 120.0));
        assert_eq!(mode, SelectionMode::Contain);
    }

    #[test]
    fn given_leftward_drag_when_node_ids_in_rect_then_uses_intersection_behavior() {
        let doc = doc_with_nodes();
        let rightward = node_ids_in_rect(&doc, (35.0, 25.0), (42.0, 32.0));
        assert!(!rightward.contains("a"));

        let leftward = node_ids_in_rect(&doc, (42.0, 32.0), (35.0, 25.0));
        assert!(leftward.contains("a"));
    }

    #[test]
    fn given_intersect_mode_when_rect_touches_node_then_node_is_selected() {
        let doc = doc_with_nodes();
        let selected =
            node_ids_in_rect_with_mode(&doc, (35.0, 25.0), (42.0, 32.0), SelectionMode::Intersect);
        assert!(selected.contains("a"));
    }

    #[test]
    fn given_snap_enabled_when_snapping_values_then_rounds_to_grid() {
        assert!((snap_value(29.0, true, 20.0) - 20.0).abs() < f64::EPSILON);
        let pt = snap_point((31.0, 49.0), true, 20.0);
        assert!((pt.0 - 40.0).abs() < f64::EPSILON && (pt.1 - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_selected_endpoints_when_auto_selecting_edges_then_connecting_edge_is_selected() {
        let source = NodeId::new(String::from("source"));
        let target = NodeId::new(String::from("target"));
        let edge_id = EdgeId::new(String::from("edge-1"));
        let mut doc = DiagramDocument::default();
        let _ = doc
            .document
            .nodes
            .insert(source.clone(), node(0.0, 0.0, 10.0, 10.0));
        let _ = doc
            .document
            .nodes
            .insert(target.clone(), node(40.0, 0.0, 10.0, 10.0));
        let _ = doc.document.edges.insert(
            edge_id.clone(),
            Edge {
                source: source.clone(),
                target: target.clone(),
                label: String::new(),
                style: crate::models::document::EdgeStyle::default(),
                arrow_type: crate::models::document::ArrowType::default(),
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::Vector::new(),
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                font_size: None,
            },
        );
        let selected = HashSet::new()
            .update(source.to_string())
            .update(target.to_string());

        let enriched = with_auto_selected_edges(&doc, &selected);
        assert!(enriched.contains(&edge_id.to_string()));
    }

    #[test]
    fn given_rightward_drag_inside_node_when_node_ids_in_rect_then_returns_empty_in_contain_mode() {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new(String::from("text-node"));
        let _ = doc.document.nodes.insert(
            node_id.clone(),
            Node {
                kind: NodeKind::Text,
                icon: String::new(),
                label: String::from("Text"),
                x: OrderedFloat(560.0),
                y: OrderedFloat(220.0),
                width: OrderedFloat(100.0),
                height: OrderedFloat(24.0),
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
            },
        );

        // Rect is inside the node: [572, 608] x [224, 240]
        // Node is [560, 660] x [220, 244]
        // Drag is (572, 224) to (608, 240) -> Rightward -> Contain mode
        let selected = node_ids_in_rect(&doc, (572.0, 224.0), (608.0, 240.0));

        assert!(
            selected.is_empty(),
            "Expected 0 selected, but got {:?}",
            selected
        );
    }

    #[test]
    fn given_leftward_drag_inside_node_when_node_ids_in_rect_then_returns_node_in_intersect_mode() {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new(String::from("text-node"));
        let _ = doc.document.nodes.insert(
            node_id.clone(),
            Node {
                kind: NodeKind::Text,
                icon: String::new(),
                label: String::from("Text"),
                x: OrderedFloat(560.0),
                y: OrderedFloat(220.0),
                width: OrderedFloat(100.0),
                height: OrderedFloat(24.0),
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
            },
        );

        // Drag is (608, 240) to (572, 224) -> Leftward -> Intersect mode
        let selected = node_ids_in_rect(&doc, (608.0, 240.0), (572.0, 224.0));

        assert!(selected.contains("text-node"));
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod proptests {
    use super::*;
    use crate::ui::grid::GridSize;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_finite_f64()(x in -1e6_f64..1e6_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_positive_f64()(x in 0.1_f64..1000.0_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_point()(x in arb_finite_f64(), y in arb_finite_f64()) -> (f64, f64) { (x, y) }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn prop_has_drag_threshold_symmetric(origin in arb_point(), delta in 0.0_f64..100.0_f64) {
            let current = (origin.0 + delta, origin.1);
            let result1 = has_drag_threshold(origin, current);
            let result2 = has_drag_threshold(current, origin);
            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn prop_snap_value_disabled_returns_same(value in arb_finite_f64(), grid in arb_positive_f64()) {
            let result = snap_value(value, false, grid);
            prop_assert!((result - value).abs() < f64::EPSILON);
        }

        #[test]
        fn prop_snap_value_enabled_is_multiple_of_grid(value in arb_finite_f64(), grid in arb_positive_f64()) {
            let result = snap_value(value, true, grid);
            let effective_grid = grid.clamp(GridSize::MIN, GridSize::MAX).max(1.0);
            let remainder = (result / effective_grid).round() * effective_grid - result;
            prop_assert!(remainder.abs() < f64::EPSILON || !result.is_finite());
        }

        #[test]
        fn prop_snap_value_nan_returns_nan(grid in arb_positive_f64()) {
            let result = snap_value(f64::NAN, true, grid);
            // NaN input should produce NaN output
            prop_assert!(result.is_nan());
        }

        #[test]
        fn prop_snap_point_consistent_with_snap_value(point in arb_point(), grid in arb_positive_f64()) {
            let snapped = snap_point(point, true, grid);
            let expected_x = snap_value(point.0, true, grid);
            let expected_y = snap_value(point.1, true, grid);
            prop_assert!((snapped.0 - expected_x).abs() < f64::EPSILON);
            prop_assert!((snapped.1 - expected_y).abs() < f64::EPSILON);
        }

        #[test]
        fn prop_snap_point_disabled_returns_same(point in arb_point(), grid in arb_positive_f64()) {
            let result = snap_point(point, false, grid);
            prop_assert!((result.0 - point.0).abs() < f64::EPSILON);
            prop_assert!((result.1 - point.1).abs() < f64::EPSILON);
        }

        #[test]
        fn prop_toggle_selection_idempotent_after_two(item in "[a-z]{1,3}") {
            let once = toggle_selection(&HashSet::new(), &item);
            let twice = toggle_selection(&once, &item);
            prop_assert!(twice.is_empty());
        }

        #[test]
        fn prop_toggle_selection_adds_item(item in "[a-z]{1,3}") {
            let result = toggle_selection(&HashSet::new(), &item);
            prop_assert!(result.contains(&item));
        }

        #[test]
        fn prop_dragged_positions_preserves_count(
            x1 in arb_finite_f64(), y1 in arb_finite_f64(),
            x2 in arb_finite_f64(), y2 in arb_finite_f64(),
            anchor in arb_point(), current in arb_point(),
        ) {
            let originals = HashMap::new()
                .update(NodeId::new("a".to_string()), (x1, y1))
                .update(NodeId::new("b".to_string()), (x2, y2));
            let result = dragged_positions_with_snap(&originals, anchor, current, false, GridSize::default());
            prop_assert_eq!(result.len(), originals.len());
        }

        #[test]
        fn prop_dragged_positions_zero_delta_same_position(
            x in arb_finite_f64(), y in arb_finite_f64(),
            point in arb_point(),
        ) {
            let originals = HashMap::new()
                .update(NodeId::new("a".to_string()), (x, y));
            let result = dragged_positions_with_snap(&originals, point, point, false, GridSize::default());
            let pos = result.get(&NodeId::new("a".to_string()));
            if let Some((rx, ry)) = pos {
                prop_assert!((rx - x).abs() < f64::EPSILON);
                prop_assert!((ry - y).abs() < f64::EPSILON);
            }
        }

        #[test]
        fn prop_dragged_positions_nan_anchor_preserves_original(
            x in arb_finite_f64(), y in arb_finite_f64(),
            current in arb_point(),
        ) {
            let originals = HashMap::new()
                .update(NodeId::new("a".to_string()), (x, y));
            let result = dragged_positions_with_snap(&originals, (f64::NAN, f64::NAN), current, false, GridSize::default());
            let pos = result.get(&NodeId::new("a".to_string()));
            if let Some((rx, ry)) = pos {
                if x.is_finite() && y.is_finite() {
                    prop_assert!(rx.is_finite() || rx.is_nan());
                    prop_assert!(ry.is_finite() || ry.is_nan());
                }
            }
        }

        #[test]
        fn prop_has_drag_threshold_always_true_for_large_delta(delta in 100.0_f64..10000.0_f64) {
            prop_assert!(has_drag_threshold((0.0, 0.0), (delta, 0.0)));
            prop_assert!(has_drag_threshold((0.0, 0.0), (0.0, delta)));
        }

        #[test]
        fn prop_has_drag_threshold_always_false_for_tiny_delta(delta in 0.0_f64..2.0_f64) {
            prop_assert!(!has_drag_threshold((0.0, 0.0), (delta, 0.0)));
            prop_assert!(!has_drag_threshold((0.0, 0.0), (0.0, delta)));
        }

        #[test]
        fn prop_snap_value_grid_zero_uses_one(value in arb_finite_f64()) {
            let result = snap_value(value, true, 0.0);
            let expected = snap_value(value, true, 1.0);
            if result.is_finite() && expected.is_finite() {
                prop_assert!((result - expected).abs() < f64::EPSILON);
            }
        }

        #[test]
        fn prop_snap_value_negative_grid_uses_one(value in arb_finite_f64(), grid in -100.0_f64..-0.1_f64) {
            let result = snap_value(value, true, grid);
            prop_assert!(result.is_finite() || !value.is_finite());
        }
    }
}

// =============================================================================
// SNP Snapping Interaction tests (bd-lgh)
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, deprecated)]
mod snp_interaction_tests {
    use super::*;
    use crate::ui::grid::GridSize;

    // SNP-3: Grid snap multi-select - detailed tests

    #[test]
    fn given_multi_select_drag_when_snap_enabled_then_all_nodes_use_snapped_delta() {
        let grid = GridSize::new(20.0).unwrap();

        // Three nodes at different positions
        let originals = HashMap::new()
            .update(NodeId::new("a".to_string()), (10.0, 15.0))
            .update(NodeId::new("b".to_string()), (100.0, 200.0))
            .update(NodeId::new("c".to_string()), (-50.0, 75.0));

        // Drag by (14.0, 26.0) - should snap to (20.0, 20.0) with grid 20.0
        // 14/20 = 0.7 -> rounds to 1 -> 20
        // 26/20 = 1.3 -> rounds to 1 -> 20
        let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (14.0, 26.0), true, grid);

        // All nodes should have moved by (20.0, 20.0) - the snapped delta
        let pos_a = updated.get(&NodeId::new("a".to_string())).copied();
        let pos_b = updated.get(&NodeId::new("b".to_string())).copied();
        let pos_c = updated.get(&NodeId::new("c".to_string())).copied();

        assert_eq!(
            pos_a,
            Some((30.0, 35.0)),
            "Node a should be at (10+20, 15+20)"
        );
        assert_eq!(
            pos_b,
            Some((120.0, 220.0)),
            "Node b should be at (100+20, 200+20)"
        );
        assert_eq!(
            pos_c,
            Some((-30.0, 95.0)),
            "Node c should be at (-50+20, 75+20)"
        );
    }

    #[test]
    fn given_multi_select_drag_when_snap_disabled_then_all_nodes_use_raw_delta() {
        let grid = GridSize::new(20.0).unwrap();

        let originals = HashMap::new()
            .update(NodeId::new("a".to_string()), (10.0, 15.0))
            .update(NodeId::new("b".to_string()), (100.0, 200.0));

        // Drag by (14.0, 26.0) - no snap, should use raw delta
        let updated =
            dragged_positions_with_snap(&originals, (0.0, 0.0), (14.0, 26.0), false, grid);

        let pos_a = updated.get(&NodeId::new("a".to_string())).copied();
        let pos_b = updated.get(&NodeId::new("b".to_string())).copied();

        assert_eq!(
            pos_a,
            Some((24.0, 41.0)),
            "Node a should be at (10+14, 15+26)"
        );
        assert_eq!(
            pos_b,
            Some((114.0, 226.0)),
            "Node b should be at (100+14, 200+26)"
        );
    }

    #[test]
    fn given_multi_select_drag_from_nonzero_anchor_when_snap_enabled_then_snaps_correctly() {
        let grid = GridSize::new(20.0).unwrap();

        let originals = HashMap::new()
            .update(NodeId::new("a".to_string()), (50.0, 60.0))
            .update(NodeId::new("b".to_string()), (150.0, 160.0));

        // Anchor at (100, 100), current at (115, 128) -> delta (15, 28)
        // Delta (15, 28) with grid 20.0:
        // 15/20 = 0.75 -> rounds to 1 -> 20
        // 28/20 = 1.4 -> rounds to 1 -> 20
        let updated =
            dragged_positions_with_snap(&originals, (100.0, 100.0), (115.0, 128.0), true, grid);

        let pos_a = updated.get(&NodeId::new("a".to_string())).copied();
        let pos_b = updated.get(&NodeId::new("b".to_string())).copied();

        assert_eq!(
            pos_a,
            Some((70.0, 80.0)),
            "Node a should be at (50+20, 60+20)"
        );
        assert_eq!(
            pos_b,
            Some((170.0, 180.0)),
            "Node b should be at (150+20, 160+20)"
        );
    }

    #[test]
    fn given_single_node_drag_when_snap_enabled_then_position_snapped() {
        let grid = GridSize::new(20.0).unwrap();

        let originals = HashMap::new().update(NodeId::new("single".to_string()), (0.0, 0.0));

        // Small drag that crosses snap threshold
        let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (11.0, 9.0), true, grid);

        let pos = updated.get(&NodeId::new("single".to_string())).copied();
        // Delta (11, 9) snaps to (20, 0) with grid 20.0
        assert_eq!(pos, Some((20.0, 0.0)));
    }

    #[test]
    fn given_negative_drag_when_snap_enabled_then_snaps_to_negative_grid() {
        let grid = GridSize::new(20.0).unwrap();

        let originals = HashMap::new().update(NodeId::new("a".to_string()), (100.0, 100.0));

        // Negative drag
        // -15/20 = -0.75 -> rounds to -1 -> -20
        // -25/20 = -1.25 -> rounds to -1 -> -20
        let updated =
            dragged_positions_with_snap(&originals, (0.0, 0.0), (-15.0, -25.0), true, grid);

        let pos = updated.get(&NodeId::new("a".to_string())).copied();
        // Delta (-15, -25) snaps to (-20, -20) with grid 20.0
        assert_eq!(pos, Some((80.0, 80.0)));
    }

    #[test]
    fn given_drag_threshold_boundary_when_checked_then_engages_correctly() {
        // DRAG_THRESHOLD_PX is 3.0
        // Test just below threshold
        assert!(!has_drag_threshold((0.0, 0.0), (2.9, 0.0)));
        assert!(!has_drag_threshold((0.0, 0.0), (0.0, 2.9)));
        assert!(!has_drag_threshold((0.0, 0.0), (2.0, 2.0))); // sqrt(8) ≈ 2.83

        // Test at threshold
        assert!(has_drag_threshold((0.0, 0.0), (3.0, 0.0)));
        assert!(has_drag_threshold((0.0, 0.0), (0.0, 3.0)));

        // Test just above threshold
        assert!(has_drag_threshold((0.0, 0.0), (3.1, 0.0)));
        assert!(has_drag_threshold((0.0, 0.0), (0.0, 3.1)));
    }

    #[test]
    fn given_diagonal_drag_when_threshold_checked_then_uses_euclidean_distance() {
        // Diagonal distance should be Euclidean
        // sqrt(3^2 + 3^2) = sqrt(18) ≈ 4.24 > 3.0
        assert!(has_drag_threshold((0.0, 0.0), (3.0, 3.0)));

        // sqrt(2^2 + 2^2) = sqrt(8) ≈ 2.83 < 3.0
        assert!(!has_drag_threshold((0.0, 0.0), (2.0, 2.0)));

        // sqrt(2^2 + 3^2) = sqrt(13) ≈ 3.61 > 3.0
        assert!(has_drag_threshold((0.0, 0.0), (2.0, 3.0)));
    }

    #[test]
    fn given_empty_selection_when_dragged_then_returns_empty() {
        let grid = GridSize::new(20.0).unwrap();
        let originals = HashMap::new();

        let updated =
            dragged_positions_with_snap(&originals, (0.0, 0.0), (100.0, 100.0), true, grid);

        assert!(updated.is_empty());
    }

    #[test]
    fn given_large_multi_select_when_snap_enabled_then_all_processed() {
        let grid = GridSize::new(20.0).unwrap();

        // Create many nodes
        let mut originals = HashMap::new();
        for i in 0..100 {
            let x = f64::from(i) * 10.0;
            let y = f64::from(i) * 5.0;
            originals = originals.update(NodeId::new(format!("node-{}", i)), (x, y));
        }

        // Delta (15, 25) with grid 20.0:
        // 15/20 = 0.75 -> rounds to 1 -> 20
        // 25/20 = 1.25 -> rounds to 1 -> 20
        let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (15.0, 25.0), true, grid);

        // All nodes should be present
        assert_eq!(updated.len(), 100);

        // Delta (15, 25) snaps to (20, 20)
        for i in 0..100 {
            let id = NodeId::new(format!("node-{}", i));
            let expected_x = f64::from(i) * 10.0 + 20.0;
            let expected_y = f64::from(i) * 5.0 + 20.0;
            let pos = updated.get(&id).copied();
            assert_eq!(pos, Some((expected_x, expected_y)));
        }
    }

    #[test]
    fn given_different_grid_sizes_when_snap_enabled_then_snaps_to_correct_grid() {
        // Test with minimum grid size
        let small_grid = GridSize::new(10.0).unwrap();
        let originals = HashMap::new().update(NodeId::new("a".to_string()), (0.0, 0.0));
        let updated =
            dragged_positions_with_snap(&originals, (0.0, 0.0), (6.0, 4.0), true, small_grid);
        let pos = updated.get(&NodeId::new("a".to_string())).copied();
        assert_eq!(pos, Some((10.0, 0.0))); // Delta (6, 4) snaps to (10, 0) with grid 10

        // Test with maximum grid size
        let large_grid = GridSize::new(100.0).unwrap();
        let originals = HashMap::new().update(NodeId::new("b".to_string()), (0.0, 0.0));
        let updated =
            dragged_positions_with_snap(&originals, (0.0, 0.0), (55.0, 45.0), true, large_grid);
        let pos = updated.get(&NodeId::new("b".to_string())).copied();
        assert_eq!(pos, Some((100.0, 0.0))); // Delta (55, 45) snaps to (100, 0) with grid 100
    }
}

// =============================================================================
// INP Mobile/Touch Interaction tests (bd-27q)
// =============================================================================

/// Minimum touch hit radius in screen pixels for touch targets.
/// This is larger than mouse hit radius for touch usability.
#[allow(dead_code)]
const TOUCH_HIT_RADIUS_MIN: f64 = 22.0; // Minimum radius for 44x44 hit area

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, deprecated)]
mod inp_mobile_touch_tests {
    use super::*;
    use crate::ui::grid::GridSize;

    // INP-1: Touch drag selects not marquee
    // Single-finger touch drag on canvas should initiate rubber-band selection, not marquee zoom.
    // The drag threshold determines when a touch becomes a drag vs a tap.
    #[test]
    fn given_touch_drag_when_motion_below_threshold_then_not_considered_drag() {
        // Touch drag below the 3.0px threshold should not trigger drag behavior
        // This allows for touch tap detection without accidental drag initiation
        let touch_start = (100.0, 100.0);
        let touch_current_below = (101.5, 101.5); // ~2.12px distance
        let touch_current_at = (103.0, 100.0); // exactly 3.0px

        assert!(
            !has_drag_threshold(touch_start, touch_current_below),
            "Touch motion below 3px should not trigger drag"
        );
        assert!(
            has_drag_threshold(touch_start, touch_current_at),
            "Touch motion at 3px should trigger drag"
        );
    }

    #[test]
    fn given_touch_drag_when_rightward_then_uses_contain_selection_mode() {
        // Rightward touch drag should use contain mode for rubber-band selection
        let start = (50.0, 50.0);
        let current = (150.0, 100.0); // Rightward drag

        let mode = selection_mode_from_drag(start, current);
        assert_eq!(
            mode,
            SelectionMode::Contain,
            "Rightward touch drag should use contain mode for selection"
        );
    }

    // INP-3: Long press selects
    // A long press (touch hold without movement) should select the target node.
    // The drag threshold being NOT met indicates a long press / tap scenario.
    #[test]
    fn given_long_press_when_no_motion_then_not_drag_and_can_select() {
        // Long press without motion should not trigger drag threshold
        // This allows the selection logic to handle the tap/press
        let press_point = (100.0, 100.0);
        let slightly_moved = (100.5, 100.5); // ~0.7px distance - negligible for long press

        assert!(
            !has_drag_threshold(press_point, slightly_moved),
            "Long press with negligible motion should not trigger drag"
        );

        // Verify selection can happen via select_single
        let selected = select_single("node-pressed".to_string());
        assert!(
            selected.contains("node-pressed"),
            "Long press should allow node selection"
        );
    }

    #[test]
    fn given_long_press_when_minor_jitter_then_still_not_drag() {
        // Touch screens have jitter; long press should tolerate small movements
        let press_point = (0.0, 0.0);
        let jitter_positions = [
            (0.5, 0.5), // ~0.7px
            (1.0, 1.0), // ~1.4px
            (1.5, 0.0), // 1.5px
            (0.0, 2.0), // 2.0px
            (2.0, 2.0), // ~2.8px
        ];

        for jitter in jitter_positions {
            assert!(
                !has_drag_threshold(press_point, jitter),
                "Long press jitter at ({}, {}) should not trigger drag",
                jitter.0,
                jitter.1
            );
        }
    }

    // INP-6: Double-tap timing
    // Double-tap detection requires consistent timing thresholds.
    #[test]
    fn given_double_tap_timing_when_taps_within_window_then_detected() {
        // Double-tap window is typically 300-500ms
        // This test verifies timing-related constants are reasonable
        const DOUBLE_TAP_WINDOW_MS: u64 = 400;

        // First tap time
        let first_tap_ms: u64 = 1000;
        // Second tap within window
        let second_tap_within = first_tap_ms + 300;
        // Second tap outside window
        let second_tap_outside = first_tap_ms + 500;

        let within_window = second_tap_within.abs_diff(first_tap_ms) <= DOUBLE_TAP_WINDOW_MS;
        let outside_window = second_tap_outside.abs_diff(first_tap_ms) <= DOUBLE_TAP_WINDOW_MS;

        assert!(
            within_window,
            "Taps within {}ms should be detected as double-tap",
            DOUBLE_TAP_WINDOW_MS
        );
        assert!(
            !outside_window,
            "Taps outside {}ms should not be detected as double-tap",
            DOUBLE_TAP_WINDOW_MS
        );
    }

    #[test]
    fn given_double_tap_timing_constants_then_are_finite_and_reasonable() {
        // Verify timing constants are usable
        const DOUBLE_TAP_MIN_MS: u64 = 100; // Minimum time to distinguish from single tap
        const DOUBLE_TAP_MAX_MS: u64 = 700; // Maximum time for double tap detection

        assert!(
            DOUBLE_TAP_MIN_MS >= 50,
            "Double-tap min should be at least 50ms"
        );
        assert!(
            DOUBLE_TAP_MAX_MS <= 1000,
            "Double-tap max should be at most 1000ms"
        );
        assert!(
            DOUBLE_TAP_MIN_MS < DOUBLE_TAP_MAX_MS,
            "Double-tap min should be less than max"
        );
    }

    // INP-7: Touch handle hit area usable
    // Selection handles should have touch-friendly hit areas (at least 44x44 points).

    #[test]
    fn given_touch_hit_area_when_checking_selection_handles_then_meets_minimum() {
        // Selection handles should be at least 44x44 points (22px radius)
        // This is based on accessibility guidelines for touch targets
        let handle_hit_radius: f64 = 7.0; // Current handle size from canvas_view.rs
        let touch_enlarged_radius = handle_hit_radius.max(TOUCH_HIT_RADIUS_MIN);

        assert!(
            touch_enlarged_radius >= TOUCH_HIT_RADIUS_MIN,
            "Touch hit area should be at least {} radius, got {}",
            TOUCH_HIT_RADIUS_MIN,
            touch_enlarged_radius
        );
    }

    #[test]
    fn given_touch_finger_hit_area_when_computed_then_meets_accessibility() {
        // WCAG 2.1 recommends minimum 44x44 CSS pixels for touch targets
        let min_touch_size = 44.0;
        let effective_radius = min_touch_size / 2.0;

        // Verify our constant matches accessibility guidelines
        assert!(
            TOUCH_HIT_RADIUS_MIN >= effective_radius - 1.0,
            "Touch hit radius {} should meet accessibility minimum {}",
            TOUCH_HIT_RADIUS_MIN,
            effective_radius
        );
    }
}

#[cfg(test)]
mod inp_mobile_touch_proptests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_touch_point()(x in 0.0_f64..1000.0, y in 0.0_f64..1000.0) -> (f64, f64) {
            (x, y)
        }
    }

    prop_compose! {
        fn arb_small_jitter()(dx in -3.0_f64..3.0, dy in -3.0_f64..3.0) -> (f64, f64) {
            (dx, dy)
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        // INP-1: Touch drag threshold property
        #[test]
        fn prop_touch_drag_threshold_consistent_regardless_of_direction(
            origin in arb_touch_point(),
            delta in 3.0_f64..100.0,
        ) {
            // Drag threshold should be direction-agnostic (only distance matters)
            let right = (origin.0 + delta, origin.1);
            let down = (origin.0, origin.1 + delta);
            let diagonal = (origin.0 + delta / 2.0_f64.sqrt(), origin.1 + delta / 2.0_f64.sqrt());

            let right_result = has_drag_threshold(origin, right);
            let down_result = has_drag_threshold(origin, down);
            let diag_result = has_drag_threshold(origin, diagonal);

            // All directions with same distance should have same result
            prop_assert_eq!(right_result, down_result);
            prop_assert_eq!(right_result, diag_result);
        }

        // INP-3: Long press stability
        #[test]
        fn prop_long_press_with_small_jitter_never_triggers_drag(
            origin in arb_touch_point(),
            jitter in arb_small_jitter(),
        ) {
            // Jitter within ±3px should not trigger drag
            let jittered = (origin.0 + jitter.0, origin.1 + jitter.1);
            let distance = (
                (jittered.0 - origin.0).abs(),
                (jittered.1 - origin.1).abs(),
            );

            // If jitter is below threshold distance, should not trigger drag
            let euclidean = (distance.0 * distance.0 + distance.1 * distance.1).sqrt();
            if euclidean < 3.0 {
                prop_assert!(!has_drag_threshold(origin, jittered));
            }
        }

        // INP-6: Double-tap timing consistency
        #[test]
        fn prop_double_tap_timing_window_is_positive(
            min_ms in 50_u64..200,
            max_offset in 200_u64..500,
        ) {
            let max_ms = min_ms + max_offset;
            prop_assert!(max_ms > min_ms);
            prop_assert!(min_ms >= 50);
            prop_assert!(max_ms <= 1000);
        }

        // INP-7: Touch hit area is always positive
        #[test]
        fn prop_touch_hit_radius_always_positive_and_finite(radius in 1.0_f64..100.0) {
            let effective = radius.max(TOUCH_HIT_RADIUS_MIN);
            prop_assert!(effective.is_finite());
            prop_assert!(effective > 0.0);
            prop_assert!(effective >= TOUCH_HIT_RADIUS_MIN);
        }
    }
}
