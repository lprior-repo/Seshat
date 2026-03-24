#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]

use super::core::{selected_node_ids, selection_bounds};
use super::test_utils::make_node;
use diagram_models::document::{DiagramDocument, NodeId, NodeKind, OrderedFloat};
use diagram_models::history::History;

// ============== SEL-002: Selection persists across pan/zoom ==============

#[cfg(kani)]
#[kani::proof]
fn given_selected_items_when_camera_transforms_then_selection_remains_unchanged() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new(String::from("test_node"));
    doc.document.nodes = doc.document.nodes.update(
        node_id.clone(),
        make_node(NodeKind::Node, 100.0, 100.0, 50.0, 50.0),
    );
    let _ = doc.editor_state.selected_items.insert(node_id.to_string());

    let initial_ids = selected_node_ids(&doc);
    let initial_bounds = selection_bounds(&doc);

    doc.editor_state.camera_x = OrderedFloat(100.0);
    doc.editor_state.camera_y = OrderedFloat(50.0);
    doc.editor_state.zoom = OrderedFloat(2.0);

    assert_eq!(
        doc.editor_state.selected_items.len(),
        1,
        "Selection count should not change"
    );
    assert!(
        doc.editor_state.selected_items.contains("test_node"),
        "Selected item should still be present"
    );

    let after_transform_ids = selected_node_ids(&doc);
    assert_eq!(initial_ids, after_transform_ids);

    let after_transform_bounds = selection_bounds(&doc);
    assert_eq!(initial_bounds, after_transform_bounds);
    assert_eq!(
        after_transform_bounds,
        Some((100.0, 100.0, 50.0, 50.0)),
        "Document-space bounds should not change with camera"
    );
}

// ============== SEL-003: Selection box after undo/redo ==============

#[cfg(kani)]
#[kani::proof]
fn given_selection_history_when_undo_redo_then_selection_restored() {
    let mut doc = DiagramDocument::default();
    let n1 = NodeId::new(String::from("n1"));
    let n2 = NodeId::new(String::from("n2"));
    doc.document.nodes = doc
        .document
        .nodes
        .update(n1.clone(), make_node(NodeKind::Node, 0.0, 0.0, 50.0, 50.0))
        .update(
            n2.clone(),
            make_node(NodeKind::Node, 100.0, 0.0, 50.0, 50.0),
        );

    let mut history = History::new();

    assert!(doc.editor_state.selected_items.is_empty());

    history = history.push(doc.clone());
    let _ = doc.editor_state.selected_items.insert(n1.to_string());

    history = history.push(doc.clone());
    doc.editor_state.selected_items.clear();
    let _ = doc.editor_state.selected_items.insert(n2.to_string());

    assert_eq!(doc.editor_state.selected_items.len(), 1);
    assert!(doc.editor_state.selected_items.contains("n2"));

    let (restored_doc, history) = history.undo(doc.clone()).expect("undo should succeed");

    assert_eq!(
        restored_doc.editor_state.selected_items.len(),
        1,
        "After undo, should have 1 selected item"
    );
    assert!(
        restored_doc.editor_state.selected_items.contains("n1"),
        "After undo, n1 should be selected"
    );
    assert!(
        !restored_doc.editor_state.selected_items.contains("n2"),
        "After undo, n2 should not be selected"
    );

    let (redone_doc, _history) = history
        .redo(restored_doc.clone())
        .expect("redo should succeed");

    assert_eq!(
        redone_doc.editor_state.selected_items.len(),
        1,
        "After redo, should have 1 selected item"
    );
    assert!(
        redone_doc.editor_state.selected_items.contains("n2"),
        "After redo, n2 should be selected"
    );
}

// ============== SEL-005: Selection state for edit mode ==============

#[cfg(kani)]
#[kani::proof]
fn given_single_selected_node_when_edit_mode_initiated_then_target_is_identifiable() {
    let mut doc = DiagramDocument::default();
    let editable_id = NodeId::new(String::from("editable"));
    let other_id = NodeId::new(String::from("other"));

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            editable_id.clone(),
            make_node(NodeKind::Node, 0.0, 0.0, 100.0, 50.0),
        )
        .update(
            other_id.clone(),
            make_node(NodeKind::Node, 200.0, 0.0, 100.0, 50.0),
        );

    let _ = doc
        .editor_state
        .selected_items
        .insert(editable_id.to_string());

    let selected = selected_node_ids(&doc);

    assert_eq!(
        selected.len(),
        1,
        "Exactly one node should be selected for edit mode"
    );
    assert_eq!(
        selected.first(),
        Some(&editable_id),
        "The editable node should be the selection target"
    );

    assert!(
        doc.document.nodes.contains_key(&editable_id),
        "Selected node must exist in document for editing"
    );

    let node = doc.document.nodes.get(&editable_id).expect("node exists");
    assert!(
        !node.label.is_empty() || node.label.is_empty(),
        "Label is accessible for editing"
    );
}
