use super::super::test_helpers::{make_edge, make_node};
use crate::document::{DiagramDocument, OrderedFloat};
use crate::validation::{validate_document, ValidationCode};

// --- Editor State Tests ---

#[test]
fn ep19_editor_state_camera_x_nan_produces_error() {
    let mut doc = DiagramDocument::default();
    doc.editor_state.camera_x = OrderedFloat::new_unchecked(f64::NAN);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDITOR_INVALID_STATE));
}

#[test]
fn ep20_editor_state_camera_y_infinity_produces_error() {
    let mut doc = DiagramDocument::default();
    doc.editor_state.camera_y = OrderedFloat::new_unchecked(f64::INFINITY);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDITOR_INVALID_STATE));
}

#[test]
fn ep21_editor_state_zoom_neg_infinity_produces_error() {
    let mut doc = DiagramDocument::default();
    doc.editor_state.zoom = OrderedFloat::new_unchecked(f64::NEG_INFINITY);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDITOR_INVALID_STATE));
}

#[test]
fn ep22_editor_state_multiple_bad_fields_produces_multiple_issues() {
    let mut doc = DiagramDocument::default();
    doc.editor_state.camera_x = OrderedFloat::new_unchecked(f64::NAN);
    doc.editor_state.camera_y = OrderedFloat::new_unchecked(f64::NAN);
    doc.editor_state.zoom = OrderedFloat::new_unchecked(f64::NAN);
    let issues = validate_document(&doc);
    let count = issues
        .iter()
        .filter(|i| i.code == ValidationCode::EDITOR_INVALID_STATE)
        .count();
    assert!(
        count >= 3,
        "Expected >= 3 EDITOR_INVALID_STATE issues, got {count}"
    );
}

#[test]
fn ep23_editor_state_valid_produces_no_error() {
    let doc = DiagramDocument::default();
    let issues = validate_document(&doc);
    assert!(!issues
        .iter()
        .any(|i| i.code == ValidationCode::EDITOR_INVALID_STATE));
}

#[test]
fn ep24_edge_color_hash_only_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.color = Some("#".to_string());
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_COLOR));
}

#[test]
fn ep25_edge_color_uppercase_hex_produces_no_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    for (eid_str, color) in [
        ("e1", "#FFF"),
        ("e2", "#ABCDEF"),
        ("e3", "#AABBCCDD"),
        ("e4", "#AbC123"),
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
fn ep26_edge_label_offset_t_neg_infinity_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.label_offset_t = OrderedFloat::new_unchecked(f64::NEG_INFINITY);
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_OFFSET));
}

#[test]
fn ep27_edge_label_offset_t_pos_infinity_produces_error() {
    let mut doc = DiagramDocument::default();
    let (nid, node) = make_node("A");
    doc.document.nodes = doc.document.nodes.update(nid, node);
    let (eid, mut edge) = make_edge("e1", "A", "A");
    edge.label_offset_t = OrderedFloat::new_unchecked(f64::INFINITY);
    doc.document.edges = doc.document.edges.update(eid, edge);
    let issues = validate_document(&doc);
    assert!(issues
        .iter()
        .any(|i| i.code == ValidationCode::EDGE_INVALID_OFFSET));
}
