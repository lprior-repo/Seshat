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
        ArrowType::Arrow => EdgePathSemantics::Default,
        ArrowType::Open => EdgePathSemantics::Straight,
        ArrowType::Diamond => EdgePathSemantics::Step,
        ArrowType::Circle => EdgePathSemantics::Curved,
        ArrowType::None => EdgePathSemantics::Sharp,
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
    bend_points: &[Point],
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
pub(super) fn selection_handles_overlay(
    doc: &DiagramDocument,
    interaction_mode: Signal<InteractionMode>,
    doc_signal: Signal<DiagramDocument>,
    to_screen_coords: impl Fn(f64, f64, f64, f64, f64) -> (f64, f64),
) -> Element {
    let selected_nodes = selected_node_ids(doc);
    let selected_count = selected_nodes.len();
    let selection = selection_bounds(doc);
    if let Some((bx, by, bw, bh)) = selection {
        let s = &doc.editor_state;
        let (sx, sy) = to_screen_coords(bx, by, s.camera_x.0, s.camera_y.0, s.zoom.0);
        let sw = bw * s.zoom.0;
        let sh = bh * s.zoom.0;
        let is_multi = selected_count >= 2;
        let pad = if is_multi { 6.0 } else { 4.0 };
        let box_w = (2.0_f64).mul_add(pad, sw);
        let box_h = (2.0_f64).mul_add(pad, sh);
        let hs = if is_multi { 8.0 } else { 7.0 };
        let handles = [
            (ResizeHandle::Nw, sx - pad, sy - pad, "nwse-resize"),
            (ResizeHandle::N, sx + (sw / 2.0), sy - pad, "ns-resize"),
            (ResizeHandle::Ne, sx + sw + pad, sy - pad, "nesw-resize"),
            (ResizeHandle::E, sx + sw + pad, sy + (sh / 2.0), "ew-resize"),
            (
                ResizeHandle::Se,
                sx + sw + pad,
                sy + sh + pad,
                "nwse-resize",
            ),
            (ResizeHandle::S, sx + (sw / 2.0), sy + sh + pad, "ns-resize"),
            (ResizeHandle::Sw, sx - pad, sy + sh + pad, "nesw-resize"),
            (ResizeHandle::W, sx - pad, sy + (sh / 2.0), "ew-resize"),
        ];

        rsx! {
            if is_multi {
                div {
                    style: "position:absolute; left:{sx - pad}px; top:{sy - pad}px; width:{box_w}px; height:{box_h}px; border:{SELECTION_BOUNDS_STROKE}; pointer-events:none; z-index:15;"
                }
            }
            if !selected_nodes.is_empty() {
                for (handle, hx, hy, cursor) in handles {
                    button {
                        key: "{hx}-{hy}",
                        style: "position:absolute; left:{hx - hs/2.0}px; top:{hy - hs/2.0}px; width:{hs}px; height:{hs}px; border-radius:2px; border:1px solid {BG_BASE}; background:{ACCENT}; cursor:{cursor}; z-index:16;",
                        onmousedown: move |evt| {
                            if evt.data.trigger_button() != Some(MouseButton::Primary) {
                                return;
                            }
                            evt.stop_propagation();
                            let c = evt.data.coordinates().client();
                            start_resize_interaction(
                                interaction_mode,
                                doc_signal,
                                handle,
                                c.x,
                                c.y,
                            );
                        }
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
    doc.document.edges.iter().find_map(|(id, edge)| {
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
                        for step in 1..=20 {
                            let t = f64::from(step) / 20.0;
                            let curr = quadratic_bezier_point((sx, sy), (cx, cy), (tx, ty), t);
                            min_dist =
                                min_dist.min(dist_to_segment(x, y, prev.0, prev.1, curr.0, curr.1));
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
                (hit_distance < 8.0).then(|| id.clone())
            })
    })
}
