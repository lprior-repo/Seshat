#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    unused_variables,
    unused_imports
)]

use super::*;
use crate::document::{Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat};
use crate::grouping::validation::GroupingError;
use im::{HashMap, HashSet};
use std::collections::BTreeSet;

fn test_node(id: &str, parent: Option<&str>) -> (NodeId, Node) {
    let node_id = NodeId::new(id.to_string());
    let node = Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat::new_unchecked(0.0),
        y: OrderedFloat::new_unchecked(0.0),
        width: OrderedFloat::new_unchecked(100.0),
        height: OrderedFloat::new_unchecked(100.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: parent.map(|s| NodeId::new(s.to_string())),
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };
    (node_id, node)
}

fn test_edge(id: &str, source: &str, target: &str) -> (EdgeId, Edge) {
    let edge_id = EdgeId::new(id.to_string());
    let edge = Edge {
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
    };
    (edge_id, edge)
}

// ---------------------------------------------------------------------------
// calculate_bounding_box
// ---------------------------------------------------------------------------

#[test]
fn given_single_node_at_100_100_when_calculate_bounding_box_then_returns_100_100_200_200() {
    let (id, mut node) = test_node("n1", None);
    node.x = OrderedFloat::new_unchecked(100.0);
    node.y = OrderedFloat::new_unchecked(100.0);
    node.width = OrderedFloat::new_unchecked(100.0);
    node.height = OrderedFloat::new_unchecked(100.0);

    let nodes = HashMap::unit(id.clone(), node);
    let selected = HashSet::unit(id);

    assert_eq!(
        calculate_bounding_box(&nodes, &selected),
        Some((100.0, 100.0, 200.0, 200.0))
    );
}

#[test]
fn given_two_nodes_when_calculate_bounding_box_then_returns_combined_bounds() {
    let (id1, mut node1) = test_node("n1", None);
    node1.x = OrderedFloat::new_unchecked(0.0);
    node1.y = OrderedFloat::new_unchecked(0.0);
    node1.width = OrderedFloat::new_unchecked(10.0);
    node1.height = OrderedFloat::new_unchecked(10.0);

    let (id2, mut node2) = test_node("n2", None);
    node2.x = OrderedFloat::new_unchecked(100.0);
    node2.y = OrderedFloat::new_unchecked(100.0);
    node2.width = OrderedFloat::new_unchecked(10.0);
    node2.height = OrderedFloat::new_unchecked(10.0);

    let nodes = HashMap::from_iter([(id1.clone(), node1), (id2.clone(), node2)]);
    let selected = HashSet::from_iter([id1, id2]);

    assert_eq!(
        calculate_bounding_box(&nodes, &selected),
        Some((0.0, 0.0, 110.0, 110.0))
    );
}

#[test]
fn given_empty_selection_when_calculate_bounding_box_then_returns_none() {
    let nodes = HashMap::new();
    let selected = HashSet::new();

    assert_eq!(calculate_bounding_box(&nodes, &selected), None);
}

// ---------------------------------------------------------------------------
// compute_padded_bounds
// ---------------------------------------------------------------------------

#[test]
fn given_single_node_when_compute_padded_bounds_then_includes_24px_padding() {
    let (id, mut node) = test_node("n1", None);
    node.x = OrderedFloat::new_unchecked(10.0);
    node.y = OrderedFloat::new_unchecked(10.0);
    node.width = OrderedFloat::new_unchecked(10.0);
    node.height = OrderedFloat::new_unchecked(10.0);

    let nodes = HashMap::unit(id.clone(), node);
    let selected = HashSet::unit(id);

    // Bounding box: (10, 10, 20, 20)
    // Padded min: (10-24, 10-24) = (-14, -14)
    // Padded max: (20+24, 20+24) = (44, 44)
    // Width  = 44 - (-14) = 58
    // Height = 44 - (-14) = 58
    assert_eq!(
        compute_padded_bounds(&nodes, &selected),
        Ok((-14.0, -14.0, 58.0, 58.0))
    );
}

