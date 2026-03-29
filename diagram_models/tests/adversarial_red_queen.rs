#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::similar_names,
    clippy::redundant_clone
)]
use diagram_models::document::{
    ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, LockState, Node, NodeId, NodeKind,
    OrderedFloat,
};
use im::HashMap;

fn create_base_node(_id: &str) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: "test".to_string(),
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
    }
}

fn create_test_edge(source: &str, target: &str) -> Edge {
    Edge {
        source: NodeId::new(source.to_string()),
        target: NodeId::new(target.to_string()),
        label: String::new(),
        style: EdgeStyle::default(),
        arrow_type: ArrowType::default(),
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
    }
}

#[test]
fn attack_6_parent_child_cycle() {
    let mut doc = DiagramDocument::default();
    let node_a_id = NodeId::new("node_a".to_string());
    let node_b_id = NodeId::new("node_b".to_string());

    let mut node_a = create_base_node("node_a");
    node_a.parent = Some(node_b_id.clone());

    let mut node_b = create_base_node("node_b");
    node_b.parent = Some(node_a_id.clone());

    doc.document.nodes.insert(node_a_id.clone(), node_a);
    doc.document.nodes.insert(node_b_id.clone(), node_b);

    // This should detect a cycle or prevent it, but instead might infinite loop or panic
    let leaf_node = doc.document.nodes.get(&node_a_id).unwrap();
    let _ = leaf_node.get_world_coords_im(&doc.document.nodes);
}

#[test]
fn attack_7_zero_width_height() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("zero_node".to_string());
    let mut node = create_base_node("zero_node");
    node.width = OrderedFloat(0.0);
    node.height = OrderedFloat(0.0);

    doc.document.nodes.insert(node_id.clone(), node);

    let edge_id = EdgeId::new("edge_1".to_string());
    let edge = create_test_edge("zero_node", "zero_node");
    doc.document.edges.insert(edge_id, edge);

    // Force some calculation
    let _ = doc.clone();
}

#[test]
fn attack_8_extreme_z_index() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("z_node".to_string());
    let mut node = create_base_node("z_node");
    node.z_index = i64::MAX;

    let node_id2 = NodeId::new("z_node2".to_string());
    let mut node2 = create_base_node("z_node2");
    node2.z_index = i64::MIN;

    doc.document.nodes.insert(node_id, node);
    doc.document.nodes.insert(node_id2, node2);

    // Sort nodes by z-index - checking for overflow when subtracting z-indexes in custom sort
    let mut nodes: Vec<_> = doc.document.nodes.values().collect();
    nodes.sort_by(|a, b| a.z_index.cmp(&b.z_index));
}

#[test]
fn attack_9_self_referencing_edge() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("node_1".to_string());
    doc.document
        .nodes
        .insert(node_id.clone(), create_base_node("node_1"));

    let edge_id = EdgeId::new("edge_1".to_string());
    let edge = create_test_edge("node_1", "node_1");
    let res = doc.add_edge(edge_id.clone(), edge);

    // Should ideally be rejected or handled gracefully
    assert!(res.is_ok());
}

#[test]
fn attack_10_missing_source_node_in_edge() {
    let mut doc = DiagramDocument::default();
    let target_id = NodeId::new("target".to_string());
    doc.document
        .nodes
        .insert(target_id.clone(), create_base_node("target"));

    let edge_id = EdgeId::new("edge_1".to_string());
    let edge = create_test_edge("missing_source", "target");

    // By passing add_edge validator
    doc.document.edges.insert(edge_id.clone(), edge);

    let _ = doc.clone();
}
