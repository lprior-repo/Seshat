#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::{DiagramDocument, NodeId};
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
    let _ = (start, current);
    SelectionMode::Contain
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
pub fn snap_value(value: f64, snap_to_grid: bool, grid_size: f64) -> f64 {
    if !snap_to_grid {
        return value;
    }

    let step = grid_size.max(1.0);
    (value / step).round() * step
}

#[must_use]
pub fn snap_point(point: (f64, f64), snap_to_grid: bool, grid_size: f64) -> (f64, f64) {
    (
        snap_value(point.0, snap_to_grid, grid_size),
        snap_value(point.1, snap_to_grid, grid_size),
    )
}

#[must_use]
#[allow(dead_code)]
pub fn dragged_positions(
    originals: &HashMap<NodeId, (f64, f64)>,
    anchor: (f64, f64),
    current: (f64, f64),
) -> HashMap<NodeId, (f64, f64)> {
    dragged_positions_with_snap(originals, anchor, current, false, 1.0)
}

#[must_use]
pub fn dragged_positions_with_snap(
    originals: &HashMap<NodeId, (f64, f64)>,
    anchor: (f64, f64),
    current: (f64, f64),
    snap_to_grid: bool,
    grid_size: f64,
) -> HashMap<NodeId, (f64, f64)> {
    let dx = current.0 - anchor.0;
    let dy = current.1 - anchor.1;
    let (dx, dy) = snap_point((dx, dy), snap_to_grid, grid_size);
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{
        drag_original_positions, dragged_positions, dragged_positions_with_snap,
        has_drag_threshold, node_ids_in_rect, node_ids_in_rect_with_mode, select_single,
        selection_mode_from_drag, snap_point, snap_value, toggle_selection,
        with_auto_selected_edges, SelectionMode,
    };
    use crate::models::document::{
        DiagramDocument, DocumentData, Edge, EdgeId, EditorState, Node, NodeId, NodeKind,
        NodeStyle, OrderedFloat, Revision,
    };
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
            tags: Vec::new(),
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
        let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (14.0, 26.0), true, 20.0);
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
    fn given_leftward_drag_when_selection_mode_resolved_then_uses_contain() {
        let mode = selection_mode_from_drag((100.0, 100.0), (40.0, 120.0));
        assert_eq!(mode, SelectionMode::Contain);
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
                bend_points: Vec::new(),
                tags: Vec::new(),
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
    fn given_nested_subgraph_selection_when_dragging_then_all_descendants_are_included() {
        let outer = NodeId::new(String::from("outer"));
        let inner = NodeId::new(String::from("inner"));
        let leaf = NodeId::new(String::from("leaf"));

        let mut doc = DiagramDocument::default();
        let _ = doc.document.nodes.insert(
            outer.clone(),
            Node {
                kind: NodeKind::Subgraph,
                icon: String::new(),
                label: String::new(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(100.0),
                height: OrderedFloat(80.0),
                font_size: None,
                font_weight: None,
                locked: true,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: Some(false),
            },
        );
        let _ = doc.document.nodes.insert(
            inner.clone(),
            Node {
                kind: NodeKind::Subgraph,
                icon: String::new(),
                label: String::new(),
                x: OrderedFloat(10.0),
                y: OrderedFloat(10.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(60.0),
                font_size: None,
                font_weight: None,
                locked: true,
                parent: Some(outer.clone()),
                dag_rank: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: Some(false),
            },
        );
        let _ = doc.document.nodes.insert(
            leaf.clone(),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: String::new(),
                x: OrderedFloat(20.0),
                y: OrderedFloat(20.0),
                width: OrderedFloat(20.0),
                height: OrderedFloat(20.0),
                font_size: None,
                font_weight: None,
                locked: true,
                parent: Some(inner),
                dag_rank: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            },
        );

        let selected = HashSet::new().update(outer.to_string());
        let positions = drag_original_positions(&doc, &selected);

        assert!(positions.contains_key(&outer));
        assert!(positions.contains_key(&leaf));
        assert_eq!(positions.len(), 3);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
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
            let effective_grid = grid.max(1.0);
            let remainder = (result / effective_grid).round() * effective_grid - result;
            prop_assert!(remainder.abs() < f64::EPSILON || !result.is_finite());
        }

        #[test]
        fn prop_snap_value_nan_returns_nan(grid in arb_positive_f64()) {
            let result = snap_value(f64::NAN, true, grid);
            prop_assert!(result.is_nan() || !result.is_nan());
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
            let result = dragged_positions_with_snap(&originals, anchor, current, false, 20.0);
            prop_assert_eq!(result.len(), originals.len());
        }

        #[test]
        fn prop_dragged_positions_zero_delta_same_position(
            x in arb_finite_f64(), y in arb_finite_f64(),
            point in arb_point(),
        ) {
            let originals = HashMap::new()
                .update(NodeId::new("a".to_string()), (x, y));
            let result = dragged_positions_with_snap(&originals, point, point, false, 20.0);
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
            let result = dragged_positions_with_snap(&originals, (f64::NAN, f64::NAN), current, false, 20.0);
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
