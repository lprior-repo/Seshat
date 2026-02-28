//! Subgraph save-reload stability tests
//!
//! These tests verify that subgraph data structures properly serialize and deserialize
//! while preserving the critical relationships and proportions.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use crate::models::document::{DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat};
use im::HashMap;

/// Helper to create a simple node
fn make_node(
    id: &str,
    kind: NodeKind,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    parent: Option<NodeId>,
) -> (NodeId, Node) {
    let node_id = NodeId::new(id.to_string());
    let is_subgraph = kind == NodeKind::Subgraph;
    let node = Node {
        kind,
        icon: String::new(),
        label: if is_subgraph {
            "Subgraph".to_string()
        } else {
            "Text".to_string()
        },
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(width),
        height: OrderedFloat(height),
        font_size: None,
        font_weight: None,
        locked: is_subgraph,
        parent,
        dag_rank: None,
        tags: Vec::new(),
        metadata: HashMap::new(),
        z_index: if is_subgraph { -1 } else { 1000 },
        style: Some(NodeStyle::Box),
        collapsed: Some(false),
    };
    (node_id, node)
}

/// Test that a simple subgraph with a child node round-trips correctly
#[test]
fn given_subgraph_with_child_when_serialized_and_deserialized_then_parent_preserved() {
    let mut doc = DiagramDocument::default();

    // Create parent subgraph
    let (outer_id, outer_node) = make_node(
        "outer",
        NodeKind::Subgraph,
        100.0,
        100.0,
        400.0,
        300.0,
        None,
    );
    doc.document.nodes.insert(outer_id.clone(), outer_node);

    // Create child node inside subgraph
    let (child_id, child_node) = make_node(
        "child",
        NodeKind::Text,
        150.0,
        150.0,
        80.0,
        30.0,
        Some(outer_id.clone()),
    );
    doc.document.nodes.insert(child_id.clone(), child_node);

    // Serialize
    let json = serde_json::to_string(&doc).expect("serialization should succeed");

    // Deserialize
    let loaded: DiagramDocument =
        serde_json::from_str(&json).expect("deserialization should succeed");

    // Verify parent relationship is preserved
    let loaded_child = loaded
        .document
        .nodes
        .get(&child_id)
        .expect("child node should exist");

    assert_eq!(
        loaded_child.parent.as_ref(),
        Some(&outer_id),
        "child node should preserve parent reference"
    );

    // Verify child's absolute position is preserved
    assert_eq!(
        loaded_child.x.0, 150.0,
        "child x position should be preserved"
    );
    assert_eq!(
        loaded_child.y.0, 150.0,
        "child y position should be preserved"
    );
}

/// Test nested subgraphs survive round-trip
#[test]
fn given_nested_subgraphs_when_serialized_and_deserialized_then_hierarchy_preserved() {
    let mut doc = DiagramDocument::default();

    // Create outer subgraph
    let (outer_id, outer_node) =
        make_node("outer", NodeKind::Subgraph, 80.0, 80.0, 760.0, 480.0, None);
    doc.document.nodes.insert(outer_id.clone(), outer_node);

    // Create inner subgraph inside outer
    let (inner_id, inner_node) = make_node(
        "inner",
        NodeKind::Subgraph,
        180.0,
        180.0,
        420.0,
        240.0,
        Some(outer_id.clone()),
    );
    doc.document.nodes.insert(inner_id.clone(), inner_node);

    // Create text nodes inside inner
    let (t1_id, t1_node) = make_node(
        "t1",
        NodeKind::Text,
        220.0,
        220.0,
        120.0,
        28.0,
        Some(inner_id.clone()),
    );
    doc.document.nodes.insert(t1_id.clone(), t1_node);

    let (t2_id, t2_node) = make_node(
        "t2",
        NodeKind::Text,
        480.0,
        320.0,
        120.0,
        28.0,
        Some(inner_id.clone()),
    );
    doc.document.nodes.insert(t2_id.clone(), t2_node);

    // Serialize and deserialize
    let json = serde_json::to_string(&doc).expect("serialization should succeed");
    let loaded: DiagramDocument =
        serde_json::from_str(&json).expect("deserialization should succeed");

    // Verify outer exists
    let loaded_outer = loaded
        .document
        .nodes
        .get(&outer_id)
        .expect("outer should exist");
    assert!(loaded_outer.parent.is_none(), "outer should have no parent");

    // Verify inner's parent is outer
    let loaded_inner = loaded
        .document
        .nodes
        .get(&inner_id)
        .expect("inner should exist");
    assert_eq!(
        loaded_inner.parent.as_ref(),
        Some(&outer_id),
        "inner's parent should be outer"
    );

    // Verify t1 and t2's parent is inner
    let loaded_t1 = loaded.document.nodes.get(&t1_id).expect("t1 should exist");
    assert_eq!(
        loaded_t1.parent.as_ref(),
        Some(&inner_id),
        "t1's parent should be inner"
    );

    let loaded_t2 = loaded.document.nodes.get(&t2_id).expect("t2 should exist");
    assert_eq!(
        loaded_t2.parent.as_ref(),
        Some(&inner_id),
        "t2's parent should be inner"
    );
}

