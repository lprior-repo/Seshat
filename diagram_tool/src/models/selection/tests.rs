use super::*;
use crate::models::document::{
    DiagramDocument, DocumentData, EditorState, LockState, Node, NodeId, NodeKind, OrderedFloat,
};
use im::HashMap;
use serde_json::json;

fn setup_doc() -> DiagramDocument {
    let mut nodes = HashMap::new();
    let n1 = NodeId::new("n1".to_string());
    let n2 = NodeId::new("n2".to_string());

    let n1_node = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "n1".to_string(),
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
        style: None,
        collapsed: None,
    };

    let n2_node = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "n2".to_string(),
        x: OrderedFloat(200.0),
        y: OrderedFloat(200.0),
        width: OrderedFloat(50.0),
        height: OrderedFloat(50.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };

    nodes.insert(n1.clone(), n1_node);
    nodes.insert(n2.clone(), n2_node);

    let doc_data = DocumentData {
        nodes,
        edges: HashMap::new(),
    };

    DiagramDocument {
        version: 2,
        revision: crate::models::document::Revision::INITIAL,
        document: doc_data,
        editor_state: EditorState::default(),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_021_bounding_box_covers_rotated_nodes() {
    let mut doc = setup_doc();
    // Rotate n1 by 45 degrees
    let mut meta = HashMap::new();
    meta.insert("rotation".to_string(), json!(std::f64::consts::FRAC_PI_4));
    doc.document
        .nodes
        .get_mut(&NodeId::new("n1".to_string()))
        .unwrap()
        .metadata = meta;

    doc.editor_state.selected_items.insert("n1".to_string());

    let bounds = compute_selection_bounds(&doc).unwrap();
    // 100x100 box at 0,0 rotated 45deg around center 50,50
    // distance from center to corner is sqrt(50^2 + 50^2) = 70.7106
    // So bounds width = 70.7106 * 2 = 141.421

    assert!((bounds.width - 141.421).abs() < 0.1);
    assert!((bounds.height - 141.421).abs() < 0.1);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_022_long_press_adds_node_to_selection_without_drag() {
    let mut doc = setup_doc();
    let res = handle_long_press(&mut doc, NodeId::new("n1".to_string()), 2.0);
    assert!(res.is_ok());
    assert!(doc.editor_state.selected_items.contains("n1"));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_023_double_click_enters_edit_mode_on_shape() {
    let mut doc = setup_doc();
    let res = handle_double_click(&mut doc, NodeId::new("n1".to_string()));
    assert!(res.is_ok());
    assert_eq!(doc.editor_state.edit_mode_target.as_deref(), Some("n1"));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_024_selection_persists_across_camera_zoom_and_pan() {
    let mut doc = setup_doc();
    doc.editor_state.selected_items.insert("n1".to_string());

    let selection_before = doc.editor_state.selected_items.clone();

    // Zoom and pan
    doc.editor_state.camera_x = OrderedFloat(100.0);
    doc.editor_state.zoom = OrderedFloat(2.0);

    let selection_after = doc.editor_state.selected_items.clone();

    assert_eq!(selection_before, selection_after);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_025_marquee_selects_nodes_inside_subgraphs() {
    let mut doc = setup_doc();
    let child = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "child".to_string(),
        x: OrderedFloat(50.0),
        y: OrderedFloat(50.0),
        width: OrderedFloat(10.0),
        height: OrderedFloat(10.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: Some(NodeId::new("n1".to_string())),
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };
    doc.document
        .nodes
        .insert(NodeId::new("child".to_string()), child);

    // Make marquee large enough to fully enclose the parent n1 (0,0 to 100,100)
    let marquee = Rect::new(-10.0, -10.0, 120.0, 120.0).unwrap();
    let selected = compute_marquee_selection(&doc, marquee).unwrap();

    assert!(selected.contains(&NodeId::new("child".to_string())));
    assert!(selected.contains(&NodeId::new("n1".to_string()))); // n1 bounds 0,0 100,100, fully enclosed
    assert!(!selected.contains(&NodeId::new("n2".to_string())));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_when_computing_bounds_for_missing_nodes() {
    let mut doc = setup_doc();
    doc.editor_state
        .selected_items
        .insert("n3_missing".to_string());

    let res = compute_selection_bounds(&doc);
    assert_eq!(res.unwrap_err(), SelectionError::NodeNotFound);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_long_press_fails_when_movement_exceeds_threshold() {
    let mut doc = setup_doc();
    let res = handle_long_press(&mut doc, NodeId::new("n1".to_string()), 15.0);
    assert_eq!(
        res.unwrap_err(),
        SelectionError::MovementExceededDragThreshold
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_double_click_fails_on_uneditable_nodes() {
    let mut doc = setup_doc();
    doc.document
        .nodes
        .get_mut(&NodeId::new("n1".to_string()))
        .unwrap()
        .lock_state = LockState::Locked;

    let res = handle_double_click(&mut doc, NodeId::new("n1".to_string()));
    assert_eq!(res.unwrap_err(), SelectionError::NodeNotEditable);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p1_violation_returns_node_not_found() {
    let mut doc = setup_doc();
    doc.editor_state.selected_items.insert("n3".to_string());
    let res = compute_selection_bounds(&doc);
    assert_eq!(res, Err(SelectionError::NodeNotFound));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p2_violation_returns_movement_exceeded_drag_threshold() {
    let mut doc = setup_doc();
    let res = handle_long_press(&mut doc, NodeId::new("n1".to_string()), 6.0);
    assert_eq!(res, Err(SelectionError::MovementExceededDragThreshold));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p3_violation_returns_node_not_editable() {
    let mut doc = setup_doc();
    doc.document
        .nodes
        .get_mut(&NodeId::new("n1".to_string()))
        .unwrap()
        .lock_state = LockState::Locked;
    let res = handle_double_click(&mut doc, NodeId::new("n1".to_string()));
    assert_eq!(res, Err(SelectionError::NodeNotEditable));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p5_violation_returns_marquee_invalid() {
    let res = Rect::new(0.0, 0.0, -10.0, 10.0);
    assert_eq!(res.unwrap_err(), SelectionError::InvalidMarqueeBounds);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_021_bounding_box_with_mixed_rotated_and_unrotated_nodes() {
    let mut doc = setup_doc();
    // n1 is unrotated: 0,0 -> 100,100
    // n2 is rotated 90 deg: 200,200 -> 250,250
    // bounding box of both should envelop both
    let mut meta = HashMap::new();
    meta.insert("rotation".to_string(), json!(std::f64::consts::FRAC_PI_2));
    doc.document
        .nodes
        .get_mut(&NodeId::new("n2".to_string()))
        .unwrap()
        .metadata = meta;

    doc.editor_state.selected_items.insert("n1".to_string());
    doc.editor_state.selected_items.insert("n2".to_string());

    let bounds = compute_selection_bounds(&doc).unwrap();

    // Unrotated n1: 0, 0 to 100, 100
    // Rotated n2 by 90deg doesn't change its AABB size since it's square: 200, 200 to 250, 250
    assert!((bounds.x - 0.0).abs() < 0.1);
    assert!((bounds.y - 0.0).abs() < 0.1);
    assert!((bounds.width - 250.0).abs() < 0.1);
    assert!((bounds.height - 250.0).abs() < 0.1);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_025_marquee_partially_overlapping_parent_selects_fully_enclosed_child() {
    let mut doc = setup_doc();
    // Group A
    let group_a = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "Group A".to_string(),
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
        style: None,
        collapsed: None,
    };
    doc.document
        .nodes
        .insert(NodeId::new("group_a".to_string()), group_a);

    // Child B
    let child_b = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "Child B".to_string(),
        x: OrderedFloat(10.0),
        y: OrderedFloat(10.0),
        width: OrderedFloat(20.0),
        height: OrderedFloat(20.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: Some(NodeId::new("group_a".to_string())),
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };
    doc.document
        .nodes
        .insert(NodeId::new("child_b".to_string()), child_b);

    // Node C
    let node_c = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "Node C".to_string(),
        x: OrderedFloat(150.0),
        y: OrderedFloat(10.0),
        width: OrderedFloat(20.0),
        height: OrderedFloat(20.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };
    doc.document
        .nodes
        .insert(NodeId::new("node_c".to_string()), node_c);

    // Marquee hits Child B fully, and partially hits Group A
    let marquee = Rect::new(5.0, 5.0, 200.0, 30.0).unwrap();
    let selected = compute_marquee_selection(&doc, marquee).unwrap();

    assert!(selected.contains(&NodeId::new("child_b".to_string())));
    assert!(selected.contains(&NodeId::new("node_c".to_string())));
    assert!(!selected.contains(&NodeId::new("group_a".to_string())));
}
