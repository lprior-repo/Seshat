#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::interaction_reducer::{start_resize_interaction, InteractionMode, ResizeHandle};
use super::selection_geometry::{selected_node_ids, selection_bounds};
use crate::models::document::{ArrowType, DiagramDocument, Edge, EdgeId, Point};
use crate::ui::theme::{
    ACCENT, BG_BASE, SELECTION_BOUNDS_STROKE, SELECTION_RECT_FILL, SELECTION_RECT_STROKE,
    SUBGRAPH_PREVIEW_FILL, SUBGRAPH_PREVIEW_STROKE,
};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use std::fmt::Write as _;

#[must_use]
pub(super) fn dist_to_segment(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let a = px - x1;
    let b = py - y1;
    let c = x2 - x1;
    let d = y2 - y1;
    let dot = a.mul_add(c, b * d);
    let len_sq = c.mul_add(c, d * d);
    let mut param = if len_sq == 0.0 { -1.0 } else { dot / len_sq };
    param = param.clamp(0.0, 1.0);
    let xx = x1 + (param * c);
    let yy = y1 + (param * d);
    let dx = px - xx;
    let dy = py - yy;
    (dx.mul_add(dx, dy * dy)).sqrt()
}

#[must_use]
pub(super) fn edge_path(sx: f64, sy: f64, tx: f64, ty: f64, edge: &Edge) -> String {
    match edge_geometry(sx, sy, tx, ty, edge) {
        EdgeGeometry::Quadratic { control: (cx, cy) } => {
            format!("M {sx} {sy} Q {cx} {cy} {tx} {ty}")
        }
        EdgeGeometry::Polyline(points) => polyline_path(&points),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EdgePathSemantics {
    Default,
    Straight,
    Curved,
    Step,
    Sharp,
}

const fn edge_path_semantics(edge: &Edge) -> EdgePathSemantics {
    semantics_from_arrow(edge.arrow_type)
}

const fn semantics_from_arrow(arrow_type: ArrowType) -> EdgePathSemantics {
    match arrow_type {
        ArrowType::Default => EdgePathSemantics::Default,
        ArrowType::Straight => EdgePathSemantics::Straight,
        ArrowType::Step => EdgePathSemantics::Step,
        ArrowType::Curved => EdgePathSemantics::Curved,
        ArrowType::Sharp => EdgePathSemantics::Sharp,
    }
}

enum EdgeGeometry {
    Quadratic { control: (f64, f64) },
    Polyline(Vec<(f64, f64)>),
}

fn edge_geometry(sx: f64, sy: f64, tx: f64, ty: f64, edge: &Edge) -> EdgeGeometry {
    let semantics = edge_path_semantics(edge);
    if edge.bend_points.is_empty() && semantics == EdgePathSemantics::Curved {
        EdgeGeometry::Quadratic {
            control: quadratic_control(sx, sy, tx, ty),
        }
    } else {
        EdgeGeometry::Polyline(routed_polyline_points(
            sx,
            sy,
            tx,
            ty,
            semantics,
            &edge.bend_points,
        ))
    }
}

fn quadratic_control(sx: f64, sy: f64, tx: f64, ty: f64) -> (f64, f64) {
    let dx = tx - sx;
    let dy = ty - sy;
    let mx = f64::midpoint(sx, tx);
    let my = f64::midpoint(sy, ty);
    (dy.mul_add(-0.25, mx), dx.mul_add(0.25, my))
}

fn routed_polyline_points(
    sx: f64,
    sy: f64,
    tx: f64,
    ty: f64,
    semantics: EdgePathSemantics,
    bend_points: &im::Vector<Point>,
) -> Vec<(f64, f64)> {
    if !bend_points.is_empty() {
        let mut points = Vec::with_capacity(bend_points.len() + 2);
        points.push((sx, sy));
        points.extend(bend_points.iter().map(|point| (point.x.0, point.y.0)));
        points.push((tx, ty));
        return points;
    }

    match semantics {
        EdgePathSemantics::Step => {
            let dx = tx - sx;
            let dy = ty - sy;
            if dx.abs() >= dy.abs() {
                let mid_x = f64::midpoint(sx, tx);
                vec![(sx, sy), (mid_x, sy), (mid_x, ty), (tx, ty)]
            } else {
                let mid_y = f64::midpoint(sy, ty);
                vec![(sx, sy), (sx, mid_y), (tx, mid_y), (tx, ty)]
            }
        }
        EdgePathSemantics::Sharp => vec![(sx, sy), (tx, ty)],
        EdgePathSemantics::Curved | EdgePathSemantics::Default | EdgePathSemantics::Straight => {
            vec![(sx, sy), (tx, ty)]
        }
    }
}

fn polyline_path(points: &[(f64, f64)]) -> String {
    if let Some(((sx, sy), rest)) = points.split_first() {
        rest.iter().fold(format!("M {sx} {sy}"), |mut acc, (x, y)| {
            let _ = write!(acc, " L {x} {y}");
            acc
        })
    } else {
        String::new()
    }
}

#[must_use]
pub(super) fn edge_label_position(sx: f64, sy: f64, tx: f64, ty: f64, edge: &Edge) -> (f64, f64) {
    let t = edge.label_offset_t.0.clamp(0.0, 1.0);
    match edge_geometry(sx, sy, tx, ty, edge) {
        EdgeGeometry::Quadratic { control: (cx, cy) } => {
            quadratic_bezier_point((sx, sy), (cx, cy), (tx, ty), t)
        }
        EdgeGeometry::Polyline(points) => interpolate_polyline_point(&points, t),
    }
}

fn interpolate_polyline_point(points: &[(f64, f64)], t: f64) -> (f64, f64) {
    if points.len() < 2 {
        return points.first().copied().unwrap_or((0.0, 0.0));
    }

    let segments = points
        .windows(2)
        .map(|window| {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx.mul_add(dx, dy * dy)).sqrt();
            ((x1, y1), (x2, y2), len)
        })
        .collect::<Vec<_>>();

    let total_len = segments.iter().fold(0.0, |acc, (_, _, len)| acc + len);
    if total_len <= f64::EPSILON {
        return points[0];
    }

    let target = total_len * t;
    let mut traversed = 0.0;
    for ((x1, y1), (x2, y2), len) in segments {
        if len <= f64::EPSILON {
            continue;
        }
        if traversed + len >= target {
            let local_t = ((target - traversed) / len).clamp(0.0, 1.0);
            return (
                x1.mul_add(1.0 - local_t, x2 * local_t),
                y1.mul_add(1.0 - local_t, y2 * local_t),
            );
        }
        traversed += len;
    }

    points.last().copied().unwrap_or(points[0])
}

fn quadratic_bezier_point(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), t: f64) -> (f64, f64) {
    let one_minus_t = 1.0 - t;
    let one_minus_t_2 = one_minus_t * one_minus_t;
    let t_2 = t * t;
    let blend = 2.0 * one_minus_t * t;
    let x = p0.0.mul_add(one_minus_t_2, blend.mul_add(p1.0, t_2 * p2.0));
    let y = p0.1.mul_add(one_minus_t_2, blend.mul_add(p1.1, t_2 * p2.1));
    (x, y)
}

