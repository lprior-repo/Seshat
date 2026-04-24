use super::subgraph_helpers::{make_child_node, make_subgraph_node};
use crate::interaction_reducer::InteractionMode;
use crate::stubs::drag_original_positions;
use diagram_models::document::DiagramDocument;

// SUB-005: Parent-child relationship preservation during selection

/// Given a container with children,
/// when the container is selected and resized,
/// then children are included in resize targets and parent references are preserved.
#[cfg(kani)]
#[kani::proof]
fn given_container_with_children_when_selected_then_children_included_in_resize_targets() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 100.0, 100.0, 300.0, 200.0, false, None, None);
    doc.document.nodes.insert(container_id.clone(), container);

    let (child1_id, child1) = make_child_node(
        "child1",
        120.0,
        130.0,
        60.0,
        30.0,
        false,
        Some(container_id.clone()),
    );
    doc.document.nodes.insert(child1_id.clone(), child1);

    let (child2_id, child2) = make_child_node(
        "child2",
        200.0,
        180.0,
        60.0,
        30.0,
        false,
        Some(container_id.clone()),
    );
    doc.document.nodes.insert(child2_id.clone(), child2);

    let (outside_id, outside) =
        make_child_node("outside", 500.0, 100.0, 60.0, 30.0, false, None);
    doc.document.nodes.insert(outside_id.clone(), outside);

    let _ = doc
        .editor_state
        .selected_items
        .insert(container_id.to_string());

    let selected = doc
        .editor_state
        .selected_items
        .iter()
        .map(|s| diagram_models::document::NodeId::new(s.clone()))
        .collect::<Vec<_>>();
    let node_geometry = doc
        .document
        .nodes
        .iter()
        .map(|(id, node)| {
            (
                id.clone(),
                (
                    node.x.0,
                    node.y.0,
                    node.width.0,
                    node.height.0,
                    node.kind == diagram_models::document::NodeKind::Subgraph,
                ),
            )
        })
        .collect::<im::HashMap<_, _>>();
    let targets = crate::drag_math::subgraphs::calculate_resize_target_ids(&selected, &node_geometry);

    assert!(
        targets.contains(&container_id),
        "Container should be in resize targets"
    );
    assert!(
        targets.contains(&child1_id),
        "Child1 inside container should be in resize targets"
    );
    assert!(
        targets.contains(&child2_id),
        "Child2 inside container should be in resize targets"
    );
    assert!(
        !targets.contains(&outside_id),
        "Node outside container should NOT be in resize targets"
    );
}

/// Given a container with children,
/// when the container is selected for resize,
/// then the parent references of children remain intact.
#[cfg(kani)]
#[kani::proof]
fn given_container_with_children_when_resizing_then_parent_references_preserved() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 100.0, 100.0, 300.0, 200.0, false, None, None);
    doc.document.nodes.insert(container_id.clone(), container);

    let (child_id, child) = make_child_node(
        "child",
        150.0,
        150.0,
        60.0,
        30.0,
        false,
        Some(container_id.clone()),
    );
    doc.document.nodes.insert(child_id.clone(), child);

    let _ = doc
        .editor_state
        .selected_items
        .insert(container_id.to_string());

    let mut mode = InteractionMode::Select;
    let _ = crate::interaction_reducer::finalize_motion_release(&mut mode, &mut doc, &None);

    let child_node = doc.document.nodes.get(&child_id).expect("child exists");
    assert_eq!(
        child_node.parent,
        Some(container_id.clone()),
        "Child's parent reference should be preserved after resize operation"
    );
}

/// Given nested containers,
/// when checking parent-child relationships,
/// then each node correctly references its immediate parent.
#[cfg(kani)]
#[kani::proof]
fn given_nested_containers_then_parent_chain_is_correct() {
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
        false,
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
        outer_node.parent.is_none(),
        "Outer container should have no parent"
    );
    assert_eq!(
        inner_node.parent,
        Some(outer_id.clone()),
        "Inner's parent should be outer"
    );
    assert_eq!(
        child_node.parent,
        Some(inner_id.clone()),
        "Child's parent should be inner (not outer)"
    );
}
