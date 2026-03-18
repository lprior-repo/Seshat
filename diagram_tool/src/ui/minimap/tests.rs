#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::models::{MinimapProjection, MinimapSnapshot, ProjectionKey};
use diagram_models::document::{
    ArrowType, DocumentData, Edge, EdgeId, EdgeStyle, LockState, Node, NodeId, NodeKind, NodeStyle,
    OrderedFloat, Revision,
};
use im::HashMap;

fn make_node(id: &str, x: f64, y: f64, width: f64, height: f64) -> (NodeId, Node) {
    (
        NodeId::new(id.to_string()),
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
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
            style: Some(NodeStyle::default()),
            collapsed: None,
        },
    )
}

fn make_document(nodes: Vec<(NodeId, Node)>) -> DocumentData {
    DocumentData {
        nodes: nodes.into_iter().collect(),
        edges: HashMap::new(),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_degenerate_world_bounds_when_projecting_then_projection_is_finite() {
    let tiny_node = make_node("tiny", 0.0, 0.0, 0.001, 0.001);
    let doc = make_document(vec![tiny_node]);

    let snapshot = MinimapSnapshot::from_document(&doc);
    assert!(snapshot.is_some());

    let snap = snapshot.unwrap();
    let projection = snap.project(snap.min_x, snap.min_y, 0.01);

    for (_, x, y, w, h, _) in projection.node_rects.iter() {
        assert!(x.is_finite(), "x should be finite");
        assert!(y.is_finite(), "y should be finite");
        assert!(w.is_finite(), "width should be finite");
        assert!(h.is_finite(), "height should be finite");
        assert!(*w >= 2.0, "width should be >= 2.0 after floor");
        assert!(*h >= 2.0, "height should be >= 2.0 after floor");
    }

    for (sx, sy, tx, ty) in projection.edge_segments.iter() {
        assert!(sx.is_finite() && sy.is_finite() && tx.is_finite() && ty.is_finite());
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_near_zero_node_sizes_when_snapshot_then_bounds_remain_finite() {
    let nodes = vec![
        make_node("a", 0.0, 0.0, 1e-10, 1e-10),
        make_node("b", 100.0, 100.0, 1e-10, 1e-10),
    ];
    let doc = make_document(nodes);

    let snapshot = MinimapSnapshot::from_document(&doc);
    assert!(snapshot.is_some());

    let snap = snapshot.unwrap();
    assert!(snap.min_x.is_finite());
    assert!(snap.min_y.is_finite());
    assert!(snap.max_x.is_finite());
    assert!(snap.max_y.is_finite());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_overlapping_nodes_when_projecting_then_no_panic() {
    let nodes = vec![
        make_node("a", 50.0, 50.0, 100.0, 100.0),
        make_node("b", 50.0, 50.0, 100.0, 100.0),
        make_node("c", 75.0, 75.0, 50.0, 50.0),
    ];
    let doc = make_document(nodes);

    let snapshot = MinimapSnapshot::from_document(&doc);
    assert!(snapshot.is_some());

    let snap = snapshot.unwrap();
    let projection = snap.project(snap.min_x, snap.min_y, 1.0);

    assert_eq!(projection.node_rects.len(), 3);
    for (_, x, y, w, h, _) in projection.node_rects.iter() {
        assert!(x.is_finite());
        assert!(y.is_finite());
        assert!(w.is_finite());
        assert!(h.is_finite());
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_nan_node_geometry_when_snapshot_built_then_invalid_geometry_is_sanitized_or_ignored() {
    let valid_node = make_node("valid", 100.0, 100.0, 64.0, 64.0);
    let nan_node = make_node("nan", f64::NAN, f64::NAN, f64::NAN, f64::NAN);

    let nodes = vec![valid_node, nan_node];
    let doc = make_document(nodes);

    let snapshot = MinimapSnapshot::from_document(&doc);
    assert!(snapshot.is_some());

    let snap = snapshot.unwrap();

    assert!(snap.min_x.is_finite() || snap.min_x.is_infinite());
    assert!(snap.min_y.is_finite() || snap.min_y.is_infinite());

    if snap.min_x.is_finite()
        && snap.min_y.is_finite()
        && snap.max_x.is_finite()
        && snap.max_y.is_finite()
    {
        let projection = snap.project(snap.min_x, snap.min_y, 1.0);

        for (_, x, y, w, h, _) in projection.node_rects.iter() {
            assert!(x.is_finite(), "projected x should be finite");
            assert!(y.is_finite(), "projected y should be finite");
            assert!(w.is_finite(), "projected width should be finite");
            assert!(h.is_finite(), "projected height should be finite");
        }
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_inf_node_geometry_when_snapshot_built_then_handles_gracefully() {
    let valid_node = make_node("valid", 100.0, 100.0, 64.0, 64.0);
    let inf_node = make_node(
        "inf",
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::INFINITY,
    );

    let nodes = vec![valid_node, inf_node];
    let doc = make_document(nodes);

    let snapshot = MinimapSnapshot::from_document(&doc);
    assert!(snapshot.is_some());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_single_node_when_projecting_then_bounds_are_correct() {
    let node = make_node("single", 200.0, 150.0, 100.0, 80.0);
    let doc = make_document(vec![node]);

    let snapshot = MinimapSnapshot::from_document(&doc);
    assert!(snapshot.is_some());

    let snap = snapshot.unwrap();
    assert_eq!(snap.min_x, 200.0);
    assert_eq!(snap.min_y, 150.0);
    assert_eq!(snap.max_x, 300.0);
    assert_eq!(snap.max_y, 230.0);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_empty_document_when_snapshot_then_returns_none() {
    let doc = DocumentData {
        nodes: HashMap::new(),
        edges: HashMap::new(),
    };

    let snapshot = MinimapSnapshot::from_document(&doc);
    assert!(snapshot.is_none());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_nodes_with_edges_when_snapshot_then_edge_segments_included() {
    let node_a = make_node("a", 0.0, 0.0, 100.0, 50.0);
    let node_b = make_node("b", 200.0, 0.0, 100.0, 50.0);

    let edge = Edge {
        source: NodeId::new("a".to_string()),
        target: NodeId::new("b".to_string()),
        label: String::new(),
        style: EdgeStyle::default(),
        arrow_type: ArrowType::default(),
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
    };

    let mut doc = make_document(vec![node_a, node_b]);
    doc.edges = doc.edges.update(EdgeId::new("e1".to_string()), edge);

    let snapshot = MinimapSnapshot::from_document(&doc);
    assert!(snapshot.is_some());

    let snap = snapshot.unwrap();
    assert_eq!(snap.edge_segments.len(), 1);

    let (sx, sy, tx, ty) = snap.edge_segments[0];
    assert!(sx.is_finite());
    assert!(sy.is_finite());
    assert!(tx.is_finite());
    assert!(ty.is_finite());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_very_large_scale_when_projecting_then_sizes_floored_to_minimum() {
    let node = make_node("big", 0.0, 0.0, 1000.0, 1000.0);
    let doc = make_document(vec![node]);

    let snapshot = MinimapSnapshot::from_document(&doc);
    let snap = snapshot.unwrap();

    let tiny_scale = 1e-10;
    let projection = snap.project(snap.min_x, snap.min_y, tiny_scale);

    for (_, _, _, w, h, _) in projection.node_rects.iter() {
        assert!(*w >= 2.0);
        assert!(*h >= 2.0);
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_projection_key_when_state_identical_then_keys_equal() {
    let rev = Revision::INITIAL;
    let key1 = ProjectionKey::from_state(rev, 100.0, 200.0, 0.5);
    let key2 = ProjectionKey::from_state(rev, 100.0, 200.0, 0.5);

    assert_eq!(key1, key2);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_projection_key_when_state_differs_then_keys_differ() {
    let rev = Revision::INITIAL;
    let key1 = ProjectionKey::from_state(rev, 100.0, 200.0, 0.5);
    let key2 = ProjectionKey::from_state(rev, 100.0, 200.0, 0.6);

    assert_ne!(key1, key2);
}
