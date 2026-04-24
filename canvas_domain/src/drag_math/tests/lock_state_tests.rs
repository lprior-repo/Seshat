use super::subgraph_helpers::{make_child_node, make_subgraph_node};
use diagram_models::document::DiagramDocument;

// SUB-004: Locked container with unlocked children

/// Given a locked container with unlocked children,
/// when checking lock status,
/// then children are independently unlocked (not inheriting parent's locked state).
#[cfg(kani)]
#[kani::proof]
fn given_locked_container_with_unlocked_children_then_children_are_independently_unlocked() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 100.0, 100.0, 200.0, 150.0, true, None, None);
    doc.document.nodes.insert(container_id.clone(), container);

    let (child_id, child) = make_child_node(
        "child",
        120.0,
        120.0,
        50.0,
        30.0,
        false,
        Some(container_id.clone()),
    );
    doc.document.nodes.insert(child_id.clone(), child);

    let container_node = doc
        .document
        .nodes
        .get(&container_id)
        .expect("container exists");
    let child_node = doc.document.nodes.get(&child_id).expect("child exists");

    assert!(
        container_node.lock_state.is_locked(),
        "Container should be locked"
    );
    assert!(
        !child_node.lock_state.is_locked(),
        "Child should be unlocked despite parent being locked"
    );
}

/// Given a locked container with unlocked child,
/// when selecting the child,
/// then the child can be selected independently.
#[cfg(kani)]
#[kani::proof]
fn given_locked_container_when_selecting_unlocked_child_then_child_is_selectable() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 100.0, 100.0, 200.0, 150.0, true, None, None);
    doc.document.nodes.insert(container_id.clone(), container);

    let (child_id, child) = make_child_node(
        "child",
        120.0,
        120.0,
        50.0,
        30.0,
        false,
        Some(container_id.clone()),
    );
    doc.document.nodes.insert(child_id.clone(), child);

    let _ = doc.editor_state.selected_items.insert(child_id.to_string());

    assert_eq!(
        doc.editor_state.selected_items.len(),
        1,
        "Child should be selectable"
    );
    assert!(
        doc.editor_state.selected_items.contains("child"),
        "Unlocked child should be selectable inside locked container"
    );
    assert!(
        !doc.editor_state.selected_items.contains("container"),
        "Locked container should not be selected when clicking child"
    );
}

/// Given mixed lock states in a hierarchy,
/// when checking each node's lock state,
/// then each node maintains its own lock state without inheritance.
#[cfg(kani)]
#[kani::proof]
fn given_mixed_lock_hierarchy_then_lock_states_are_per_node() {
    let mut doc = DiagramDocument::default();

    let (outer_id, outer) =
        make_subgraph_node("outer", 50.0, 50.0, 400.0, 300.0, false, None, None);
    doc.document.nodes.insert(outer_id.clone(), outer);

    let (inner_id, inner) = make_subgraph_node(
        "inner",
        100.0,
        100.0,
        250.0,
        180.0,
        true,
        None,
        Some(outer_id.clone()),
    );
    doc.document.nodes.insert(inner_id.clone(), inner);

    let (child_id, child) = make_child_node(
        "child",
        150.0,
        150.0,
        60.0,
        30.0,
        false,
        Some(inner_id.clone()),
    );
    doc.document.nodes.insert(child_id.clone(), child);

    let outer_node = doc.document.nodes.get(&outer_id).expect("outer exists");
    let inner_node = doc.document.nodes.get(&inner_id).expect("inner exists");
    let child_node = doc.document.nodes.get(&child_id).expect("child exists");

    assert!(
        !outer_node.lock_state.is_locked(),
        "Outer should be unlocked"
    );
    assert!(inner_node.lock_state.is_locked(), "Inner should be locked");
    assert!(
        !child_node.lock_state.is_locked(),
        "Child should be unlocked (not inheriting inner's lock)"
    );
}
