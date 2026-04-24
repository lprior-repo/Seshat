#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]

use super::core::{selected_node_ids, selection_bounds};
use super::test_utils::make_node_with_lock;
use diagram_models::document::{DiagramDocument, NodeId, NodeKind};

// ============== GEO-024: Locked node exclusion from selection ==============

#[test]
fn given_locked_and_unlocked_nodes_when_selection_bounds_then_exclude_locked() {
    let mut doc = DiagramDocument::default();
    let unlocked_id = NodeId::new(String::from("unlocked"));
    let locked_id = NodeId::new(String::from("locked"));

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            unlocked_id.clone(),
            make_node_with_lock(NodeKind::Node, 10.0, 10.0, 50.0, 50.0, false),
        )
        .update(
            locked_id.clone(),
            make_node_with_lock(NodeKind::Node, 100.0, 100.0, 50.0, 50.0, true),
        );

    let _ = doc
        .editor_state
        .selected_items
        .insert(unlocked_id.to_string());
    let _ = doc
        .editor_state
        .selected_items
        .insert(locked_id.to_string());

    let bounds = selection_bounds(&doc);

    assert_eq!(bounds, Some((10.0, 10.0, 50.0, 50.0)));
}

#[test]
fn given_all_locked_nodes_when_selection_bounds_then_none() {
    let mut doc = DiagramDocument::default();
    let locked_a = NodeId::new(String::from("locked_a"));
    let locked_b = NodeId::new(String::from("locked_b"));

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            locked_a.clone(),
            make_node_with_lock(NodeKind::Node, 10.0, 10.0, 50.0, 50.0, true),
        )
        .update(
            locked_b.clone(),
            make_node_with_lock(NodeKind::Node, 100.0, 100.0, 50.0, 50.0, true),
        );

    let _ = doc.editor_state.selected_items.insert(locked_a.to_string());
    let _ = doc.editor_state.selected_items.insert(locked_b.to_string());

    let bounds = selection_bounds(&doc);

    assert_eq!(bounds, None);
}

#[test]
fn given_mixed_selection_when_selected_node_ids_then_exclude_locked() {
    let mut doc = DiagramDocument::default();
    let unlocked_id = NodeId::new(String::from("unlocked"));
    let locked_id = NodeId::new(String::from("locked"));

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            unlocked_id.clone(),
            make_node_with_lock(NodeKind::Node, 10.0, 10.0, 50.0, 50.0, false),
        )
        .update(
            locked_id.clone(),
            make_node_with_lock(NodeKind::Node, 100.0, 100.0, 50.0, 50.0, true),
        );

    let _ = doc
        .editor_state
        .selected_items
        .insert(unlocked_id.to_string());
    let _ = doc
        .editor_state
        .selected_items
        .insert(locked_id.to_string());

    let ids = selected_node_ids(&doc);

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&unlocked_id));
    assert!(!ids.contains(&locked_id));
}

// ============== GEO-024: Kani proofs (for formal verification) ==============

#[cfg(kani)]
#[kani::proof]
fn given_locked_and_unlocked_nodes_when_selection_bounds_then_exclude_locked() {
    let mut doc = DiagramDocument::default();
    let unlocked_id = NodeId::new(String::from("unlocked"));
    let locked_id = NodeId::new(String::from("locked"));

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            unlocked_id.clone(),
            make_node_with_lock(NodeKind::Node, 10.0, 10.0, 50.0, 50.0, false),
        )
        .update(
            locked_id.clone(),
            make_node_with_lock(NodeKind::Node, 100.0, 100.0, 50.0, 50.0, true),
        );

    let _ = doc
        .editor_state
        .selected_items
        .insert(unlocked_id.to_string());
    let _ = doc
        .editor_state
        .selected_items
        .insert(locked_id.to_string());

    let bounds = selection_bounds(&doc);

    assert_eq!(bounds, Some((10.0, 10.0, 50.0, 50.0)));
}

#[cfg(kani)]
#[kani::proof]
fn given_all_locked_nodes_when_selection_bounds_then_none() {
    let mut doc = DiagramDocument::default();
    let locked_a = NodeId::new(String::from("locked_a"));
    let locked_b = NodeId::new(String::from("locked_b"));

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            locked_a.clone(),
            make_node_with_lock(NodeKind::Node, 10.0, 10.0, 50.0, 50.0, true),
        )
        .update(
            locked_b.clone(),
            make_node_with_lock(NodeKind::Node, 100.0, 100.0, 50.0, 50.0, true),
        );

    let _ = doc.editor_state.selected_items.insert(locked_a.to_string());
    let _ = doc.editor_state.selected_items.insert(locked_b.to_string());

    let bounds = selection_bounds(&doc);

    assert_eq!(bounds, None);
}

#[cfg(kani)]
#[kani::proof]
fn given_mixed_selection_when_selected_node_ids_then_exclude_locked() {
    let mut doc = DiagramDocument::default();
    let unlocked_id = NodeId::new(String::from("unlocked"));
    let locked_id = NodeId::new(String::from("locked"));

    doc.document.nodes = doc
        .document
        .nodes
        .update(
            unlocked_id.clone(),
            make_node_with_lock(NodeKind::Node, 10.0, 10.0, 50.0, 50.0, false),
        )
        .update(
            locked_id.clone(),
            make_node_with_lock(NodeKind::Node, 100.0, 100.0, 50.0, 50.0, true),
        );

    let _ = doc
        .editor_state
        .selected_items
        .insert(unlocked_id.to_string());
    let _ = doc
        .editor_state
        .selected_items
        .insert(locked_id.to_string());

    let ids = selected_node_ids(&doc);

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&unlocked_id));
    assert!(!ids.contains(&locked_id));
}