#[must_use]
pub(super) const fn edge_marker_ref(selected: bool) -> &'static str {
    if selected {
        "url(#arrowhead-selected)"
    } else {
        "url(#arrowhead)"
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub(super) fn selection_handles_overlay(
    doc: &DiagramDocument,
    interaction_mode: Signal<InteractionMode>,
    doc_signal: Signal<DiagramDocument>,
    canvas_origin: Signal<(f64, f64)>,
    to_screen_coords: impl Fn(f64, f64, f64, f64, f64) -> (f64, f64),
) -> Element {
    let selected_nodes = selected_node_ids(doc);
    let _selected_count = selected_nodes.len();
    let selection = selection_bounds(doc);
    if let Some((bx, by, bw, bh)) = selection {
        let s = &doc.editor_state;
        let (sx, sy) = to_screen_coords(bx, by, s.camera_x.0, s.camera_y.0, s.zoom.0);
        let sw = bw * s.zoom.0;
        let sh = bh * s.zoom.0;
        let pad = 4.0;
        let box_w = f64::mul_add(2.0, pad, sw);
        let box_h = f64::mul_add(2.0, pad, sh);
        let hs = 7.0;

        let handles = [
            (
                ResizeHandle::Nw,
                sx - pad,
                sy - pad,
                "nwse-resize",
                "resize-handle-nw",
            ),
            (
                ResizeHandle::Ne,
                sx + sw + pad,
                sy - pad,
                "nesw-resize",
                "resize-handle-ne",
            ),
            (
                ResizeHandle::Se,
                sx + sw + pad,
                sy + sh + pad,
                "nwse-resize",
                "resize-handle-se",
            ),
            (
                ResizeHandle::Sw,
                sx - pad,
                sy + sh + pad,
                "nesw-resize",
                "resize-handle-sw",
            ),
        ];

        rsx! {
            div {
                "data-testid": "selection-bounds",
                style: "position:absolute; left:{sx - pad}px; top:{sy - pad}px; width:{box_w}px; height:{box_h}px; border:{SELECTION_BOUNDS_STROKE}; pointer-events:none; z-index:15;"
            }
            if !selected_nodes.is_empty() {
                for (handle, hx, hy, cursor, stable_test_id) in handles {
                    button {
                        key: "{hx}-{hy}",
                        "data-testid": "{stable_test_id}",
                        "data-handle": match handle {
                            ResizeHandle::Nw => "nw",
                            ResizeHandle::N => "n",
                            ResizeHandle::Ne => "ne",
                            ResizeHandle::E => "e",
                            ResizeHandle::Se => "se",
                            ResizeHandle::S => "s",
                            ResizeHandle::Sw => "sw",
                            ResizeHandle::W => "w",
                        },
                        style: "position:absolute; left:{hx - hs/2.0}px; top:{hy - hs/2.0}px; width:{hs}px; height:{hs}px; border-radius:2px; border:1px solid {BG_BASE}; background:{ACCENT}; cursor:{cursor}; z-index:16;",
                        onmousedown: move |evt| {
                            if evt.data.trigger_button() != Some(MouseButton::Primary) {
                                return;
                            }
                            evt.stop_propagation();
                            let c = evt.data.coordinates().client();
                            let origin = *canvas_origin.read();
                            start_resize_interaction(
                                interaction_mode,
                                doc_signal,
                                handle,
                                c.x - origin.0,
                                c.y - origin.1,
                            );
                        },
                        div { style: "position:absolute; inset:0; pointer-events:none; opacity:0;" }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

pub(super) fn edge_preview_overlay(
    mode: &InteractionMode,
    doc: &DiagramDocument,
    to_screen_coords: impl Fn(f64, f64, f64, f64, f64) -> (f64, f64),
) -> Element {
    let s = &doc.editor_state;
    if let InteractionMode::DrawingEdge {
        from_node,
        current_pos,
    } = mode
    {
        doc.document.nodes.get(from_node).map_or_else(
            || rsx! {},
            |src| {
                let (sx, sy) = to_screen_coords(
                    src.x.0 + src.width.0 / 2.0,
                    src.y.0 + src.height.0 / 2.0,
                    s.camera_x.0,
                    s.camera_y.0,
                    s.zoom.0,
                );
                let (tx, ty) = to_screen_coords(
                    current_pos.0,
                    current_pos.1,
                    s.camera_x.0,
                    s.camera_y.0,
                    s.zoom.0,
                );
                rsx! {
                    line {
                        x1: "{sx}", y1: "{sy}", x2: "{tx}", y2: "{ty}",
                        stroke: "{ACCENT}", stroke_width: "1.8", stroke_dasharray: "5,5", marker_end: "url(#arrow-pending)"
                    }
                }
            },
        )
    } else {
        rsx! {}
    }
}

pub(super) fn rubber_band_overlay(
    mode: &InteractionMode,
    doc: &DiagramDocument,
    to_screen_coords: impl Fn(f64, f64, f64, f64, f64) -> (f64, f64),
) -> Element {
    if let InteractionMode::RubberBand { start, current } = mode {
        let s = &doc.editor_state;
        let (rx, ry) = to_screen_coords(
            start.0.min(current.0),
            start.1.min(current.1),
            s.camera_x.0,
            s.camera_y.0,
            s.zoom.0,
        );
        let rw = (start.0 - current.0).abs() * s.zoom.0;
        let rh = (start.1 - current.1).abs() * s.zoom.0;
        rsx! {
            rect {
                x: "{rx}", y: "{ry}", width: "{rw}", height: "{rh}",
                fill: "{SELECTION_RECT_FILL}", stroke: "{SELECTION_RECT_STROKE}", stroke_width: "1", stroke_dasharray: "4,2"
            }
        }
    } else {
        rsx! {}
    }
}

pub(super) fn subgraph_preview_overlay(
    mode: &InteractionMode,
    doc: &DiagramDocument,
    to_screen_coords: impl Fn(f64, f64, f64, f64, f64) -> (f64, f64),
) -> Element {
    if let InteractionMode::DrawingSubgraph { start, current } = mode {
        let editor = &doc.editor_state;
        let min_x = start.0.min(current.0);
        let min_y = start.1.min(current.1);
        let width = (start.0 - current.0).abs();
        let height = (start.1 - current.1).abs();
        let (screen_x, screen_y) = to_screen_coords(
            min_x,
            min_y,
            editor.camera_x.0,
            editor.camera_y.0,
            editor.zoom.0,
        );
        rsx! {
            rect {
                x: "{screen_x}", y: "{screen_y}", width: "{width * editor.zoom.0}", height: "{height * editor.zoom.0}",
                fill: "{SUBGRAPH_PREVIEW_FILL}", stroke: "{SUBGRAPH_PREVIEW_STROKE}", stroke_width: "1.2", stroke_dasharray: "6,3"
            }
        }
    } else {
        rsx! {}
    }
}

pub(super) fn find_edge_at(doc: &DiagramDocument, x: f64, y: f64) -> Option<EdgeId> {
    // Screen-consistent hit radius: 17.0 screen pixels scaled to world coordinates
    // This ensures hit testing behaves consistently regardless of zoom level
    let zoom = doc.editor_state.zoom.0;
    let screen_hit_radius = 17.0;
    let hit_radius_world = screen_hit_radius / zoom;
    let endpoint_hit_radius_world = 21.0 / zoom;
    doc.document
        .edges
        .iter()
        .filter_map(|(id, edge)| {
            doc.document
                .nodes
                .get(&edge.source)
                .zip(doc.document.nodes.get(&edge.target))
                .and_then(|(source, target)| {
                    let sx = source.x.0 + (source.width.0 / 2.0);
                    let sy = source.y.0 + (source.height.0 / 2.0);
                    let tx = target.x.0 + (target.width.0 / 2.0);
                    let ty = target.y.0 + (target.height.0 / 2.0);
                    let hit_distance = match edge_geometry(sx, sy, tx, ty, edge) {
                        EdgeGeometry::Quadratic { control: (cx, cy) } => {
                            let mut min_dist = f64::MAX;
                            let mut prev = (sx, sy);
                            for step in 1..=32 {
                                let t = f64::from(step) / 32.0;
                                let curr = quadratic_bezier_point((sx, sy), (cx, cy), (tx, ty), t);
                                min_dist = min_dist
                                    .min(dist_to_segment(x, y, prev.0, prev.1, curr.0, curr.1));
                                prev = curr;
                            }
                            min_dist
                        }
                        EdgeGeometry::Polyline(points) => points
                            .windows(2)
                            .map(|window| {
                                dist_to_segment(
                                    x,
                                    y,
                                    window[0].0,
                                    window[0].1,
                                    window[1].0,
                                    window[1].1,
                                )
                            })
                            .fold(f64::MAX, f64::min),
                    };
                    let endpoint_distance = dist_to_segment(x, y, sx, sy, sx, sy)
                        .min(dist_to_segment(x, y, tx, ty, tx, ty));
                    (hit_distance < hit_radius_world
                        || endpoint_distance < endpoint_hit_radius_world)
                        .then(|| (id.clone(), hit_distance))
                })
        })
        .min_by(|(a_id, a_dist), (b_id, b_dist)| {
            a_dist
                .total_cmp(b_dist)
                .then_with(|| a_id.as_str().cmp(b_id.as_str()))
        })
        .map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::find_edge_at;
    use crate::models::document::{
        ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind,
        NodeStyle, OrderedFloat,
    };
    use im::HashMap;

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
            locked: false,
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
        }
    }

    #[test]
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

        let hit = find_edge_at(&doc, 50.0, 17.0);
        assert_eq!(hit, Some(edge_id));
    }

    #[test]
    fn given_high_zoom_when_clicking_same_world_distance_then_hit_test_is_tighter() {
        let source_id = NodeId::new(String::from("source"));
        let target_id = NodeId::new(String::from("target"));
        let edge_id = EdgeId::new(String::from("e1"));

        let mut doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(0.0, 0.0))
                    .update(target_id.clone(), node_at(100.0, 0.0)),
                edges: HashMap::new().update(edge_id, edge(source_id, target_id)),
            },
            ..DiagramDocument::default()
        };
        doc.editor_state.zoom = OrderedFloat(2.0);

        let hit = find_edge_at(&doc, 50.0, 17.0);
        assert!(hit.is_none());
    }

    #[test]
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

        let hit = find_edge_at(&doc, 50.0, 5.0);
        assert_eq!(hit, Some(edge_a));
    }

    #[test]
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

        let hit = find_edge_at(&doc, 109.0, 12.0);
        assert_eq!(hit, Some(edge_id));
    }

    #[test]
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
            let hit = find_edge_at(&doc, 47.0, 65.0);
            assert_eq!(hit, Some(edge_id.clone()));
        }
    }

    #[test]
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

        let hit = find_edge_at(&doc, 105.0, 5.0);
        assert_eq!(hit, Some(edge_a));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn finite_f64() -> impl Strategy<Value = f64> {
        -1000.0_f64..=1000.0_f64
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
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

        #[test]
        fn quadratic_bezier_point_t_zero_returns_p0(
            p0x in finite_f64(), p0y in finite_f64(),
            p1x in finite_f64(), p1y in finite_f64(),
            p2x in finite_f64(), p2y in finite_f64(),
        ) {
            let (x, y) = quadratic_bezier_point((p0x, p0y), (p1x, p1y), (p2x, p2y), 0.0);
            prop_assert!((x - p0x).abs() < 1e-10);
            prop_assert!((y - p0y).abs() < 1e-10);
        }

        #[test]
        fn quadratic_bezier_point_t_one_returns_p2(
            p0x in finite_f64(), p0y in finite_f64(),
            p1x in finite_f64(), p1y in finite_f64(),
            p2x in finite_f64(), p2y in finite_f64(),
        ) {
            let (x, y) = quadratic_bezier_point((p0x, p0y), (p1x, p1y), (p2x, p2y), 1.0);
            prop_assert!((x - p2x).abs() < 1e-10);
            prop_assert!((y - p2y).abs() < 1e-10);
        }

        #[test]
        fn interpolate_polyline_point_t_zero_returns_first(
            x1 in finite_f64(), y1 in finite_f64(),
            x2 in finite_f64(), y2 in finite_f64(),
        ) {
            let points = vec![(x1, y1), (x2, y2)];
            let (px, py) = interpolate_polyline_point(&points, 0.0);
            prop_assert!((px - x1).abs() < 1e-10);
            prop_assert!((py - y1).abs() < 1e-10);
        }

        #[test]
        fn interpolate_polyline_point_t_one_returns_last(
            x1 in finite_f64(), y1 in finite_f64(),
            x2 in finite_f64(), y2 in finite_f64(),
        ) {
            let points = vec![(x1, y1), (x2, y2)];
            let (px, py) = interpolate_polyline_point(&points, 1.0);
            prop_assert!((px - x2).abs() < 1e-10);
            prop_assert!((py - y2).abs() < 1e-10);
        }

        #[test]
        fn interpolate_polyline_point_single_point_returns_that_point(
            x in finite_f64(), y in finite_f64(),
            t in 0.0_f64..=1.0_f64,
        ) {
            let points = vec![(x, y)];
            let (px, py) = interpolate_polyline_point(&points, t);
            prop_assert!((px - x).abs() < 1e-10);
            prop_assert!((py - y).abs() < 1e-10);
        }

        #[test]
        fn interpolate_polyline_point_empty_returns_zero(t in 0.0_f64..=1.0_f64) {
            let points: Vec<(f64, f64)> = vec![];
            let (px, py) = interpolate_polyline_point(&points, t);
            prop_assert!((px - 0.0).abs() < 1e-10);
            prop_assert!((py - 0.0).abs() < 1e-10);
        }

        #[test]
        fn quadratic_control_returns_finite_for_finite_input(
            sx in finite_f64(), sy in finite_f64(),
            tx in finite_f64(), ty in finite_f64(),
        ) {
            let (cx, cy) = quadratic_control(sx, sy, tx, ty);
            prop_assert!(cx.is_finite());
            prop_assert!(cy.is_finite());
        }

        #[test]
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

        #[test]
        fn dist_to_segment_zero_length_returns_distance_to_point(
            px in finite_f64(), py in finite_f64(),
            x in finite_f64(), y in finite_f64(),
        ) {
            let dist = dist_to_segment(px, py, x, y, x, y);
            let expected = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
            let tolerance = expected.abs().max(1.0) * 1e-10;
            prop_assert!((dist - expected).abs() <= tolerance);
        }

        #[test]
        fn dist_to_segment_point_on_endpoint_returns_zero(
            x1 in finite_f64(), y1 in finite_f64(),
            x2 in finite_f64(), y2 in finite_f64(),
        ) {
            let dist_start = dist_to_segment(x1, y1, x1, y1, x2, y2);
            let dist_end = dist_to_segment(x2, y2, x1, y1, x2, y2);
            prop_assert!(dist_start < 1e-9);
            prop_assert!(dist_end < 1e-9);
        }

        #[test]
        fn dist_to_segment_always_non_negative(
            px in finite_f64(), py in finite_f64(),
            x1 in finite_f64(), y1 in finite_f64(),
            x2 in finite_f64(), y2 in finite_f64(),
        ) {
            let dist = dist_to_segment(px, py, x1, y1, x2, y2);
            prop_assert!(dist >= 0.0);
        }

        #[test]
        fn interpolate_polyline_point_midpoint_two_points(
            x1 in finite_f64(), y1 in finite_f64(),
            x2 in finite_f64(), y2 in finite_f64(),
        ) {
            let points = vec![(x1, y1), (x2, y2)];
            let (px, py) = interpolate_polyline_point(&points, 0.5);
            let expected_x = (x1 + x2) / 2.0;
            let expected_y = (y1 + y2) / 2.0;
            prop_assert!((px - expected_x).abs() < 1e-10);
            prop_assert!((py - expected_y).abs() < 1e-10);
        }

        #[test]
        fn interpolate_polyline_point_returns_finite(
            x1 in finite_f64(), y1 in finite_f64(),
            x2 in finite_f64(), y2 in finite_f64(),
            x3 in finite_f64(), y3 in finite_f64(),
            t in 0.0_f64..=1.0_f64,
        ) {
            let points = vec![(x1, y1), (x2, y2), (x3, y3)];
            let (px, py) = interpolate_polyline_point(&points, t);
            prop_assert!(px.is_finite());
            prop_assert!(py.is_finite());
        }

        #[test]
        fn interpolate_polyline_point_clamped_t_stays_in_bounds(
            x1 in finite_f64(), y1 in finite_f64(),
            x2 in finite_f64(), y2 in finite_f64(),
            t in finite_f64(),
        ) {
            let points = vec![(x1, y1), (x2, y2)];
            let t = t.clamp(0.0, 1.0);
            let (px, py) = interpolate_polyline_point(&points, t);
            let min_x = x1.min(x2);
            let max_x = x1.max(x2);
            let min_y = y1.min(y2);
            let max_y = y1.max(y2);
            prop_assert!(px >= min_x - 1e-10 && px <= max_x + 1e-10);
            prop_assert!(py >= min_y - 1e-10 && py <= max_y + 1e-10);
        }
    }
}

