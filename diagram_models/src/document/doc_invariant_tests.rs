#![allow(
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
//! Document and Scene Graph Invariant Tests: DOC-001 to DOC-020
//!
//! These 20 test cases verify node CRUD, edge CRUD, DAG integrity,
//! revision monotonicity, schema validation, optimistic concurrency,
//! atomic operations, and cascade delete.

use crate::dag::validate_dag;
use crate::document::{
    DiagramDocument, DocumentData, DocumentError, Edge, EdgeId, LockState, Node, NodeId, NodeKind,
    OrderedFloat, OrderedFloatError, Revision,
};
use crate::validation::rules::validate_document_data;
use crate::validation::types::ValidationCode;
use im::HashMap;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn test_node(id: &str) -> Node {
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

fn test_node_at(id: &str, x: f64, y: f64) -> Node {
    Node {
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        ..test_node(id)
    }
}

fn test_edge(source: &str, target: &str) -> Edge {
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

fn doc_with_nodes(ids: &[&str]) -> DiagramDocument {
    let mut doc = DiagramDocument::default();
    for id in ids {
        let node_id = NodeId::new(id.to_string());
        doc.document.nodes.insert(node_id, test_node(id));
    }
    doc
}

fn doc_with_chain(n: usize) -> DiagramDocument {
    let mut doc = DiagramDocument::default();
    for i in 0..n {
        let id = format!("N{i}");
        let node_id = NodeId::new(id.clone());
        doc.document.nodes.insert(node_id, test_node(&id));
    }
    for i in 0..n.saturating_sub(1) {
        let src = format!("N{i}");
        let tgt = format!("N{}", i + 1);
        let edge_id = EdgeId::new(format!("E{i}"));
        doc.document
            .edges
            .insert(edge_id, test_edge(&src, &tgt));
    }
    doc
}

// ===========================================================================
// DOC-001: Create node with valid data
// ===========================================================================

#[test]
fn doc_001_create_node_with_valid_data() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("N1".to_string());
    let node = test_node("N1");

    let result = doc.add_node(node_id.clone(), node.clone());

    assert!(result.is_ok(), "add_node should succeed for valid data");
    assert_eq!(doc.document.nodes.len(), 1);
    assert_eq!(doc.document.nodes.get(&node_id), Some(&node));
}

// ===========================================================================
// DOC-002: Create node with duplicate ID rejected
// ===========================================================================

#[test]
fn doc_002_create_node_with_duplicate_id_rejected() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("N1".to_string());

    let first = doc.add_node(node_id.clone(), test_node("N1"));
    assert!(first.is_ok(), "First add_node should succeed");

    let second = doc.add_node(node_id.clone(), test_node("N1-v2"));
    assert_eq!(
        second,
        Err(DocumentError::NodeAlreadyExists(node_id)),
        "Duplicate node ID must be rejected"
    );

    // Only one entry should exist
    assert_eq!(doc.document.nodes.len(), 1);
}

// ===========================================================================
// DOC-003: Delete existing node
// ===========================================================================

#[test]
fn doc_003_delete_existing_node() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("N1".to_string());
    doc.document.nodes.insert(node_id.clone(), test_node("N1"));

    let result = doc.remove_node(&node_id);

    assert!(result.is_ok(), "Removing existing node should succeed");
    assert!(
        !doc.document.nodes.contains_key(&node_id),
        "Node should be removed"
    );
    assert!(doc.document.nodes.is_empty(), "No nodes should remain");
}

// ===========================================================================
// DOC-004: Delete non-existent node returns error
// ===========================================================================

#[test]
fn doc_004_delete_non_existent_node_returns_error() {
    let doc = DiagramDocument::default();
    let missing_id = NodeId::new("N999".to_string());

    // Use remove_node which already checks existence
    let mut doc = doc;
    let result = doc.remove_node(&missing_id);

    assert_eq!(
        result,
        Err(DocumentError::NodeNotFound(missing_id)),
        "Removing non-existent node must return NodeNotFound"
    );
}

