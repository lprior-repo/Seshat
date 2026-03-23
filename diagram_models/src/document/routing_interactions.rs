#![allow(clippy::suboptimal_flops)]

use super::{DiagramDocument, EdgeId};
use crate::document::edge::{Edge, SerializedPoint};
use crate::document::routing::{EdgeRoutingError, EdgeRoutingMutator, MAX_BEND_POINTS};
use crate::document::types::OrderedFloat;
use crate::geometry::{FinitePoint, Point};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanvasInteractionError {
    PointIndexOutOfBounds(usize),
    SegmentIntersectionFailed,
    EdgeNotFound,
    InvalidCoordinate,
}

impl std::fmt::Display for CanvasInteractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PointIndexOutOfBounds(idx) => write!(f, "Point index out of bounds: {idx}"),
            Self::SegmentIntersectionFailed => write!(f, "Segment intersection failed"),
            Self::EdgeNotFound => write!(f, "Edge not found"),
            Self::InvalidCoordinate => write!(f, "Invalid coordinate"),
        }
    }
}

impl std::error::Error for CanvasInteractionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BendPointIndex(pub usize);

/// Calculate the index where a new bend point should be inserted.
///
/// # Errors
/// Returns `CanvasInteractionError` if the click is invalid or far from any segment.
pub fn calculate_insertion_index(
    edge: &Edge,
    click: FinitePoint,
) -> Result<BendPointIndex, CanvasInteractionError> {
    let p = Point::from(click);

    let pts: Vec<Point> = edge
        .bend_points
        .iter()
        .map(|bp| Point::new(bp.x.0, bp.y.0))
        .collect();
    if pts.len() < 2 {
        return Ok(BendPointIndex(0));
    }

    let (best, dist) = pts
        .windows(2)
        .enumerate()
        .fold((0, f64::MAX), |(b, md), (i, w)| {
            let d = p.dist_to_segment_squared(w[0], w[1]);
            if d < md {
                (i + 1, d)
            } else {
                (b, md)
            }
        });

    if dist > 100.0 {
        Err(CanvasInteractionError::SegmentIntersectionFailed)
    } else {
        Ok(BendPointIndex(best))
    }
}

/// Handles the insertion of a new bend point into an edge's routing.
///
/// # Errors
/// Returns `EdgeRoutingError` if edge not found or coordinates invalid.
pub fn handle_bend_point_insertion(
    doc: &DiagramDocument,
    edge_id: &EdgeId,
    click: FinitePoint,
) -> Result<DiagramDocument, EdgeRoutingError> {
    let edge = doc
        .document
        .edges
        .get(edge_id)
        .ok_or_else(|| EdgeRoutingError::EdgeNotFound(edge_id.to_string()))?;

    if edge.bend_points.len() >= MAX_BEND_POINTS {
        return Err(EdgeRoutingError::BendPointLimitExceeded(MAX_BEND_POINTS));
    }

    let insertion_index = calculate_insertion_index(edge, click)
        .map_err(|_| EdgeRoutingError::InvalidCoordinate(click.x))?;

    let mut bps = edge.bend_points.clone();
    bps.insert(
        insertion_index.0,
        SerializedPoint {
            x: OrderedFloat(click.x),
            y: OrderedFloat(click.y),
        },
    );

    doc.update_edge_routing(edge_id, bps)
}

/// Handles dragging an existing bend point to a new position.
///
/// # Errors
/// Returns `EdgeRoutingError` on out of bounds or invalid coordinates.
pub fn handle_bend_point_drag(
    doc: &DiagramDocument,
    edge_id: &EdgeId,
    bend_index: BendPointIndex,
    new_pos: FinitePoint,
) -> Result<DiagramDocument, EdgeRoutingError> {
    let edge = doc
        .document
        .edges
        .get(edge_id)
        .ok_or_else(|| EdgeRoutingError::EdgeNotFound(edge_id.to_string()))?;

    if bend_index.0 >= edge.bend_points.len() {
        #[allow(clippy::cast_precision_loss)]
        return Err(EdgeRoutingError::InvalidCoordinate(bend_index.0 as f64));
    }

    let mut bps = edge.bend_points.clone();
    bps[bend_index.0] = SerializedPoint {
        x: OrderedFloat(new_pos.x),
        y: OrderedFloat(new_pos.y),
    };

    doc.update_edge_routing(edge_id, bps)
}