#[test]
fn given_empty_selection_when_compute_padded_bounds_then_returns_empty_selection_error() {
    let nodes = HashMap::new();
    let selected = HashSet::new();

    assert_eq!(
        compute_padded_bounds(&nodes, &selected),
        Err(GroupingError::EmptySelection)
    );
}

// ---------------------------------------------------------------------------
// find_lca
// ---------------------------------------------------------------------------

#[test]
fn given_siblings_under_same_parent_when_find_lca_then_returns_parent() {
    let (parent_id, parent) = test_node("parent", None);
    let (child_a, child_node_a) = test_node("child_a", Some("parent"));
    let (child_b, child_node_b) = test_node("child_b", Some("parent"));

    let nodes = HashMap::from_iter([
        (parent_id.clone(), parent),
        (child_a.clone(), child_node_a),
        (child_b.clone(), child_node_b),
    ]);
    let selected = HashSet::from_iter([child_a, child_b]);

    assert_eq!(find_lca(&nodes, &selected), Some(parent_id));
}

#[test]
fn given_nodes_at_different_parents_when_find_lca_then_returns_common_ancestor() {
    // grandparent
    //   ├── parentA
    //   │     └── childA  (selected)
    //   └── parentB
    //         └── childB  (selected)
    let (grandparent_id, grandparent) = test_node("grandparent", None);
    let (parent_a, parent_node_a) = test_node("parentA", Some("grandparent"));
    let (parent_b, parent_node_b) = test_node("parentB", Some("grandparent"));
    let (child_a, child_node_a) = test_node("childA", Some("parentA"));
    let (child_b, child_node_b) = test_node("childB", Some("parentB"));

    let nodes = HashMap::from_iter([
        (grandparent_id.clone(), grandparent),
        (parent_a, parent_node_a),
        (parent_b, parent_node_b),
        (child_a.clone(), child_node_a),
        (child_b.clone(), child_node_b),
    ]);
    let selected = HashSet::from_iter([child_a, child_b]);

    assert_eq!(find_lca(&nodes, &selected), Some(grandparent_id));
}

#[test]
fn given_empty_selection_when_find_lca_then_returns_none() {
    let nodes = HashMap::new();
    let selected = HashSet::new();

    assert_eq!(find_lca(&nodes, &selected), None);
}

// ---------------------------------------------------------------------------
// calculate_edge_cleanup
// ---------------------------------------------------------------------------

#[test]
fn given_edges_to_deleted_subgraphs_when_calculate_edge_cleanup_then_removes_them() {
    let (e1_id, e1) = test_edge("e1", "deleted_sg", "n1");
    let (e2_id, e2) = test_edge("e2", "n2", "deleted_sg");

    let edges = HashMap::from_iter([(e1_id, e1), (e2_id, e2)]);
    let deleted: BTreeSet<NodeId> = BTreeSet::from_iter([NodeId::new("deleted_sg".to_string())]);

    let result = calculate_edge_cleanup(&edges, &deleted);
    assert_eq!(result.len(), 0);
}

#[test]
fn given_edges_to_non_deleted_nodes_when_calculate_edge_cleanup_then_preserves_them() {
    let (e1_id, e1) = test_edge("e1", "n1", "n2");
    let (e2_id, e2) = test_edge("e2", "n3", "n4");

    let edges = HashMap::from_iter([(e1_id, e1), (e2_id, e2)]);
    let deleted: BTreeSet<NodeId> = BTreeSet::from_iter([NodeId::new("deleted_sg".to_string())]);

    let result = calculate_edge_cleanup(&edges, &deleted);
    assert_eq!(result.len(), 2);
}

#[test]
fn given_edge_between_two_deleted_when_calculate_edge_cleanup_then_removes() {
    let (e1_id, e1) = test_edge("e1", "deleted_a", "deleted_b");

    let edges = HashMap::unit(e1_id, e1);
    let deleted: BTreeSet<NodeId> = BTreeSet::from_iter([
        NodeId::new("deleted_a".to_string()),
        NodeId::new("deleted_b".to_string()),
    ]);

    let result = calculate_edge_cleanup(&edges, &deleted);
    assert_eq!(result.len(), 0);
}
