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
use crate::document::{DiagramDocument, NodeId, OrderedFloat};
use crate::validation::{validate_document, ValidationCode};

// --- Edge Case Tests ---

#[test]
fn ec01_empty_document_zero_issues() {
    let doc = DiagramDocument::default();
    assert!(validate_document(&doc).is_empty());
}

#[test]
fn ec02_document_with_hundreds_of_nodes_no_panics() {
    let mut doc = DiagramDocument::default();
    for i in 0..500 {
        let (nid, node) = make_node(&format!("n{i}"));
        doc.document.nodes = doc.document.nodes.update(nid, node);
    }
    let _ = validate_document(&doc);
}

#[test]
fn ec03_only_editor_state_issues_no_node_edge_issues() {
    let mut doc = DiagramDocument::default();
    doc.editor_state.zoom = OrderedFloat::new_unchecked(f64::NAN);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDITOR_INVALID_STATE));
    assert!(!issues.iter().any(|i| {
        let code = &i.code;
        code == &ValidationCode::EDGE_DANGLING
            || code == &ValidationCode::INVALID_PARENT
            || code == &ValidationCode::INVALID_NUMERIC
    }));
}

#[test]
fn ec04_edge_label_offset_exactly_zero_valid() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.label_offset_t = OrderedFloat::new_unchecked(0.0);
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_OFFSET));
}

#[test]
fn ec05_edge_label_offset_exactly_one_valid() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.label_offset_t = OrderedFloat::new_unchecked(1.0);
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_OFFSET));
}

#[test]
fn ec06_edge_color_valid_3digit_hex() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.color = Some("#abc".to_string());
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ec07_edge_color_valid_4digit_hex_with_alpha() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.color = Some("#abcd".to_string());
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ec08_edge_color_valid_6digit_hex() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.color = Some("#aabbcc".to_string());
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ec09_edge_color_valid_8digit_hex_with_alpha() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.color = Some("#aabbccdd".to_string());
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ec10_parent_cycle_deep_chain_of_five_nodes() {
    let mut doc = DiagramDocument::default();
    let names = ["A", "B", "C", "D", "E"];
    for (i, name) in names.iter().enumerate() {
        let (nid, mut node) = make_node(name);
        node.parent = Some(NodeId::new(names[(i + 1) % 5].to_string()));
        doc.document.nodes = doc.document.nodes.update(nid, node);
    }
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::PARENT_CYCLE));
}

#[test]
fn ec11_version_zero_produces_error() {
    let mut doc = DiagramDocument::default();
    doc.version = 0;
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_VERSION));
}

#[test]
fn ec12_version_large_number_produces_error() {
    let mut doc = DiagramDocument::default();
    doc.version = 999;
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_VERSION));
}

#[test]
fn ec13_edge_color_uppercase_3digit_hex() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.color = Some("#FFF".to_string());
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ec14_edge_color_mixed_case_hex() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.color = Some("#aAbBcC".to_string());
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ec15_edge_thickness_zero_is_valid() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.thickness = OrderedFloat::new_unchecked(0.0);
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_THICKNESS));
}

#[test]
fn ec16_edge_thickness_positive_finite_is_valid() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.thickness = OrderedFloat::new_unchecked(1.5);
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_THICKNESS));
}

#[test]
fn ec17_edge_font_size_zero_is_valid() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.font_size = Some(OrderedFloat::new_unchecked(0.0));
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_FONT_SIZE));
}

#[test]
fn ec18_edge_font_size_positive_is_valid() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.font_size = Some(OrderedFloat::new_unchecked(12.0));
    doc.document.edges = doc.document.edges.update(eid, edge);
    assert!(!validate_document(&doc)
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_FONT_SIZE));
}

#[test]
fn ec19_version_three_produces_error() {
    let mut doc = DiagramDocument::default();
    doc.version = 3;
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_VERSION));
}

#[test]
fn ec20_version_u32_max_produces_error() {
    let mut doc = DiagramDocument::default();
    doc.version = u32::MAX;
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_VERSION));
}

#[test]
fn ec21_negative_coordinates_valid() {
    let mut doc = DiagramDocument::default();
    let (nid, mut node) = make_node("A");
    node.x = OrderedFloat::new_unchecked(-100.0);
    node.y = OrderedFloat::new_unchecked(-200.0);
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let issues = validate_document(&doc);
    assert!(!issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_NUMERIC));
}

#[test]
fn ec22_zero_dimensions_valid() {
    let mut doc = DiagramDocument::default();
    let (nid, mut node) = make_node("A");
    node.width = OrderedFloat::new_unchecked(0.0);
    node.height = OrderedFloat::new_unchecked(0.0);
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let issues = validate_document(&doc);
    assert!(!issues
        .iter()
        .any(|i| i.code == ValidationCode::INVALID_NUMERIC));
}
