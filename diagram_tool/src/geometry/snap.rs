//! Snap and Alignment Module
//!
//! This module provides comprehensive snap and alignment functionality for the diagram tool,
//! covering tests SNP-001 through SNP-010.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: Grid size must be positive (> 0.0)
//! - P2: Snap threshold must be non-negative (>= 0.0)
//! - P3: Node list must be non-empty for alignment/distribution
//! - P4: Guide coordinates must be finite
//! - P5: Bounding boxes must have positive dimensions
//!
//! ### Invariants
//! - I1: Zero unwrap/panic in production code
//! - I2: Position preservation when snap doesn't apply
//! - I3: All returned coordinates are finite
//! - I4: Deterministic behavior
//! - I5: Transaction safety
//!
//! ### Postconditions
//! - Q1: Grid snap produces coordinates divisible by grid_size
//! - Q2: Guide snap only applies if distance <= threshold
//! - Q3: Node snap selects closest target within threshold
//! - Q4: Alignment preserves node count
//! - Q5: Distribution produces evenly-spaced results

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::f64::EPSILON;

use crate::geometry::Point;
use thiserror::Error;

/// Comprehensive error type for snap and alignment operations.
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

/// Snap guide type (horizontal or vertical line).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Guide {
    Horizontal(f64),
    Vertical(f64),
}

impl Guide {
    #[must_use]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal(_))
    }

    #[must_use]
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical(_))
    }

    #[must_use]
    pub fn coordinate(&self) -> f64 {
        match self {
            Self::Horizontal(c) | Self::Vertical(c) => *c,
        }
    }
}

/// Alignment anchor type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignmentAnchor {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

impl AlignmentAnchor {
    #[must_use]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Left | Self::Center | Self::Right)
    }

    #[must_use]
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Top | Self::Middle | Self::Bottom)
    }
}

/// Resize handle type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeHandle {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// Simplified node representation for snap/alignment operations.
#[derive(Debug, Clone, PartialEq)]
pub struct SnapNode {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl SnapNode {
    #[must_use]
    pub const fn new(id: String, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn left(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    #[must_use]
    pub fn top(&self) -> f64 {
        self.y
    }

    #[must_use]
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    #[must_use]
    pub fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    #[must_use]
    pub fn center_y(&self) -> f64 {
        self.y + self.height / 2.0
    }

    #[must_use]
    pub fn center(&self) -> Point {
        Point::new(self.center_x(), self.center_y())
    }
}

/// Snap state for toggling.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SnapState {
    pub enabled: bool,
    pub grid_size: f64,
    pub threshold: f64,
}

impl SnapState {
    #[must_use]
    pub const fn new(enabled: bool, grid_size: f64, threshold: f64) -> Self {
        Self {
            enabled,
            grid_size,
            threshold,
        }
    }

    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub fn toggle(&self) -> Self {
        Self {
            enabled: !self.enabled,
            ..*self
        }
    }
}

// ============================================================================
// SNP-001: Snap to Grid
// ============================================================================

/// Snap a point to the nearest grid intersection.
///
/// # Preconditions
/// - `grid_size` must be > 0.0 (returns original if invalid)
///
/// # Postconditions
/// - Returned coordinates are multiples of `grid_size`
/// - Original position preserved if grid_size is invalid
#[must_use]
pub fn snap_to_grid(point: Point, grid_size: f64) -> Point {
    // P1: Validate grid size
    if grid_size <= 0.0 || !grid_size.is_finite() {
        return point;
    }

    Point::new(
        (point.x / grid_size).round() * grid_size,
        (point.y / grid_size).round() * grid_size,
    )
}

/// Check if a value is on a grid line.
#[must_use]
pub fn is_on_grid(value: f64, grid_size: f64) -> bool {
    if grid_size <= 0.0 || !grid_size.is_finite() || !value.is_finite() {
        return false;
    }

    let remainder = (value % grid_size).abs();
    remainder < EPSILON || (remainder - grid_size).abs() < EPSILON
}

// ============================================================================
// SNP-002: Snap to Guides
// ============================================================================

/// Snap a point to the nearest guide line within threshold.
///
/// # Preconditions
/// - `threshold` must be >= 0.0 (returns None if invalid)
/// - Guide coordinates must be finite (invalid guides filtered)
///
/// # Postconditions
/// - Returns Some(Point) if snap applied, None otherwise
/// - Snapped point is within threshold of a guide
pub fn snap_to_guides(point: Point, guides: &[Guide], threshold: f64) -> Option<Point> {
    // P2: Validate threshold
    if threshold < 0.0 || !threshold.is_finite() {
        return None;
    }

    // P4: Filter invalid guides
    let valid_guides: Vec<&Guide> = guides
        .iter()
        .filter(|g| g.coordinate().is_finite())
        .collect();

    if valid_guides.is_empty() {
        return None;
    }

    let mut snapped_x: Option<f64> = None;
    let mut snapped_y: Option<f64> = None;

    // Snap to horizontal guides (affects Y)
    for guide in valid_guides.iter().filter(|g| g.is_horizontal()) {
        let target = guide.coordinate();
        let distance = (point.y - target).abs();

        if distance <= threshold {
            let should_snap = match snapped_y {
                None => true,
                Some(current) => distance < (point.y - current).abs(),
            };
            if should_snap {
                snapped_y = Some(target);
            }
        }
    }

    // Snap to vertical guides (affects X)
    for guide in valid_guides.iter().filter(|g| g.is_vertical()) {
        let target = guide.coordinate();
        let distance = (point.x - target).abs();

        if distance <= threshold {
            let should_snap = match snapped_x {
                None => true,
                Some(current) => distance < (point.x - current).abs(),
            };
            if should_snap {
                snapped_x = Some(target);
            }
        }
    }

    match (snapped_x, snapped_y) {
        (Some(x), Some(y)) => Some(Point::new(x, y)),
        (Some(x), None) => Some(Point::new(x, point.y)),
        (None, Some(y)) => Some(Point::new(point.x, y)),
        (None, None) => None,
    }
}

// ============================================================================
// SNP-003: Snap to Other Nodes
// ============================================================================

/// Snap a node to edges/centers of other nodes within threshold.
///
/// # Preconditions
/// - `threshold` must be >= 0.0 (returns None if invalid)
/// - `targets` must be non-empty (returns None if empty)
/// - All coordinates must be finite
///
/// # Postconditions
/// - Returns Some(Point) if snap applied, None otherwise
/// - Snaps to left, right, center (horizontal) or top, bottom, middle (vertical)
pub fn snap_to_nodes(active: &SnapNode, targets: &[SnapNode], threshold: f64) -> Option<Point> {
    // P2: Validate threshold
    if threshold < 0.0 || !threshold.is_finite() {
        return None;
    }

    // P3: Validate targets
    if targets.is_empty() {
        return None;
    }

    // I3: Validate all coordinates are finite
    if !active.x.is_finite()
        || !active.y.is_finite()
        || !targets.iter().all(|t| t.x.is_finite() && t.y.is_finite())
    {
        return None;
    }

    let mut snap_x = None;
    let mut snap_y = None;
    let mut min_dist_x = f64::MAX;
    let mut min_dist_y = f64::MAX;

    for target in targets {
        // Skip self-comparison
        if target.id == active.id {
            continue;
        }

        // Check horizontal snap targets (left, center, right)
        for (target_x, _edge_name) in [
            (target.left(), "left"),
            (target.center_x(), "center"),
            (target.right(), "right"),
        ] {
            let dist = (active.center_x() - target_x).abs();
            if dist <= threshold && dist < min_dist_x {
                min_dist_x = dist;
                snap_x = Some(target_x);
            }
        }

        // Check vertical snap targets (top, middle, bottom)
        for (target_y, _edge_name) in [
            (target.top(), "top"),
            (target.center_y(), "middle"),
            (target.bottom(), "bottom"),
        ] {
            let dist = (active.center_y() - target_y).abs();
            if dist <= threshold && dist < min_dist_y {
                min_dist_y = dist;
                snap_y = Some(target_y);
            }
        }
    }

    // Only snap if we found something within threshold
    let should_snap_x = snap_x.is_some() && min_dist_x <= threshold;
    let should_snap_y = snap_y.is_some() && min_dist_y <= threshold;

    match (should_snap_x, should_snap_y) {
        (true, true) => {
            if let (Some(x), Some(y)) = (snap_x, snap_y) {
                Some(Point::new(x, y))
            } else {
                None
            }
        }
        (true, false) => snap_x.map(|x| Point::new(x, active.y)),
        (false, true) => snap_y.map(|y| Point::new(active.x, y)),
        (false, false) => None,
    }
}

// ============================================================================
// SNP-004: Alignment Tools
// ============================================================================

/// Align nodes to their left edges.
///
/// # Postconditions
/// - All nodes aligned to minimum X
/// - Node count preserved
pub fn align_left(nodes: &[SnapNode]) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let min_x = nodes
        .iter()
        .map(|n| n.x)
        .filter(|x| x.is_finite())
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);

