#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use super::geometry::{interpolate_polyline_point, quadratic_bezier_point, quadratic_control};
use diagram_models::document::{ArrowType, Edge, Node, SerializedPoint};
use std::fmt::Write as _;

#[must_use]
pub fn edge_path(sx: f64, sy: f64, tx: f64, ty: f64, edge: &Edge) -> String {
    match edge_geometry(sx, sy, tx, ty, edge) {
        EdgeGeometry::Quadratic { control: (cx, cy) } => {
            format!("M {sx} {sy} Q {cx} {cy} {tx} {ty}")
        }
        EdgeGeometry::Polyline(points) => polyline_path(&points),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgePathSemantics {
    Default,
    Straight,
    Curved,
    Step,
    Sharp,
}

pub const fn edge_path_semantics(edge: &Edge) -> EdgePathSemantics {
    semantics_from_arrow(edge.arrow_type)
}

pub const fn semantics_from_arrow(arrow_type: ArrowType) -> EdgePathSemantics {
    match arrow_type {
        ArrowType::Default => EdgePathSemantics::Default,
        ArrowType::Straight => EdgePathSemantics::Straight,
        ArrowType::Step => EdgePathSemantics::Step,
        ArrowType::Curved => EdgePathSemantics::Curved,
        ArrowType::Sharp => EdgePathSemantics::Sharp,
    }
}

pub enum EdgeGeometry {
    Quadratic { control: (f64, f64) },
    Polyline(Vec<(f64, f64)>),
}

#[must_use]
pub fn edge_endpoints(edge: &Edge, src: &Node, tgt: &Node) -> ((f64, f64), (f64, f64)) {
    (
        port_position(&edge.source_port, src),
        port_position(&edge.target_port, tgt),
    )
}

pub fn edge_geometry(sx: f64, sy: f64, tx: f64, ty: f64, edge: &Edge) -> EdgeGeometry {
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

fn port_position(port: &Option<diagram_models::port::PortAnchor>, node: &Node) -> (f64, f64) {
    let point = port.as_ref().map_or_else(
        || {
            diagram_models::geometry::Point::new(
                node.x.0 + node.width.0 / 2.0,
                node.y.0 + node.height.0 / 2.0,
            )
        },
        |anchor| diagram_models::port::compute_port_absolute_position(node, anchor),
    );
    (point.x, point.y)
}

pub fn routed_polyline_points(
    sx: f64,
    sy: f64,
    tx: f64,
    ty: f64,
    semantics: EdgePathSemantics,
    bend_points: &im::Vector<SerializedPoint>,
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

pub fn polyline_path(points: &[(f64, f64)]) -> String {
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
pub fn edge_label_position(sx: f64, sy: f64, tx: f64, ty: f64, edge: &Edge) -> (f64, f64) {
    let t = edge.label_offset_t.0.clamp(0.0, 1.0);
    match edge_geometry(sx, sy, tx, ty, edge) {
        EdgeGeometry::Quadratic { control: (cx, cy) } => {
            quadratic_bezier_point((sx, sy), (cx, cy), (tx, ty), t)
        }
        EdgeGeometry::Polyline(points) => interpolate_polyline_point(&points, t),
    }
}