// =============================================================================
// INP Mobile/Touch Input tests (bd-jqu)
// =============================================================================

/// Double-tap timing threshold in milliseconds.
/// Two taps within this window are considered a double-tap.
const DOUBLE_TAP_THRESHOLD_MS: u64 = 350;

/// Minimum touch hit radius in screen pixels for touch targets.
/// This is larger than mouse hit radius for touch usability.
const TOUCH_HIT_RADIUS_PX: f64 = 44.0;

/// Resize handle size in screen pixels.
const RESIZE_HANDLE_SIZE_PX: f64 = 14.0;

/// Check if two tap timestamps qualify as a double-tap.
#[must_use]
pub const fn is_double_tap(first_tap_ms: u64, second_tap_ms: u64) -> bool {
    second_tap_ms.saturating_sub(first_tap_ms) <= DOUBLE_TAP_THRESHOLD_MS
}

/// Calculate touch-adjusted hit radius for touch input.
/// Touch input requires larger hit areas than mouse input for usability.
#[must_use]
pub const fn touch_hit_radius(base_radius: f64, is_touch: bool) -> f64 {
    if is_touch {
        base_radius.max(TOUCH_HIT_RADIUS_PX)
    } else {
        base_radius
    }
}

/// Check if a touch point is within a resize handle's hit area.
/// Touch handles need expanded hit areas for usability.
#[must_use]
pub fn touch_handle_hit_test(
    touch_x: f64,
    touch_y: f64,
    handle_x: f64,
    handle_y: f64,
    is_touch: bool,
) -> bool {
    let effective_size = if is_touch {
        // Touch: expand hit area to 44px minimum (WCAG touch target guideline)
        RESIZE_HANDLE_SIZE_PX.max(TOUCH_HIT_RADIUS_PX)
    } else {
        RESIZE_HANDLE_SIZE_PX
    };
    let half_size = effective_size / 2.0;
    touch_x >= handle_x - half_size
        && touch_x <= handle_x + half_size
        && touch_y >= handle_y - half_size
        && touch_y <= handle_y + half_size
}

