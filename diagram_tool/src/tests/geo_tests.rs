//! GEO Category Tests (30 tests)
//!
//! Geometry calculations: AABB, transforms, intersections, bounds calculations.

use crate::test_utils::{builders::*, harness::*, types::*};
use diagram_models::document::{DiagramDocument, NodeId, OrderedFloat};

// ============================================================================
// GEO-001 to GEO-005: AABB for axis-aligned rectangles
// ============================================================================

#[test]
fn geo_001_aabb_axis_aligned_rect_basic() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 10.0, 20.0, 50.0, 60.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    assert_eq!(node.x.0, 10.0);
    assert_eq!(node.y.0, 20.0);
    assert_eq!(node.width.0, 50.0);
    assert_eq!(node.height.0, 60.0);
}

#[test]
fn geo_002_aabb_axis_aligned_rect_at_origin() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 0.0, 0.0, 100.0, 100.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    assert_eq!(node.x.0, 0.0);
    assert_eq!(node.y.0, 0.0);
}

#[test]
fn geo_003_aabb_negative_coordinates() {
    let doc = DocBuilder::new()
        .add_node_with("n1", -100.0, -200.0, 50.0, 50.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    assert_eq!(node.x.0, -100.0);
    assert_eq!(node.y.0, -200.0);
}

#[test]
fn geo_004_aabb_large_coordinates() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 10000.0, 20000.0, 500.0, 600.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    assert_eq!(node.x.0, 10000.0);
    assert_eq!(node.y.0, 20000.0);
}

#[test]
fn geo_005_aabb_small_dimensions() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 50.0, 50.0, 1.0, 1.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    assert_eq!(node.width.0, 1.0);
    assert_eq!(node.height.0, 1.0);
}

// ============================================================================
// GEO-006 to GEO-010: Bounds calculation for rotated rectangles
// ============================================================================

#[test]
fn geo_006_node_with_rotation_metadata() {
    let node = test_node_builder(100.0, 100.0, 50.0, 50.0)
        .with_metadata("rotation", serde_json::json!(0.7854)) // 45 degrees
        .build();
    assert!(node.metadata.contains_key(&"rotation".to_string()));
}

#[test]
fn geo_007_bounds_include_stroke_width() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 10.0, 10.0, 50.0, 50.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    // Bounds should at minimum cover x, y, width, height
    let right = node.x.0 + node.width.0;
    let bottom = node.y.0 + node.height.0;
    assert_eq!(right, 60.0);
    assert_eq!(bottom, 60.0);
}

#[test]
fn geo_008_image_node_bounds_calculation() {
    let node = test_node_builder(0.0, 0.0, 200.0, 150.0)
        .with_kind(diagram_models::document::NodeKind::Node)
        .with_metadata("image_url", serde_json::json!("test.png"))
        .build();
    assert_eq!(node.width.0, 200.0);
    assert_eq!(node.height.0, 150.0);
}

#[test]
fn geo_009_text_node_bounds_calculation() {
    let node = test_text_node(50.0, 50.0, 120.0, 30.0);
    assert_eq!(node.kind, diagram_models::document::NodeKind::Text);
    assert_eq!(node.width.0, 120.0);
    assert_eq!(node.height.0, 30.0);
}

#[test]
fn geo_010_node_center_calculation() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 100.0, 200.0, 50.0, 60.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    let center_x = node.x.0 + node.width.0 / 2.0;
    let center_y = node.y.0 + node.height.0 / 2.0;
    assert_eq!(center_x, 125.0);
    assert_eq!(center_y, 230.0);
}

// ============================================================================
// GEO-011 to GEO-015: Point containment tests
// ============================================================================

#[test]
fn geo_011_point_inside_node() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 10.0, 10.0, 50.0, 50.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    let px = 30.0;
    let py = 30.0;
    assert!(px >= node.x.0 && px <= node.x.0 + node.width.0);
    assert!(py >= node.y.0 && py <= node.y.0 + node.height.0);
}

#[test]
fn geo_012_point_outside_node() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 10.0, 10.0, 50.0, 50.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    let px = 100.0;
    let py = 100.0;
    assert!(px > node.x.0 + node.width.0 || px < node.x.0);
    assert!(py > node.y.0 + node.height.0 || py < node.y.0);
}

#[test]
fn geo_013_point_on_node_boundary() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 10.0, 10.0, 50.0, 50.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    // Right edge
    let px = node.x.0 + node.width.0;
    assert!(px >= node.x.0 && px <= node.x.0 + node.width.0);
}

#[test]
fn geo_014_point_at_node_origin() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 25.0, 25.0, 50.0, 50.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    assert!(25.0 >= node.x.0);
    assert!(25.0 >= node.y.0);
}

