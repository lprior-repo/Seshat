#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use super::*;
use crate::test_utils::builders::edge::EdgeBuilder;
use crate::test_utils::builders::node::NodeBuilder;
use crate::ui::grid::GridSize;
use diagram_models::document::{
    DiagramDocument, EdgeId, LockState, NodeId, NodeKind, OrderedFloat,
};

fn create_test_doc() -> DiagramDocument {
    DiagramDocument::default()
}

#[test]
fn test_provider_color() {
    assert_eq!(provider_color("aws"), "#FF9900");
    assert_eq!(provider_color("gcp"), "#4285F4");
    assert_eq!(provider_color("azure"), "#0078D4");
    assert_eq!(provider_color("k8s"), "#326CE5");
    assert_eq!(provider_color("unknown"), "#6B7280");
}

#[test]
fn test_initials() {
    assert_eq!(initials("Amazon Web Services"), "AWS");
    assert_eq!(initials("Google/Cloud/Platform"), "GCP");
    assert_eq!(initials("SingleWord"), "SIN");
    assert_eq!(initials("A/B/C"), "ABC");
    assert_eq!(initials(""), "");
}

#[test]
fn test_icon_tags() {
    assert_eq!(icon_tags(""), vec!["".to_string()]);
    assert_eq!(icon_tags("aws"), vec!["aws".to_string()]);
    assert_eq!(
        icon_tags("aws/compute/ec2"),
        vec!["aws".to_string(), "compute".to_string()]
    );
}

#[test]
fn test_fallback_icon_label() {
    assert_eq!(fallback_icon_label("aws/compute/ec2"), "Ec2");
    assert_eq!(fallback_icon_label("node"), "Node");
    assert_eq!(fallback_icon_label(""), "");
}

#[test]
fn test_ordered_node_ids() {
    let mut doc = create_test_doc();
    let n1 = NodeId::new("n1".to_string());
    let n2 = NodeId::new("n2".to_string());
    let sub1 = NodeId::new("sub1".to_string());

    doc.document.nodes.insert(
        n1.clone(),
        NodeBuilder::new(0.0, 0.0, 10.0, 10.0)
            .with_z_index(1)
            .build(),
    );
    doc.document.nodes.insert(
        n2.clone(),
        NodeBuilder::new(0.0, 0.0, 10.0, 10.0)
            .with_z_index(2)
            .build(),
    );
    doc.document.nodes.insert(
        sub1.clone(),
        NodeBuilder::new(0.0, 0.0, 10.0, 10.0)
            .with_kind(NodeKind::Subgraph)
            .with_z_index(10)
            .build(),
    );

    let ordered = ordered_node_ids(&doc);
    assert_eq!(ordered, vec![sub1.clone(), n1.clone(), n2.clone()]);
}

#[test]
fn test_find_node_at_hit_margin() {
    let mut doc = create_test_doc();
    doc.editor_state.zoom = OrderedFloat::new_unchecked(1.0); // Margin = 5.0

    let n1 = NodeId::new("n1".to_string());
    doc.document.nodes.insert(
        n1.clone(),
        NodeBuilder::new(10.0, 10.0, 100.0, 100.0).build(),
    );

    // Inside bounds
    assert_eq!(find_node_at(&doc, 50.0, 50.0), Some(n1.clone()));

    // Top-left within margin (5.0 margin allows hits down to 5.0)
    assert_eq!(find_node_at(&doc, 7.0, 7.0), Some(n1.clone()));
    assert_eq!(find_node_at(&doc, 4.0, 4.0), None);

    // Bottom-right within margin (down to 115.0)
    assert_eq!(find_node_at(&doc, 112.0, 112.0), Some(n1.clone()));
    assert_eq!(find_node_at(&doc, 116.0, 116.0), None);
}

#[test]
fn test_find_node_at_z_order() {
    let mut doc = create_test_doc();
    doc.editor_state.zoom = OrderedFloat::new_unchecked(1.0);

    let n1 = NodeId::new("n1".to_string());
    let n2 = NodeId::new("n2".to_string());

    doc.document.nodes.insert(
        n1.clone(),
        NodeBuilder::new(10.0, 10.0, 100.0, 100.0)
            .with_z_index(1)
            .build(),
    );
    doc.document.nodes.insert(
        n2.clone(),
        NodeBuilder::new(10.0, 10.0, 100.0, 100.0)
            .with_z_index(2)
            .build(),
    );

    // Should return node with higher z-index (n2)
    assert_eq!(find_node_at(&doc, 50.0, 50.0), Some(n2.clone()));
}