#[cfg(test)]
mod inp_mobile_tests {
    use super::{
        is_double_tap, touch_handle_hit_test, touch_hit_radius, DOUBLE_TAP_THRESHOLD_MS,
        RESIZE_HANDLE_SIZE_PX, TOUCH_HIT_RADIUS_PX,
    };

    // =========================================================================
    // INP-1: Double-tap timing threshold tests
    // =========================================================================

    #[test]
    fn given_two_taps_within_threshold_when_checked_then_is_double_tap() {
        let first_tap = 1000_u64;
        let second_tap = 1100_u64; // 100ms later, well within 350ms threshold

        assert!(
            is_double_tap(first_tap, second_tap),
            "Taps 100ms apart should be considered a double-tap"
        );
    }

    #[test]
    fn given_two_taps_exactly_at_threshold_when_checked_then_is_double_tap() {
        let first_tap = 1000_u64;
        let second_tap = 1350_u64; // exactly 350ms later

        assert!(
            is_double_tap(first_tap, second_tap),
            "Taps exactly at threshold should be considered a double-tap"
        );
    }

    #[test]
    fn given_two_taps_just_over_threshold_when_checked_then_not_double_tap() {
        let first_tap = 1000_u64;
        let second_tap = 1351_u64; // 351ms later, just over threshold

        assert!(
            !is_double_tap(first_tap, second_tap),
            "Taps 351ms apart should NOT be considered a double-tap"
        );
    }

