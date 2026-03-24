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

// ============== generate_svg_string tests ==============

#[cfg(kani)]
#[kani::proof]
fn given_empty_document_when_generate_svg_string_then_contains_valid_svg_structure(
) -> Result<(), anyhow::Error> {
    // Given
    let doc = create_empty_document();

    // When
    let svg = generate_svg_string(&doc);

    // Then
    assert!(svg.starts_with("<svg"), "Should start with svg tag");
    assert!(svg.ends_with("</svg>"), "Should end with closing svg tag");
    assert!(svg.contains("xmlns='http://www.w3.org/2000/svg'"));
    assert!(svg.contains("viewBox="));
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_empty_document_when_generate_svg_string_then_uses_default_viewbox(
) -> Result<(), anyhow::Error> {
    // Given
    let doc = create_empty_document();

    // When
    let svg = generate_svg_string(&doc);

    // Then - empty doc uses default bounds (0, 0, 800, 600) with margin
    // view_min_x = 0 - 50 = -50, view_min_y = 0 - 50 = -50
    // width = 2*50 + (800-0) = 900, height = 2*50 + (600-0) = 700
    assert!(svg.contains("viewBox='-50 -50 900 700'"));
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_single_node_when_generate_svg_string_then_viewbox_contains_node_with_margin(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id, node) = create_node("n1", 100.0, 100.0, 100.0, 50.0, "Test");
    doc.document.nodes.insert(id, node);

    // When
    let svg = generate_svg_string(&doc);

    // Then
    // Bounds: min_x=100, min_y=100, max_x=200, max_y=150
    // view_min_x = 100 - 50 = 50, view_min_y = 100 - 50 = 50
    // width = 2*50 + (200-100) = 200, height = 2*50 + (150-100) = 150
    assert!(svg.contains("viewBox='50 50 200 150'"));
    assert!(svg.contains("width='200'"));
    assert!(svg.contains("height='150'"));
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_node_when_generate_svg_string_then_contains_node_rect() -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id, node) = create_node("n1", 100.0, 100.0, 120.0, 80.0, "MyNode");
    doc.document.nodes.insert(id, node);

    // When
    let svg = generate_svg_string(&doc);

    // Then
    assert!(svg.contains("<rect width='120' height='80'"));
    assert!(svg.contains("<text"));
    assert!(svg.contains(">MyNode<"));
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_node_when_generate_svg_string_then_transform_uses_node_position(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id, node) = create_node("n1", 250.0, 175.0, 100.0, 50.0, "Test");
    doc.document.nodes.insert(id, node);

    // When
    let svg = generate_svg_string(&doc);

    // Then - transform should use exact x, y coordinates
    assert!(svg.contains("transform='translate(250, 175)'"));
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_edge_between_nodes_when_generate_svg_string_then_line_connects_centers(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id1, node1) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "Source");
    let (id2, node2) = create_node("n2", 200.0, 100.0, 100.0, 50.0, "Target");
    doc.document.nodes.insert(id1, node1);
    doc.document.nodes.insert(id2, node2);

    let (edge_id, edge) = create_edge("e1", "n1", "n2");
    doc.document.edges.insert(edge_id, edge);

    // When
    let svg = generate_svg_string(&doc);

    // Then - line should connect node centers
    // Source center: (0 + 100/2, 0 + 50/2) = (50, 25)
    // Target center: (200 + 100/2, 100 + 50/2) = (250, 125)
    assert!(svg.contains("<line"));
    assert!(svg.contains("x1='50'"), "x1 should be source center x = 50");
    assert!(svg.contains("y1='25'"), "y1 should be source center y = 25");
    assert!(
        svg.contains("x2='250'"),
        "x2 should be target center x = 250"
    );
    assert!(
        svg.contains("y2='125'"),
        "y2 should be target center y = 125"
    );
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_edge_with_offset_nodes_when_generate_svg_string_then_line_uses_correct_arithmetic(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id1, node1) = create_node("n1", 50.0, 75.0, 80.0, 40.0, "A");
    let (id2, node2) = create_node("n2", 300.0, 250.0, 120.0, 60.0, "B");
    doc.document.nodes.insert(id1, node1);
    doc.document.nodes.insert(id2, node2);

    let (edge_id, edge) = create_edge("e1", "n1", "n2");
    doc.document.edges.insert(edge_id, edge);

    // When
    let svg = generate_svg_string(&doc);

    // Then - verify exact arithmetic for center calculation
    // Source center: (50 + 80/2, 75 + 40/2) = (50 + 40, 75 + 20) = (90, 95)
    // Target center: (300 + 120/2, 250 + 60/2) = (300 + 60, 250 + 30) = (360, 280)
    assert!(svg.contains("x1='90'"), "x1 should be 50 + 80/2 = 90");
    assert!(svg.contains("y1='95'"), "y1 should be 75 + 40/2 = 95");
    assert!(svg.contains("x2='360'"), "x2 should be 300 + 120/2 = 360");
    assert!(svg.contains("y2='280'"), "y2 should be 250 + 60/2 = 280");
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_edge_with_missing_source_node_when_generate_svg_string_then_skips_edge(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id2, node2) = create_node("n2", 200.0, 100.0, 100.0, 50.0, "Target");
    doc.document.nodes.insert(id2, node2);

    let (edge_id, edge) = create_edge("e1", "missing", "n2");
    doc.document.edges.insert(edge_id, edge);

    // When
    let svg = generate_svg_string(&doc);

    // Then - no line should be rendered for edge with missing source
    assert!(!svg.contains("<line"));
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_edge_with_missing_target_node_when_generate_svg_string_then_skips_edge(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id1, node1) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "Source");
    doc.document.nodes.insert(id1, node1);

    let (edge_id, edge) = create_edge("e1", "n1", "missing");
    doc.document.edges.insert(edge_id, edge);

    // When
    let svg = generate_svg_string(&doc);

    // Then - no line should be rendered for edge with missing target
    assert!(!svg.contains("<line"));
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_small_content_when_generate_svg_string_then_enforces_minimum_dimensions(
) -> Result<(), anyhow::Error> {
    // Given - node at origin with small dimensions
    let mut doc = create_empty_document();
    let (id, node) = create_node("n1", 0.0, 0.0, 10.0, 10.0, "Tiny");
    doc.document.nodes.insert(id, node);

    // When
    let svg = generate_svg_string(&doc);

    // Then - width/height should be at least 100
    // Raw: width = 2*50 + (10-0) = 110, height = 2*50 + (10-0) = 110
    // Both are > 100, so should be used as-is
    assert!(svg.contains("width='110'"));
    assert!(svg.contains("height='110'"));
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_wide_document_when_generate_svg_string_then_viewbox_reflects_width(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id1, node1) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "Left");
    let (id2, node2) = create_node("n2", 1000.0, 0.0, 100.0, 50.0, "Right");
    doc.document.nodes.insert(id1, node1);
    doc.document.nodes.insert(id2, node2);

    // When
    let svg = generate_svg_string(&doc);

    // Then
    // Bounds: min_x=0, max_x=1100, min_y=0, max_y=50
    // width = 2*50 + (1100-0) = 1200
    assert!(svg.contains("width='1200'"));
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_tall_document_when_generate_svg_string_then_viewbox_reflects_height(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id1, node1) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "Top");
    let (id2, node2) = create_node("n2", 0.0, 800.0, 100.0, 50.0, "Bottom");
    doc.document.nodes.insert(id1, node1);
    doc.document.nodes.insert(id2, node2);

    // When
    let svg = generate_svg_string(&doc);

    // Then
    // Bounds: min_y=0, max_y=850
    // height = 2*50 + (850-0) = 950
    assert!(svg.contains("height='950'"));
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_node_with_exact_position_when_generate_svg_string_then_text_is_centered(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id, node) = create_node("n1", 100.0, 100.0, 200.0, 100.0, "Label");
    doc.document.nodes.insert(id, node);

    // When
    let svg = generate_svg_string(&doc);

    // Then - text x should be width/2 = 100, y should be height - 5 = 95
    assert!(
        svg.contains("text x='100'"),
        "text x should be width/2 = 200/2 = 100"
    );
    assert!(
        svg.contains("y='95'"),
        "text y should be height - 5 = 100 - 5 = 95"
    );
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_multiple_edges_when_generate_svg_string_then_all_edges_rendered(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id1, node1) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "A");
    let (id2, node2) = create_node("n2", 200.0, 0.0, 100.0, 50.0, "B");
    let (id3, node3) = create_node("n3", 100.0, 150.0, 100.0, 50.0, "C");
    doc.document.nodes.insert(id1, node1);
    doc.document.nodes.insert(id2, node2);
    doc.document.nodes.insert(id3, node3);

    let (e1_id, e1) = create_edge("e1", "n1", "n2");
    let (e2_id, e2) = create_edge("e2", "n2", "n3");
    let (e3_id, e3) = create_edge("e3", "n3", "n1");
    doc.document.edges.insert(e1_id, e1);
    doc.document.edges.insert(e2_id, e2);
    doc.document.edges.insert(e3_id, e3);

    // When
    let svg = generate_svg_string(&doc);

    // Then - should have 3 lines
    let line_count = svg.matches("<line").count();
    assert_eq!(line_count, 3, "Should have 3 lines for 3 edges");
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_viewbox_margin_when_generate_svg_string_then_subtracts_50_from_bounds(
) -> Result<(), anyhow::Error> {
    // Given
    let mut doc = create_empty_document();
    let (id, node) = create_node("n1", 100.0, 200.0, 100.0, 50.0, "Test");
    doc.document.nodes.insert(id, node);

    // When
    let svg = generate_svg_string(&doc);

    // Then - view_min_x = 100 - 50 = 50, view_min_y = 200 - 50 = 150
    assert!(
        svg.contains("viewBox='50 150"),
        "viewBox should start at (min_x-50, min_y-50)"
    );
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_node_extent_calculation_when_calculate_bounds_then_adds_width_and_height(
) -> Result<(), anyhow::Error> {
    // Given - node at (100, 200) with size (150, 80)
    let mut doc = create_empty_document();
    let (id, node) = create_node("n1", 100.0, 200.0, 150.0, 80.0, "Test");
    doc.document.nodes.insert(id, node);

    // When
    let (_min_x, _min_y, max_x, max_y) = crate::export::svg::grid::calculate_bounds(&doc);

    // Then - max values should be position + dimension
    assert_eq!(max_x, 250.0, "max_x should be 100 + 150 = 250");
    assert_eq!(max_y, 280.0, "max_y should be 200 + 80 = 280");
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_center_calculation_when_edge_rendered_then_uses_division_by_2() -> Result<(), anyhow::Error>
{
    // Given - odd width/height to verify division
    let mut doc = create_empty_document();
    let (id1, node1) = create_node("n1", 0.0, 0.0, 99.0, 77.0, "A");
    let (id2, node2) = create_node("n2", 0.0, 0.0, 201.0, 303.0, "B");
    doc.document.nodes.insert(id1, node1);
    doc.document.nodes.insert(id2, node2);

    let (edge_id, edge) = create_edge("e1", "n1", "n2");
    doc.document.edges.insert(edge_id, edge);

    // When
    let svg = generate_svg_string(&doc);

    // Then - centers should use exact division
    // Node1 center: (0 + 99/2, 0 + 77/2) = (49.5, 38.5)
    // Node2 center: (0 + 201/2, 0 + 303/2) = (100.5, 151.5)
    assert!(svg.contains("x1='49.5'"), "x1 should be 99/2 = 49.5");
    assert!(svg.contains("y1='38.5'"), "y1 should be 77/2 = 38.5");
    assert!(svg.contains("x2='100.5'"), "x2 should be 201/2 = 100.5");
    assert!(svg.contains("y2='151.5'"), "y2 should be 303/2 = 151.5");
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_node_with_icon_when_generate_svg_string_then_icon_is_centered_horizontally(
) -> Result<(), anyhow::Error> {
    // Given - node with known width and an icon that exists
    let mut doc = create_empty_document();
    let mut node = create_node("n1", 0.0, 0.0, 100.0, 60.0, "Test");
    node.1.icon = String::from("aws/compute/ec2.png"); // Known icon
    doc.document.nodes.insert(node.0, node.1);

    // When
    let svg = generate_svg_string(&doc);

    // Then - icon should be centered: ix = (width - 32) / 2 = (100 - 32) / 2 = 34
    assert!(
        svg.contains("x='34"),
        "icon x should be (100 - 32) / 2 = 34"
    );
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_node_with_icon_when_generate_svg_string_then_icon_is_centered_vertically_with_offset(
) -> Result<(), anyhow::Error> {
    // Given - node with known height and an icon that exists
    let mut doc = create_empty_document();
    let mut node = create_node("n1", 0.0, 0.0, 100.0, 60.0, "Test");
    node.1.icon = String::from("aws/compute/ec2.png"); // Known icon
    doc.document.nodes.insert(node.0, node.1);

    // When
    let svg = generate_svg_string(&doc);

    // Then - icon y = (height - 32) / 2 - 5 = (60 - 32) / 2 - 5 = 14 - 5 = 9
    assert!(
        svg.contains("y='9"),
        "icon y should be (60 - 32) / 2 - 5 = 9"
    );
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_node_with_large_dimensions_when_generate_svg_string_then_icon_position_uses_subtraction(
) -> Result<(), anyhow::Error> {
    // Given - node with 200x100 dimensions
    let mut doc = create_empty_document();
    let mut node = create_node("n1", 0.0, 0.0, 200.0, 100.0, "Test");
    node.1.icon = String::from("aws/compute/ec2.png");
    doc.document.nodes.insert(node.0, node.1);

    // When
    let svg = generate_svg_string(&doc);

    // Then
    // ix = (200 - 32) / 2 = 84
    // iy = (100 - 32) / 2 - 5 = 34 - 5 = 29
    assert!(
        svg.contains("x='84"),
        "icon x should be (200 - 32) / 2 = 84"
    );
    assert!(
        svg.contains("y='29"),
        "icon y should be (100 - 32) / 2 - 5 = 29"
    );
    Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn given_node_with_icon_when_generate_svg_string_then_icon_size_is_32() -> Result<(), anyhow::Error>
{
    // Given
    let mut doc = create_empty_document();
    let mut node = create_node("n1", 0.0, 0.0, 100.0, 60.0, "Test");
    node.1.icon = String::from("aws/compute/ec2.png");
    doc.document.nodes.insert(node.0, node.1);

    // When
    let svg = generate_svg_string(&doc);

    // Then - icon should have width and height of 32
    assert!(svg.contains("width='32"), "icon width should be 32");
    assert!(svg.contains("height='32"), "icon height should be 32");
    Ok(())
}
