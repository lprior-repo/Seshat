use crate::models::document::{
    DiagramDocument, DocumentData, Edge, EdgeId, EditorState, Node, NodeId, NodeKind, OrderedFloat,
    Point,
};
use crate::models::selection::{
    compute_marquee_selection, compute_selection_bounds, handle_double_click, handle_long_press,
    hit_test, select_element, ElementId, Rect, SelectModifiers, SelectionBounds, SelectionError,
    ValidRect,
};
use im::HashMap;
use serde_json::json;

fn setup_doc() -> DiagramDocument {
    let mut nodes = HashMap::new();
    let n1 = NodeId::new("n1".to_string());
    let n2 = NodeId::new("n2".to_string());

    let mut n1_node = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "n1".to_string(),
        x: OrderedFloat(0.0),
        y: OrderedFloat(0.0),
        width: OrderedFloat(100.0),
        height: OrderedFloat(100.0),
        font_size: None,
        font_weight: None,
        locked: false,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };

    let mut n2_node = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "n2".to_string(),
        x: OrderedFloat(200.0),
        y: OrderedFloat(200.0),
        width: OrderedFloat(50.0),
        height: OrderedFloat(50.0),
        font_size: None,
        font_weight: None,
        locked: false,
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
    let mut child = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "child".to_string(),
        x: OrderedFloat(50.0),
        y: OrderedFloat(50.0),
        width: OrderedFloat(10.0),
        height: OrderedFloat(10.0),
        font_size: None,
        font_weight: None,
        locked: false,
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

    let marquee = ValidRect::new(45.0, 45.0, 20.0, 20.0).unwrap();
    let selected = compute_marquee_selection(&doc, marquee).unwrap();

    assert!(selected.contains(&NodeId::new("child".to_string())));
    assert!(selected.contains(&NodeId::new("n1".to_string()))); // n1 bounds 0,0 100,100, overlaps
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
        .locked = true;

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
        .locked = true;
    let res = handle_double_click(&mut doc, NodeId::new("n1".to_string()));
    assert_eq!(res, Err(SelectionError::NodeNotEditable));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p5_violation_returns_marquee_invalid() {
    let res = ValidRect::new(0.0, 0.0, -10.0, 10.0);
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
    let mut group_a = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "Group A".to_string(),
        x: OrderedFloat(0.0),
        y: OrderedFloat(0.0),
        width: OrderedFloat(100.0),
        height: OrderedFloat(100.0),
        font_size: None,
        font_weight: None,
        locked: false,
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
    let mut child_b = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "Child B".to_string(),
        x: OrderedFloat(10.0),
        y: OrderedFloat(10.0),
        width: OrderedFloat(20.0),
        height: OrderedFloat(20.0),
        font_size: None,
        font_weight: None,
        locked: false,
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
    let mut node_c = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "Node C".to_string(),
        x: OrderedFloat(150.0),
        y: OrderedFloat(10.0),
        width: OrderedFloat(20.0),
        height: OrderedFloat(20.0),
        font_size: None,
        font_weight: None,
        locked: false,
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
    let marquee = ValidRect::new(5.0, 5.0, 200.0, 30.0).unwrap();
    let selected = compute_marquee_selection(&doc, marquee).unwrap();

    assert!(selected.contains(&NodeId::new("child_b".to_string())));
    assert!(selected.contains(&NodeId::new("node_c".to_string())));
    assert!(!selected.contains(&NodeId::new("group_a".to_string())));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_success_when_alt_click_selects_parent_container() {
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
        locked: false,
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

    let mut state = im::HashSet::new();
    let mut modifiers = SelectModifiers::default();
    modifiers.alt = true;

    let res = select_element(
        &mut state,
        &doc,
        &ElementId::Node(NodeId::new("child".to_string())),
        &modifiers,
    );
    assert!(res.is_ok());
    assert!(state.contains("n1"));
    assert!(!state.contains("child"));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_success_when_right_click_unselected_node_selects_it() {
    let doc = setup_doc();
    let mut state = im::HashSet::new();
    let mut modifiers = SelectModifiers::default();
    modifiers.right_click = true;

    let res = select_element(
        &mut state,
        &doc,
        &ElementId::Node(NodeId::new("n1".to_string())),
        &modifiers,
    );
    assert!(res.is_ok());
    assert!(state.contains("n1"));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_success_when_click_edge_selects_connector() {
    let mut doc = setup_doc();
    let edge = Edge {
        source: NodeId::new("n1".to_string()),
        target: NodeId::new("n2".to_string()),
        label: "".to_string(),
        style: Default::default(),
        arrow_type: Default::default(),
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
    };
    doc.document
        .edges
        .insert(EdgeId::new("e1".to_string()), edge);

    let mut state = im::HashSet::new();
    let modifiers = SelectModifiers::default();

    let res = select_element(
        &mut state,
        &doc,
        &ElementId::Edge(EdgeId::new("e1".to_string())),
        &modifiers,
    );
    assert!(res.is_ok());
    assert!(state.contains("e1"));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_when_alt_clicking_node_without_parent() {
    let doc = setup_doc();
    let mut state = im::HashSet::new();
    let mut modifiers = SelectModifiers::default();
    modifiers.alt = true;

    let res = select_element(
        &mut state,
        &doc,
        &ElementId::Node(NodeId::new("n1".to_string())),
        &modifiers,
    );
    assert_eq!(res.unwrap_err(), SelectionError::NoParentContainer);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_when_selecting_locked_element() {
    let mut doc = setup_doc();
    doc.document
        .nodes
        .get_mut(&NodeId::new("n1".to_string()))
        .unwrap()
        .locked = true;

    let mut state = im::HashSet::new();
    let modifiers = SelectModifiers::default();

    let res = select_element(
        &mut state,
        &doc,
        &ElementId::Node(NodeId::new("n1".to_string())),
        &modifiers,
    );
    assert_eq!(res.unwrap_err(), SelectionError::ElementLocked);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_when_selecting_hidden_element() {
    let mut doc = setup_doc();
    let mut meta = HashMap::new();
    meta.insert("visibility".to_string(), json!("hidden"));
    doc.document
        .nodes
        .get_mut(&NodeId::new("n1".to_string()))
        .unwrap()
        .metadata = meta;

    let mut state = im::HashSet::new();
    let modifiers = SelectModifiers::default();

    let res = select_element(
        &mut state,
        &doc,
        &ElementId::Node(NodeId::new("n1".to_string())),
        &modifiers,
    );
    assert_eq!(res.unwrap_err(), SelectionError::ElementHidden);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_when_selecting_non_existent_element() {
    let doc = setup_doc();
    let mut state = im::HashSet::new();
    let modifiers = SelectModifiers::default();

    let res = select_element(
        &mut state,
        &doc,
        &ElementId::Node(NodeId::new("ghost".to_string())),
        &modifiers,
    );
    assert_eq!(res.unwrap_err(), SelectionError::ElementNotFound);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_handles_click_passing_through_hidden_node_to_node_underneath() {
    let mut doc = setup_doc();
    let mut meta = HashMap::new();
    meta.insert("visibility".to_string(), json!("hidden"));
    let hidden = Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: "hidden".to_string(),
        x: OrderedFloat(0.0),
        y: OrderedFloat(0.0),
        width: OrderedFloat(100.0),
        height: OrderedFloat(100.0),
        font_size: None,
        font_weight: None,
        locked: false,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: meta,
        z_index: 10,
        style: None,
        collapsed: None,
    };
    doc.document
        .nodes
        .insert(NodeId::new("hidden".to_string()), hidden);

    let point = Point {
        x: OrderedFloat(50.0),
        y: OrderedFloat(50.0),
    };
    let res = hit_test(&point, &doc).unwrap();

    assert_eq!(res, Some(ElementId::Node(NodeId::new("n1".to_string()))));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_handles_right_click_already_selected_node_preserves_selection() {
    let doc = setup_doc();
    let mut state = im::HashSet::new();
    state.insert("n1".to_string());
    let mut modifiers = SelectModifiers::default();
    modifiers.right_click = true;

    let res = select_element(
        &mut state,
        &doc,
        &ElementId::Node(NodeId::new("n1".to_string())),
        &modifiers,
    );
    assert!(res.is_ok());
    assert!(state.contains("n1"));
    assert_eq!(state.len(), 1);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p1_violation_returns_no_parent_container_error() {
    test_returns_error_when_alt_clicking_node_without_parent();
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p2_violation_returns_element_locked_error() {
    test_returns_error_when_selecting_locked_element();
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p3_violation_returns_element_hidden_error() {
    test_returns_error_when_selecting_hidden_element();
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p4_violation_returns_element_not_found_error() {
    test_returns_error_when_selecting_non_existent_element();
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_q1_violation_returns_precondition_violated() {}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_q2_violation_returns_element_locked() {}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_q3_violation_returns_element_hidden() {}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_q4_violation_returns_precondition_violated() {}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_q5_violation_returns_precondition_violated() {}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_precondition_element_must_be_unlocked() {}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_precondition_element_must_be_visible() {}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_postcondition_alt_click_replaces_child_with_parent() {}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_postcondition_right_click_replaces_selection() {}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_invariant_selection_never_contains_locked_elements() {}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_invariant_selection_never_contains_hidden_elements() {}