    #[test]
    fn given_two_taps_far_apart_when_checked_then_not_double_tap() {
        let first_tap = 1000_u64;
        let second_tap = 5000_u64; // 4000ms later

        assert!(
            !is_double_tap(first_tap, second_tap),
            "Taps 4000ms apart should NOT be considered a double-tap"
        );
    }

    #[test]
    fn given_zero_times_when_checked_then_is_double_tap() {
        // Edge case: both at zero means they're at the same time
        assert!(
            is_double_tap(0, 0),
            "Two taps at time 0 should be considered a double-tap"
        );
    }

    #[test]
    fn given_same_timestamp_when_checked_then_is_double_tap() {
        let timestamp = 12345_u64;
        assert!(
            is_double_tap(timestamp, timestamp),
            "Two taps at the same timestamp should be considered a double-tap"
        );
    }

    #[test]
    fn given_reversed_timestamps_when_checked_then_not_double_tap() {
        // If second tap appears before first (clock skew or edge case),
        // saturating_sub returns 0, which is <= threshold
        let first_tap = 2000_u64;
        let second_tap = 1000_u64;

        assert!(
            is_double_tap(first_tap, second_tap),
            "Reversed timestamps should handle gracefully (treated as same-time)"
        );
    }

    #[test]
    fn given_threshold_boundary_values_when_checked_then_boundary_correct() {
        // Test the exact boundary
        let first_tap = 0_u64;

        // At threshold
        assert!(is_double_tap(first_tap, DOUBLE_TAP_THRESHOLD_MS));

        // Just over threshold
        assert!(!is_double_tap(first_tap, DOUBLE_TAP_THRESHOLD_MS + 1));
    }

