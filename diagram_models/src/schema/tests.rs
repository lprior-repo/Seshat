#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use crate::document::{
    ArrowType, Edge, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use im::HashMap;

fn node(kind: NodeKind, parent: Option<NodeId>) -> Node {
    Node {
        kind,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat::new_unchecked(0.0),
        y: OrderedFloat::new_unchecked(0.0),
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

fn edge(source: &NodeId, target: &NodeId) -> Edge {
    Edge {
        source: source.clone(),
        target: target.clone(),
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

#[cfg(kani)]
#[kani::proof]
fn given_default_document_when_validated_then_schema_passes() {
    let doc = DiagramDocument::default();
    let result = validate_schema(&doc);
    assert!(result.is_ok());
}

#[cfg(kani)]
#[kani::proof]
fn given_non_v2_document_when_validated_then_schema_fails_without_runtime_gate() {
    let doc = DiagramDocument {
        version: 3,
        ..DiagramDocument::default()
    };

    let result = validate_schema(&doc);
    assert!(result.is_err());
}

#[cfg(kani)]
#[kani::proof]
fn given_node_parent_that_is_not_subgraph_when_validated_then_schema_fails() {
    let parent_id = NodeId::new(String::from("parent"));
    let child_id = NodeId::new(String::from("child"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new()
        .update(parent_id.clone(), node(NodeKind::Node, None))
        .update(child_id, node(NodeKind::Node, Some(parent_id)));

    assert!(validate_schema(&doc).is_err());
}

#[cfg(kani)]
#[kani::proof]
fn given_edge_with_missing_target_when_validated_then_schema_fails() {
    let a = NodeId::new(String::from("a"));
    let b = NodeId::new(String::from("b"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new().update(a.clone(), node(NodeKind::Node, None));
    doc.document.edges = HashMap::new().update(EdgeId::new(String::from("e1")), edge(&a, &b));

    assert!(validate_schema(&doc).is_err());
}

#[cfg(kani)]
#[kani::proof]
fn given_node_with_missing_parent_reference_when_validated_then_schema_fails() {
    let missing_parent = NodeId::new(String::from("missing-parent"));
    let child_id = NodeId::new(String::from("child"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes =
        HashMap::new().update(child_id, node(NodeKind::Node, Some(missing_parent)));

    assert!(validate_schema(&doc).is_err());
}

#[cfg(kani)]
#[kani::proof]
fn given_node_with_existing_subgraph_parent_when_validated_then_schema_passes() {
    let parent_id = NodeId::new(String::from("parent"));
    let child_id = NodeId::new(String::from("child"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new()
        .update(parent_id.clone(), node(NodeKind::Subgraph, None))
        .update(child_id, node(NodeKind::Node, Some(parent_id)));

    assert!(validate_schema(&doc).is_ok());
}

// =============================================================================
// SUB subgraph tests (bd-163) - Parent cycle prevention
// =============================================================================

#[cfg(kani)]
#[kani::proof]
fn given_circular_parent_chain_when_validated_then_schema_fails() {
    // Create a cycle: A -> B -> C -> A
    let a_id = NodeId::new(String::from("subgraph-a"));
    let b_id = NodeId::new(String::from("subgraph-b"));
    let c_id = NodeId::new(String::from("subgraph-c"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new()
        // A's parent is C
        .update(a_id.clone(), node(NodeKind::Subgraph, Some(c_id.clone())))
        // B's parent is A
        .update(b_id.clone(), node(NodeKind::Subgraph, Some(a_id.clone())))
        // C's parent is B
        .update(c_id, node(NodeKind::Subgraph, Some(b_id)));

    let result = validate_schema(&doc);
    assert!(
        result.is_err(),
        "circular parent chain should fail validation"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.to_lowercase().contains("circular") || err_msg.to_lowercase().contains("cycle"),
        "error message should mention circular or cycle: {}",
        err_msg
    );
}

#[cfg(kani)]
#[kani::proof]
fn given_self_referential_parent_when_validated_then_schema_fails() {
    // A node that is its own parent
    let a_id = NodeId::new(String::from("subgraph-a"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new().update(a_id.clone(), node(NodeKind::Subgraph, Some(a_id)));

    let result = validate_schema(&doc);
    assert!(
        result.is_err(),
        "self-referential parent should fail validation"
    );
}

#[cfg(kani)]
#[kani::proof]
fn given_two_node_parent_cycle_when_validated_then_schema_fails() {
    // Create a 2-node cycle: A -> B -> A
    let a_id = NodeId::new(String::from("subgraph-a"));
    let b_id = NodeId::new(String::from("subgraph-b"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new()
        .update(a_id.clone(), node(NodeKind::Subgraph, Some(b_id.clone())))
        .update(b_id, node(NodeKind::Subgraph, Some(a_id)));

    let result = validate_schema(&doc);
    assert!(
        result.is_err(),
        "two-node parent cycle should fail validation"
    );
}