// ===========================================================================
// DOC-005: Update node position
// ===========================================================================

#[test]
fn doc_005_update_node_position() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("N1".to_string());
    doc.document
        .nodes
        .insert(node_id.clone(), test_node_at("N1", 0.0, 0.0));

    let result = doc.update_node(&node_id, |node| Node {
        x: OrderedFloat(150.0),
        y: OrderedFloat(250.0),
        ..node
    });

    assert!(result.is_ok(), "update_node should succeed");
    let updated = doc.document.nodes.get(&node_id);
    assert!(updated.is_some());
    let updated = updated.unwrap();
    assert_eq!(updated.x, OrderedFloat(150.0));
    assert_eq!(updated.y, OrderedFloat(250.0));
    // Other fields preserved
    assert_eq!(updated.width, OrderedFloat(100.0));
    assert_eq!(updated.height, OrderedFloat(100.0));
    assert_eq!(updated.label, "N1");
}

// ===========================================================================
// DOC-006: Create edge between valid nodes
// ===========================================================================

#[test]
fn doc_006_create_edge_between_valid_nodes() {
    let mut doc = doc_with_nodes(&["N1", "N2"]);
    let edge_id = EdgeId::new("E1".to_string());
    let edge = test_edge("N1", "N2");

    let result = doc.add_edge(edge_id.clone(), edge);

    assert!(result.is_ok(), "add_edge between valid nodes should succeed");
    assert!(doc.document.edges.contains_key(&edge_id));
    assert_eq!(doc.document.edges.len(), 1);
}

// ===========================================================================
// DOC-007: Create edge with non-existent source rejected
// ===========================================================================

#[test]
fn doc_007_create_edge_with_nonexistent_source_rejected() {
    let mut doc = doc_with_nodes(&["N2"]);
    let edge = test_edge("N1", "N2"); // N1 does not exist

    let result = doc.add_edge(EdgeId::new("E1".to_string()), edge);

    assert_eq!(
        result,
        Err(DocumentError::NodeNotFound(NodeId::new("N1".to_string()))),
        "Edge with non-existent source must be rejected"
    );
    assert!(
        doc.document.edges.is_empty(),
        "No edges should be created"
    );
}

// ===========================================================================
// DOC-008: Create edge that would form cycle rejected (validation)
// ===========================================================================

#[test]
fn doc_008_create_edge_forms_cycle_detected_by_validation() {
    // Build N1 -> N2 -> N3 chain, then add N3 -> N1 (cycle)
    let mut doc = doc_with_nodes(&["N1", "N2", "N3"]);
    // Directly insert edges bypassing DAG check (add_edge doesn't check cycles)
    doc.document
        .edges
        .insert(EdgeId::new("E1".to_string()), test_edge("N1", "N2"));
    doc.document
        .edges
        .insert(EdgeId::new("E2".to_string()), test_edge("N2", "N3"));
    doc.document
        .edges
        .insert(EdgeId::new("E3".to_string()), test_edge("N3", "N1"));

    // Validate DAG - should detect cycle
    let dag_result = validate_dag(&doc.document.nodes, &doc.document.edges);
    assert!(dag_result.is_err(), "DAG validation should detect the cycle");

    // Also check via document validation
    let issues = validate_document_data(&doc.document);
    let cycle_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.code == ValidationCode::DAG_CYCLE)
        .collect();
    assert!(
        !cycle_issues.is_empty(),
        "Document validation should report a DAG cycle issue"
    );
}

// ===========================================================================
// DOC-009: Delete edge
// ===========================================================================

