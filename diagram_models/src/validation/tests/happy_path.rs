use super::super::test_helpers::{make_edge, make_node};
use crate::document::DiagramDocument;
use crate::document::OrderedFloat;
use crate::validation::{validate_document, validate_document_data, ValidationCode};

// --- Happy Path Tests ---

#[test]
fn hp01_valid_default_document_produces_zero_issues() {
    let doc = DiagramDocument::default();
    assert!(validate_document(&doc).is_empty());
}

#[test]
fn hp02_valid_document_with_single_node_produces_zero_issues() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    assert!(validate_document(&doc).is_empty());
}

#[test]
fn hp03_valid_document_with_nodes_and_edges_produces_zero_issues() {
    let mut doc = DiagramDocument::default();
    let (a, node_a) = make_node("A");
    let (b, node_b) = make_node("B");
    doc.document.nodes = doc.document.nodes.update(a, node_a).update(b, node_b);
    let (e, edge) = make_edge("e1", "A", "B");
    doc.document.edges = doc.document.edges.update(e, edge);
    assert!(validate_document(&doc).is_empty());
}

#[test]
fn hp04_valid_document_with_subgraph_parent_produces_zero_issues() {
    let mut doc = DiagramDocument::default();
    let (pid, mut parent) = make_node("P");
    parent.kind = crate::document::NodeKind::Subgraph;
    let (cid, mut child) = make_node("C");
    child.parent = Some(pid.clone());
    doc.document.nodes = doc.document.nodes.update(pid, parent).update(cid, child);
    let issues = validate_document(&doc);
    assert!(!issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_PARENT));
}

#[test]
fn hp05_valid_document_data_produces_same_issues_as_document_minus_editor_state() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);

    doc.editor_state.camera_x = OrderedFloat::new_unchecked(f64::NAN);

    let full_issues = validate_document(&doc);
    let data_issues = validate_document_data(&doc.document);

    assert!(full_issues
        .iter()
        .any(|i| i.code == ValidationCode::EDITOR_INVALID_STATE));
    assert!(!data_issues
        .iter()
        .any(|i| i.code == ValidationCode::EDITOR_INVALID_STATE));
}

#[test]
fn hp06_mixed_node_and_editor_state_violations_filtered_correctly() {
    let mut doc = DiagramDocument::default();
    let (nid, mut node) = make_node("nan");
    node.x = OrderedFloat::new_unchecked(f64::NAN);
    doc.document.nodes = doc.document.nodes.update(nid, node);
    doc.editor_state.zoom = OrderedFloat::new_unchecked(f64::NAN);

    let full_issues = validate_document(&doc);
    let data_issues = validate_document_data(&doc.document);

    assert!(full_issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_NUMERIC));
    assert!(full_issues
        .iter()
        .any(|i| i.code == ValidationCode::EDITOR_INVALID_STATE));
    assert!(data_issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_NUMERIC));
    assert!(!data_issues
        .iter()
        .any(|i| i.code == ValidationCode::EDITOR_INVALID_STATE));
}
