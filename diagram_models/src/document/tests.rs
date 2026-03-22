#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
//! Tests for the document module.

use crate::document::{
    DiagramDocument, DocumentError, Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat,
    Revision,
};

fn create_test_node(id: &str) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: id.to_string(),
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

fn create_test_edge(source: &str, target: &str) -> Edge {
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
fn valid_edge_creation() {
    let mut doc = DiagramDocument::default();
    doc.document
        .nodes
        .insert(NodeId::new("N1".into()), create_test_node("N1"));
    doc.document
        .nodes
        .insert(NodeId::new("N2".into()), create_test_node("N2"));

    let edge = create_test_edge("N1", "N2");
    let result = doc.add_edge(EdgeId::new("E1".into()), edge);

    assert!(result.is_ok());
    assert!(doc.document.edges.contains_key(&EdgeId::new("E1".into())));
}

#[test]
fn invalid_edge_missing_source() {
    let mut doc = DiagramDocument::default();
    doc.document
        .nodes
        .insert(NodeId::new("N2".into()), create_test_node("N2"));

    let edge = create_test_edge("N1", "N2");
    let result = doc.add_edge(EdgeId::new("E1".into()), edge);

    assert_eq!(
        result,
        Err(DocumentError::NodeNotFound(NodeId::new("N1".into())))
    );
    assert!(!doc.document.edges.contains_key(&EdgeId::new("E1".into())));
}

#[test]
fn invalid_edge_missing_target() {
    let mut doc = DiagramDocument::default();
    doc.document
        .nodes
        .insert(NodeId::new("N1".into()), create_test_node("N1"));

    let edge = create_test_edge("N1", "N2");
    let result = doc.add_edge(EdgeId::new("E1".into()), edge);

    assert_eq!(
        result,
        Err(DocumentError::NodeNotFound(NodeId::new("N2".into())))
    );
    assert!(!doc.document.edges.contains_key(&EdgeId::new("E1".into())));
}

#[test]
fn edge_deletion_isolated() {
    let mut doc = DiagramDocument::default();
    doc.document
        .nodes
        .insert(NodeId::new("N1".into()), create_test_node("N1"));
    doc.document
        .nodes
        .insert(NodeId::new("N2".into()), create_test_node("N2"));

    let edge = create_test_edge("N1", "N2");
    doc.add_edge(EdgeId::new("E1".into()), edge).unwrap();

    let result = doc.remove_edge(&EdgeId::new("E1".into()));
    assert!(result.is_ok());
    assert!(!doc.document.edges.contains_key(&EdgeId::new("E1".into())));
    assert!(doc.document.nodes.contains_key(&NodeId::new("N1".into())));
    assert!(doc.document.nodes.contains_key(&NodeId::new("N2".into())));
}

#[test]
fn node_deletion_cascades_edges() {
    let mut doc = DiagramDocument::default();
    doc.document
        .nodes
        .insert(NodeId::new("N1".into()), create_test_node("N1"));
    doc.document
        .nodes
        .insert(NodeId::new("N2".into()), create_test_node("N2"));
    doc.document
        .nodes
        .insert(NodeId::new("N3".into()), create_test_node("N3"));

    doc.add_edge(EdgeId::new("E1".into()), create_test_edge("N1", "N2"))
        .unwrap();
    doc.add_edge(EdgeId::new("E2".into()), create_test_edge("N2", "N3"))
        .unwrap();

    let result = doc.remove_node(&NodeId::new("N2".into()));
    assert!(result.is_ok());
    assert!(!doc.document.nodes.contains_key(&NodeId::new("N2".into())));
    assert!(!doc.document.edges.contains_key(&EdgeId::new("E1".into())));
    assert!(!doc.document.edges.contains_key(&EdgeId::new("E2".into())));
    assert!(doc.document.nodes.contains_key(&NodeId::new("N1".into())));
    assert!(doc.document.nodes.contains_key(&NodeId::new("N3".into())));
}

#[test]
fn returns_error_when_creating_edge_with_duplicate_id() {
    let mut doc = DiagramDocument::default();
    doc.document
        .nodes
        .insert(NodeId::new("N1".into()), create_test_node("N1"));
    doc.document
        .nodes
        .insert(NodeId::new("N2".into()), create_test_node("N2"));

    doc.add_edge(EdgeId::new("E1".into()), create_test_edge("N1", "N2"))
        .unwrap();
    let result = doc.add_edge(EdgeId::new("E1".into()), create_test_edge("N1", "N2"));
    assert_eq!(
        result,
        Err(DocumentError::EdgeAlreadyExists(EdgeId::new("E1".into())))
    );
}

#[test]
fn returns_error_when_deleting_missing_edge() {
    let mut doc = DiagramDocument::default();
    let result = doc.remove_edge(&EdgeId::new("E1".into()));
    assert_eq!(
        result,
        Err(DocumentError::EdgeNotFound(EdgeId::new("E1".into())))
    );
}

#[test]
fn cascading_deletion_handles_multiple_edges_on_same_node() {
    let mut doc = DiagramDocument::default();
    doc.document
        .nodes
        .insert(NodeId::new("N1".into()), create_test_node("N1"));
    doc.document
        .nodes
        .insert(NodeId::new("N2".into()), create_test_node("N2"));

    doc.add_edge(EdgeId::new("E1".into()), create_test_edge("N1", "N2"))
        .unwrap();
    doc.add_edge(EdgeId::new("E2".into()), create_test_edge("N1", "N2"))
        .unwrap();
    doc.add_edge(EdgeId::new("E3".into()), create_test_edge("N2", "N1"))
        .unwrap();

    doc.remove_node(&NodeId::new("N1".into())).unwrap();

    assert!(doc.document.edges.is_empty());
}

#[test]
fn cascading_deletion_handles_self_loop() {
    let mut doc = DiagramDocument::default();
    doc.document
        .nodes
        .insert(NodeId::new("N1".into()), create_test_node("N1"));
    doc.add_edge(EdgeId::new("E1".into()), create_test_edge("N1", "N1"))
        .unwrap();

    doc.remove_node(&NodeId::new("N1".into())).unwrap();
    assert!(doc.document.edges.is_empty());
}

#[test]
fn invariant_all_edges_reference_existing_nodes() {
    let mut doc = DiagramDocument::default();
    doc.document
        .nodes
        .insert(NodeId::new("N1".into()), create_test_node("N1"));
    doc.document
        .nodes
        .insert(NodeId::new("N2".into()), create_test_node("N2"));
    doc.add_edge(EdgeId::new("E1".into()), create_test_edge("N1", "N2"))
        .unwrap();

    // Remove node cascades, keeping invariant intact
    doc.remove_node(&NodeId::new("N1".into())).unwrap();
    for edge in doc.document.edges.values() {
        assert!(doc.document.nodes.contains_key(&edge.source));
        assert!(doc.document.nodes.contains_key(&edge.target));
    }
}

#[test]
fn document_default_version_is_2() {
    let doc = DiagramDocument::default();
    assert_eq!(doc.version, 2);
}

#[test]
fn document_default_revision_is_initial() {
    let doc = DiagramDocument::default();
    assert_eq!(doc.revision, Revision::INITIAL);
}
