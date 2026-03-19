use super::*;
use diagram_models::document::{
    DiagramDocument, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat,
};

fn test_node() -> Node {
    Node {
        kind: NodeKind::Text,
        icon: String::new(),
        label: "Test".to_string(),
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
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    }
}

#[test]
fn test_returns_error_when_source_node_missing() {
    let mut doc = DiagramDocument::default();
    let t1 = NodeId::new("t1".to_string());
    doc.document.nodes.insert(t1.clone(), test_node());

    let err = create_edge(
        &mut doc,
        NodeId::new("s1".to_string()),
        t1,
        EdgeId::new("e1".to_string()),
    )
    .unwrap_err();
    assert_eq!(err, RoutingError::SourceNotFound("s1".to_string()));
}

#[test]
fn test_allows_self_loop_at_routing_layer() {
    // Self-loop validation is now handled at the policy layer (CyclePolicy).
    // At the routing layer, self-loops are allowed - the policy layer
    // enforces rejection in Deny mode and allows in Allow mode.
    let mut doc = DiagramDocument::default();
    let s1 = NodeId::new("s1".to_string());
    doc.document.nodes.insert(s1.clone(), test_node());

    // Self-loop should now succeed at routing layer
    let result = create_edge(&mut doc, s1.clone(), s1, EdgeId::new("e1".to_string()));
    assert!(
        result.is_ok(),
        "Self-loop should be allowed at routing layer"
    );

    // Verify edge was created
    let edge = doc.document.edges.get(&EdgeId::new("e1".to_string()));
    assert!(edge.is_some(), "Edge should exist in document");
}

#[test]
fn test_adding_edge_that_creates_cycle_returns_cycle_detected_error() {
    let mut doc = DiagramDocument::default();
    let n1 = NodeId::new("1".to_string());
    let n2 = NodeId::new("2".to_string());
    let n3 = NodeId::new("3".to_string());

    doc.document.nodes.insert(n1.clone(), test_node());
    doc.document.nodes.insert(n2.clone(), test_node());
    doc.document.nodes.insert(n3.clone(), test_node());

    create_edge(
        &mut doc,
        n1.clone(),
        n2.clone(),
        EdgeId::new("e1".to_string()),
    )
    .unwrap();
    create_edge(&mut doc, n2, n3.clone(), EdgeId::new("e2".to_string())).unwrap();

    let err = create_edge(&mut doc, n3, n1, EdgeId::new("e3".to_string())).unwrap_err();
    assert_eq!(err, RoutingError::CycleDetected);
}

use diagram_models::document::Edge;
use diagram_models::port::PortAnchor;

fn create_edge_obj(source: &str, target: &str) -> Edge {
    Edge {
        source: NodeId::new(source.to_string()),
        target: NodeId::new(target.to_string()),
        label: String::new(),
        style: Default::default(),
        arrow_type: Default::default(),
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

#[test]
fn test_compute_straight_line_route_between_centers_successfully() {
    let mut doc = DiagramDocument::default();
    let n1_id = NodeId::new("n1".to_string());
    let n2_id = NodeId::new("n2".to_string());

    let mut n1 = test_node();
    n1.x = OrderedFloat(0.0);
    n1.y = OrderedFloat(0.0);
    n1.width = OrderedFloat(100.0);
    n1.height = OrderedFloat(100.0);

    let mut n2 = test_node();
    n2.x = OrderedFloat(200.0);
    n2.y = OrderedFloat(0.0);
    n2.width = OrderedFloat(100.0);
    n2.height = OrderedFloat(100.0);

    doc.document.nodes.insert(n1_id, n1);
    doc.document.nodes.insert(n2_id, n2);

    let edge = create_edge_obj("n1", "n2");
    let (start, end) = compute_straight_line_route(&doc, &edge).unwrap();

    assert_eq!(start.x, 50.0);
    assert_eq!(start.y, 50.0);
    assert_eq!(end.x, 250.0);
    assert_eq!(end.y, 50.0);
}

#[test]
fn test_compute_straight_line_route_between_named_ports_successfully() {
    let mut doc = DiagramDocument::default();
    let n1_id = NodeId::new("n1".to_string());
    let n2_id = NodeId::new("n2".to_string());

    let mut n1 = test_node();
    n1.x = OrderedFloat(0.0);
    n1.y = OrderedFloat(0.0);
    n1.width = OrderedFloat(100.0);
    n1.height = OrderedFloat(100.0);

    let mut n2 = test_node();
    n2.x = OrderedFloat(0.0);
    n2.y = OrderedFloat(200.0);
    n2.width = OrderedFloat(100.0);
    n2.height = OrderedFloat(100.0);

    doc.document.nodes.insert(n1_id, n1);
    doc.document.nodes.insert(n2_id, n2);

    let mut edge = create_edge_obj("n1", "n2");
    edge.source_port = Some(PortAnchor::Bottom);
    edge.target_port = Some(PortAnchor::Top);

    let (start, end) = compute_straight_line_route(&doc, &edge).unwrap();

    assert_eq!(start.x, 50.0);
    assert_eq!(start.y, 100.0);
    assert_eq!(end.x, 50.0);
    assert_eq!(end.y, 200.0);
}

#[test]
fn test_compute_straight_line_route_for_self_loop_returns_same_points() {
    let mut doc = DiagramDocument::default();
    let n1_id = NodeId::new("n1".to_string());

    let mut n1 = test_node();
    n1.x = OrderedFloat(0.0);
    n1.y = OrderedFloat(0.0);
    n1.width = OrderedFloat(100.0);
    n1.height = OrderedFloat(100.0);

    doc.document.nodes.insert(n1_id, n1);

    let edge = create_edge_obj("n1", "n1");
    let (start, end) = compute_straight_line_route(&doc, &edge).unwrap();

    assert_eq!(start, end);
    assert_eq!(start.x, 50.0);
    assert_eq!(start.y, 50.0);
}

#[test]
fn test_routing_returns_error_when_source_node_missing() {
    let mut doc = DiagramDocument::default();
    let n2_id = NodeId::new("n2".to_string());
    doc.document.nodes.insert(n2_id, test_node());

    let edge = create_edge_obj("n1", "n2");
    let err = compute_straight_line_route(&doc, &edge).unwrap_err();
    assert_eq!(err, RoutingError::SourceNotFound("n1".to_string()));
}

#[test]
fn test_routing_returns_error_when_target_node_missing() {
    let mut doc = DiagramDocument::default();
    let n1_id = NodeId::new("n1".to_string());
    doc.document.nodes.insert(n1_id, test_node());

    let edge = create_edge_obj("n1", "n2");
    let err = compute_straight_line_route(&doc, &edge).unwrap_err();
    assert_eq!(err, RoutingError::TargetNotFound("n2".to_string()));
}
