//! Domain types for subgraph event handling.
//!
//! Separated from main module to keep file sizes under 300 lines.

use thiserror::Error;

use crate::models::document::{NodeId, OrderedFloat};
use crate::models::projection::types::DiagramProjection;

/// Alias to align with contract terminology
pub type DiagramState = DiagramProjection;

/// Bounding box representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: OrderedFloat,
    pub y: OrderedFloat,
    pub width: OrderedFloat,
    pub height: OrderedFloat,
}

impl Rect {
    /// Creates a new `Rect` ensuring valid bounds.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidBounds` if width or height is negative.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, Error> {
        if width < 0.0 || height < 0.0 {
            return Err(Error::InvalidBounds(Self {
                x: OrderedFloat::new_unchecked(x),
                y: OrderedFloat::new_unchecked(y),
                width: OrderedFloat::new_unchecked(width),
                height: OrderedFloat::new_unchecked(height),
            }));
        }
        Ok(Self {
            x: OrderedFloat::new_unchecked(x),
            y: OrderedFloat::new_unchecked(y),
            width: OrderedFloat::new_unchecked(width),
            height: OrderedFloat::new_unchecked(height),
        })
    }
}

/// Errors defined in the contract taxonomy
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("Cycle detected: {0} -> {1}")]
    CycleDetected(NodeId, NodeId),
    #[error("Invalid bounds: {0:?}")]
    InvalidBounds(Rect),
}
