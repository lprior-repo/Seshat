use super::super::test_helpers::{make_edge, make_node};
use crate::document::{DiagramDocument, NodeId, OrderedFloat};
use crate::validation::{validate_document, ValidationCode};

// --- Collect-All Pattern Tests ---

#[test]
fn ca01_multiple_issues_collected_from_single_document() {
    let mut doc = DiagramDocument::default();
    doc.version = 1;
    let (nid, mut node) = make_node("nan");
    node.x = OrderedFloat::new_unchecked(f64::NAN);
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, edge) = make_edge("e1", "MISSING", "MISSING");
    doc.document.edges = doc.document.edges.update(eid, edge);

    let issues = validate_document(&doc);
    assert!(issues.len() >= 3);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_VERSION));
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_NUMERIC));
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_DANGLING));
}

#[test]
fn ca02_all_dangling_edges_reported_not_just_first() {
    let mut doc = DiagramDocument::default();
    for i in 0..3 {
        let (eid, edge) = make_edge(&format!("e{i}"), "MISSING", "MISSING");
        doc.document.edges = doc.document.edges.update(eid, edge);
    }
    let issues = validate_document(&doc);
    let dangling_count = issues
        .iter()
        .filter(|i| i.code == ValidationCode::EDGE_DANGLING)
        .count();
    assert!(
        dangling_count >= 3,
        "Expected >= 3 EDGE_DANGLING, got {dangling_count}"
    );
}

#[test]
fn ca03_all_nodes_with_bad_geometry_reported() {
    let mut doc = DiagramDocument::default();
    for i in 0..5 {
        let (nid, mut node) = make_node(&format!("n{i}"));
        node.width = OrderedFloat::new_unchecked(-1.0);
        doc.document.nodes = doc.document.nodes.update(nid, node);
    }
    let issues = validate_document(&doc);
    let bad_count = issues
        .iter()
        .filter(|i| i.code == ValidationCode::INVALID_NUMERIC)
        .count();
    assert!(
        bad_count >= 5,
        "Expected >= 5 INVALID_NUMERIC, got {bad_count}"
    );
}

#[test]
fn ca04_mixed_issue_types_all_reported() {
    let mut doc = DiagramDocument::default();
    doc.version = 1;
    let (aid, mut a) = make_node("A");
    a.parent = Some(NodeId::new("B".to_string()));
    let (bid, mut b) = make_node("B");
    b.parent = Some(NodeId::new("A".to_string()));
    doc.document.nodes = doc.document.nodes.update(aid, a).update(bid, b);
    let (eid, edge) = make_edge("e1", "MISSING", "MISSING");
    doc.document.edges = doc.document.edges.update(eid, edge);
    doc.editor_state.camera_x = OrderedFloat::new_unchecked(f64::NAN);

    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_VERSION));
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::PARENT_CYCLE));
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_DANGLING));
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDITOR_INVALID_STATE));
    assert!(issues.len() >= 4);
}

#[test]
fn ca05_single_node_all_properties_invalid_collects_all() {
    let mut doc = DiagramDocument::default();
    let (nid, mut node) = make_node("bad");
    node.x = OrderedFloat::new_unchecked(f64::NAN);
    node.y = OrderedFloat::new_unchecked(f64::INFINITY);
    node.width = OrderedFloat::new_unchecked(-1.0);
    node.height = OrderedFloat::new_unchecked(-1.0);
    node.parent = Some(NodeId::new("nonexistent".to_string()));
    doc.document.nodes = doc.document.nodes.update(nid, node);

    let issues = validate_document(&doc);
    assert!(
        issues.len() >= 3,
        "Expected >= 3 issues, got {}",
        issues.len()
    );
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_NUMERIC && i.message.contains("coordinates")));
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_NUMERIC && i.message.contains("dimensions")));
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_PARENT));
}

#[test]
fn ca06_edge_color_none_does_not_suppress_other_errors() {
    let mut doc = DiagramDocument::default();
    let (eid, mut edge) = make_edge("e1", "MISSING", "MISSING");
    edge.color = None;
    edge.thickness = OrderedFloat::new_unchecked(f64::NAN);
    doc.document.edges = doc.document.edges.update(eid, edge);

    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_DANGLING));
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_THICKNESS));
    assert!(!issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}