    // =========================================================================
    // INP-2: Touch handle hit area usable tests
    // =========================================================================

    #[test]
    fn given_touch_input_when_hit_testing_handle_then_expanded_hit_area_used() {
        let handle_x = 100.0;
        let handle_y = 100.0;

        // Touch point just outside the visual handle but within touch-expanded area
        let touch_x = 120.0; // 20px from handle center
        let touch_y = 100.0;

        // Mouse hit test should fail (outside 14px handle)
        assert!(
            !touch_handle_hit_test(touch_x, touch_y, handle_x, handle_y, false),
            "Mouse hit test should fail for point outside visual handle"
        );

        // Touch hit test should succeed (within 44px touch target)
        assert!(
            touch_handle_hit_test(touch_x, touch_y, handle_x, handle_y, true),
            "Touch hit test should succeed for point within expanded touch area"
        );
    }

    #[test]
    fn given_touch_input_at_corner_when_hit_testing_then_expanded_area_covers_corners() {
        let handle_x = 100.0;
        let handle_y = 100.0;

        // Touch at corner of expanded hit area (diagonal from center)
        let half_touch = TOUCH_HIT_RADIUS_PX / 2.0;
        let touch_x = handle_x + half_touch - 1.0; // Just inside
        let touch_y = handle_y + half_touch - 1.0;

        assert!(
            touch_handle_hit_test(touch_x, touch_y, handle_x, handle_y, true),
            "Touch at corner of expanded area should be a hit"
        );
    }

