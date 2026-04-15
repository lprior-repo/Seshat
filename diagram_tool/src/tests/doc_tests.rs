//! DOC Category Tests (12 tests)
//!
//! Document serialization, schema validation, versioning, and round-trip tests.

use crate::test_utils::{builders::*, fixtures::*, harness::*, types::*};

// ============================================================================
// DOC-001: Schema version 2 is accepted
// ============================================================================

#[test]
fn doc_001_schema_version_2_accepted() {
    let doc = serde_json::json!({
        "version": 2,
        "document": { "nodes": {}, "edges": {} }
    });
    let result = validate_fixture_schema(&doc);
    assert!(result.is_ok(), "Schema version 2 should be accepted");
}

// ============================================================================
// DOC-002: Schema version mismatch is rejected
// ============================================================================

#[test]
fn doc_002_schema_version_mismatch_rejected() {
    let doc = serde_json::json!({
        "version": 99,
        "document": { "nodes": {}, "edges": {} }
    });
    let result = validate_fixture_schema(&doc);
    assert!(result.is_err(), "Wrong schema version should be rejected");
}

// ============================================================================
// DOC-003: Missing version field is rejected
// ============================================================================

#[test]
fn doc_003_missing_version_field_rejected() {
    let doc = serde_json::json!({
        "document": { "nodes": {}, "edges": {} }
    });
    let result = validate_fixture_schema(&doc);
    assert!(result.is_err(), "Missing version field should be rejected");
}

// ============================================================================
// DOC-004: Document round-trip preserves nodes
// ============================================================================

#[test]
fn doc_004_round_trip_preserves_nodes() {
    let original = setup_doc();
    let serialized = serde_json::to_string(&original).unwrap_or_default();
    let deserialized: diagram_models::document::DiagramDocument =
        serde_json::from_str(&serialized).unwrap_or_default();

    assert_eq!(
        original.document.nodes.len(),
        deserialized.document.nodes.len(),
        "Node count must be preserved after round-trip"
    );
}

// ============================================================================
// DOC-005: Document round-trip preserves edges
// ============================================================================

#[test]
fn doc_005_round_trip_preserves_edges() {
    let mut doc = setup_doc();
    let source = diagram_models::document::NodeId::new("A".to_string());
    let target = diagram_models::document::NodeId::new("B".to_string());
    doc.document.edges.insert(
        diagram_models::document::EdgeId::new("e1".to_string()),
        test_edge(source, target),
    );

    let serialized = serde_json::to_string(&doc).unwrap_or_default();
    let deserialized: diagram_models::document::DiagramDocument =
        serde_json::from_str(&serialized).unwrap_or_default();

    assert_eq!(
        deserialized.document.edges.len(),
        1,
        "Edge count must be preserved after round-trip"
    );
}

// ============================================================================
// DOC-006: Default document has version 2
// ============================================================================

#[test]
fn doc_006_default_document_version_2() {
    let doc = diagram_models::document::DiagramDocument::default();
    assert_eq!(doc.version, 2, "Default document must have version 2");
}

// ============================================================================
// DOC-007: Default document has zero revision
// ============================================================================

#[test]
fn doc_007_default_document_zero_revision() {
    let doc = diagram_models::document::DiagramDocument::default();
    assert_eq!(
        doc.revision.value(),
        0,
        "Default document must have revision 0"
    );
}

// ============================================================================
// DOC-008: Document hash is deterministic
// ============================================================================

#[test]
fn doc_008_document_hash_deterministic() {
    let doc = setup_doc();
    let hash1 = compute_document_hash(&doc);
    let hash2 = compute_document_hash(&doc);
    assert_eq!(hash1, hash2, "Document hash must be deterministic");
}

// ============================================================================
// DOC-009: Different documents produce different hashes
// ============================================================================

#[test]
fn doc_009_different_documents_different_hashes() {
    let doc1 = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .build();
    let doc2 = DocBuilder::new()
        .add_node_with("A", 20.0, 20.0, 50.0, 50.0)
        .build();

    let hash1 = compute_document_hash(&doc1);
    let hash2 = compute_document_hash(&doc2);
    assert_ne!(hash1, hash2, "Different documents must have different hashes");
}

// ============================================================================
// DOC-010: Invariant verification passes for valid document
// ============================================================================

#[test]
fn doc_010_invariant_verification_valid_document() {
    let doc = setup_doc();
    let result = verify_invariants(&doc);
    assert!(
        result.is_ok(),
        "Valid document must pass invariant verification"
    );
}

// ============================================================================
// DOC-011: Mixed selection fixture loads and validates
// ============================================================================

#[test]
fn doc_011_mixed_selection_fixture_loads() {
    let result = load_fixture("mixed_selection.json");
    assert!(result.is_ok(), "mixed_selection.json must load");
    let doc = result.unwrap();
    let schema_result = validate_fixture_schema(&doc);
    assert!(schema_result.is_ok(), "Fixture must pass schema validation");
}

// ============================================================================
// DOC-012: Nested subgraph fixture loads and validates
// ============================================================================

#[test]
fn doc_012_nested_subgraph_fixture_loads() {
    let result = load_fixture("nested_subgraph.json");
    assert!(result.is_ok(), "nested_subgraph.json must load");
    let doc = result.unwrap();
    let schema_result = validate_fixture_schema(&doc);
    assert!(schema_result.is_ok(), "Fixture must pass schema validation");
}
