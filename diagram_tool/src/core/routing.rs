#![allow(dead_code)]

use crate::models::document::{
    ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, NodeId, OrderedFloat, Point,
};
use crate::models::port::{compute_port_absolute_position, PortAnchor};
use thiserror::Error;

/// Constant for maximum number of edges between same node pair
pub const MAX_EDGE_MULTIPLICITY: usize = 1;

/// Padding applied to bounding box when creating subgraphs (20.0 units)
pub const SUBGRAPH_PADDING: f64 = 20.0;

/// Bounding box for node/group positioning
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    /// Create a new bounding box from coordinates
    #[must_use]
    pub const fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Calculate the width of the bounding box
    #[must_use]
    pub const fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Calculate the height of the bounding box
    #[must_use]
    pub const fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Check if this bounding box contains another
    #[must_use]
    pub fn contains(&self, other: &Self) -> bool {
        other.min_x >= self.min_x
            && other.max_x <= self.max_x
            && other.min_y >= self.min_y
            && other.max_y <= self.max_y
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min_x: f64::MAX,
            min_y: f64::MAX,
            max_x: f64::MIN,
            max_y: f64::MIN,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoutingError {
    #[error("Source node {0} not found")]
    SourceNotFound(NodeId),
    #[error("Target node {0} not found")]
    TargetNotFound(NodeId),
    #[error("Cannot create self-loop on node {0}")]
    SelfLoop(NodeId),
    #[error("Adding this edge creates a cycle")]
    CycleDetected,
    #[error("Invalid coordinates on node {0}")]
    InvalidNodeCoordinates(NodeId),
    #[error("Duplicate edge {0}")]
    DuplicateEdge(EdgeId),
}

/// Validates edge endpoints - returns error if invalid
fn validate_edge_endpoints(
    doc: &DiagramDocument,
    source: &NodeId,
    target: &NodeId,
) -> Result<(), RoutingError> {
    if source == target {
        return Err(RoutingError::SelfLoop(source.clone()));
    }
    if !doc.document.nodes.contains_key(source) {
        return Err(RoutingError::SourceNotFound(source.clone()));
    }
    if !doc.document.nodes.contains_key(target) {
        return Err(RoutingError::TargetNotFound(target.clone()));
    }
    Ok(())
}

/// Check if adding edge from source to target would create a cycle using DFS
fn would_create_cycle(doc: &DiagramDocument, source: &NodeId, target: &NodeId) -> bool {
    doc.document.edges.values().any(|e| {
        e.source == *target && (e.target == *source || would_create_cycle(doc, source, &e.target))
    })
}

/// Create a new edge with default styling
fn make_default_edge(source: NodeId, target: NodeId) -> Edge {
    Edge {
        source,
        target,
        label: String::new(),
        style: EdgeStyle::default(),
        arrow_type: ArrowType::default(),
        label_offset_t: OrderedFloat::default(),
        color: None,
        thickness: OrderedFloat::default(),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

/// Creates a new edge between two nodes.
///
/// # Errors
///
/// Returns `RoutingError::SelfLoop` if source and target are the same.
/// Returns `RoutingError::SourceNotFound` if source node doesn't exist.
/// Returns `RoutingError::TargetNotFound` if target node doesn't exist.
/// Returns `RoutingError::CycleDetected` if the edge would create a cycle.
pub fn create_edge(
    doc: &mut DiagramDocument,
    source: NodeId,
    target: NodeId,
    edge_id: EdgeId,
) -> Result<(), RoutingError> {
    validate_edge_endpoints(doc, &source, &target)?;
    if would_create_cycle(doc, &source, &target) {
        return Err(RoutingError::CycleDetected);
    }
    let new_edge = make_default_edge(source, target);
    doc.document.edges.insert(edge_id, new_edge);
    Ok(())
}

/// Computes a straight-line route between the source and target ports of an edge.
///
/// Falls back to center port if no port anchor is specified.
///
/// # Errors
///
/// Returns `RoutingError::SourceNotFound` if the source node does not exist.
/// Returns `RoutingError::TargetNotFound` if the target node does not exist.
pub fn compute_straight_line_route(
    doc: &DiagramDocument,
    edge: &Edge,
) -> Result<(Point, Point), RoutingError> {
    let source_node = doc
        .document
        .nodes
        .get(&edge.source)
        .ok_or_else(|| RoutingError::SourceNotFound(edge.source.clone()))?;

    let target_node = doc
        .document
        .nodes
        .get(&edge.target)
        .ok_or_else(|| RoutingError::TargetNotFound(edge.target.clone()))?;

    let source_port = edge.source_port.as_ref().unwrap_or(&PortAnchor::Center);
    let target_port = edge.target_port.as_ref().unwrap_or(&PortAnchor::Center);

    let start = compute_port_absolute_position(source_node, source_port);
    let end = compute_port_absolute_position(target_node, target_port);

    Ok((start, end))
}

#[cfg(test)]
#[path = "routing_tests.rs"]
mod tests;
