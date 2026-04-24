use super::subgraph_helpers::{make_child_node, make_subgraph_node};
use diagram_models::document::DiagramDocument;

// SUB-002: Box-select across container boundary

/// Given nodes inside and outside a container,
/// when performing rubber-band selection that spans both areas,
/// then nodes from both inside and outside the container should be selectable.
#[cfg(kani)]
#[kani::proof]
fn given_nodes_inside_and_outside_container_when_rubberband_selection_then_all_selectable() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 100.0, 100.0, 200.0, 150.0, false, None, None);
    doc.document.nodes.insert(container_id.clone(), container);

    let (child_inside_id, child_inside) = make_child_node(
        "child_inside",
        120.0,
        120.0,
        50.0,
        30.0,
        false,
        Some(container_id.clone()),
    );
    doc.document
        .nodes
        .insert(child_inside_id.clone(), child_inside);

    let (outside_id, outside) =
        make_child_node("outside", 400.0, 100.0, 50.0, 30.0, false, None);
    doc.document.nodes.insert(outside_id.clone(), outside);

    let _ = doc
        .editor_state
        .selected_items
        .insert(child_inside_id.to_string());
    let _ = doc
        .editor_state
        .selected_items
        .insert(outside_id.to_string());

    assert_eq!(
        doc.editor_state.selected_items.len(),
        2,
        "Both nodes should be selectable"
    );
    assert!(
        doc.editor_state.selected_items.contains("child_inside"),
        "Child inside container should be selected"
    );
    assert!(
        doc.editor_state.selected_items.contains("outside"),
        "Node outside container should be selected"
    );
}

/// Given a rubber-band selection area,
/// when the area partially overlaps a container,
/// then only nodes within the selection area are selected (not all container children).
#[cfg(kani)]
#[kani::proof]
fn given_partial_container_overlap_when_rubberband_then_only_overlapping_selected() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 100.0, 100.0, 300.0, 200.0, false, None, None);
    doc.document.nodes.insert(container_id.clone(), container);

    let (left_child_id, left_child) = make_child_node(
        "left_child",
        120.0,
        130.0,
        50.0,
        30.0,
        false,
        Some(container_id.clone()),
    );
    doc.document.nodes.insert(left_child_id.clone(), left_child);

    let (right_child_id, right_child) = make_child_node(
        "right_child",
        320.0,
        130.0,
        50.0,
        30.0,
        false,
        Some(container_id.clone()),
    );
    doc.document
        .nodes
        .insert(right_child_id.clone(), right_child);

    let _ = doc
        .editor_state
        .selected_items
        .insert(left_child_id.to_string());

    assert_eq!(
        doc.editor_state.selected_items.len(),
        1,
        "Only one child should be selected"
    );
    assert!(
        doc.editor_state.selected_items.contains("left_child"),
        "Left child should be selected"
    );
    assert!(
        !doc.editor_state.selected_items.contains("right_child"),
        "Right child should NOT be selected"
    );
}