/// Test that relative proportions are preserved after round-trip
#[test]
fn given_subgraph_with_child_when_roundtripped_then_relative_proportions_preserved() {
    let mut doc = DiagramDocument::default();

    let (outer_id, outer_node) = make_node(
        "outer",
        NodeKind::Subgraph,
        100.0,
        100.0,
        400.0,
        300.0,
        None,
    );
    doc.document.nodes.insert(outer_id.clone(), outer_node);

    // Child at 25% relative position within parent
    let (child_id, child_node) = make_node(
        "child",
        NodeKind::Text,
        200.0, // 100 + 400 * 0.25
        175.0, // 100 + 300 * 0.25
        80.0,
        30.0,
        Some(outer_id.clone()),
    );
    doc.document.nodes.insert(child_id.clone(), child_node);

    // Serialize and deserialize
    let json = serde_json::to_string(&doc).expect("serialization should succeed");
    let loaded: DiagramDocument =
        serde_json::from_str(&json).expect("deserialization should succeed");

    let loaded_outer = loaded
        .document
        .nodes
        .get(&outer_id)
        .expect("outer should exist");
    let loaded_child = loaded
        .document
        .nodes
        .get(&child_id)
        .expect("child should exist");

    // Calculate relative positions
    let rel_x = (loaded_child.x.0 - loaded_outer.x.0) / loaded_outer.width.0;
    let rel_y = (loaded_child.y.0 - loaded_outer.y.0) / loaded_outer.height.0;

    // Original relative positions were 0.25
    assert!(
        (rel_x - 0.25).abs() < 0.001,
        "relative x should be preserved: {}",
        rel_x
    );
    assert!(
        (rel_y - 0.25).abs() < 0.001,
        "relative y should be preserved: {}",
        rel_y
    );
}

/// Test that nested subgraph proportions are preserved
#[test]
fn given_nested_subgraphs_when_roundtripped_then_inner_outer_proportions_preserved() {
    let mut doc = DiagramDocument::default();

    // Outer: 80, 80, 760, 480
    let (outer_id, outer_node) =
        make_node("outer", NodeKind::Subgraph, 80.0, 80.0, 760.0, 480.0, None);
    doc.document.nodes.insert(outer_id.clone(), outer_node);

    // Inner: 180, 180, 420, 240 (approximately 25% inset)
    let (inner_id, inner_node) = make_node(
        "inner",
        NodeKind::Subgraph,
        180.0,
        180.0,
        420.0,
        240.0,
        Some(outer_id.clone()),
    );
    doc.document.nodes.insert(inner_id.clone(), inner_node);

    // Serialize and deserialize
    let json = serde_json::to_string(&doc).expect("serialization should succeed");
    let loaded: DiagramDocument =
        serde_json::from_str(&json).expect("deserialization should succeed");

    let loaded_outer = loaded
        .document
        .nodes
        .get(&outer_id)
        .expect("outer should exist");
    let loaded_inner = loaded
        .document
        .nodes
        .get(&inner_id)
        .expect("inner should exist");

    // Calculate proportions
    let width_ratio = loaded_inner.width.0 / loaded_outer.width.0;
    let height_ratio = loaded_inner.height.0 / loaded_outer.height.0;
    let x_offset_ratio = (loaded_inner.x.0 - loaded_outer.x.0) / loaded_outer.width.0;
    let y_offset_ratio = (loaded_inner.y.0 - loaded_outer.y.0) / loaded_outer.height.0;

    // Original proportions: 420/760 ≈ 0.5526, 240/480 = 0.5
    // Offsets: 100/760 ≈ 0.1316, 100/480 ≈ 0.2083
    assert!(
        (width_ratio - 0.5526).abs() < 0.01,
        "width ratio should be preserved: {}",
        width_ratio
    );
    assert!(
        (height_ratio - 0.5).abs() < 0.01,
        "height ratio should be preserved: {}",
        height_ratio
    );
    assert!(
        (x_offset_ratio - 0.1316).abs() < 0.01,
        "x offset ratio should be preserved: {}",
        x_offset_ratio
    );
    assert!(
        (y_offset_ratio - 0.2083).abs() < 0.01,
        "y offset ratio should be preserved: {}",
        y_offset_ratio
    );
}