#[test]
fn doc_009_delete_edge() {
    let mut doc = doc_with_nodes(&["N1", "N2"]);
    let edge_id = EdgeId::new("E1".to_string());
    doc.add_edge(edge_id.clone(), test_edge("N1", "N2")).ok();

    let result = doc.remove_edge(&edge_id);

    assert!(result.is_ok(), "Removing existing edge should succeed");
    assert!(
        !doc.document.edges.contains_key(&edge_id),
        "Edge should be removed"
    );
    // Nodes should be unaffected
    assert_eq!(doc.document.nodes.len(), 2);
}

// ===========================================================================
// DOC-010: Delete node cascades connected edges
// ===========================================================================

#[test]
fn doc_010_delete_node_cascades_connected_edges() {
    let mut doc = doc_with_nodes(&["N1", "N2", "N3"]);
    doc.add_edge(EdgeId::new("E1".to_string()), test_edge("N1", "N2"))
        .ok();
    doc.add_edge(EdgeId::new("E2".to_string()), test_edge("N2", "N3"))
        .ok();

    let result = doc.remove_node(&NodeId::new("N2".to_string()));

    assert!(result.is_ok());
    assert!(
        !doc.document.nodes.contains_key(&NodeId::new("N2".to_string())),
        "N2 should be removed"
    );
    assert!(
        !doc.document.edges.contains_key(&EdgeId::new("E1".to_string())),
        "E1 (connected to N2) should be cascade-deleted"
    );
    assert!(
        !doc.document.edges.contains_key(&EdgeId::new("E2".to_string())),
        "E2 (connected to N2) should be cascade-deleted"
    );
    assert!(
        doc.document.nodes.contains_key(&NodeId::new("N1".to_string())),
        "N1 should remain"
    );
    assert!(
        doc.document.nodes.contains_key(&NodeId::new("N3".to_string())),
        "N3 should remain"
    );
}

// ===========================================================================
// DOC-011: Revision increments on every mutation
// ===========================================================================

#[test]
fn doc_011_revision_increments_on_add_node() {
    let mut doc = DiagramDocument::default();
    assert_eq!(doc.revision, Revision::INITIAL, "Initial revision should be 0");

    doc.add_node(NodeId::new("N1".to_string()), test_node("N1")).ok();
    assert_eq!(doc.revision.value(), 1, "Revision should be 1 after first add_node");
}

#[test]
fn doc_011_revision_increments_on_add_edge() {
    let mut doc = doc_with_nodes(&["N1", "N2"]);
    let rev_before = doc.revision;

    doc.add_edge(EdgeId::new("E1".to_string()), test_edge("N1", "N2")).ok();
    assert!(
        doc.revision.value() > rev_before.value(),
        "Revision should increase after add_edge"
    );
}

#[test]
fn doc_011_revision_increments_on_remove_node() {
    let mut doc = DiagramDocument::default();
    doc.add_node(NodeId::new("N1".to_string()), test_node("N1")).ok();
    let rev_before = doc.revision;

    doc.remove_node(&NodeId::new("N1".to_string())).ok();
    assert!(
        doc.revision.value() > rev_before.value(),
        "Revision should increase after remove_node"
    );
}

#[test]
fn doc_011_revision_increments_on_remove_edge() {
    let mut doc = doc_with_nodes(&["N1", "N2"]);
    doc.add_edge(EdgeId::new("E1".to_string()), test_edge("N1", "N2")).ok();
    let rev_before = doc.revision;

    doc.remove_edge(&EdgeId::new("E1".to_string())).ok();
    assert!(
        doc.revision.value() > rev_before.value(),
        "Revision should increase after remove_edge"
    );
}

#[test]
fn doc_011_revision_increments_on_update_node() {
    let mut doc = DiagramDocument::default();
    doc.add_node(NodeId::new("N1".to_string()), test_node("N1")).ok();
    let rev_before = doc.revision;

    doc.update_node(&NodeId::new("N1".to_string()), |n| Node {
        x: OrderedFloat(50.0),
        ..n
    })
    .ok();
    assert!(
        doc.revision.value() > rev_before.value(),
        "Revision should increase after update_node"
    );
}