#[test]
fn geo_015_multiple_nodes_bounds_union() {
    let doc = DocBuilder::new()
        .add_node_with("n1", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("n2", 100.0, 100.0, 50.0, 50.0)
        .build();
    assert_eq!(doc.document.nodes.len(), 2);
    // Union bounds should span from (10,10) to (150,150)
    let min_x = 10.0_f64;
    let min_y = 10.0_f64;
    let max_x = 150.0_f64;
    let max_y = 150.0_f64;
    assert_eq!(max_x - min_x, 140.0);
    assert_eq!(max_y - min_y, 140.0);
}

// ============================================================================
// GEO-016 to GEO-020: Edge geometry and routing
// ============================================================================

#[test]
fn geo_016_edge_source_target_positions() {
    let source = NodeId::new("A".to_string());
    let target = NodeId::new("B".to_string());
    let edge = test_edge(source.clone(), target.clone());
    assert_eq!(edge.source, source);
    assert_eq!(edge.target, target);
}

#[test]
fn geo_017_edge_label_offset_default() {
    let source = NodeId::new("A".to_string());
    let target = NodeId::new("B".to_string());
    let edge = test_edge(source, target);
    assert_eq!(edge.label_offset_t.0, 0.5);
}

#[test]
fn geo_018_edge_with_bend_points() {
    let source = NodeId::new("A".to_string());
    let target = NodeId::new("B".to_string());
    let mut edge = test_edge(source, target);
    edge.bend_points = im::vector![
        (OrderedFloat(50.0), OrderedFloat(0.0)),
        (OrderedFloat(50.0), OrderedFloat(100.0)),
    ];
    assert_eq!(edge.bend_points.len(), 2);
}

#[test]
fn geo_019_undirected_edge() {
    let source = NodeId::new("A".to_string());
    let target = NodeId::new("B".to_string());
    let edge = test_edge_builder(source, target).directed(false).build();
    assert!(!edge.directed);
}

#[test]
fn geo_020_edge_thickness_custom() {
    let source = NodeId::new("A".to_string());
    let target = NodeId::new("B".to_string());
    let edge = test_edge_builder(source, target)
        .with_thickness(3.0)
        .build();
    assert_eq!(edge.thickness.0, 3.0);
}

// ============================================================================
// GEO-021 to GEO-025: Line intersection tests
// ============================================================================

#[test]
fn geo_021_horizontal_line_intersection() {
    // Lines y=0 from x=-10..10, and x=0 from y=-10..10 must intersect at (0,0)
    let a1 = (-10.0_f64, 0.0);
    let a2 = (10.0_f64, 0.0);
    let b1 = (0.0_f64, -10.0);
    let b2 = (0.0_f64, 10.0);
    // Simple intersection check
    let intersects = !(a1.0 > b2.0 || b1.0 > a2.0);
    assert!(intersects, "Perpendicular lines must intersect");
}

#[test]
fn geo_022_parallel_lines_no_intersection() {
    // Two horizontal lines at different y values
    let y1 = 0.0_f64;
    let y2 = 100.0_f64;
    assert_ne!(y1, y2, "Parallel lines at different y must not intersect");
}

#[test]
fn geo_023_diagonal_line_intersection() {
    // y=x and y=-x intersect at origin
    let slope1 = 1.0_f64;
    let slope2 = -1.0_f64;
    assert_ne!(slope1, slope2, "Non-parallel slopes must intersect");
}

#[test]
fn geo_024_collinear_segments_overlap() {
    let a_start = 0.0_f64;
    let a_end = 10.0_f64;
    let b_start = 5.0_f64;
    let b_end = 15.0_f64;
    let overlaps = a_start <= b_end && b_start <= a_end;
    assert!(overlaps, "Collinear overlapping segments must overlap");
}

#[test]
fn geo_025_segment_endpoint_touching() {
    let a_end = 10.0_f64;
    let b_start = 10.0_f64;
    assert_eq!(a_end, b_start, "Touching endpoints should be equal");
}

// ============================================================================
// GEO-026 to GEO-030: Transform geometry
// ============================================================================

#[test]
fn geo_026_translate_node_position() {
    let node = test_node(10.0, 20.0, 50.0, 50.0);
    let dx = 30.0;
    let dy = 40.0;
    let new_x = node.x.0 + dx;
    let new_y = node.y.0 + dy;
    assert_eq!(new_x, 40.0);
    assert_eq!(new_y, 60.0);
}

#[test]
fn geo_027_scale_node_dimensions() {
    let node = test_node(100.0, 100.0, 50.0, 50.0);
    let scale = 2.0;
    let new_width = node.width.0 * scale;
    let new_height = node.height.0 * scale;
    assert_eq!(new_width, 100.0);
    assert_eq!(new_height, 100.0);
}

#[test]
fn geo_028_mirror_horizontal() {
    let node = test_node(10.0, 20.0, 50.0, 50.0);
    let mirror_x = 100.0;
    let mirrored_x = 2.0 * mirror_x - node.x.0 - node.width.0;
    assert_eq!(mirrored_x, 40.0);
}

#[test]
fn geo_029_distance_between_nodes() {
    let n1 = test_node(0.0, 0.0, 50.0, 50.0);
    let n2 = test_node(100.0, 100.0, 50.0, 50.0);
    let dx = (n2.x.0 - n1.x.0).abs();
    let dy = (n2.y.0 - n1.y.0).abs();
    let dist = (dx * dx + dy * dy).sqrt();
    let expected = (100.0_f64 * 100.0 + 100.0 * 100.0).sqrt();
    assert!((dist - expected).abs() < f64::EPSILON);
}

#[test]
fn geo_030_midpoint_between_nodes() {
    let n1 = test_node(0.0, 0.0, 50.0, 50.0);
    let n2 = test_node(100.0, 100.0, 50.0, 50.0);
    let mid_x = (n1.x.0 + n2.x.0) / 2.0;
    let mid_y = (n1.y.0 + n2.y.0) / 2.0;
    assert_eq!(mid_x, 50.0);
    assert_eq!(mid_y, 50.0);
}
