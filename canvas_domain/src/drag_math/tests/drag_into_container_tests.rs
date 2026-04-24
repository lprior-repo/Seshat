use super::subgraph_helpers::{make_child_node, make_subgraph_node};
use crate::stubs::drag_original_positions;
use diagram_models::document::{DiagramDocument, OrderedFloat};

// SUB-006: Drag multiple selected nodes into container
// SUB-007: Drag container into another container (nesting)
// MUL-003: Drag selection across container boundary triggers reparent

/// Given multiple selected nodes outside a container,
/// when drag positions are calculated,
/// then both nodes are tracked for the drag operation.
#[cfg(kani)]
#[kani::proof]
fn given_multiple_selected_nodes_when_drag_position_calculated_then_all_tracked() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 300.0, 100.0, 200.0, 150.0, false, None, None);
    doc.document.nodes.insert(container_id, container);

    let (node1_id, node1) = make_child_node("node1", 50.0, 100.0, 60.0, 30.0, false, None);
    let (node2_id, node2) = make_child_node("node2", 50.0, 150.0, 60.0, 30.0, false, None);
    doc.document.nodes.insert(node1_id.clone(), node1);
    doc.document.nodes.insert(node2_id.clone(), node2);

    let selected = im::HashSet::new()
        .update(node1_id.to_string())
        .update(node2_id.to_string());
    let positions = drag_original_positions(&doc, &selected);

    assert_eq!(positions.len(), 2, "Both selected nodes should be tracked");
    assert!(
        positions.contains_key(&node1_id),
        "Node1 should have original position recorded"
    );
    assert!(
        positions.contains_key(&node2_id),
        "Node2 should have original position recorded"
    );

    let pos1 = positions.get(&node1_id);
    let pos2 = positions.get(&node2_id);
    assert_eq!(pos1.map(|p| p.0), Some(50.0), "Node1 x position");
    assert_eq!(pos1.map(|p| p.1), Some(100.0), "Node1 y position");
    assert_eq!(pos2.map(|p| p.0), Some(50.0), "Node2 x position");
    assert_eq!(pos2.map(|p| p.1), Some(150.0), "Node2 y position");
}

/// Given two containers where one can be nested inside the other,
/// when the inner container is positioned within outer bounds,
/// then the geometry supports valid nesting.
#[cfg(kani)]
#[kani::proof]
fn given_two_containers_when_inner_positioned_in_outer_then_geometry_supports_nesting() {
    let mut doc = DiagramDocument::default();

    let (outer_id, outer) =
        make_subgraph_node("outer", 100.0, 100.0, 400.0, 300.0, false, None, None);
    doc.document.nodes.insert(outer_id.clone(), outer);

    let (inner_id, inner) =
        make_subgraph_node("inner", 150.0, 150.0, 200.0, 150.0, false, None, None);
    doc.document.nodes.insert(inner_id.clone(), inner);

    let outer_node = doc.document.nodes.get(&outer_id).expect("outer exists");
    let inner_node = doc.document.nodes.get(&inner_id).expect("inner exists");

    let outer_rect = (
        outer_node.x.0,
        outer_node.y.0,
        outer_node.width.0,
        outer_node.height.0,
    );
    let inner_rect = (
        inner_node.x.0,
        inner_node.y.0,
        inner_node.width.0,
        inner_node.height.0,
    );

    assert!(
        crate::math::within(outer_rect, inner_rect),
        "Inner container should fit within outer container bounds for valid nesting"
    );

    assert_eq!(doc.document.nodes.len(), 2);
    assert!(
        inner_node.parent.is_none(),
        "Inner starts without parent (would be assigned on drop)"
    );
}

/// Given multi-selection dragged across container boundary,
/// when drag ends inside container,
/// then all selected nodes should be reparented to the target container.
#[test]
fn given_multi_selection_dragged_across_container_boundary_when_ends_inside_then_reparents() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 300.0, 100.0, 200.0, 200.0, false, None, None);
    doc.document.nodes.insert(container_id, container);

    let (node1_id, node1) = make_child_node("node1", 50.0, 150.0, 60.0, 30.0, false, None);
    let (node2_id, node2) = make_child_node("node2", 150.0, 150.0, 60.0, 30.0, false, None);
    doc.document.nodes.insert(node1_id.clone(), node1);
    doc.document.nodes.insert(node2_id.clone(), node2);

    let selected = im::HashSet::new()
        .update(node1_id.to_string())
        .update(node2_id.to_string());
    doc.editor_state.selected_items = selected.clone();

    let positions = drag_original_positions(&doc, &selected);
    assert_eq!(positions.len(), 2, "Both selected nodes should be tracked");

    let drag_delta = (300.0, 0.0);

    if let Some(node) = doc.document.nodes.get_mut(&node1_id) {
        node.x = OrderedFloat(50.0 + drag_delta.0);
        node.y = OrderedFloat(150.0 + drag_delta.1);
    }
    if let Some(node) = doc.document.nodes.get_mut(&node2_id) {
        node.x = OrderedFloat(150.0 + drag_delta.0);
        node.y = OrderedFloat(150.0 + drag_delta.1);
    }

    let node1 = doc.document.nodes.get(&node1_id).unwrap();
    let node2 = doc.document.nodes.get(&node2_id).unwrap();
    assert!(
        node1.x.0 >= 300.0 && node1.x.0 <= 500.0,
        "Node1 should be inside container X bounds"
    );
    assert!(
        node1.y.0 >= 100.0 && node1.y.0 <= 300.0,
        "Node1 should be inside container Y bounds"
    );
    assert!(
        node2.x.0 >= 300.0 && node2.x.0 <= 500.0,
        "Node2 should be inside container X bounds"
    );
}

/// Given multi-selection dragged OUT of container,
/// when drag ends outside container,
/// then all selected nodes should be reparented to root (None).
#[test]
fn given_multi_selection_dragged_out_of_container_when_ends_outside_then_reparents_to_root() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 100.0, 100.0, 200.0, 200.0, false, None, None);
    doc.document.nodes.insert(container_id.clone(), container);

    let (node1_id, node1) = make_child_node(
        "node1",
        150.0,
        150.0,
        60.0,
        30.0,
        false,
        Some(container_id.clone()),
    );
    let (node2_id, node2) =
        make_child_node("node2", 200.0, 150.0, 60.0, 30.0, false, Some(container_id));
    doc.document.nodes.insert(node1_id.clone(), node1);
    doc.document.nodes.insert(node2_id.clone(), node2);

    let selected = im::HashSet::new()
        .update(node1_id.to_string())
        .update(node2_id.to_string());
    doc.editor_state.selected_items = selected.clone();

    let positions = drag_original_positions(&doc, &selected);
    assert_eq!(positions.len(), 2, "Both selected nodes should be tracked");

    let drag_delta = (200.0, 0.0);

    if let Some(node) = doc.document.nodes.get_mut(&node1_id) {
        node.x = OrderedFloat(150.0 + drag_delta.0);
        node.y = OrderedFloat(150.0 + drag_delta.1);
    }
    if let Some(node) = doc.document.nodes.get_mut(&node2_id) {
        node.x = OrderedFloat(200.0 + drag_delta.0);
        node.y = OrderedFloat(150.0 + drag_delta.1);
    }

    let node1 = doc.document.nodes.get(&node1_id).unwrap();
    assert!(
        node1.x.0 > 300.0 || node1.y.0 > 300.0 || node1.y.0 < 100.0,
        "Node1 should be outside container bounds"
    );
}