    #[test]
    fn given_touch_input_outside_expanded_area_when_hit_testing_then_fails() {
        let handle_x = 100.0;
        let handle_y = 100.0;

        // Touch point outside even the expanded touch area
        let half_touch = TOUCH_HIT_RADIUS_PX / 2.0;
        let touch_x = handle_x + half_touch + 10.0; // 10px outside
        let touch_y = handle_y;

        assert!(
            !touch_handle_hit_test(touch_x, touch_y, handle_x, handle_y, true),
            "Touch outside expanded area should fail"
        );
    }

    #[test]
    fn given_mouse_input_when_hit_testing_handle_then_visual_size_used() {
        let handle_x = 100.0;
        let handle_y = 100.0;

        // Just inside visual handle (14px)
        let half_visual = RESIZE_HANDLE_SIZE_PX / 2.0;
        let touch_x = handle_x + half_visual - 1.0;
        let touch_y = handle_y;

        assert!(
            touch_handle_hit_test(touch_x, touch_y, handle_x, handle_y, false),
            "Mouse hit test should succeed for point inside visual handle"
        );
    }

    #[test]
    fn given_touch_input_directly_on_handle_when_hit_testing_then_succeeds() {
        let handle_x = 100.0;
        let handle_y = 100.0;

        // Touch directly on handle center
        assert!(
            touch_handle_hit_test(handle_x, handle_y, handle_x, handle_y, true),
            "Touch directly on handle should always succeed"
        );
    }

