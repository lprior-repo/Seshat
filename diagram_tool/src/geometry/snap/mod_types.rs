use crate::geometry::primitives::Point;
pub use diagram_models::document::types::NodeId;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnapMode {
    #[default]
    Disabled,
    Enabled,
}

impl SnapMode {
    /// Converts a bool to `SnapMode` for backward compatibility with existing callers.
    #[must_use]
    pub const fn from_bool(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }

    /// Returns true if snapping is enabled.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum SnapError {
    #[error("invalid grid size: {0} (must be > 0)")]
    InvalidGridSize(f64),
    #[error("invalid threshold: {0} (must be >= 0)")]
    InvalidThreshold(f64),
    #[error("invalid node list")]
    InvalidNodeList,
    #[error("invalid alignment anchor")]
    InvalidAlignmentAnchor,
    #[error("invalid resize handle")]
    InvalidResizeHandle,
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
// SnapType for smart alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapType {
    CenterX,
    CenterY,
    EdgeTop,
    EdgeBottom,
    EdgeLeft,
    EdgeRight,
}

// SnapResult for smart alignment
#[derive(Debug, Clone, PartialEq)]
pub enum SnapResult {
    Snapped {
        snap_type: SnapType,
        target_node_id: NodeId,
        snapped_position: Point,
    },
    Unsnapped,
}

impl SnapResult {
    #[must_use]
    pub const fn to_position(&self) -> Option<Point> {
        match self {
            Self::Snapped {
                snapped_position, ..
            } => Some(*snapped_position),
            Self::Unsnapped => None,
        }
    }
    #[must_use]
    pub const fn inactive() -> Self {
        Self::Unsnapped
    }
    #[must_use]
    pub const fn new(snap_type: SnapType, target_node_id: NodeId, snapped_position: Point) -> Self {
        Self::Snapped {
            snap_type,
            target_node_id,
            snapped_position,
        }
    }
}

impl Default for SnapResult {
    fn default() -> Self {
        Self::inactive()
    }
}

// SnapThreshold for validated thresholds
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapThreshold(f64);
impl SnapThreshold {
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(if value < 0.0 { 0.0 } else { value })
    }
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }
}
impl Default for SnapThreshold {
    fn default() -> Self {
        Self(10.0)
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
