#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use crate::export::svg::generate_svg_string;
use diagram_models::document::{
    DiagramDocument, DocumentData, Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat,
    Revision,
};
use im::HashMap;

fn create_node(id: &str, x: f64, y: f64, width: f64, height: f64, label: &str) -> (NodeId, Node) {
    (
        NodeId::new(id.to_string()),
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: label.to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(width),
            height: OrderedFloat(height),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        },
    )
}

fn create_edge(id: &str, source: &str, target: &str) -> (EdgeId, Edge) {
    (
        EdgeId::new(id.to_string()),
        Edge {
            source: NodeId::new(source.to_string()),
            target: NodeId::new(target.to_string()),
            label: String::new(),
            style: diagram_models::document::EdgeStyle::Solid,
            arrow_type: diagram_models::document::ArrowType::Default,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        },
    )
}

fn create_empty_document() -> DiagramDocument {
    DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        },
        editor_state: diagram_models::document::EditorState::default(),
    }
}

// ============== calculate_bounds tests ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_empty_document_when_calculate_bounds_then_returns_default_bounds(
) -> Result<(), anyhow::Error> {
    // Given
    let doc = create_empty_document();

    // When
    let (min_x, min_y, max_x, max_y) = crate::export::svg::grid::calculate_bounds(&doc);

    // Then
    assert_eq!(min_x, 0.0);
    assert_eq!(min_y, 0.0);
    assert_eq!(max_x, 800.0);
    assert_eq!(max_y, 600.0);
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_single_node_at_origin_when_calculate_bounds_then_returns_node_bounds(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id, node) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "Test");
    doc.document.nodes.insert(id, node);

    // When
    let (min_x, min_y, max_x, max_y) = crate::export::svg::grid::calculate_bounds(&doc);

    // Then - bounds should include full node extent (x + width, y + height)
    assert_eq!(min_x, 0.0);
    assert_eq!(min_y, 0.0);
    assert_eq!(max_x, 100.0, "max_x should be x + width = 0 + 100");
    assert_eq!(max_y, 50.0, "max_y should be y + height = 0 + 50");
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_single_node_with_positive_coords_when_calculate_bounds_then_returns_node_bounds(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id, node) = create_node("n1", 200.0, 150.0, 100.0, 80.0, "Test");
    doc.document.nodes.insert(id, node);

    // When
    let (min_x, min_y, max_x, max_y) = crate::export::svg::grid::calculate_bounds(&doc);

    // Then
    assert_eq!(min_x, 200.0);
    assert_eq!(min_y, 150.0);
    assert_eq!(max_x, 300.0, "max_x should be 200 + 100 = 300");
    assert_eq!(max_y, 230.0, "max_y should be 150 + 80 = 230");
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_two_nodes_when_calculate_bounds_then_returns_combined_bounds() -> Result<(), anyhow::Error>
{
    // Given
    let mut doc = create_empty_document();
    let (id1, node1) = create_node("n1", 100.0, 100.0, 50.0, 50.0, "Node1");
    let (id2, node2) = create_node("n2", 200.0, 300.0, 60.0, 40.0, "Node2");
    doc.document.nodes.insert(id1, node1);
    doc.document.nodes.insert(id2, node2);

    // When
    let (min_x, min_y, max_x, max_y) = crate::export::svg::grid::calculate_bounds(&doc);

    // Then
    assert_eq!(min_x, 100.0, "min_x should be minimum x of all nodes");
    assert_eq!(min_y, 100.0, "min_y should be minimum y of all nodes");
    assert_eq!(
        max_x, 260.0,
        "max_x should be max of (100+50, 200+60) = 260"
    );
    assert_eq!(
        max_y, 340.0,
        "max_y should be max of (100+50, 300+40) = 340"
    );
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_nodes_with_negative_coords_when_calculate_bounds_then_handles_negative_values(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id, node) = create_node("n1", -100.0, -50.0, 200.0, 100.0, "Test");
    doc.document.nodes.insert(id, node);

    // When
    let (min_x, min_y, max_x, max_y) = crate::export::svg::grid::calculate_bounds(&doc);

    // Then
    assert_eq!(min_x, -100.0);
    assert_eq!(min_y, -50.0);
    assert_eq!(max_x, 100.0, "max_x should be -100 + 200 = 100");
    assert_eq!(max_y, 50.0, "max_y should be -50 + 100 = 50");
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_overlapping_nodes_when_calculate_bounds_then_returns_union_bounds(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id1, node1) = create_node("n1", 100.0, 100.0, 200.0, 200.0, "Big");
    let (id2, node2) = create_node("n2", 150.0, 150.0, 50.0, 50.0, "Small");
    doc.document.nodes.insert(id1, node1);
    doc.document.nodes.insert(id2, node2);

    // When
    let (min_x, min_y, max_x, max_y) = crate::export::svg::grid::calculate_bounds(&doc);

    // Then - should be union of both nodes
    assert_eq!(min_x, 100.0);
    assert_eq!(min_y, 100.0);
    assert_eq!(
        max_x, 300.0,
        "max_x should be max of (100+200, 150+50) = 300"
    );
    assert_eq!(
        max_y, 300.0,
        "max_y should be max of (100+200, 150+50) = 300"
    );
    Ok(())
}