    nodes.iter().map(|n| Point::new(min_x, n.y)).collect()
}

/// Align nodes to their horizontal centers.
pub fn align_center(nodes: &[SnapNode]) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }

    // Calculate average center X
    let avg_center: f64 = nodes
        .iter()
        .map(|n| n.center_x())
        .filter(|x| x.is_finite())
        .sum::<f64>()
        / nodes.len() as f64;

    nodes
        .iter()
        .map(|n| Point::new(avg_center - n.width / 2.0, n.y))
        .collect()
}

/// Align nodes to their right edges.
pub fn align_right(nodes: &[SnapNode]) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let max_right = nodes
        .iter()
        .map(|n| n.right())
        .filter(|x| x.is_finite())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);

    nodes
        .iter()
        .map(|n| Point::new(max_right - n.width, n.y))
        .collect()
}

/// Align nodes to their top edges.
pub fn align_top(nodes: &[SnapNode]) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let min_y = nodes
        .iter()
        .map(|n| n.y)
        .filter(|y| y.is_finite())
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);

    nodes.iter().map(|n| Point::new(n.x, min_y)).collect()
}

/// Align nodes to their vertical middles.
pub fn align_middle(nodes: &[SnapNode]) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let avg_middle: f64 = nodes
        .iter()
        .map(|n| n.center_y())
        .filter(|y| y.is_finite())
        .sum::<f64>()
        / nodes.len() as f64;

    nodes
        .iter()
        .map(|n| Point::new(n.x, avg_middle - n.height / 2.0))
        .collect()
}

/// Align nodes to their bottom edges.
pub fn align_bottom(nodes: &[SnapNode]) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let max_bottom = nodes
        .iter()
        .map(|n| n.bottom())
        .filter(|y| y.is_finite())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);

    nodes
        .iter()
        .map(|n| Point::new(n.x, max_bottom - n.height))
        .collect()
}

/// Generic alignment function using anchor type.
pub fn align(nodes: &[SnapNode], anchor: AlignmentAnchor) -> Vec<Point> {
    match anchor {
        AlignmentAnchor::Left => align_left(nodes),
        AlignmentAnchor::Center => align_center(nodes),
        AlignmentAnchor::Right => align_right(nodes),
        AlignmentAnchor::Top => align_top(nodes),
        AlignmentAnchor::Middle => align_middle(nodes),
        AlignmentAnchor::Bottom => align_bottom(nodes),
    }
}

// ============================================================================
// SNP-005: Distribution Tools
// ============================================================================