// ===========================================================================
// DOC-012: Revision never decreases
// ===========================================================================

#[test]
fn doc_012_revision_never_decreases() {
    let mut doc = DiagramDocument::default();
    // Perform several mutations
    doc.add_node(NodeId::new("N1".to_string()), test_node("N1")).ok();
    doc.add_node(NodeId::new("N2".to_string()), test_node("N2")).ok();
    doc.add_edge(EdgeId::new("E1".to_string()), test_edge("N1", "N2")).ok();
    doc.remove_edge(&EdgeId::new("E1".to_string())).ok();
    doc.remove_node(&NodeId::new("N2".to_string())).ok();

    let final_rev = doc.revision.value();

    // Revision type has no decrement method - this is by design
    // Verify revision is monotonically larger than initial
    assert!(
        final_rev > Revision::INITIAL.value(),
        "Revision must be > 0 after mutations"
    );

    // Verify Revision type does not expose decrement
    // (Compile-time guarantee: no `decrement` method exists on Revision)
    let r = Revision::new(5);
    let incremented = r.increment();
    assert_eq!(incremented.value(), 6);
    assert!(
        incremented.value() > r.value(),
        "Increment always increases"
    );
}

// ===========================================================================
// DOC-013: Concurrent revision mismatch detected
// ===========================================================================

#[test]
fn doc_013_concurrent_revision_mismatch_detected() {
    let mut doc = DiagramDocument::default();
    doc.add_node(NodeId::new("N1".to_string()), test_node("N1")).ok();
    let client_revision = doc.revision; // e.g., 1

    // Simulate server-side mutations the client doesn't know about
    doc.add_node(NodeId::new("N2".to_string()), test_node("N2")).ok();
    // Now doc.revision is 2, client thinks it's 1

    let result = doc.check_revision(client_revision);
    assert_eq!(
        result,
        Err(DocumentError::RevisionMismatch {
            expected: client_revision.value(),
            actual: doc.revision.value()
        }),
        "Stale revision should be detected"
    );
}

#[test]
fn doc_013_current_revision_accepted() {
    let mut doc = DiagramDocument::default();
    doc.add_node(NodeId::new("N1".to_string()), test_node("N1")).ok();

    let result = doc.check_revision(doc.revision);
    assert!(result.is_ok(), "Current revision should be accepted");
}

// ===========================================================================
// DOC-014: Multi-node delete is atomic
// ===========================================================================

#[test]
fn doc_014_multi_node_delete_is_atomic() {
    let mut doc = doc_with_nodes(&["N1", "N2", "N3"]);
    doc.add_edge(EdgeId::new("E1".to_string()), test_edge("N1", "N2"))
        .ok();
    doc.add_edge(EdgeId::new("E2".to_string()), test_edge("N2", "N3"))
        .ok();

    let ids_to_remove = vec![
        NodeId::new("N1".to_string()),
        NodeId::new("N2".to_string()),
    ];

    let result = doc.remove_nodes_batch(&ids_to_remove);

    assert!(result.is_ok(), "Batch delete should succeed");
    // Only N3 should remain
    assert_eq!(doc.document.nodes.len(), 1);
    assert!(
        doc.document.nodes.contains_key(&NodeId::new("N3".to_string())),
        "N3 should remain"
    );
    // All edges should be gone (E1 connected to N1/N2, E2 connected to N2)
    assert!(
        doc.document.edges.is_empty(),
        "All edges connected to deleted nodes should be removed"
    );
}

#[test]
fn doc_014_batch_delete_rolls_back_on_missing_node() {
    let mut doc = doc_with_nodes(&["N1", "N2"]);

    let ids_to_remove = vec![
        NodeId::new("N1".to_string()),
        NodeId::new("N999".to_string()), // Does not exist
    ];

    let result = doc.remove_nodes_batch(&ids_to_remove);

    assert!(
        result.is_err(),
        "Batch delete with missing node should fail"
    );
    // N1 should still exist (atomic rollback)
    assert_eq!(
        doc.document.nodes.len(),
        2,
        "No nodes should be removed on failure (atomic)"
    );
}

