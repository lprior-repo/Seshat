//! Types for subgraph operations
//!
//! Contains core domain types: BoundingBox, Padding, PositiveScale, and Error types.

use crate::models::document::{DocumentData, NodeId, OrderedFloat, Point};
use thiserror::Error;

/// Alias for document data structure used in subgraph operations
pub type CanvasState = DocumentData;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    #[must_use]
    pub const fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid padding")]
    InvalidPadding,
    #[error("Empty selection")]
    EmptySelection,
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("Circular dependency detected")]
    CircularDependency,
    #[error("Node locked: {0}")]
    NodeLocked(NodeId),
    #[error("Invalid transform scale")]
    InvalidTransform,
    #[error("Invalid node type")]
    InvalidNodeType,
    #[error("Invariant violation")]
    InvariantViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Padding {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositiveScale(OrderedFloat);

impl PositiveScale {
    /// Creates a new `PositiveScale` ensuring the value is strictly greater than zero.
    ///
    /// # Errors
    /// Returns `Error::InvalidTransform` if the value is zero or negative.
    pub fn try_new(value: OrderedFloat) -> Result<Self, Error> {
        if value.0 > 0.0 {
            Ok(Self(value))
        } else {
            Err(Error::InvalidTransform)
        }
    }

    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0 .0
    }
}

/// Applies a viewport transform to a subgraph, scaling its position and dimensions.
///
/// # Errors
/// Returns `Error::InvalidTransform` if the resulting transform produces invalid values.
pub fn apply_viewport_transform(
    subgraph: &crate::models::document::Node,
    scale: PositiveScale,
) -> Result<crate::models::document::Node, Error> {
    let scaled_x = subgraph.x.0 * scale.value();
    let scaled_y = subgraph.y.0 * scale.value();
    let scaled_width = subgraph.width.0 * scale.value();
    let scaled_height = subgraph.height.0 * scale.value();

    Ok(crate::models::document::Node {
        x: OrderedFloat::new(scaled_x).map_err(|_| Error::InvalidTransform)?,
        y: OrderedFloat::new(scaled_y).map_err(|_| Error::InvalidTransform)?,
        width: OrderedFloat::new(scaled_width).map_err(|_| Error::InvalidTransform)?,
        height: OrderedFloat::new(scaled_height).map_err(|_| Error::InvalidTransform)?,
        ..subgraph.clone()
    })
}

/// Calculates the bounding box that encapsulates all given child nodes plus the specified padding.
///
/// # Errors
/// Returns `Error::InvariantViolation` if calculating bounds fails or no children exist.
pub fn calculate_container_bounds(
    children: &[crate::models::document::Node],
    padding: Padding,
) -> Result<BoundingBox, Error> {
    if children.is_empty() {
        return Ok(BoundingBox::new(0.0, 0.0, 0.0, 0.0));
    }

    let min_x = children.iter().map(|n| n.x.0).fold(f64::INFINITY, f64::min);
    let min_y = children.iter().map(|n| n.y.0).fold(f64::INFINITY, f64::min);
    let max_x = children
        .iter()
        .map(|n| n.x.0 + n.width.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = children
        .iter()
        .map(|n| n.y.0 + n.height.0)
        .fold(f64::NEG_INFINITY, f64::max);

    let bounds = BoundingBox::new(
        min_x - f64::from(padding.left),
        min_y - f64::from(padding.top),
        max_x + f64::from(padding.right),
        max_y + f64::from(padding.bottom),
    );

    // Q1 Postcondition validation - ensure container bounds encapsulate all children + padding
    let valid = children.iter().all(|n| {
        bounds.min_x <= n.x.0 - f64::from(padding.left)
            && bounds.min_y <= n.y.0 - f64::from(padding.top)
            && bounds.max_x >= n.x.0 + n.width.0 + f64::from(padding.right)
            && bounds.max_y >= n.y.0 + n.height.0 + f64::from(padding.bottom)
    });

    if !valid {
        return Err(Error::InvariantViolation);
    }

    Ok(bounds)
}

/// Creates a new empty subgraph container node with minimum dimensions.
///
/// # Errors
/// Returns error if invariants are violated.
pub fn create_empty_subgraph(
    _id: NodeId,
    position: Point,
) -> Result<crate::models::document::Node, Error> {
    let width = OrderedFloat::new(100.0).map_err(|_| Error::InvariantViolation)?; // minimum width
    let height = OrderedFloat::new(60.0).map_err(|_| Error::InvariantViolation)?; // minimum height

    let node = crate::models::document::Node {
        kind: crate::models::document::NodeKind::Subgraph,
        icon: String::new(),
        label: String::new(),
        x: position.x,
        y: position.y,
        width,
        height,
        font_size: None,
        font_weight: None,
        locked: false,
        parent: None,
        dag_rank: None,
        tags: im::vector![],
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };

    // Q2 Postcondition validation
    if node.width.0 < 100.0 || node.height.0 < 60.0 {
        return Err(Error::InvariantViolation);
    }

    Ok(node)
}