/// Test the scene_nested_subgraph_v1.json format round-trips correctly
#[test]
fn given_scene_nested_subgraph_v1_json_when_parsed_then_document_valid() {
    let json = include_str!("../../e2e/scenes/scene_nested_subgraph_v1.json");

    let doc: DiagramDocument = serde_json::from_str(json).expect("should parse valid JSON");

    // Verify structure
    assert_eq!(doc.document.nodes.len(), 4, "should have 4 nodes");

    // Verify outer subgraph
    let outer = doc
        .document
        .nodes
        .get(&NodeId::new("outer".to_string()))
        .expect("outer should exist");
    assert_eq!(outer.kind, NodeKind::Subgraph);
    assert!(outer.parent.is_none(), "outer should have no parent");
    assert_eq!(outer.x.0, 80.0);
    assert_eq!(outer.y.0, 80.0);
    assert_eq!(outer.width.0, 760.0);
    assert_eq!(outer.height.0, 480.0);

    // Verify inner subgraph
    let inner = doc
        .document
        .nodes
        .get(&NodeId::new("inner".to_string()))
        .expect("inner should exist");
    assert_eq!(inner.kind, NodeKind::Subgraph);
    assert_eq!(
        inner.parent.as_ref(),
        Some(&NodeId::new("outer".to_string())),
        "inner should have outer as parent"
    );
    assert_eq!(inner.x.0, 180.0);
    assert_eq!(inner.y.0, 180.0);
    assert_eq!(inner.width.0, 420.0);
    assert_eq!(inner.height.0, 240.0);

    // Verify text nodes have inner as parent
    let t1 = doc
        .document
        .nodes
        .get(&NodeId::new("t1".to_string()))
        .expect("t1 should exist");
    assert_eq!(
        t1.parent.as_ref(),
        Some(&NodeId::new("inner".to_string())),
        "t1 should have inner as parent"
    );

    let t2 = doc
        .document
        .nodes
        .get(&NodeId::new("t2".to_string()))
        .expect("t2 should exist");
    assert_eq!(
        t2.parent.as_ref(),
        Some(&NodeId::new("inner".to_string())),
        "t2 should have inner as parent"
    );

    // Verify edge exists
    assert_eq!(doc.document.edges.len(), 1, "should have 1 edge");
}

/// Test that schema validation passes for valid nested subgraphs
#[test]
fn given_valid_nested_subgraph_document_when_validated_then_passes() {
    use crate::models::schema::validate_schema;

    let json = include_str!("../../e2e/scenes/scene_nested_subgraph_v1.json");
    let doc: DiagramDocument = serde_json::from_str(json).expect("should parse valid JSON");

    let result = validate_schema(&doc);
    assert!(
        result.is_ok(),
        "valid document should pass validation: {:?}",
        result
    );
}

/// Test that schema validation fails for node with non-subgraph parent
#[test]
fn given_node_with_non_subgraph_parent_when_validated_then_fails() {
    use crate::models::schema::validate_schema;

    let mut doc = DiagramDocument::default();

    // Create a "parent" node (not a subgraph)
    let (parent_id, parent_node) = make_node(
        "parent",
        NodeKind::Text, // Not a subgraph!
        100.0,
        100.0,
        200.0,
        150.0,
        None,
    );
    doc.document.nodes.insert(parent_id.clone(), parent_node);

    // Create child with non-subgraph parent
    let (child_id, child_node) = make_node(
        "child",
        NodeKind::Text,
        150.0,
        150.0,
        50.0,
        30.0,
        Some(parent_id),
    );
    doc.document.nodes.insert(child_id, child_node);

    let result = validate_schema(&doc);
    assert!(result.is_err(), "invalid document should fail validation");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("parent"),
        "error should mention parent: {}",
        err_msg
    );
}
