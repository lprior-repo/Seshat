#![cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
use super::z_order::*;
use crate::models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};
use std::collections::BTreeSet;

fn make_node(label: &str) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: label.to_string(),
        x: OrderedFloat(0.0),
        y: OrderedFloat(0.0),
        width: OrderedFloat(100.0),
        height: OrderedFloat(50.0),
        font_size: None,
        font_weight: None,
        locked: false,
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
fn given_selected_middle_node_when_bring_to_front_then_relative_order_preserved() {
    let mut ids = vec![
        NodeId::new(String::from("a")),
        NodeId::new(String::from("b")),
        NodeId::new(String::from("c")),
    ];
    let mut selected = BTreeSet::new();
    let _ = selected.insert(NodeId::new(String::from("b")));

    apply_z_order_to_ids(&mut ids, &selected, ZOrderOp::BringToFront);

    assert_eq!(
        ids,
        vec![
            NodeId::new(String::from("a")),
            NodeId::new(String::from("c")),
            NodeId::new(String::from("b")),
        ]
    );
}

#[test]
fn given_selected_middle_node_when_send_to_back_then_relative_order_preserved() {
    let mut ids = vec![
        NodeId::new(String::from("a")),
        NodeId::new(String::from("b")),
        NodeId::new(String::from("c")),
    ];
    let mut selected = BTreeSet::new();
    let _ = selected.insert(NodeId::new(String::from("b")));

    apply_z_order_to_ids(&mut ids, &selected, ZOrderOp::SendToBack);

    assert_eq!(
        ids,
        vec![
            NodeId::new(String::from("b")),
            NodeId::new(String::from("a")),
            NodeId::new(String::from("c")),
        ]
    );
}

#[test]
fn test_bring_forward() {
    let mut doc = DiagramDocument::default();
    let id_a = NodeId::new("a".to_string());
    let id_b = NodeId::new("b".to_string());
    let id_c = NodeId::new("c".to_string());

    doc.document.nodes.insert(id_a.clone(), make_node("a"));
    doc.document.nodes.insert(id_b.clone(), make_node("b"));
    doc.document.nodes.insert(id_c.clone(), make_node("c"));

    doc.document.nodes.get_mut(&id_a).unwrap().z_index = 0;
    doc.document.nodes.get_mut(&id_b).unwrap().z_index = 1;
    doc.document.nodes.get_mut(&id_c).unwrap().z_index = 2;

    doc.editor_state.selected_items.insert("b".to_string());

    assert!(bring_forward(&mut doc));

    // Order should now be a, c, b (z-indexes: a=0, c=1, b=2)
    assert_eq!(doc.document.nodes.get(&id_a).unwrap().z_index, 0);
    assert_eq!(doc.document.nodes.get(&id_c).unwrap().z_index, 1);
    assert_eq!(doc.document.nodes.get(&id_b).unwrap().z_index, 2);
}

#[test]
fn test_send_backward() {
    let mut doc = DiagramDocument::default();
    let id_a = NodeId::new("a".to_string());
    let id_b = NodeId::new("b".to_string());
    let id_c = NodeId::new("c".to_string());

    doc.document.nodes.insert(id_a.clone(), make_node("a"));
    doc.document.nodes.insert(id_b.clone(), make_node("b"));
    doc.document.nodes.insert(id_c.clone(), make_node("c"));

    doc.document.nodes.get_mut(&id_a).unwrap().z_index = 0;
    doc.document.nodes.get_mut(&id_b).unwrap().z_index = 1;
    doc.document.nodes.get_mut(&id_c).unwrap().z_index = 2;

    doc.editor_state.selected_items.insert("b".to_string());

    assert!(send_backward(&mut doc));

    // Order should now be b, a, c (z-indexes: b=0, a=1, c=2)
    assert_eq!(doc.document.nodes.get(&id_b).unwrap().z_index, 0);
    assert_eq!(doc.document.nodes.get(&id_a).unwrap().z_index, 1);
    assert_eq!(doc.document.nodes.get(&id_c).unwrap().z_index, 2);
}

#[test]
fn test_bring_to_front() {
    let mut doc = DiagramDocument::default();
    let id_a = NodeId::new("a".to_string());
    let id_b = NodeId::new("b".to_string());
    let id_c = NodeId::new("c".to_string());

    doc.document.nodes.insert(id_a.clone(), make_node("a"));
    doc.document.nodes.insert(id_b.clone(), make_node("b"));
    doc.document.nodes.insert(id_c.clone(), make_node("c"));

    doc.document.nodes.get_mut(&id_a).unwrap().z_index = 0;
    doc.document.nodes.get_mut(&id_b).unwrap().z_index = 1;
    doc.document.nodes.get_mut(&id_c).unwrap().z_index = 2;

    doc.editor_state.selected_items.insert("a".to_string());

    assert!(bring_to_front(&mut doc));

    // Order should now be b, c, a (z-indexes: b=0, c=1, a=2)
    assert_eq!(doc.document.nodes.get(&id_b).unwrap().z_index, 0);
    assert_eq!(doc.document.nodes.get(&id_c).unwrap().z_index, 1);
    assert_eq!(doc.document.nodes.get(&id_a).unwrap().z_index, 2);
}

#[test]
fn test_send_to_back() {
    let mut doc = DiagramDocument::default();
    let id_a = NodeId::new("a".to_string());
    let id_b = NodeId::new("b".to_string());
    let id_c = NodeId::new("c".to_string());

    doc.document.nodes.insert(id_a.clone(), make_node("a"));
    doc.document.nodes.insert(id_b.clone(), make_node("b"));
    doc.document.nodes.insert(id_c.clone(), make_node("c"));

    doc.document.nodes.get_mut(&id_a).unwrap().z_index = 0;
    doc.document.nodes.get_mut(&id_b).unwrap().z_index = 1;
    doc.document.nodes.get_mut(&id_c).unwrap().z_index = 2;

    doc.editor_state.selected_items.insert("c".to_string());

    assert!(send_to_back(&mut doc));

    // Order should now be c, a, b (z-indexes: c=0, a=1, b=2)
    assert_eq!(doc.document.nodes.get(&id_c).unwrap().z_index, 0);
    assert_eq!(doc.document.nodes.get(&id_a).unwrap().z_index, 1);
    assert_eq!(doc.document.nodes.get(&id_b).unwrap().z_index, 2);
}
