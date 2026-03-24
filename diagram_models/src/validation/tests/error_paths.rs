use super::super::test_helpers::{make_edge, make_node};
use crate::document::{DiagramDocument, NodeId, OrderedFloat};
use crate::validation::{validate_document, ValidationCode};

// --- Error Path Tests ---

#[test]
fn ep01_invalid_version_produces_invalid_version_error() {
    let mut doc = DiagramDocument::default();
    doc.version = 1;
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_VERSION));
    assert!(issues
        .iter()
        .any(|i| { i.code == ValidationCode::INVALID_VERSION && i.message.contains("1") }));
}

#[test]
fn ep02_parent_cycle_detected_produces_parent_cycle_error() {
    let mut doc = DiagramDocument::default();
    let (aid, mut a) = make_node("A");
    a.parent = Some(NodeId::new("B".to_string()));
    let (bid, mut b) = make_node("B");
    b.parent = Some(NodeId::new("A".to_string()));
    doc.document.nodes = doc.document.nodes.update(aid, a).update(bid, b);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::PARENT_CYCLE));
}

#[test]
fn ep03_parent_cycle_self_reference_produces_parent_cycle_error() {
    let mut doc = DiagramDocument::default();
    let (aid, mut a) = make_node("A");
    a.parent = Some(NodeId::new("A".to_string()));
    doc.document.nodes = doc.document.nodes.update(aid, a);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::PARENT_CYCLE));
}

#[test]
fn ep04_parent_cycle_three_node_chain_produces_parent_cycle_error() {
    let mut doc = DiagramDocument::default();
    let (aid, mut a) = make_node("A");
    a.parent = Some(NodeId::new("B".to_string()));
    let (bid, mut b) = make_node("B");
    b.parent = Some(NodeId::new("C".to_string()));
    let (cid, mut c) = make_node("C");
    c.parent = Some(NodeId::new("A".to_string()));
    doc.document.nodes = doc
        .document
        .nodes
        .update(aid, a)
        .update(bid, b)
        .update(cid, c);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::PARENT_CYCLE));
}

#[test]
fn ep05_edge_label_offset_t_outside_range_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.label_offset_t = OrderedFloat::new_unchecked(1.5);
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_OFFSET));
}

#[test]
fn ep06_edge_label_offset_t_negative_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.label_offset_t = OrderedFloat::new_unchecked(-0.1);
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_OFFSET));
}

#[test]
fn ep07_edge_label_offset_t_nan_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.label_offset_t = OrderedFloat::new_unchecked(f64::NAN);
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_OFFSET));
}

#[test]
fn ep08_edge_label_offset_t_at_boundaries_produces_no_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (e1id, mut e1) = make_edge("e1", "A", "A");
    e1.label_offset_t = OrderedFloat::new_unchecked(0.0);
    let (e2id, mut e2) = make_edge("e2", "A", "A");
    e2.label_offset_t = OrderedFloat::new_unchecked(1.0);
    doc.document.edges = doc.document.edges.update(e1id, e1).update(e2id, e2);
    let issues = validate_document(&doc);
    assert!(!issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_OFFSET));
}

#[test]
fn ep09_edge_thickness_infinity_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.thickness = OrderedFloat::new_unchecked(f64::INFINITY);
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_THICKNESS));
}

#[test]
fn ep10_edge_thickness_negative_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.thickness = OrderedFloat::new_unchecked(-1.0);
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_THICKNESS));
}

#[test]
fn ep11_edge_thickness_nan_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.thickness = OrderedFloat::new_unchecked(f64::NAN);
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_THICKNESS));
}

#[test]
fn ep12_edge_color_invalid_format_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.color = Some("not-a-color".to_string());
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ep13_edge_color_empty_string_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.color = Some(String::new());
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ep14_edge_color_valid_formats_produce_no_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    for (eid_str, color) in [
        ("e1", "#fff"),
        ("e2", "#ffffff"),
        ("e3", "#ffff"),
        ("e4", "#ffffffff"),
    ] {
        let (eid, mut edge) = make_edge(eid_str, "A", "A");
        edge.color = Some(color.to_string());
        doc.document.edges = doc.document.edges.update(eid, edge);
    }
    let issues = validate_document(&doc);
    assert!(!issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ep15_edge_color_none_produces_no_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, edge) = make_edge("e1", "A", "A");
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(!issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ep16_edge_font_size_infinity_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.font_size = Some(OrderedFloat::new_unchecked(f64::INFINITY));
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_FONT_SIZE));
}

#[test]
fn ep17_edge_font_size_nan_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.font_size = Some(OrderedFloat::new_unchecked(f64::NAN));
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_FONT_SIZE));
}

#[test]
fn ep18_edge_font_size_none_produces_no_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, edge) = make_edge("e1", "A", "A");
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(!issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_FONT_SIZE));
}