// ===========================================================================
// DOC-015: Schema rejects NaN coordinates
// ===========================================================================

#[test]
fn doc_015_schema_rejects_nan_coordinates() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("NanNode".to_string());
    let node = Node {
        x: OrderedFloat(f64::NAN),
        y: OrderedFloat(0.0),
        ..test_node("NanNode")
    };
    doc.document.nodes.insert(node_id, node);

    let issues = validate_document_data(&doc.document);
    let nan_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.code == ValidationCode::INVALID_NUMERIC)
        .collect();
    assert!(
        !nan_issues.is_empty(),
        "Validation should detect NaN coordinate"
    );
}

#[test]
fn doc_015_ordered_float_new_rejects_nan() {
    let result = OrderedFloat::new(f64::NAN);
    assert_eq!(result, Err(OrderedFloatError::NaN));
}

// ===========================================================================
// DOC-016: Schema rejects negative dimensions
// ===========================================================================

#[test]
fn doc_016_schema_rejects_negative_dimensions() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("NegDim".to_string());
    let node = Node {
        width: OrderedFloat(-10.0),
        height: OrderedFloat(50.0),
        ..test_node("NegDim")
    };
    doc.document.nodes.insert(node_id, node);

    let issues = validate_document_data(&doc.document);
    let dim_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.code == ValidationCode::INVALID_NUMERIC)
        .collect();
    assert!(
        !dim_issues.is_empty(),
        "Validation should detect negative dimension"
    );
}

#[test]
fn doc_016_schema_rejects_negative_height() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("NegHeight".to_string());
    let node = Node {
        width: OrderedFloat(50.0),
        height: OrderedFloat(-5.0),
        ..test_node("NegHeight")
    };
    doc.document.nodes.insert(node_id, node);

    let issues = validate_document_data(&doc.document);
    let dim_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.code == ValidationCode::INVALID_NUMERIC)
        .collect();
    assert!(
        !dim_issues.is_empty(),
        "Validation should detect negative height"
    );
}

// ===========================================================================
// DOC-017: Schema rejects empty ID
// ===========================================================================

#[test]
fn doc_017_schema_rejects_empty_node_id() {
    let result = NodeId::try_new(String::new());
    assert!(result.is_err(), "Empty NodeId should be rejected");
    assert_eq!(result.unwrap_err(), "NodeId cannot be empty");
}

#[test]
fn doc_017_schema_rejects_empty_edge_id() {
    let result = EdgeId::try_new(String::new());
    assert!(result.is_err(), "Empty EdgeId should be rejected");
    assert_eq!(result.unwrap_err(), "EdgeId cannot be empty");
}

#[test]
fn doc_017_valid_id_accepted() {
    let node_result = NodeId::try_new("valid-id".to_string());
    assert!(node_result.is_ok());
    let edge_result = EdgeId::try_new("valid-edge".to_string());
    assert!(edge_result.is_ok());
}

// ===========================================================================
// DOC-018: Circular parent chain rejected
// ===========================================================================

#[test]
fn doc_018_circular_parent_chain_rejected() {
    let mut doc = DiagramDocument::default();
    let n1_id = NodeId::new("N1".to_string());
    let n2_id = NodeId::new("N2".to_string());

    let mut n1 = test_node("N1");
    n1.parent = Some(n2_id.clone());

    let mut n2 = test_node("N2");
    n2.parent = Some(n1_id.clone());

    doc.document.nodes.insert(n1_id.clone(), n1);
    doc.document.nodes.insert(n2_id.clone(), n2);

    let issues = validate_document_data(&doc.document);
    let cycle_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.code == ValidationCode::PARENT_CYCLE)
        .collect();
    assert!(
        !cycle_issues.is_empty(),
        "Validation should detect circular parent chain"
    );
}

