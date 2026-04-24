use super::subgraph_helpers::{make_child_node, make_subgraph_node};
use crate::stubs::drag_original_positions;
use diagram_models::document::DiagramDocument;

// SUB-008: Grab parent prevents reparent gesture
// SUB-009: Container auto-expand when child crosses boundary
// SUB-010: Drag selection with nested descendants

/// Given a nested container hierarchy,
/// when a middle container (which has children) is selected,
/// then dragging includes both the container and its descendants.
#[cfg(kani)]
#[kani::proof]
fn given_nested_container_with_children_when_middle_selected_then_descendants_included() {
    let mut doc = DiagramDocument::default();

    let (outer_id, outer) =
        make_subgraph_node("outer", 100.0, 100.0, 400.0, 300.0, false, None, None);
    doc.document.nodes.insert(outer_id.clone(), outer);

    let (inner_id, inner) = make_subgraph_node(
        "inner",
        150.0,
        150.0,
        200.0,
        150.0,
        false,
        None,
        Some(outer_id.clone()),
    );
    doc.document.nodes.insert(inner_id.clone(), inner);

    let (child_id, child) = make_child_node(
        "child",
        180.0,
        180.0,
        60.0,
        30.0,
        false,
        Some(inner_id.clone()),
    );
    doc.document.nodes.insert(child_id.clone(), child);

    let selected = im::HashSet::new().update(inner_id.to_string());
    let positions = drag_original_positions(&doc, &selected);

    assert!(
        positions.contains_key(&inner_id),
        "Selected inner container should be in drag positions"
    );
    assert!(
        positions.contains_key(&child_id),
        "Child of selected container should be included in drag positions"
    );
    assert!(
        !positions.contains_key(&outer_id),
        "Outer (ancestor) should NOT be included when selecting inner"
    );
}

/// Given a container with a child near the edge,
/// when calculating resize targets,
/// then both container and child are included for boundary calculations.
#[cfg(kani)]
#[kani::proof]
fn given_container_with_child_near_edge_when_resize_targets_then_both_included() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 100.0, 100.0, 200.0, 150.0, false, None, None);
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
        targets.contains(&child_id),
        "Child inside container should be in resize targets"
    );
    assert_eq!(
        targets.len(),
        2,
        "Should have exactly container and child in targets"
    );
}

/// Given a three-level hierarchy (outer -> inner -> leaf),
/// when the outer container is selected,
/// then drag positions include all descendants.
#[cfg(kani)]
#[kani::proof]
fn given_three_level_hierarchy_when_outer_selected_then_all_descendants_in_drag_positions() {
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

    let (leaf_id, leaf) = make_child_node(
        "leaf",
        150.0,
        150.0,
        60.0,
        30.0,
        false,
        Some(inner_id.clone()),
    );
    doc.document.nodes.insert(leaf_id.clone(), leaf);

    let selected = im::HashSet::new().update(outer_id.to_string());
    let positions = drag_original_positions(&doc, &selected);

    assert_eq!(
        positions.len(),
        3,
        "All three nodes in hierarchy should be in drag positions"
    );
    assert!(
        positions.contains_key(&outer_id),
        "Outer container should be in drag positions"
    );
    assert!(
        positions.contains_key(&inner_id),
        "Inner container (descendant) should be in drag positions"
    );
    assert!(
        positions.contains_key(&leaf_id),
        "Leaf node (descendant of descendant) should be in drag positions"
    );

    let outer_pos = positions.get(&outer_id);
    let inner_pos = positions.get(&inner_id);
    let leaf_pos = positions.get(&leaf_id);

    assert_eq!(outer_pos.map(|p| (p.0, p.1)), Some((50.0, 50.0)));
    assert_eq!(inner_pos.map(|p| (p.0, p.1)), Some((100.0, 100.0)));
    assert_eq!(leaf_pos.map(|p| (p.0, p.1)), Some((150.0, 150.0)));
}
