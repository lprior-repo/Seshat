use super::subgraph_helpers::{make_child_node, make_subgraph_node};
use diagram_models::document::DiagramDocument;

// SUB-003: Collapse/expand container behavior

/// Given a container with collapsed state,
/// when serialized and deserialized,
/// then the collapsed state is preserved.
#[cfg(kani)]
#[kani::proof]
fn given_container_with_collapsed_state_when_roundtripped_then_state_preserved() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) = make_subgraph_node(
        "container",
        100.0,
        100.0,
        200.0,
        150.0,
        false,
        Some(true),
        None,
    );
    doc.document.nodes.insert(container_id.clone(), container);

    let json = serde_json::to_string(&doc).expect("serialization should succeed");
    let loaded: DiagramDocument =
        serde_json::from_str(&json).expect("deserialization should succeed");

    let loaded_container = loaded
        .document
        .nodes
        .get(&container_id)
        .expect("container should exist");
    assert_eq!(
        loaded_container.collapsed,
        Some(true),
        "Collapsed state should be preserved as true"
    );
}

/// Given an expanded container with children,
/// when the container is set to collapsed,
/// then the collapsed field reflects this but children remain in document.
#[cfg(kani)]
#[kani::proof]
fn given_expanded_container_when_collapsed_then_children_remain_in_document() {
    let mut doc = DiagramDocument::default();

    let (container_id, mut container) = make_subgraph_node(
        "container",
        100.0,
        100.0,
        200.0,
        150.0,
        false,
        Some(false),
        None,
    );

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
    doc.document
        .nodes
        .insert(container_id.clone(), container.clone());

    container.collapsed = Some(true);
    doc.document
        .nodes
        .insert(container_id.clone(), container.clone());

    assert!(
        doc.document.nodes.contains_key(&child_id),
        "Child should still exist in document after collapse"
    );
    assert_eq!(
        doc.document.nodes.len(),
        2,
        "Both container and child should exist"
    );

    let container_node = doc
        .document
        .nodes
        .get(&container_id)
        .expect("container exists");
    assert_eq!(
        container_node.collapsed,
        Some(true),
        "Container should be marked as collapsed"
    );
}

/// Given containers with different collapsed states,
/// when queried, each container maintains its own collapsed state independently.
#[cfg(kani)]
#[kani::proof]
fn given_multiple_containers_when_collapsed_independently_then_states_are_independent() {
    let mut doc = DiagramDocument::default();

    let (expanded_id, expanded) = make_subgraph_node(
        "expanded",
        50.0,
        50.0,
        200.0,
        100.0,
        false,
        Some(false),
        None,
    );
    let (collapsed_id, collapsed) = make_subgraph_node(
        "collapsed",
        300.0,
        50.0,
        200.0,
        100.0,
        false,
        Some(true),
        None,
    );

    doc.document.nodes.insert(expanded_id.clone(), expanded);
    doc.document.nodes.insert(collapsed_id.clone(), collapsed);

    let expanded_node = doc
        .document
        .nodes
        .get(&expanded_id)
        .expect("expanded exists");
    let collapsed_node = doc
        .document
        .nodes
        .get(&collapsed_id)
        .expect("collapsed exists");

    assert_eq!(
        expanded_node.collapsed,
        Some(false),
        "First container should be expanded"
    );
    assert_eq!(
        collapsed_node.collapsed,
        Some(true),
        "Second container should be collapsed"
    );
}