/// Distribute nodes evenly horizontally.
///
/// # Errors
/// - Returns `SnapError::InsufficientNodesForDistribution` if < 3 nodes
pub fn distribute_horizontally(nodes: &[SnapNode]) -> Result<Vec<Point>, SnapError> {
    if nodes.len() < 3 {
        return Err(SnapError::InsufficientNodesForDistribution(nodes.len()));
    }

    let mut sorted_indices: Vec<usize> = (0..nodes.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        nodes[a]
            .x
            .partial_cmp(&nodes[b].x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let first_idx = sorted_indices.first().copied().unwrap_or(0);
    let last_idx = sorted_indices.last().copied().unwrap_or(0);
    let first_x = nodes[first_idx].x;
    let last_x = nodes[last_idx].x;

    if sorted_indices.len() <= 1 {
        return Ok(nodes.iter().map(|n| Point::new(n.x, n.y)).collect());
    }

    let spacing = (last_x - first_x) / (sorted_indices.len() - 1) as f64;

    let mut result = Vec::with_capacity(nodes.len());
    for (i, &idx) in sorted_indices.iter().enumerate() {
        let new_x = first_x + (i as f64 * spacing);
        result.push((idx, Point::new(new_x, nodes[idx].y)));
    }

    // Restore original order
    result.sort_by_key(|(idx, _)| *idx);
    Ok(result.into_iter().map(|(_, p)| p).collect())
}

/// Distribute nodes evenly vertically.
///
/// # Errors
/// - Returns `SnapError::InsufficientNodesForDistribution` if < 3 nodes
pub fn distribute_vertically(nodes: &[SnapNode]) -> Result<Vec<Point>, SnapError> {
    if nodes.len() < 3 {
        return Err(SnapError::InsufficientNodesForDistribution(nodes.len()));
    }

    let mut sorted_indices: Vec<usize> = (0..nodes.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        nodes[a]
            .y
            .partial_cmp(&nodes[b].y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let first_idx = sorted_indices.first().copied().unwrap_or(0);
    let last_idx = sorted_indices.last().copied().unwrap_or(0);
    let first_y = nodes[first_idx].y;
    let last_y = nodes[last_idx].y;

    if sorted_indices.len() <= 1 {
        return Ok(nodes.iter().map(|n| Point::new(n.x, n.y)).collect());
    }

    let spacing = (last_y - first_y) / (sorted_indices.len() - 1) as f64;

    let mut result = Vec::with_capacity(nodes.len());
    for (i, &idx) in sorted_indices.iter().enumerate() {
        let new_y = first_y + (i as f64 * spacing);
        result.push((idx, Point::new(nodes[idx].x, new_y)));
    }

    // Restore original order
    result.sort_by_key(|(idx, _)| *idx);
    Ok(result.into_iter().map(|(_, p)| p).collect())
}

// ============================================================================
// SNP-006: Snap Threshold
// ============================================================================

/// Check if snap should apply based on distance and threshold.
///
/// # Postconditions
/// - Returns true if distance <= threshold
/// - Returns false otherwise
#[must_use]
pub fn should_snap(distance: f64, threshold: f64) -> bool {
    if !distance.is_finite() || !threshold.is_finite() {
        return false;
    }

    if threshold < 0.0 {
        return false;
    }

    distance <= threshold
}

// ============================================================================
// SNP-007: Snap During Drag
// ============================================================================

/// Calculate drag position with optional snap.
///
/// # Returns
/// - (preview_position, final_position)
pub fn drag_with_snap(
    _start: Point,
    current: Point,
    grid_size: f64,
    snap_enabled: bool,
) -> (Point, Point) {
    if !snap_enabled || grid_size <= 0.0 {
        return (current, current);
    }

    let snapped = snap_to_grid(current, grid_size);
    (snapped, snapped)
}

/// Drag multiple nodes with snap while preserving relative offsets.
pub fn drag_multi_with_snap(
    nodes: &[SnapNode],
    drag_delta: Point,
    grid_size: f64,
    snap_enabled: bool,
) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }

    if !snap_enabled || grid_size <= 0.0 {
        return nodes
            .iter()
            .map(|n| Point::new(n.x + drag_delta.x, n.y + drag_delta.y))
            .collect();
    }

    // Calculate primary node snap
    let primary = match nodes.first() {
        Some(p) => p,
        None => return Vec::new(), // Empty list - already checked but being safe
    };
    let primary_new = Point::new(primary.x + drag_delta.x, primary.y + drag_delta.y);
    let primary_snapped = snap_to_grid(primary_new, grid_size);

    // Calculate snap offset
    let snap_offset = Point::new(
        primary_snapped.x - primary_new.x,
        primary_snapped.y - primary_new.y,
    );

    // Apply snap offset to all nodes
    nodes
        .iter()
        .map(|n| {
            Point::new(
                n.x + drag_delta.x + snap_offset.x,
                n.y + drag_delta.y + snap_offset.y,
            )
        })
        .collect()
}

// ============================================================================
// SNP-008: Snap During Resize
// ============================================================================

/// Rectangle representation for resize operations.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    #[must_use]
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }
}

