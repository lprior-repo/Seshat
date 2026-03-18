#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]

use super::core::{selected_node_ids, selection_bounds};
use super::test_utils::make_node;
use diagram_models::document::{DiagramDocument, NodeId, NodeKind};

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_selected_nodes_when_bounds_requested_then_bounds_cover_selection() {
    let mut doc = DiagramDocument::default();
    let id_a = NodeId::new(String::from("a"));
    let id_b = NodeId::new(String::from("b"));
    let _ = doc.document.nodes.insert(
        id_a.clone(),
        make_node(NodeKind::Node, 10.0, 20.0, 50.0, 30.0),
    );
    let _ = doc.document.nodes.insert(
        id_b.clone(),
        make_node(NodeKind::Node, 100.0, 120.0, 40.0, 20.0),
    );
    let _ = doc.editor_state.selected_items.insert(id_a.to_string());
    let _ = doc.editor_state.selected_items.insert(id_b.to_string());

    let ids = selected_node_ids(&doc);
    assert_eq!(ids.len(), 2);
    assert_eq!(selection_bounds(&doc), Some((10.0, 20.0, 130.0, 120.0)));
}

// ============== SEL-001: Multi-type selection (shape+text+connector) ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_multi_type_selection_when_bounds_requested_then_all_types_included() {
    let mut doc = DiagramDocument::default();
    let shape_id = NodeId::new(String::from("shape_node"));
    let text_id = NodeId::new(String::from("text_node"));

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            shape_id.clone(),
            make_node(NodeKind::Node, 50.0, 50.0, 80.0, 60.0),
        )
        .update(
            text_id.clone(),
            make_node(NodeKind::Text, 200.0, 100.0, 100.0, 30.0),
        );

    let _ = doc.editor_state.selected_items.insert(shape_id.to_string());
    let _ = doc.editor_state.selected_items.insert(text_id.to_string());

    let ids = selected_node_ids(&doc);

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&shape_id));
    assert!(ids.contains(&text_id));

    let bounds = selection_bounds(&doc);
    assert_eq!(bounds, Some((50.0, 50.0, 250.0, 80.0)));
}

// ============== SEL-004: Selection box handles negative coordinates ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_nodes_at_negative_coords_when_selected_then_bounds_correct() {
    let mut doc = DiagramDocument::default();
    let neg_x = NodeId::new(String::from("neg_x"));
    let neg_y = NodeId::new(String::from("neg_y"));
    let neg_both = NodeId::new(String::from("neg_both"));

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            neg_x.clone(),
            make_node(NodeKind::Node, -100.0, 50.0, 80.0, 60.0),
        )
        .update(
            neg_y.clone(),
            make_node(NodeKind::Node, 50.0, -100.0, 80.0, 60.0),
        )
        .update(
            neg_both.clone(),
            make_node(NodeKind::Node, -200.0, -200.0, 100.0, 100.0),
        );

    let _ = doc.editor_state.selected_items.insert(neg_x.to_string());
    let _ = doc.editor_state.selected_items.insert(neg_y.to_string());
    let _ = doc.editor_state.selected_items.insert(neg_both.to_string());

    let bounds = selection_bounds(&doc);
    assert!(
        bounds.is_some(),
        "Bounds should be computed for negative coords"
    );

    let (min_x, min_y, width, height) = bounds.unwrap();

    assert_eq!(min_x, -200.0, "min_x should be -200");
    assert_eq!(min_y, -200.0, "min_y should be -200");
    assert_eq!(width, 330.0, "width should be 330 (130 - (-200))");
    assert_eq!(height, 310.0, "height should be 310 (110 - (-200))");
}
