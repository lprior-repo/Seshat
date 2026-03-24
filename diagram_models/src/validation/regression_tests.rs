#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]

use super::test_helpers::{make_edge, make_node};
use crate::document::DiagramDocument;
use crate::validation::{validate_document, ValidationCode};

// --- Regression Tests ---

#[test]
fn er01_edge_dangling_source() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, edge) = make_edge("e1", "MISSING", "A");
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_DANGLING && i.message.contains("MISSING")));
}

#[test]
fn er02_edge_dangling_target() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, edge) = make_edge("e1", "A", "MISSING");
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_DANGLING && i.message.contains("MISSING")));
}

#[test]
fn er03_edge_dangling_source_and_target_two_issues() {
    let mut doc = DiagramDocument::default();
    let (eid, edge) = make_edge("e1", "MISSING_SRC", "MISSING_TGT");
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    let dangling_count = issues
        .iter()
        .filter(|i| i.code == ValidationCode::EDGE_DANGLING)
        .count();
    assert_eq!(dangling_count, 2);
}

#[test]
fn er04_node_parent_nonexistent() {
    let mut doc = DiagramDocument::default();
    let (nid, mut node) = make_node("A");
    node.parent = Some(crate::document::NodeId::new("missing_id".to_string()));
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_PARENT && i.subject.as_deref() == Some("A")));
}

#[test]
fn er05_node_parent_not_subgraph() {
    let mut doc = DiagramDocument::default();
    let (pid, parent) = make_node("P"); // kind: Node
    let (cid, mut child) = make_node("C");
    child.parent = Some(pid.clone());
    doc.document.nodes = doc.document.nodes.update(pid, parent).update(cid, child);
    let issues = validate_document(&doc);
    assert!(issues.iter().any(|i| {
        i.code == ValidationCode::INVALID_PARENT && i.message.contains("not a Subgraph")
    }));
}

#[test]
fn er06_node_nan_coordinates() {
    let mut doc = DiagramDocument::default();
    let (nid, mut node) = make_node("A");
    node.x = crate::document::OrderedFloat::new_unchecked(f64::NAN);
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_NUMERIC));
}

#[test]
fn er07_node_inf_coordinates() {
    let mut doc = DiagramDocument::default();
    let (nid, mut node) = make_node("A");
    node.y = crate::document::OrderedFloat::new_unchecked(f64::INFINITY);
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_NUMERIC));
}

#[test]
fn er08_node_negative_dimensions() {
    let mut doc = DiagramDocument::default();
    let (nid, mut node) = make_node("A");
    node.width = crate::document::OrderedFloat::new_unchecked(-10.0);
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_NUMERIC));
}

#[test]
fn er09_dag_cycle() {
    let mut doc = DiagramDocument::default();
    let (a, node_a) = make_node("A");
    let (b, node_b) = make_node("B");
    doc.document.nodes = doc.document.nodes.update(a, node_a).update(b, node_b);
    let (e1, edge1) = make_edge("e1", "A", "B");
    let (e2, edge2) = make_edge("e2", "B", "A");
    doc.document.edges = doc.document.edges.update(e1, edge1).update(e2, edge2);
    let issues = validate_document(&doc);
    assert!(issues.iter().any(|i| i.code == ValidationCode::DAG_CYCLE));
}