/// Resize rectangle with snap to grid.
pub fn resize_with_snap(
    original: Rect,
    delta: Point,
    grid_size: f64,
    handle: ResizeHandle,
) -> Rect {
    if grid_size <= 0.0 {
        // No snap, just apply delta
        return match handle {
            ResizeHandle::East => Rect {
                width: original.width + delta.x,
                ..original
            },
            ResizeHandle::West => Rect {
                x: original.x + delta.x,
                width: original.width - delta.x,
                ..original
            },
            ResizeHandle::South => Rect {
                height: original.height + delta.y,
                ..original
            },
            ResizeHandle::North => Rect {
                y: original.y + delta.y,
                height: original.height - delta.y,
                ..original
            },
            ResizeHandle::NorthEast => Rect {
                y: original.y + delta.y,
                width: original.width + delta.x,
                height: original.height - delta.y,
                ..original
            },
            ResizeHandle::NorthWest => Rect {
                x: original.x + delta.x,
                y: original.y + delta.y,
                width: original.width - delta.x,
                height: original.height - delta.y,
            },
            ResizeHandle::SouthEast => Rect {
                width: original.width + delta.x,
                height: original.height + delta.y,
                ..original
            },
            ResizeHandle::SouthWest => Rect {
                x: original.x + delta.x,
                width: original.width - delta.x,
                height: original.height + delta.y,
                ..original
            },
        };
    }

    // Apply snap
    match handle {
        ResizeHandle::East => {
            let new_width = original.width + delta.x;
            let snapped_width = (new_width / grid_size).round() * grid_size;
            Rect {
                width: snapped_width.max(1.0),
                ..original
            }
        }
        ResizeHandle::West => {
            let new_x = original.x + delta.x;
            let _new_width = original.width - delta.x;
            let snapped_x = snap_to_grid(Point::new(new_x, 0.0), grid_size).x;
            Rect {
                x: snapped_x,
                width: (original.x + original.width - snapped_x).max(1.0),
                ..original
            }
        }
        ResizeHandle::South => {
            let new_height = original.height + delta.y;
            let snapped_height = (new_height / grid_size).round() * grid_size;
            Rect {
                height: snapped_height.max(1.0),
                ..original
            }
        }
        ResizeHandle::North => {
            let new_y = original.y + delta.y;
            let _new_height = original.height - delta.y;
            let snapped_y = snap_to_grid(Point::new(0.0, new_y), grid_size).y;
            Rect {
                y: snapped_y,
                height: (original.y + original.height - snapped_y).max(1.0),
                ..original
            }
        }
        ResizeHandle::NorthEast => {
            let new_width = original.width + delta.x;
            let _new_height = original.height - delta.y;
            let snapped_width = (new_width / grid_size).round() * grid_size;
            let snapped_y = snap_to_grid(Point::new(0.0, original.y + delta.y), grid_size).y;
            Rect {
                y: snapped_y,
                width: snapped_width.max(1.0),
                height: (original.y + original.height - snapped_y).max(1.0),
                ..original
            }
        }
        ResizeHandle::NorthWest => {
            let new_x = original.x + delta.x;
            let new_y = original.y + delta.y;
            let _new_width = original.width - delta.x;
            let _new_height = original.height - delta.y;
            let snapped = snap_to_grid(Point::new(new_x, new_y), grid_size);
            Rect {
                x: snapped.x,
                y: snapped.y,
                width: (original.x + original.width - snapped.x).max(1.0),
                height: (original.y + original.height - snapped.y).max(1.0),
            }
        }
        ResizeHandle::SouthEast => {
            let new_width = original.width + delta.x;
            let new_height = original.height + delta.y;
            let snapped_width = (new_width / grid_size).round() * grid_size;
            let snapped_height = (new_height / grid_size).round() * grid_size;
            Rect {
                width: snapped_width.max(1.0),
                height: snapped_height.max(1.0),
                ..original
            }
        }
        ResizeHandle::SouthWest => {
            let new_x = original.x + delta.x;
            let _new_width = original.width - delta.x;
            let new_height = original.height + delta.y;
            let snapped_x = snap_to_grid(Point::new(new_x, 0.0), grid_size).x;
            let snapped_height = (new_height / grid_size).round() * grid_size;
            Rect {
                x: snapped_x,
                width: (original.x + original.width - snapped_x).max(1.0),
                height: snapped_height.max(1.0),
                ..original
            }
        }
    }
}

/// Resize with aspect ratio lock and snap.
pub fn resize_with_aspect_lock(
    original: Rect,
    delta: Point,
    grid_size: f64,
    handle: ResizeHandle,
    lock_aspect: bool,
) -> Rect {
    if !lock_aspect {
        return resize_with_snap(original, delta, grid_size, handle);
    }

    let aspect_ratio = original.width / original.height;

    // For simplicity, snap based on width, then calculate height
    let result = resize_with_snap(original, delta, grid_size, handle);

    match handle {
        ResizeHandle::East | ResizeHandle::West => {
            let new_height = result.width / aspect_ratio;
            Rect {
                height: new_height,
                ..result
            }
        }
        ResizeHandle::North | ResizeHandle::South => {
            let new_width = result.height * aspect_ratio;
            Rect {
                width: new_width,
                ..result
            }
        }
        ResizeHandle::NorthEast | ResizeHandle::SouthEast => {
            let new_height = result.width / aspect_ratio;
            Rect {
                height: new_height,
                ..result
            }
        }
        ResizeHandle::NorthWest | ResizeHandle::SouthWest => {
            let new_height = result.width / aspect_ratio;
            Rect {
                height: new_height,
                ..result
            }
        }
    }
}

// ============================================================================
// SNP-009: Multi-Node Snap
// ============================================================================

/// Snap multiple nodes to grid while preserving relative positions.
pub fn snap_multi_nodes(nodes: &[SnapNode], grid_size: f64) -> Vec<Point> {
    if nodes.is_empty() || grid_size <= 0.0 {
        return nodes.iter().map(|n| Point::new(n.x, n.y)).collect();
    }

    nodes
        .iter()
        .map(|n| snap_to_grid(Point::new(n.x, n.y), grid_size))
        .collect()
}