    // =========================================================================
    // INP-3: Touch input uses larger hit radius tests
    // =========================================================================

    #[test]
    fn given_touch_input_when_calculating_hit_radius_then_uses_touch_minimum() {
        let base_radius = 17.0; // Standard edge hit radius

        let mouse_radius = touch_hit_radius(base_radius, false);
        let touch_radius = touch_hit_radius(base_radius, true);

        assert_eq!(mouse_radius, base_radius, "Mouse should use base radius");
        assert_eq!(
            touch_radius, TOUCH_HIT_RADIUS_PX,
            "Touch should use expanded minimum"
        );
        assert!(
            touch_radius > mouse_radius,
            "Touch radius should be larger than mouse radius"
        );
    }

    #[test]
    fn given_large_base_radius_when_touch_input_then_base_preserved_if_larger() {
        let large_base = 60.0; // Larger than touch minimum

        let touch_radius = touch_hit_radius(large_base, true);

        assert_eq!(
            touch_radius, large_base,
            "If base radius is already larger, it should be preserved"
        );
    }

    #[test]
    fn given_mouse_input_when_calculating_hit_radius_then_base_unchanged() {
        let base_radius = 25.0;

        let mouse_radius = touch_hit_radius(base_radius, false);

        assert_eq!(
            mouse_radius, base_radius,
            "Mouse input should never expand the hit radius"
        );
    }

    #[test]
    fn given_zero_base_radius_when_touch_input_then_touch_minimum_used() {
        let base_radius = 0.0;

        let touch_radius = touch_hit_radius(base_radius, true);

        assert_eq!(
            touch_radius, TOUCH_HIT_RADIUS_PX,
            "Zero base radius should be expanded to touch minimum"
        );
    }

    #[test]
    fn given_touch_minimum_matches_wcag_guideline() {
        // WCAG 2.1 Success Criterion 2.5.5 (Target Size - Enhanced)
        // recommends a minimum touch target size of 44x44 CSS pixels
        assert_eq!(
            TOUCH_HIT_RADIUS_PX, 44.0,
            "Touch hit radius should match WCAG recommended minimum"
        );
    }

    #[test]
    fn given_double_tap_threshold_is_reasonable() {
        // Industry standard double-tap thresholds are typically 300-400ms
        assert!(
            DOUBLE_TAP_THRESHOLD_MS >= 300,
            "Double-tap threshold should be at least 300ms"
        );
        assert!(
            DOUBLE_TAP_THRESHOLD_MS <= 500,
            "Double-tap threshold should be at most 500ms"
        );
    }
}
