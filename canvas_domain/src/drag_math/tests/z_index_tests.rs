use super::subgraph_helpers::{make_child_node, make_subgraph_node};
use diagram_models::document::DiagramDocument;

// SUB-001: Click inside container selects child vs container

/// Given a container with a child at overlapping position,
/// when hit testing by position, the child should be prioritized due to higher z_index.
#[cfg(kani)]
#[kani::proof]
fn given_container_with_child_when_hit_testing_then_child_has_higher_z_index() {
    let mut doc = DiagramDocument::default();

    let (container_id, container) =
        make_subgraph_node("container", 100.0, 100.0, 300.0, 200.0, false, None, None);
    doc.document.nodes.insert(container_id.clone(), container);

    let (child_id, child) = make_child_node(
        "child",
        150.0,
        150.0,
        80.0,
        40.0,
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
        child_node.z_index > container_node.z_index,
        "Child z_index ({}) should be greater than container z_index ({})",
        child_node.z_index,
        container_node.z_index
    );

    let container_rect = (
        container_node.x.0,
        container_node.y.0,
        container_node.width.0,
        container_node.height.0,
    );
    let child_rect = (
        child_node.x.0,
        child_node.y.0,
        child_node.width.0,
        child_node.height.0,
    );
    assert!(
        crate::math::within(container_rect, child_rect),
        "Child should be geometrically within container bounds"
    );
}

/// Given a container with multiple children at different z_index values,
/// when selecting by position, the highest z_index node should be preferred.
#[cfg(kani)]
#[kani::proof]
fn given_nested_nodes_when_selecting_by_position_then_highest_z_index_wins() {
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

    let outer_z = doc
        .document
        .nodes
        .get(&outer_id)
        .map(|n| n.z_index)
        .unwrap_or(0);
    let inner_z = doc
        .document
        .nodes
        .get(&inner_id)
        .map(|n| n.z_index)
        .unwrap_or(0);
    let child_z = doc
        .document
        .nodes
        .get(&child_id)
        .map(|n| n.z_index)
        .unwrap_or(0);

    assert_eq!(outer_z, -1, "Outer container should have z_index -1");
    assert_eq!(inner_z, -1, "Inner container should have z_index -1");
    assert_eq!(child_z, 1000, "Child should have z_index 1000");
    assert!(child_z > outer_z && child_z > inner_z);
}
