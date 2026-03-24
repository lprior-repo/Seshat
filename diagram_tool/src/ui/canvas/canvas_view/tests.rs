#![allow(clippy::unwrap_used, clippy::panic, clippy::module_inception, clippy::let_unit_value, clippy::redundant_pattern_matching, unused_variables, unused_imports)]
use im::HashMap;
use proptest::prelude::*;

use super::geometry::quadratic_control;
use super::{dist_to_segment, find_edge_at, quadratic_bezier_point};
use diagram_models::document::{
    ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EdgeStyle, LockState, Node, NodeId,
    NodeKind, NodeStyle, OrderedFloat,
};

fn node_at(x: f64, y: f64) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(10.0),
        height: OrderedFloat(10.0),
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
    }
}

fn edge(source: NodeId, target: NodeId) -> Edge {
    Edge {
        source,
        target,
        label: String::new(),
        style: EdgeStyle::Solid,
        arrow_type: ArrowType::Straight,
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
    }
}

#[cfg(kani)]
#[kani::proof]
fn given_low_zoom_when_clicking_near_edge_then_hit_test_uses_screen_consistent_radius() {
    let source_id = NodeId::new(String::from("source"));
    let target_id = NodeId::new(String::from("target"));
    let edge_id = EdgeId::new(String::from("e1"));

    let mut doc = DiagramDocument {
        document: DocumentData {
            nodes: HashMap::new()
                .update(source_id.clone(), node_at(0.0, 0.0))
                .update(target_id.clone(), node_at(100.0, 0.0)),
            edges: HashMap::new().update(edge_id.clone(), edge(source_id, target_id)),
        },
        ..DiagramDocument::default()
    };
    doc.editor_state.zoom = OrderedFloat(0.5);

    assert_eq!(find_edge_at(&doc, 50.0, 17.0), Some(edge_id));
}

#[cfg(kani)]
#[kani::proof]
fn given_high_zoom_when_clicking_same_world_distance_then_hit_test_is_tighter() {
    let source_id = NodeId::new(String::from("source"));
    let target_id = NodeId::new(String::from("target"));

    let mut doc = DiagramDocument {
        document: DocumentData {
            nodes: HashMap::new()
                .update(source_id.clone(), node_at(0.0, 0.0))
                .update(target_id.clone(), node_at(100.0, 0.0)),
            edges: HashMap::new()
                .update(EdgeId::new(String::from("e1")), edge(source_id, target_id)),
        },
        ..DiagramDocument::default()
    };
    doc.editor_state.zoom = OrderedFloat(2.0);

    assert!(find_edge_at(&doc, 50.0, 17.0).is_none());
}

#[cfg(kani)]
#[kani::proof]
fn given_overlapping_edges_when_hit_distance_ties_then_selection_is_stable_by_edge_id() {
    let source_id = NodeId::new(String::from("source"));
    let target_id = NodeId::new(String::from("target"));
    let edge_a = EdgeId::new(String::from("edge-a"));
    let edge_b = EdgeId::new(String::from("edge-b"));

    let doc = DiagramDocument {
        document: DocumentData {
            nodes: HashMap::new()
                .update(source_id.clone(), node_at(0.0, 0.0))
                .update(target_id.clone(), node_at(100.0, 0.0)),
            edges: HashMap::new()
                .update(edge_b.clone(), edge(source_id.clone(), target_id.clone()))
                .update(edge_a.clone(), edge(source_id, target_id)),
        },
        ..DiagramDocument::default()
    };

    assert_eq!(find_edge_at(&doc, 50.0, 5.0), Some(edge_a));
}

#[cfg(kani)]
#[kani::proof]
fn given_click_near_arrow_endpoint_when_within_endpoint_radius_then_edge_is_hit() {
    let source_id = NodeId::new(String::from("source"));
    let target_id = NodeId::new(String::from("target"));
    let edge_id = EdgeId::new(String::from("e1"));

    let doc = DiagramDocument {
        document: DocumentData {
            nodes: HashMap::new()
                .update(source_id.clone(), node_at(0.0, 0.0))
                .update(target_id.clone(), node_at(100.0, 0.0)),
            edges: HashMap::new().update(edge_id.clone(), edge(source_id, target_id)),
        },
        ..DiagramDocument::default()
    };

    assert_eq!(find_edge_at(&doc, 109.0, 12.0), Some(edge_id));
}

#[cfg(kani)]
#[kani::proof]
fn given_thin_vertical_edge_when_clicking_near_segment_then_hit_is_stable_across_zooms() {
    let source_id = NodeId::new(String::from("source"));
    let target_id = NodeId::new(String::from("target"));
    let edge_id = EdgeId::new(String::from("e1"));

    let mut doc = DiagramDocument {
        document: DocumentData {
            nodes: HashMap::new()
                .update(source_id.clone(), node_at(40.0, 0.0))
                .update(target_id.clone(), node_at(40.0, 120.0)),
            edges: HashMap::new().update(edge_id.clone(), edge(source_id, target_id)),
        },
        ..DiagramDocument::default()
    };

    for zoom in [0.5_f64, 1.0_f64, 2.0_f64, 3.0_f64] {
        doc.editor_state.zoom = OrderedFloat(zoom);
        assert_eq!(find_edge_at(&doc, 47.0, 65.0), Some(edge_id.clone()));
    }
}

