#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::{DiagramDocument, DocumentData, NodeKind, Revision};
use crate::ui::theme::{ACCENT, APP_FONT, BG_ELEVATED, BG_SURFACE, BORDER, TEXT_MUTED};
use dioxus::prelude::*;

const PAD: f64 = 60.0;
const BASE_SIDE: f64 = 180.0;
const MIN_W: f64 = 120.0;
const MAX_W: f64 = 280.0;
const MIN_H: f64 = 80.0;
const MAX_H: f64 = 200.0;

type EdgeSegment = (f64, f64, f64, f64);
type NodeRect = (bool, f64, f64, f64, f64, String);
type ProjectedNodeRect = (bool, f64, f64, f64, f64, &'static str);

#[derive(Clone)]
struct MinimapProjection {
    edge_segments: Vec<EdgeSegment>,
    node_rects: Vec<ProjectedNodeRect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProjectionKey {
    revision: Revision,
    min_x_bits: u64,
    min_y_bits: u64,
    scale_bits: u64,
}

impl ProjectionKey {
    const fn from_state(revision: Revision, min_x: f64, min_y: f64, scale: f64) -> Self {
        Self {
            revision,
            min_x_bits: min_x.to_bits(),
            min_y_bits: min_y.to_bits(),
            scale_bits: scale.to_bits(),
        }
    }
}

#[derive(Clone)]
struct MinimapSnapshot {
    edge_segments: Vec<EdgeSegment>,
    node_rects: Vec<NodeRect>,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl MinimapSnapshot {
    fn from_document(document: &DocumentData) -> Option<Self> {
        if document.nodes.is_empty() {
            return None;
        }

        let mut edge_segments = Vec::new();
        let mut node_rects = Vec::new();
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for node in document.nodes.values() {
            let has_valid_geometry = node.x.0.is_finite()
                && node.y.0.is_finite()
                && node.width.0.is_finite()
                && node.height.0.is_finite();

            if !has_valid_geometry {
                continue;
            }

            min_x = min_x.min(node.x.0);
            min_y = min_y.min(node.y.0);
            max_x = max_x.max(node.x.0 + node.width.0);
            max_y = max_y.max(node.y.0 + node.height.0);

            let provider = node
                .tags
                .get(0)
                .map_or_else(|| String::from("generic"), Clone::clone);
            node_rects.push((
                node.kind == NodeKind::Subgraph,
                node.x.0,
                node.y.0,
                node.width.0,
                node.height.0,
                provider,
            ));
        }

        for edge in document.edges.values() {
            if let Some((source, target)) = document
                .nodes
                .get(&edge.source)
                .zip(document.nodes.get(&edge.target))
            {
                edge_segments.push((
                    source.x.0 + (source.width.0 / 2.0),
                    source.y.0 + (source.height.0 / 2.0),
                    target.x.0 + (target.width.0 / 2.0),
                    target.y.0 + (target.height.0 / 2.0),
                ));
            }
        }

        Some(Self {
            edge_segments,
            node_rects,
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    fn project(&self, min_x: f64, min_y: f64, scale: f64) -> MinimapProjection {
        let to_mini = |x: f64, y: f64| ((x - min_x) * scale, (y - min_y) * scale);
        let edge_segments = self
            .edge_segments
            .iter()
            .map(|(sxw, syw, txw, tyw)| {
                let (sx, sy) = to_mini(*sxw, *syw);
                let (tx, ty) = to_mini(*txw, *tyw);
                (sx, sy, tx, ty)
            })
            .collect();
        let node_rects = self
            .node_rects
            .iter()
            .map(|(is_subgraph, node_x, node_y, node_w, node_h, provider)| {
                let (x, y) = to_mini(*node_x, *node_y);
                let w = (*node_w * scale).max(2.0);
                let h = (*node_h * scale).max(2.0);
                (*is_subgraph, x, y, w, h, provider_color(provider))
            })
            .collect();

        MinimapProjection {
            edge_segments,
            node_rects,
        }
    }
}

fn provider_color(provider: &str) -> &'static str {
    match provider {
        "aws" => "#FF9900",
        "gcp" => "#4285F4",
        "azure" => "#0078D4",
        "k8s" => "#326CE5",
        _ => "#6B7280",
    }
}

#[component]
pub fn Minimap() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let viewport_size = use_context::<Signal<(f64, f64)>>();
    let mut dragging = use_signal(|| false);
    let mut cached_snapshot = use_signal(|| Option::<MinimapSnapshot>::None);
    let mut last_snapshot_revision = use_signal(|| Option::<Revision>::None);
    let mut cached_projection = use_signal(|| Option::<MinimapProjection>::None);
    let mut last_projection_key = use_signal(|| Option::<ProjectionKey>::None);

    let (cam_x, cam_y, zoom, revision) = {
        let doc = doc_signal.read();

        if doc.document.nodes.is_empty() {
            return rsx! {};
        }

        let needs_refresh = cached_snapshot.read().is_none()
            || last_snapshot_revision
                .read()
                .as_ref()
                .is_none_or(|cached| *cached != doc.revision);

        if needs_refresh {
            cached_snapshot.set(MinimapSnapshot::from_document(&doc.document));
            last_snapshot_revision.set(Some(doc.revision));
            last_projection_key.set(None);
        }

        (
            doc.editor_state.camera_x.0,
            doc.editor_state.camera_y.0,
            doc.editor_state.zoom.0,
            doc.revision,
        )
    };

    let snapshot = cached_snapshot.read();
    let Some(snapshot) = snapshot.as_ref() else {
        return rsx! {};
    };
    let doc_min_x = snapshot.min_x;
    let doc_min_y = snapshot.min_y;
    let doc_max_x = snapshot.max_x;
    let doc_max_y = snapshot.max_y;

    let (viewport_w, viewport_h) = *viewport_size.read();
    let vp_w = viewport_w.max(1.0) / zoom;
    let vp_h = viewport_h.max(1.0) / zoom;
    let vp_left = cam_x;
    let vp_top = cam_y;

    let min_x = doc_min_x.min(vp_left) - PAD;
    let min_y = doc_min_y.min(vp_top) - PAD;
    let max_x = doc_max_x.max(vp_left + vp_w) + PAD;
    let max_y = doc_max_y.max(vp_top + vp_h) + PAD;

    let world_w = (max_x - min_x).max(1.0);
    let world_h = (max_y - min_y).max(1.0);
    let aspect = world_w / world_h;
    let (mut view_w, mut view_h) = if aspect > 1.0 {
        let width = BASE_SIDE.round();
        (width, (width / aspect).round())
    } else {
        let height = BASE_SIDE.round();
        ((height * aspect).round(), height)
    };
    view_w = view_w.clamp(MIN_W, MAX_W);
    view_h = view_h.clamp(MIN_H, MAX_H);

    let scale = (view_w / world_w).min(view_h / world_h);

    let projection_key = ProjectionKey::from_state(revision, min_x, min_y, scale);
    if last_projection_key
        .read()
        .as_ref()
        .is_none_or(|cached| *cached != projection_key)
    {
        cached_projection.set(Some(snapshot.project(min_x, min_y, scale)));
        last_projection_key.set(Some(projection_key));
    }

    let projection = cached_projection.read();
    let Some(projection) = projection.as_ref() else {
        return rsx! {};
    };
    let vp_x = (vp_left - min_x) * scale;
    let vp_y = (vp_top - min_y) * scale;

    let mut nav_to = move |screen_x: f64, screen_y: f64| {
        let center_x = (screen_x / scale) + min_x;
        let center_y = (screen_y / scale) + min_y;
        let doc = doc_signal.read();
        let zoom = doc.editor_state.zoom.0;
        let viewport = *viewport_size.read();
        let vp_w = viewport.0.max(1.0) / zoom;
        let vp_h = viewport.1.max(1.0) / zoom;
        let left = center_x - (vp_w / 2.0);
        let top = center_y - (vp_h / 2.0);
        let next_camera_x = left;
        let next_camera_y = top;
        let changed = (doc.editor_state.camera_x.0 - next_camera_x).abs() > 0.25
            || (doc.editor_state.camera_y.0 - next_camera_y).abs() > 0.25;
        if changed {
            drop(doc);
            doc_signal.with_mut(|doc_mut| {
                doc_mut.editor_state.camera_x.0 = next_camera_x;
                doc_mut.editor_state.camera_y.0 = next_camera_y;
            });
        }
    };

    rsx! {
        div {
            "data-testid": "minimap-root",
            style: "position: absolute; right: 12px; bottom: 12px; width: {view_w}px; height: {view_h}px; border: 1px solid {BORDER}; border-radius: 10px; background: linear-gradient(180deg, {BG_ELEVATED}f2 0%, {BG_SURFACE}ea 100%); backdrop-filter: blur(8px); overflow: hidden; z-index: 20; user-select:none; box-shadow: 0 8px 20px color-mix(in oklch, black 28%, transparent);",
            onmousedown: move |evt| {
                evt.stop_propagation();
                dragging.set(true);
                let c = evt.data.coordinates().element();
                nav_to(c.x, c.y);
            },
            onmousemove: move |evt| {
                if *dragging.read() {
                    let c = evt.data.coordinates().element();
                    nav_to(c.x, c.y);
                }
            },
            onmouseup: move |_| dragging.set(false),
            onmouseleave: move |_| dragging.set(false),

            svg {
                width: "{view_w}",
                height: "{view_h}",
                for &(sx, sy, tx, ty) in projection.edge_segments.iter() {
                    line {
                        x1: "{sx}",
                        y1: "{sy}",
                        x2: "{tx}",
                        y2: "{ty}",
                        stroke: "color-mix(in oklch, {TEXT_MUTED} 78%, transparent)",
                        stroke_width: "0.7",
                        opacity: "0.7",
                    }
                }
                for &(is_subgraph, x, y, w, h, provider_color) in projection.node_rects.iter() {
                    rect {
                        x: "{x}",
                        y: "{y}",
                        width: "{w}",
                        height: "{h}",
                        rx: "1.5",
                        fill: if is_subgraph { "none" } else { provider_color },
                        stroke: if is_subgraph {
                            format!("color-mix(in oklch, {TEXT_MUTED} 55%, transparent)")
                        } else { String::from(ACCENT) },
                        stroke_width: "0.8",
                        opacity: "0.85",
                    }
                }
                rect {
                    "data-testid": "minimap-viewport",
                    x: "{vp_x}",
                    y: "{vp_y}",
                    width: "{(vp_w * scale).max(4.0)}",
                    height: "{(vp_h * scale).max(4.0)}",
                    fill: "color-mix(in oklch, {ACCENT} 20%, transparent)",
                    stroke: "{ACCENT}",
                    stroke_width: "1.2",
                    rx: "2",
                }
            }
            div {
                style: "position: absolute; top: 4px; right: 6px; color: {TEXT_MUTED}; font-size: 10px; font-family: {APP_FONT};",
                "{(zoom * 100.0).round()}%"
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::models::document::{
        ArrowType, Edge, EdgeId, EdgeStyle, LockState, Node, NodeId, NodeKind, NodeStyle,
        OrderedFloat,
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
}
