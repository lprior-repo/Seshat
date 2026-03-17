//! Property-based tests for schema validation - complex DAG tests.

use crate::document::{
    ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EditorState, Node, NodeId, NodeKind,
    NodeStyle, OrderedFloat, Revision,
};
use crate::schema::validate_schema;
use im::HashMap;
use proptest::prelude::*;

fn arb_node_id() -> impl Strategy<Value = NodeId> {
    "[a-z]{1,8}".prop_map(NodeId::new)
}

fn make_node(kind: NodeKind, parent: Option<NodeId>, x: f64, y: f64) -> Node {
    Node {
        kind,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat::new_unchecked(x),
        y: OrderedFloat::new_unchecked(y),
        width: OrderedFloat::new_unchecked(100.0),
        height: OrderedFloat::new_unchecked(60.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent,
        dag_rank: None,
        tags: im::vector![],
        metadata: HashMap::new(),
        z_index: 0,
        style: Some(NodeStyle::default()),
        collapsed: None,
    }
}

fn make_edge(source: NodeId, target: NodeId) -> Edge {
    Edge {
        source,
        target,
        label: String::new(),
        style: crate::document::EdgeStyle::Solid,
        arrow_type: ArrowType::Default,
        label_offset_t: OrderedFloat::new_unchecked(0.5),
        color: None,
        thickness: OrderedFloat::new_unchecked(1.5),
        directed: true,
        bend_points: im::vector![],
        tags: im::vector![],
        metadata: HashMap::new(),
        font_size: None,
            source_port: None,
            target_port: None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_complex_dag(num_nodes in 2usize..10, edge_density in 0.0f64..1.0) {
        let mut nodes = HashMap::new();
        let node_ids: Vec<NodeId> = (0..num_nodes).map(|i| NodeId::new(format!("n{}", i))).collect();
        for id in &node_ids { nodes.insert(id.clone(), make_node(NodeKind::Node, None, 0.0, 0.0)); }
        let mut edges = HashMap::new();
        let mut edge_count = 0usize;
        for (i, src) in node_ids.iter().enumerate() {
            for (j, tgt) in node_ids.iter().enumerate() {
                if i < j && (i as f64 + j as f64) * edge_density < num_nodes as f64 {
                    edges.insert(EdgeId::new(format!("e{}", edge_count)), make_edge(src.clone(), tgt.clone()));
                    edge_count += 1;
                }
            }
        }
        let doc = DiagramDocument { version: 2, revision: Revision::INITIAL, document: DocumentData { nodes, edges }, editor_state: EditorState::default() };
        let _ = validate_schema(&doc);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_linear_dag(num_nodes in 2usize..15) {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let node_ids: Vec<NodeId> = (0..num_nodes).map(|i| NodeId::new(format!("n{}", i))).collect();
        for id in &node_ids { nodes.insert(id.clone(), make_node(NodeKind::Node, None, 0.0, 0.0)); }
        for i in 0..(num_nodes - 1) {
            edges.insert(EdgeId::new(format!("e{}", i)), make_edge(node_ids[i].clone(), node_ids[i + 1].clone()));
        }
        let doc = DiagramDocument { version: 2, revision: Revision::INITIAL, document: DocumentData { nodes, edges }, editor_state: EditorState::default() };
        prop_assert!(validate_schema(&doc).is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_star_dag(center in 1usize..5, spokes in 2usize..8) {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let center_id = NodeId::new(format!("center"));
        nodes.insert(center_id.clone(), make_node(NodeKind::Node, None, 0.0, 0.0));
        for i in 0..spokes {
            let spoke_id = NodeId::new(format!("spoke{}", i));
            nodes.insert(spoke_id.clone(), make_node(NodeKind::Node, None, 100.0 * (i as f64), 0.0));
            edges.insert(EdgeId::new(format!("e{}", i)), make_edge(center_id.clone(), spoke_id));
        }
        let doc = DiagramDocument { version: 2, revision: Revision::INITIAL, document: DocumentData { nodes, edges }, editor_state: EditorState::default() };
        prop_assert!(validate_schema(&doc).is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_complete_dag(num_nodes in 2usize..6) {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let node_ids: Vec<NodeId> = (0..num_nodes).map(|i| NodeId::new(format!("n{}", i))).collect();
        for id in &node_ids { nodes.insert(id.clone(), make_node(NodeKind::Node, None, 0.0, 0.0)); }
        let mut edge_idx = 0;
        for (i, src) in node_ids.iter().enumerate() {
            for (j, tgt) in node_ids.iter().enumerate() {
                if i < j {
                    edges.insert(EdgeId::new(format!("e{}", edge_idx)), make_edge(src.clone(), tgt.clone()));
                    edge_idx += 1;
                }
            }
        }
        let doc = DiagramDocument { version: 2, revision: Revision::INITIAL, document: DocumentData { nodes, edges }, editor_state: EditorState::default() };
        prop_assert!(validate_schema(&doc).is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_disconnected_components(num_components in 2usize..5, nodes_per_component in 2usize..5) {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let mut edge_idx = 0;
        for c in 0..num_components {
            let component_nodes: Vec<NodeId> = (0..nodes_per_component).map(|i| NodeId::new(format!("c{}_n{}", c, i))).collect();
            for id in &component_nodes { nodes.insert(id.clone(), make_node(NodeKind::Node, None, 0.0, 0.0)); }
            for i in 0..(nodes_per_component - 1) {
                edges.insert(EdgeId::new(format!("e{}", edge_idx)), make_edge(component_nodes[i].clone(), component_nodes[i + 1].clone()));
                edge_idx += 1;
            }
        }
        let doc = DiagramDocument { version: 2, revision: Revision::INITIAL, document: DocumentData { nodes, edges }, editor_state: EditorState::default() };
        prop_assert!(validate_schema(&doc).is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_dag_with_reverse_edges_fails(num_nodes in 3usize..8) {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let node_ids: Vec<NodeId> = (0..num_nodes).map(|i| NodeId::new(format!("n{}", i))).collect();
        for id in &node_ids { nodes.insert(id.clone(), make_node(NodeKind::Node, None, 0.0, 0.0)); }
        // Create a cycle: n0->n1->n2->...->n0
        for i in 0..num_nodes {
            let next = (i + 1) % num_nodes;
            edges.insert(EdgeId::new(format!("e{}", i)), make_edge(node_ids[i].clone(), node_ids[next].clone()));
        }
        let doc = DiagramDocument { version: 2, revision: Revision::INITIAL, document: DocumentData { nodes, edges }, editor_state: EditorState::default() };
        prop_assert!(validate_schema(&doc).is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_mixed_node_kinds_in_dag(num_nodes in 3usize..10) {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let node_ids: Vec<NodeId> = (0..num_nodes).map(|i| NodeId::new(format!("n{}", i))).collect();
        for (i, id) in node_ids.iter().enumerate() {
            let kind = match i % 3 { 0 => NodeKind::Node, 1 => NodeKind::Subgraph, _ => NodeKind::Text };
            nodes.insert(id.clone(), make_node(kind, None, 0.0, 0.0));
        }
        // Create linear edges
        for i in 0..(num_nodes - 1) {
            edges.insert(EdgeId::new(format!("e{}", i)), make_edge(node_ids[i].clone(), node_ids[i + 1].clone()));
        }
        let doc = DiagramDocument { version: 2, revision: Revision::INITIAL, document: DocumentData { nodes, edges }, editor_state: EditorState::default() };
        prop_assert!(validate_schema(&doc).is_ok());
    }
}