#[test]
fn test_edge_preserves_dag() {
    let mut doc = create_test_doc();
    let n1 = NodeId::new("n1".to_string());
    let n2 = NodeId::new("n2".to_string());
    let n3 = NodeId::new("n3".to_string());

    doc.document
        .nodes
        .insert(n1.clone(), NodeBuilder::new(0.0, 0.0, 10.0, 10.0).build());
    doc.document
        .nodes
        .insert(n2.clone(), NodeBuilder::new(0.0, 0.0, 10.0, 10.0).build());
    doc.document
        .nodes
        .insert(n3.clone(), NodeBuilder::new(0.0, 0.0, 10.0, 10.0).build());

    let edge1 = EdgeBuilder::new(n1.clone(), n2.clone()).build();
    doc.document
        .edges
        .insert(EdgeId::new("e1".to_string()), edge1);

    // Valid edge (n2 -> n3)
    let edge2 = EdgeBuilder::new(n2.clone(), n3.clone()).build();
    assert!(edge_preserves_dag(&doc, &edge2));

    // Cycle edge (n2 -> n1)
    let cycle_edge = EdgeBuilder::new(n2.clone(), n1.clone()).build();
    assert!(!edge_preserves_dag(&doc, &cycle_edge));
}

#[test]
fn test_subgraph_release_bounds() {
    let grid = GridSize::new(20.0).expect("Valid grid size");

    // Valid snap
    let bounds = subgraph_release_bounds((0.0, 0.0), (50.0, 50.0), true, grid);
    assert_eq!(bounds, Some((0.0, 0.0, 60.0, 60.0))); // Snaps up to nearest 20

    // No snap
    let bounds_no_snap = subgraph_release_bounds((0.0, 0.0), (50.0, 50.0), false, grid);
    assert_eq!(bounds_no_snap, Some((0.0, 0.0, 50.0, 50.0)));

    // Too small bounds
    let bounds_small = subgraph_release_bounds((0.0, 0.0), (10.0, 10.0), false, grid);
    assert_eq!(bounds_small, None);
}

#[test]
fn test_safe_zoom() {
    // Tests behavior when math library evaluates
    let valid_zoom = safe_zoom(1.5);
    assert!(valid_zoom > 0.0);
}

#[test]
fn test_fit_icon_side() {
    assert_eq!(fit_icon_side(f64::NAN), 0.0);
    // test the clamp logic inside queries.rs
    assert!(fit_icon_side(10.0) >= 0.0);
}

// --- Icon URL construction tests ---

fn make_node_with_metadata(metadata: im::HashMap<String, Value>) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: "test".to_string(),
        x: OrderedFloat::new_unchecked(0.0),
        y: OrderedFloat::new_unchecked(0.0),
        width: OrderedFloat::new_unchecked(100.0),
        height: OrderedFloat::new_unchecked(100.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata,
        z_index: 0,
        style: None,
        collapsed: None,
    }
}

#[test]
fn icon_url_for_relpath_formats_path() {
    assert_eq!(
        icon_url_for_relpath("gcp/compute/gke.png"),
        "/assets/resources/gcp/compute/gke.png"
    );
}

#[test]
fn icon_url_for_relpath_empty_path() {
    assert_eq!(icon_url_for_relpath(""), "/assets/resources/");
}

#[test]
fn icon_url_for_relpath_blocks_path_traversal() {
    assert_eq!(icon_url_for_relpath("../../etc/passwd"), "");
}

#[test]
fn icon_url_for_relpath_blocks_nested_traversal() {
    assert_eq!(icon_url_for_relpath("aws/../../etc/passwd"), "");
}

#[test]
fn icon_url_returns_none_for_traversal() {
    assert_eq!(icon_url("../../etc/passwd"), None);
}

#[test]
fn node_image_url_from_metadata() {
    let metadata = im::HashMap::from(vec![(
        "icon_url".to_string(),
        Value::String("/assets/resources/aws/ec2.png".to_string()),
    )]);
    let node = make_node_with_metadata(metadata);
    assert_eq!(
        node_image_url(&node),
        Some("/assets/resources/aws/ec2.png".to_string())
    );
}

#[test]
fn node_image_url_ignores_old_icon_data_url_key() {
    // Old documents stored base64 data URLs in "icon_data_url" metadata.
    // After migration, the key is remapped to "icon_url" by persistence_compat.
    // If NOT migrated (e.g. direct metadata access), the function must ignore
    // the old key and fall back to icon_url(&node.icon).
    let metadata = im::HashMap::from(vec![(
        "icon_data_url".to_string(),
        Value::String("data:image/png;base64,abc".to_string()),
    )]);
    let node = make_node_with_metadata(metadata);
    let result = node_image_url(&node);
    // Must NOT return the old base64 data URL value
    assert!(
        result.as_deref() != Some("data:image/png;base64,abc"),
        "Legacy icon_data_url must NOT be returned as-is; got: {result:?}"
    );
    // Must fall back to icon_url(&node.icon), which returns a /assets/resources/ URL
    // (or None if the icon key is not in the index and has no valid relpath)
    match result {
        Some(url) => assert!(
            url.starts_with("/assets/resources/"),
            "Fallback must be an /assets/resources/ URL, got: {url}"
        ),
        None => {} // Valid: unknown icon key with no index entry
    }
}