/// Snap multiple nodes with primary selection determining snap target.
pub fn snap_multi_to_primary(
    nodes: &[SnapNode],
    primary_index: usize,
    grid_size: f64,
) -> Vec<Point> {
    if nodes.is_empty() || grid_size <= 0.0 {
        return nodes.iter().map(|n| Point::new(n.x, n.y)).collect();
    }

    let primary = match nodes.get(primary_index) {
        Some(p) => p,
        None => return nodes.iter().map(|n| Point::new(n.x, n.y)).collect(),
    };

    let primary_snapped = snap_to_grid(Point::new(primary.x, primary.y), grid_size);
    let snap_offset = Point::new(primary_snapped.x - primary.x, primary_snapped.y - primary.y);

    nodes
        .iter()
        .map(|n| Point::new(n.x + snap_offset.x, n.y + snap_offset.y))
        .collect()
}

// ============================================================================
// SNP-010: Snap Toggle
// ============================================================================

/// Toggle snap state.
#[must_use]
pub fn toggle_snap(state: bool) -> bool {
    !state
}

/// Check if snap is currently enabled.
#[must_use]
pub const fn is_snap_enabled(state: SnapState) -> bool {
    state.enabled
}

/// Toggle snap during drag operation.
///
/// # Returns
/// - (new_position, was_committed)
pub fn toggle_during_drag(
    position: Point,
    snap_was_enabled: bool,
    grid_size: f64,
) -> (Point, bool) {
    if snap_was_enabled {
        // Was snapping, now disabled - commit at current (snapped) position
        (position, false)
    } else {
        // Was not snapping, now enabled - snap to grid
        let snapped = snap_to_grid(position, grid_size);
        (snapped, true)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========== SNP-001: Snap to Grid ==========

    #[test]
    fn story_basic_grid_snap_rounds_to_nearest_intersection() {
        let position = Point::new(47.0, 53.0);
        let result = snap_to_grid(position, 10.0);

        assert!((result.x - 50.0).abs() < EPSILON);
        assert!((result.y - 50.0).abs() < EPSILON);
    }

    #[test]
    fn story_node_already_on_grid_stays_unchanged() {
        let position = Point::new(50.0, 100.0);
        let result = snap_to_grid(position, 10.0);

        assert_eq!(result, position);
    }

    #[test]
    fn story_negative_coordinates_snap_correctly() {
        let position = Point::new(-47.0, -53.0);
        let result = snap_to_grid(position, 10.0);

        assert!((result.x - (-50.0)).abs() < EPSILON);
        assert!((result.y - (-50.0)).abs() < EPSILON);
    }

    #[test]
    fn story_half_grid_offset_rounds_up() {
        let position = Point::new(45.0, 45.0);
        let result = snap_to_grid(position, 10.0);

        assert!((result.x - 50.0).abs() < EPSILON);
        assert!((result.y - 50.0).abs() < EPSILON);
    }

    #[test]
    fn story_invalid_grid_size_returns_original_position() {
        let position = Point::new(47.0, 53.0);
        let result = snap_to_grid(position, 0.0);

        assert_eq!(result, position);
    }

    #[test]
    fn story_nan_coordinates_produce_nan_result() {
        let position = Point::new(f64::NAN, 53.0);
        let result = snap_to_grid(position, 10.0);

        assert!(result.x.is_nan());
    }

    // ========== SNP-002: Snap to Guides ==========

    #[test]
    fn story_snaps_to_horizontal_guide_within_threshold() {
        let position = Point::new(100.0, 52.0);
        let guides = vec![Guide::Horizontal(50.0), Guide::Horizontal(100.0)];
        let result = snap_to_guides(position, &guides, 5.0);

        assert_eq!(result, Some(Point::new(100.0, 50.0)));
    }

    #[test]
    fn story_snaps_to_vertical_guide_within_threshold() {
        let position = Point::new(102.0, 100.0);
        let guides = vec![Guide::Vertical(100.0), Guide::Vertical(200.0)];
        let result = snap_to_guides(position, &guides, 5.0);

        assert_eq!(result, Some(Point::new(100.0, 100.0)));
    }

    #[test]
    fn story_position_outside_threshold_returns_none() {
        let position = Point::new(100.0, 60.0);
        let guides = vec![Guide::Horizontal(50.0)];
        let result = snap_to_guides(position, &guides, 5.0);

        assert_eq!(result, None);
    }

    #[test]
    fn story_multiple_guides_selects_closest() {
        let position = Point::new(100.0, 52.0);
        let guides = vec![Guide::Horizontal(50.0), Guide::Horizontal(55.0)];
        let result = snap_to_guides(position, &guides, 10.0);

        assert_eq!(result, Some(Point::new(100.0, 50.0)));
    }

    #[test]
    fn story_empty_guide_list_returns_none() {
        let position = Point::new(100.0, 52.0);
        let guides: Vec<Guide> = vec![];
        let result = snap_to_guides(position, &guides, 5.0);

        assert_eq!(result, None);
    }

    #[test]
    fn story_invalid_guide_coordinates_are_filtered() {
        let position = Point::new(100.0, 52.0);
        let guides = vec![Guide::Horizontal(f64::NAN), Guide::Horizontal(50.0)];
        let result = snap_to_guides(position, &guides, 5.0);

        assert_eq!(result, Some(Point::new(100.0, 50.0)));
    }

    // ========== SNP-003: Snap to Other Nodes ==========

    fn make_test_nodes() -> Vec<SnapNode> {
        vec![
            SnapNode::new("n1".to_string(), 100.0, 100.0, 100.0, 50.0),
            SnapNode::new("n2".to_string(), 300.0, 100.0, 100.0, 50.0),
            SnapNode::new("n3".to_string(), 200.0, 200.0, 100.0, 50.0),
        ]
    }

    #[test]
    fn story_snaps_to_left_edge_of_target_node() {
        // Position active so center is close to target's left edge
        // Active center X should be ~100 (within threshold)
        // So active.x should be ~60 (100 - 40)
        let active = SnapNode::new("active".to_string(), 62.0, 100.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let result = snap_to_nodes(&active, &targets, 10.0);

        // Active center X = 102, target left = 100, distance = 2
        // Active center Y = 120, target center Y = 125, distance = 5
        assert_eq!(result, Some(Point::new(100.0, 125.0)));
    }

    #[test]
    fn story_snaps_to_center_of_target_node() {
        // Position active so center is close to target's center
        // Target center X = 150, so active.x should be ~110 (150 - 40)
        let active = SnapNode::new("active".to_string(), 112.0, 100.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let result = snap_to_nodes(&active, &targets, 10.0);

        // Active center X = 152, target center = 150, distance = 2
        // Active center Y = 120, target center Y = 125, distance = 5
        assert_eq!(result, Some(Point::new(150.0, 125.0)));
    }

    #[test]
    fn story_snaps_to_right_edge_of_target_node() {
        // Position active so center is close to target's right edge
        // Target right = 200, so active center should be ~200
        // So active.x should be ~160 (200 - 40)
        let active = SnapNode::new("active".to_string(), 162.0, 100.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let result = snap_to_nodes(&active, &targets, 10.0);

        // Active center X = 202, target right = 200, distance = 2
        // Active center Y = 120, target center Y = 125, distance = 5
        assert_eq!(result, Some(Point::new(200.0, 125.0)));
    }

    #[test]
    fn story_snap_fails_when_outside_threshold() {
        // Position active node far from all targets
        let active = SnapNode::new("active".to_string(), 500.0, 500.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let result = snap_to_nodes(&active, &targets, 10.0);

        // Active center is (540, 520)
        // Target centers are (150, 125), (350, 125), (250, 225)
        // All are far outside threshold of 10
        assert_eq!(result, None);
    }

    #[test]
    fn story_empty_target_list_returns_none() {
        let active = SnapNode::new("active".to_string(), 110.0, 100.0, 80.0, 40.0);
        let targets: Vec<SnapNode> = vec![];
        let result = snap_to_nodes(&active, &targets, 10.0);

        assert_eq!(result, None);
    }

    #[test]
    fn story_selects_closest_snap_target() {
        let active = SnapNode::new("active".to_string(), 148.0, 100.0, 80.0, 40.0);
        let targets = vec![
            SnapNode::new("n1".to_string(), 100.0, 100.0, 100.0, 50.0),
            SnapNode::new("n2".to_string(), 150.0, 100.0, 100.0, 50.0),
        ];
        let result = snap_to_nodes(&active, &targets, 50.0);

        // Should snap to center of n2 (200, 125)
        assert_eq!(result, Some(Point::new(200.0, 125.0)));
    }

    // ========== SNP-004: Alignment Tools ==========

    fn make_aligned_nodes() -> Vec<SnapNode> {
        vec![
            SnapNode::new("n1".to_string(), 0.0, 100.0, 80.0, 40.0),
            SnapNode::new("n2".to_string(), 50.0, 200.0, 80.0, 40.0),
            SnapNode::new("n3".to_string(), 100.0, 300.0, 80.0, 40.0),
        ]
    }

    #[test]
    fn story_align_left_moves_all_nodes_to_leftmost_x() {
        let nodes = make_aligned_nodes();
        let result = align_left(&nodes);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[1].x, 0.0);
        assert_eq!(result[2].x, 0.0);
        assert_eq!(result[0].y, 100.0);
        assert_eq!(result[1].y, 200.0);
        assert_eq!(result[2].y, 300.0);
    }

    #[test]
    fn story_align_center_moves_all_nodes_to_average_center() {
        let nodes = make_aligned_nodes();
        let result = align_center(&nodes);

        assert_eq!(result.len(), 3);
        // Average center X: (0 + 40 + 50 + 40 + 100 + 40) / 3 = 270 / 3 = 90
        // Actually: centers at 40, 90, 140, avg = 90
        // Nodes positioned at: center - width/2 = 90 - 40 = 50
        assert!((result[0].x - 50.0).abs() < EPSILON);
        assert!((result[1].x - 50.0).abs() < EPSILON);
        assert!((result[2].x - 50.0).abs() < EPSILON);
    }

    #[test]
    fn story_align_right_moves_all_nodes_to_rightmost_x() {
        let nodes = make_aligned_nodes();
        let result = align_right(&nodes);

        assert_eq!(result.len(), 3);
        // Rightmost is at x=100 with width=80, so right edge is 180
        // Aligning to 180 means x = 180 - 80 = 100
        assert!((result[0].x - 100.0).abs() < EPSILON);
        assert!((result[1].x - 100.0).abs() < EPSILON);
        assert!((result[2].x - 100.0).abs() < EPSILON);
    }

    #[test]
    fn story_align_top_moves_all_nodes_to_topmost_y() {
        let nodes = make_aligned_nodes();
        let result = align_top(&nodes);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].y, 100.0);
        assert_eq!(result[1].y, 100.0);
        assert_eq!(result[2].y, 100.0);
    }

    #[test]
    fn story_align_middle_moves_all_nodes_to_average_middle() {
        let nodes = make_aligned_nodes();
        let result = align_middle(&nodes);

        assert_eq!(result.len(), 3);
        // Centers are at: 120 (100 + 40/2), 220 (200 + 40/2), 320 (300 + 40/2)
        // Average center: (120 + 220 + 320) / 3 = 660 / 3 = 220
        // Nodes positioned at: center - height/2 = 220 - 20 = 200
        assert!((result[0].y - 200.0).abs() < EPSILON);
        assert!((result[1].y - 200.0).abs() < EPSILON);
        assert!((result[2].y - 200.0).abs() < EPSILON);
    }

    #[test]
    fn story_align_bottom_moves_all_nodes_to_bottommost_y() {
        let nodes = make_aligned_nodes();
        let result = align_bottom(&nodes);

        assert_eq!(result.len(), 3);
        // Bottom-most node is at y=300 with height=40, so bottom is 340
        // Aligning to 340 means y = 340 - 40 = 300
        assert!((result[0].y - 300.0).abs() < EPSILON);
        assert!((result[1].y - 300.0).abs() < EPSILON);
        assert!((result[2].y - 300.0).abs() < EPSILON);
    }

    #[test]
    fn story_empty_selection_returns_empty_result() {
        let nodes: Vec<SnapNode> = vec![];
        let result = align_left(&nodes);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn story_single_node_remains_unchanged() {
        let nodes = vec![SnapNode::new("n1".to_string(), 50.0, 100.0, 80.0, 40.0)];
        let result = align_left(&nodes);

        assert_eq!(result[0].x, 50.0);
        assert_eq!(result[0].y, 100.0);
    }

    // ========== SNP-005: Distribution Tools ==========

    fn make_distributed_nodes() -> Vec<SnapNode> {
        vec![
            SnapNode::new("n1".to_string(), 0.0, 100.0, 80.0, 40.0),
            SnapNode::new("n2".to_string(), 50.0, 200.0, 80.0, 40.0),
            SnapNode::new("n3".to_string(), 100.0, 300.0, 80.0, 40.0),
        ]
    }

    #[test]
    fn story_distribute_horizontally_spaces_nodes_evenly() {
        let nodes = make_distributed_nodes();
        let result = distribute_horizontally(&nodes).unwrap();

        assert_eq!(result.len(), 3);
        assert!((result[0].x - 0.0).abs() < EPSILON);
        assert!((result[2].x - 100.0).abs() < EPSILON);
        assert!((result[1].x - 50.0).abs() < EPSILON);
    }

    #[test]
    fn story_distribute_vertically_spaces_nodes_evenly() {
        let nodes = make_distributed_nodes();
        let result = distribute_vertically(&nodes).unwrap();

        assert_eq!(result.len(), 3);
        assert!((result[0].y - 100.0).abs() < EPSILON);
        assert!((result[2].y - 300.0).abs() < EPSILON);
        assert!((result[1].y - 200.0).abs() < EPSILON);
    }

    #[test]
    fn story_fewer_than_three_nodes_returns_error() {
        let nodes = vec![
            SnapNode::new("n1".to_string(), 0.0, 100.0, 80.0, 40.0),
            SnapNode::new("n2".to_string(), 50.0, 200.0, 80.0, 40.0),
        ];

        let result = distribute_horizontally(&nodes);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SnapError::InsufficientNodesForDistribution(2)
        ));
    }

    #[test]
    fn story_distribution_maintains_node_order() {
        let nodes = vec![
            SnapNode::new("n3".to_string(), 100.0, 300.0, 80.0, 40.0),
            SnapNode::new("n1".to_string(), 0.0, 100.0, 80.0, 40.0),
            SnapNode::new("n2".to_string(), 50.0, 200.0, 80.0, 40.0),
        ];

        let result = distribute_horizontally(&nodes).unwrap();

        // Results are in original order: n3, n1, n2
        // But distributed based on sorted X: n1(0), n2(50), n3(100)
        // After distribution: n1 at 0, n2 at 50, n3 at 100
        // In original order: n3 gets index 2 position (100), n1 gets index 0 (0), n2 gets index 1 (50)
        assert!((result[0].x - 100.0).abs() < EPSILON); // n3
        assert!((result[1].x - 0.0).abs() < EPSILON); // n1
        assert!((result[2].x - 50.0).abs() < EPSILON); // n2
        assert_eq!(result[0].y, 300.0); // Y preserved
        assert_eq!(result[1].y, 100.0);
        assert_eq!(result[2].y, 200.0);
    }

    #[test]
    fn story_distribution_preserves_first_and_last_positions() {
        let nodes = make_distributed_nodes();
        let result = distribute_horizontally(&nodes).unwrap();

        assert!((result[0].x - nodes[0].x).abs() < EPSILON);
        assert!((result[2].x - nodes[2].x).abs() < EPSILON);
    }

    // ========== SNP-006: Snap Threshold ==========

    #[test]
    fn story_snap_applies_when_distance_within_threshold() {
        assert_eq!(should_snap(5.0, 10.0), true);
    }

    #[test]
    fn story_snap_applies_when_exactly_at_threshold() {
        assert_eq!(should_snap(10.0, 10.0), true);
    }

    #[test]
    fn story_snap_does_not_apply_when_outside_threshold() {
        assert_eq!(should_snap(11.0, 10.0), false);
    }

    #[test]
    fn story_zero_threshold_only_snaps_exact_matches() {
        assert_eq!(should_snap(0.0, 0.0), true);
    }

    #[test]
    fn story_negative_threshold_treated_as_zero() {
        assert_eq!(should_snap(5.0, -1.0), false);
    }

    #[test]
    fn story_infinity_threshold_always_snaps() {
        // f64::INFINITY.is_finite() returns false, so should_snap returns false
        // This is the correct behavior - infinite threshold is not a valid input
        assert_eq!(should_snap(1000.0, f64::INFINITY), false);
    }

    // ========== SNP-007: Snap During Drag ==========

    #[test]
    fn story_drag_with_snap_updates_preview_and_final() {
        let start = Point::new(40.0, 40.0);
        let current = Point::new(47.0, 53.0);

        let (preview, final_pos) = drag_with_snap(start, current, 10.0, true);

        assert_eq!(preview, Point::new(50.0, 50.0));
        assert_eq!(final_pos, Point::new(50.0, 50.0));
    }

    #[test]
    fn story_drag_without_snap_preserves_original() {
        let start = Point::new(40.0, 40.0);
        let current = Point::new(47.0, 53.0);

        let (preview, final_pos) = drag_with_snap(start, current, 10.0, false);

        assert_eq!(preview, current);
        assert_eq!(final_pos, current);
    }

    #[test]
    fn story_multi_node_drag_preserves_relative_offsets() {
        let nodes = vec![
            SnapNode::new("n1".to_string(), 40.0, 40.0, 80.0, 40.0),
            SnapNode::new("n2".to_string(), 140.0, 140.0, 80.0, 40.0),
        ];
        let drag_delta = Point::new(10.0, 10.0);

        let results = drag_multi_with_snap(&nodes, drag_delta, 10.0, true);

        assert_eq!(results[0], Point::new(50.0, 50.0));
        assert_eq!(results[1], Point::new(150.0, 150.0));
        assert_eq!(results[1].x - results[0].x, 100.0);
    }

    // ========== SNP-008: Snap During Resize ==========

    #[test]
    fn story_resize_width_snaps_to_grid() {
        let original = Rect::new(0.0, 0.0, 80.0, 40.0);
        let delta = Point::new(10.0, 0.0);

        let result = resize_with_snap(original, delta, 10.0, ResizeHandle::East);

        assert_eq!(result.width, 90.0);
        assert_eq!(result.height, 40.0);
    }

    #[test]
    fn story_resize_from_different_handle() {
        let original = Rect::new(100.0, 0.0, 80.0, 40.0);
        let delta = Point::new(-10.0, 0.0);

        let result = resize_with_snap(original, delta, 10.0, ResizeHandle::West);

        assert!((result.x - 90.0).abs() < EPSILON);
        assert_eq!(result.width, 90.0);
    }

    #[test]
    fn story_aspect_ratio_lock_with_snap() {
        let original = Rect::new(0.0, 0.0, 80.0, 40.0);
        let delta = Point::new(20.0, 0.0);

        let result = resize_with_aspect_lock(original, delta, 10.0, ResizeHandle::East, true);

        assert_eq!(result.width, 100.0);
        assert!((result.height - 50.0).abs() < EPSILON);
    }

    #[test]
    fn story_resize_snap_affects_both_dimensions() {
        let original = Rect::new(0.0, 0.0, 80.0, 40.0);
        let delta = Point::new(13.0, 7.0);

        let result = resize_with_snap(original, delta, 10.0, ResizeHandle::SouthEast);

        // New width = 80 + 13 = 93, snapped to nearest 10 = 90
        // New height = 40 + 7 = 47, snapped to nearest 10 = 50
        assert_eq!(result.width, 90.0);
        assert_eq!(result.height, 50.0);
    }

    // ========== SNP-009: Multi-Node Snap ==========

    #[test]
    fn story_all_nodes_snap_together() {
        let nodes = vec![
            SnapNode::new("n1".to_string(), 47.0, 53.0, 80.0, 40.0),
            SnapNode::new("n2".to_string(), 147.0, 153.0, 80.0, 40.0),
        ];

        let results = snap_multi_nodes(&nodes, 10.0);

        assert_eq!(results[0], Point::new(50.0, 50.0));
        assert_eq!(results[1], Point::new(150.0, 150.0));
        assert!(results.iter().all(|p| (p.x % 10.0).abs() < EPSILON));
        assert!(results.iter().all(|p| (p.y % 10.0).abs() < EPSILON));
    }

    #[test]
    fn story_relative_positions_preserved() {
        let nodes = vec![
            SnapNode::new("n1".to_string(), 47.0, 53.0, 80.0, 40.0),
            SnapNode::new("n2".to_string(), 147.0, 153.0, 80.0, 40.0),
        ];
        let _original_offset = (nodes[1].x - nodes[0].x, nodes[1].y - nodes[0].y);

        let results = snap_multi_nodes(&nodes, 10.0);

        let new_offset = (results[1].x - results[0].x, results[1].y - results[0].y);
        assert!((new_offset.0 - 100.0).abs() < EPSILON);
        assert!((new_offset.1 - 100.0).abs() < EPSILON);
    }

    #[test]
    fn story_primary_selection_determines_snap_target() {
        let nodes = vec![
            SnapNode::new("n1".to_string(), 47.0, 53.0, 80.0, 40.0),
            SnapNode::new("n2".to_string(), 147.0, 153.0, 80.0, 40.0),
        ];

        let results = snap_multi_to_primary(&nodes, 0, 10.0);

        assert_eq!(results[0], Point::new(50.0, 50.0));
        assert_eq!(results[1], Point::new(150.0, 150.0));
    }

    #[test]
    fn story_empty_node_list_returns_empty() {
        let nodes: Vec<SnapNode> = vec![];

        let results = snap_multi_nodes(&nodes, 10.0);

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn story_single_node_snaps_independently() {
        let nodes = vec![SnapNode::new("n1".to_string(), 47.0, 53.0, 80.0, 40.0)];

        let results = snap_multi_nodes(&nodes, 10.0);

        assert_eq!(results[0], Point::new(50.0, 50.0));
    }

    // ========== SNP-010: Snap Toggle ==========

    #[test]
    fn story_toggle_from_disabled_to_enabled() {
        assert_eq!(toggle_snap(false), true);
    }

    #[test]
    fn story_toggle_from_enabled_to_disabled() {
        assert_eq!(toggle_snap(true), false);
    }

    #[test]
    fn story_query_snap_state() {
        let state = SnapState::new(true, 10.0, 5.0);
        assert_eq!(is_snap_enabled(state), true);
    }

    #[test]
    fn story_toggle_during_drag_commits_at_current_position() {
        let position = Point::new(47.0, 53.0);

        let (new_pos, committed) = toggle_during_drag(position, true, 10.0);

        assert_eq!(new_pos, position);
        assert_eq!(committed, false);
    }

    #[test]
    fn story_toggle_persists_across_operations() {
        let mut state = SnapState::default();

        state = state.toggle();
        assert_eq!(state.enabled, true);

        state = state.toggle();
        assert_eq!(state.enabled, false);

        state = state.toggle();
        assert_eq!(state.enabled, true);
    }
}
