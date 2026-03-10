use crate::geometry::primitives::Point;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnapMode {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum SnapError {
    #[error("invalid grid size: {0} (must be > 0)")]
    InvalidGridSize(f64),
    #[error("invalid threshold: {0} (must be >= 0)")]
    InvalidThreshold(f64),
    #[error("invalid node list: {0}")]
    InvalidNodeList(String),
    #[error("invalid alignment anchor: {0}")]
    InvalidAlignmentAnchor(String),
    #[error("invalid resize handle: {0}")]
    InvalidResizeHandle(String),
    #[error("insufficient nodes for distribution (need >= 3, got {0})")]
    InsufficientNodesForDistribution(usize),
    #[error("NaN or Infinity in input coordinates")]
    NonFiniteCoordinate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapNode {
    pub id: NodeId,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl SnapNode {
    #[must_use]
    pub const fn new(id: NodeId, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn left(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub const fn right(&self) -> f64 {
        self.x + self.width
    }

    #[must_use]
    pub const fn top(&self) -> f64 {
        self.y
    }

    #[must_use]
    pub const fn bottom(&self) -> f64 {
        self.y + self.height
    }

    #[must_use]
    pub const fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    #[must_use]
    pub const fn center_y(&self) -> f64 {
        self.y + self.height / 2.0
    }

    #[must_use]
    pub const fn center(&self) -> Point {
        Point::new(self.center_x(), self.center_y())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SnapState {
    pub mode: SnapMode,
    pub grid_size: f64,
    pub threshold: f64,
}

impl SnapState {
    #[must_use]
    pub const fn new(mode: SnapMode, grid_size: f64, threshold: f64) -> Self {
        Self {
            mode,
            grid_size,
            threshold,
        }
    }

    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        matches!(self.mode, SnapMode::Enabled)
    }

    #[must_use]
    pub const fn toggle(&self) -> Self {
        Self {
            mode: match self.mode {
                SnapMode::Enabled => SnapMode::Disabled,
                SnapMode::Disabled => SnapMode::Enabled,
            },
            ..*self
        }
    }
}