#[cfg(kani)]
#[kani::proof]
fn given_endpoint_tie_when_clicking_shared_target_then_selection_is_stable_by_edge_id() {
    let source_a = NodeId::new(String::from("source-a"));
    let source_b = NodeId::new(String::from("source-b"));
    let target = NodeId::new(String::from("target"));
    let edge_a = EdgeId::new(String::from("edge-a"));
    let edge_b = EdgeId::new(String::from("edge-b"));

    let doc = DiagramDocument {
        document: DocumentData {
            nodes: HashMap::new()
                .update(source_a.clone(), node_at(0.0, 0.0))
                .update(source_b.clone(), node_at(0.0, 100.0))
                .update(target.clone(), node_at(100.0, 0.0)),
            edges: HashMap::new()
                .update(edge_b, edge(source_b, target.clone()))
                .update(edge_a.clone(), edge(source_a, target)),
        },
        ..DiagramDocument::default()
    };

    assert_eq!(find_edge_at(&doc, 105.0, 5.0), Some(edge_a));
}

fn finite_f64() -> impl Strategy<Value = f64> {
    -1000.0_f64..=1000.0_f64
}

proptest! {
    #[cfg(kani)]
#[kani::proof]
    fn quadratic_bezier_point_returns_finite_for_finite_inputs(
        p0x in finite_f64(), p0y in finite_f64(),
        p1x in finite_f64(), p1y in finite_f64(),
        p2x in finite_f64(), p2y in finite_f64(),
        t in 0.0_f64..=1.0_f64,
    ) {
        let (x, y) = quadratic_bezier_point((p0x, p0y), (p1x, p1y), (p2x, p2y), t);
        prop_assert!(x.is_finite());
        prop_assert!(y.is_finite());
    }

    #[cfg(kani)]
#[kani::proof]
    fn quadratic_bezier_point_t_zero_returns_p0(
        p0x in finite_f64(), p0y in finite_f64(),
        p1x in finite_f64(), p1y in finite_f64(),
        p2x in finite_f64(), p2y in finite_f64(),
    ) {
        let (x, y) = quadratic_bezier_point((p0x, p0y), (p1x, p1y), (p2x, p2y), 0.0);
        prop_assert!((x - p0x).abs() < 1e-10);
        prop_assert!((y - p0y).abs() < 1e-10);
    }

    #[cfg(kani)]
#[kani::proof]
    fn quadratic_bezier_point_t_one_returns_p2(
        p0x in finite_f64(), p0y in finite_f64(),
        p1x in finite_f64(), p1y in finite_f64(),
        p2x in finite_f64(), p2y in finite_f64(),
    ) {
        let (x, y) = quadratic_bezier_point((p0x, p0y), (p1x, p1y), (p2x, p2y), 1.0);
        prop_assert!((x - p2x).abs() < 1e-10);
        prop_assert!((y - p2y).abs() < 1e-10);
    }

    #[cfg(kani)]
#[kani::proof]
    fn quadratic_control_returns_finite_for_finite_input(
        sx in finite_f64(), sy in finite_f64(),
        tx in finite_f64(), ty in finite_f64(),
    ) {
        let (cx, cy) = quadratic_control(sx, sy, tx, ty);
        prop_assert!(cx.is_finite());
        prop_assert!(cy.is_finite());
    }

    #[cfg(kani)]
#[kani::proof]
    fn quadratic_control_lies_on_perpendicular_through_midpoint(
        sx in finite_f64(), sy in finite_f64(),
        tx in finite_f64(), ty in finite_f64(),
    ) {
        let (cx, cy) = quadratic_control(sx, sy, tx, ty);
        let mx = f64::midpoint(sx, tx);
        let my = f64::midpoint(sy, ty);
        let to_control_x = cx - mx;
        let to_control_y = cy - my;
        let edge_x = tx - sx;
        let edge_y = ty - sy;
        let dot = to_control_x * edge_x + to_control_y * edge_y;
        let scale = (edge_x.abs().max(edge_y.abs())).max(1.0);
        prop_assert!(dot.abs() < scale * 1e-9);
    }

    #[cfg(kani)]
#[kani::proof]
    fn dist_to_segment_zero_length_returns_distance_to_point(
        px in finite_f64(), py in finite_f64(),
        x in finite_f64(), y in finite_f64(),
    ) {
        let dist = dist_to_segment(px, py, x, y, x, y);
        let expected = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
        let tolerance = expected.abs().max(1.0) * 1e-10;
        prop_assert!((dist - expected).abs() <= tolerance);
    }

    #[cfg(kani)]
#[kani::proof]
    fn dist_to_segment_point_on_endpoint_returns_zero(
        x1 in finite_f64(), y1 in finite_f64(),
        x2 in finite_f64(), y2 in finite_f64(),
    ) {
        let dist_start = dist_to_segment(x1, y1, x1, y1, x2, y2);
        let dist_end = dist_to_segment(x2, y2, x1, y1, x2, y2);
        prop_assert!(dist_start < 1e-9);
        prop_assert!(dist_end < 1e-9);
    }

    #[cfg(kani)]
#[kani::proof]
    fn dist_to_segment_always_non_negative(
        px in finite_f64(), py in finite_f64(),
        x1 in finite_f64(), y1 in finite_f64(),
        x2 in finite_f64(), y2 in finite_f64(),
    ) {
        prop_assert!(dist_to_segment(px, py, x1, y1, x2, y2) >= 0.0);
    }
}
