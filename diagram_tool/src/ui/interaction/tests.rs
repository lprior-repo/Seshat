use super::*;
use crate::models::document::{
    DiagramDocument, DocumentData, Edge, EdgeId, EditorState, LockState, Node, NodeId, NodeKind,
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
        lock_state: LockState::Unlocked,
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

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_small_motion_when_threshold_checked_then_returns_false() {
    assert!(!has_drag_threshold((0.0, 0.0), (1.0, 1.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_large_motion_when_threshold_checked_then_returns_true() {
    assert!(has_drag_threshold((0.0, 0.0), (4.0, 0.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_selection_when_toggling_then_adds_and_removes_item() {
    let once = toggle_selection(&HashSet::new(), "node-1");
    assert!(once.contains("node-1"));

    let twice = toggle_selection(&once, "node-1");
    assert!(!twice.contains("node-1"));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_single_item_when_select_single_then_only_item_is_selected() {
    let selected = select_single(String::from("edge-1"));
    assert!(selected.contains("edge-1"));
    assert_eq!(selected.len(), 1);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_drag_anchor_and_current_when_dragged_positions_then_offsets_nodes() {
    let originals = HashMap::new().update(NodeId::new(String::from("a")), (2.0, 3.0));
    let updated = dragged_positions(&originals, (0.0, 0.0), (5.0, -2.0));
    let pos = updated.get(&NodeId::new(String::from("a"))).copied();
    assert_eq!(pos, Some((7.0, 1.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_snap_enabled_when_dragging_then_positions_use_grid_delta() {
    let originals = HashMap::new().update(NodeId::new(String::from("a")), (3.0, 7.0));
    let grid = GridSize::new(20.0).unwrap();
    let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (14.0, 26.0), true, grid);
    let pos = updated.get(&NodeId::new(String::from("a"))).copied();
    assert_eq!(pos, Some((23.0, 27.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_rect_when_node_ids_in_rect_then_returns_contained_nodes() {
    let doc = doc_with_nodes();
    let selected = node_ids_in_rect(&doc, (0.0, 0.0), (60.0, 60.0));
    assert!(selected.contains("a"));
    assert!(!selected.contains("b"));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_leftward_drag_when_selection_mode_resolved_then_uses_intersect() {
    let mode = selection_mode_from_drag((100.0, 100.0), (40.0, 120.0));
    assert_eq!(mode, SelectionMode::Intersect);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_rightward_drag_when_selection_mode_resolved_then_uses_contain() {
    let mode = selection_mode_from_drag((40.0, 100.0), (100.0, 120.0));
    assert_eq!(mode, SelectionMode::Contain);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_leftward_drag_when_node_ids_in_rect_then_uses_intersection_behavior() {
    let doc = doc_with_nodes();
    let rightward = node_ids_in_rect(&doc, (35.0, 25.0), (42.0, 32.0));
    assert!(!rightward.contains("a"));

    let leftward = node_ids_in_rect(&doc, (42.0, 32.0), (35.0, 25.0));
    assert!(leftward.contains("a"));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_intersect_mode_when_rect_touches_node_then_node_is_selected() {
    let doc = doc_with_nodes();
    let selected =
        node_ids_in_rect_with_mode(&doc, (35.0, 25.0), (42.0, 32.0), SelectionMode::Intersect);
    assert!(selected.contains("a"));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_snap_enabled_when_snapping_values_then_rounds_to_grid() {
    assert!((snap_value(29.0, true, 20.0) - 20.0).abs() < f64::EPSILON);
    let pt = snap_point((31.0, 49.0), true, 20.0);
    assert!((pt.0 - 40.0).abs() < f64::EPSILON && (pt.1 - 40.0).abs() < f64::EPSILON);
}

#[cfg(kani)]
#[kani::proof]
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
            source_port: None,
            target_port: None,
        },
    );
    let selected = HashSet::new()
        .update(source.to_string())
        .update(target.to_string());

    let enriched = with_auto_selected_edges(&doc, &selected);
    assert!(enriched.contains(&edge_id.to_string()));
}

#[cfg(kani)]
#[kani::proof]
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

#[cfg(kani)]
#[kani::proof]
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

    // Drag is (608, 240) to (572, 224) -> Leftward -> Intersect mode
    let selected = node_ids_in_rect(&doc, (608.0, 240.0), (572.0, 224.0));

    assert!(selected.contains("text-node"));
}