// ===========================================================================
// DOC-019: Edge with self-loop detected
// ===========================================================================

#[test]
fn doc_019_edge_self_loop_detected_by_dag() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("N1".to_string());
    doc.document.nodes.insert(node_id.clone(), test_node("N1"));
    doc.document.edges.insert(
        EdgeId::new("E1".to_string()),
        test_edge("N1", "N1"), // Self-loop
    );

    let dag_result = validate_dag(&doc.document.nodes, &doc.document.edges);
    assert!(dag_result.is_err(), "Self-loop edge should fail DAG validation");
}

#[test]
fn doc_019_self_loop_flagged_in_document_validation() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("N1".to_string());
    doc.document.nodes.insert(node_id, test_node("N1"));
    doc.document.edges.insert(
        EdgeId::new("E1".to_string()),
        test_edge("N1", "N1"),
    );

    let issues = validate_document_data(&doc.document);
    let cycle_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.code == ValidationCode::DAG_CYCLE)
        .collect();
    assert!(
        !cycle_issues.is_empty(),
        "Document validation should report DAG_CYCLE for self-loop"
    );
}

// ===========================================================================
// DOC-020: Document serialization round-trip preserves all data
// ===========================================================================

#[test]
fn doc_020_serialization_roundtrip_preserves_all_data() {
    let mut doc = DiagramDocument::default();
    doc.add_node(NodeId::new("N1".to_string()), test_node_at("N1", 10.0, 20.0))
        .ok();
    doc.add_node(NodeId::new("N2".to_string()), test_node_at("N2", 300.0, 400.0))
        .ok();
    doc.add_edge(EdgeId::new("E1".to_string()), test_edge("N1", "N2"))
        .ok();

    let json = serde_json::to_string(&doc).expect("Serialization should succeed");
    let parsed: DiagramDocument =
        serde_json::from_str(&json).expect("Deserialization should succeed");

    assert_eq!(doc, parsed, "Round-tripped document must equal original");
}

#[test]
fn doc_020_roundtrip_with_revision() {
    let mut doc = DiagramDocument::default();
    doc.add_node(NodeId::new("N1".to_string()), test_node("N1")).ok();
    doc.add_node(NodeId::new("N2".to_string()), test_node("N2")).ok();

    let json = serde_json::to_string(&doc).expect("Serialize");
    let parsed: DiagramDocument = serde_json::from_str(&json).expect("Deserialize");

    assert_eq!(doc.revision, parsed.revision, "Revision must survive round-trip");
    assert_eq!(doc.version, parsed.version, "Version must survive round-trip");
    assert_eq!(
        doc.document.nodes.len(),
        parsed.document.nodes.len(),
        "Node count must match"
    );
    assert_eq!(
        doc.document.edges.len(),
        parsed.document.edges.len(),
        "Edge count must match"
    );
}

#[test]
fn doc_020_roundtrip_preserves_edge_properties() {
    let mut doc = doc_with_nodes(&["A", "B"]);
    let mut edge = test_edge("A", "B");
    edge.label = "My Edge".to_string();
    edge.color = Some("#FF5500".to_string());
    edge.thickness = OrderedFloat(3.0);
    doc.add_edge(EdgeId::new("E1".to_string()), edge).ok();

    let json = serde_json::to_string(&doc).expect("Serialize");
    let parsed: DiagramDocument = serde_json::from_str(&json).expect("Deserialize");

    let original_edge = doc.document.edges.get(&EdgeId::new("E1".to_string()));
    let parsed_edge = parsed.document.edges.get(&EdgeId::new("E1".to_string()));
    assert!(original_edge.is_some());
    assert!(parsed_edge.is_some());
    assert_eq!(original_edge, parsed_edge, "Edge properties must survive round-trip");
}
