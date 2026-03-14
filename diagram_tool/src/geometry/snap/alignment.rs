use crate::geometry::primitives::Point;
use crate::geometry::snap::grid::snap_to_grid;
use crate::geometry::snap::mod_types::{NodeId, SnapError, SnapNode, SnapResult, SnapType};

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
    pub const fn coordinate(&self) -> f64 {
        match self {
            Self::Horizontal(c) | Self::Vertical(c) => *c,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Center | Self::Right)
    }

    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Middle | Self::Bottom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectConstraint {
    Locked,
    Unlocked,
}

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

fn closest_guide(val: f64, guides: &[&Guide], threshold: f64, is_horiz: bool) -> Option<f64> {
    guides
        .iter()
        .filter(|g| {
            if is_horiz {
                g.is_horizontal()
            } else {
                g.is_vertical()
            }
        })
        .map(|g| g.coordinate())
        .filter(|&c| (val - c).abs() <= threshold)
        .min_by(|a, b| {
            (val - a)
                .abs()
                .partial_cmp(&(val - b).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

#[must_use]
pub fn snap_to_guides(point: Point, guides: &[Guide], threshold: f64) -> SnapResult {
    if threshold < 0.0 || !threshold.is_finite() {
        return SnapResult::inactive();
    }
    let valid_guides: Vec<&Guide> = guides
        .iter()
        .filter(|g| g.coordinate().is_finite())
        .collect();
    if valid_guides.is_empty() {
        return SnapResult::inactive();
    }

    let snap_y = closest_guide(point.y, &valid_guides, threshold, true);
    let snap_x = closest_guide(point.x, &valid_guides, threshold, false);

    match (snap_x, snap_y) {
        (Some(x), Some(y)) => {
            SnapResult::new(SnapType::CenterX, NodeId("guide".into()), Point::new(x, y))
        }
        (Some(x), None) => SnapResult::new(
            SnapType::CenterX,
            NodeId("guide".into()),
            Point::new(x, point.y),
        ),
        (None, Some(y)) => SnapResult::new(
            SnapType::CenterY,
            NodeId("guide".into()),
            Point::new(point.x, y),
        ),
        (None, None) => SnapResult::inactive(),
    }
}

#[must_use]
pub fn snap_to_nodes(active: &SnapNode, targets: &[SnapNode], threshold: f64) -> SnapResult {
    if threshold < 0.0 || !threshold.is_finite() || targets.is_empty() {
        return SnapResult::inactive();
    }
    if !active.x.is_finite()
        || !active.y.is_finite()
        || !targets.iter().all(|t| t.x.is_finite() && t.y.is_finite())
    {
        return SnapResult::inactive();
    }

    // Find best snap target considering all 6 snap points
    let mut best: Option<(f64, f64, f64, NodeId, SnapType)> = None;

    for target in targets.iter().filter(|t| t.id != active.id) {
        // X snap points
        let dist_left = (active.center_x() - target.left()).abs();
        if dist_left <= threshold {
            let candidate = (
                target.left(),
                target.center_y(),
                dist_left,
                target.id.clone(),
                SnapType::EdgeLeft,
            );
            best = match best {
                None => Some(candidate),
                Some((_, _, best_dist, _, _)) if dist_left < best_dist => Some(candidate),
                _ => best,
            };
        }
        let dist_center_x = (active.center_x() - target.center_x()).abs();
        if dist_center_x <= threshold {
            let candidate = (
                target.center_x(),
                target.center_y(),
                dist_center_x,
                target.id.clone(),
                SnapType::CenterX,
            );
            best = match best {
                None => Some(candidate),
                Some((_, _, best_dist, _, _)) if dist_center_x < best_dist => Some(candidate),
                _ => best,
            };
        }
        let dist_right = (active.center_x() - target.right()).abs();
        if dist_right <= threshold {
            let candidate = (
                target.right(),
                target.center_y(),
                dist_right,
                target.id.clone(),
                SnapType::EdgeRight,
            );
            best = match best {
                None => Some(candidate),
                Some((_, _, best_dist, _, _)) if dist_right < best_dist => Some(candidate),
                _ => best,
            };
        }
        // Y snap points
        let dist_top = (active.center_y() - target.top()).abs();
        if dist_top <= threshold {
            let candidate = (
                target.center_x(),
                target.top(),
                dist_top,
                target.id.clone(),
                SnapType::EdgeTop,
            );
            best = match best {
                None => Some(candidate),
                Some((_, _, best_dist, _, _)) if dist_top < best_dist => Some(candidate),
                _ => best,
            };
        }
        let dist_center_y = (active.center_y() - target.center_y()).abs();
        if dist_center_y <= threshold {
            let candidate = (
                target.center_x(),
                target.center_y(),
                dist_center_y,
                target.id.clone(),
                SnapType::CenterY,
            );
            best = match best {
                None => Some(candidate),
                Some((_, _, best_dist, _, _)) if dist_center_y < best_dist => Some(candidate),
                _ => best,
            };
        }
        let dist_bottom = (active.center_y() - target.bottom()).abs();
        if dist_bottom <= threshold {
            let candidate = (
                target.center_x(),
                target.bottom(),
                dist_bottom,
                target.id.clone(),
                SnapType::EdgeBottom,
            );
            best = match best {
                None => Some(candidate),
                Some((_, _, best_dist, _, _)) if dist_bottom < best_dist => Some(candidate),
                _ => best,
            };
        }
    }

    match best {
        Some((x, y, _, id, st)) => SnapResult::new(st, id, Point::new(x, y)),
        None => SnapResult::inactive(),
    }
}

#[must_use]
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

#[must_use]
pub fn align_center(nodes: &[SnapNode]) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }
    let avg_center: f64 = nodes
        .iter()
        .map(SnapNode::center_x)
        .filter(|x| x.is_finite())
        .sum::<f64>()
        / nodes.len() as f64;
    nodes
        .iter()
        .map(|n| Point::new(avg_center - n.width / 2.0, n.y))
        .collect()
}

#[must_use]
pub fn align_right(nodes: &[SnapNode]) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }
    let max_right = nodes
        .iter()
        .map(SnapNode::right)
        .filter(|x| x.is_finite())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);
    nodes
        .iter()
        .map(|n| Point::new(max_right - n.width, n.y))
        .collect()
}

#[must_use]
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

#[must_use]
pub fn align_middle(nodes: &[SnapNode]) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }
    let avg_middle: f64 = nodes
        .iter()
        .map(super::mod_types::SnapNode::center_y)
        .filter(|y| y.is_finite())
        .sum::<f64>()
        / nodes.len() as f64;
    nodes
        .iter()
        .map(|n| Point::new(n.x, avg_middle - n.height / 2.0))
        .collect()
}

#[must_use]
pub fn align_bottom(nodes: &[SnapNode]) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }
    let max_bottom = nodes
        .iter()
        .map(super::mod_types::SnapNode::bottom)
        .filter(|y| y.is_finite())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);
    nodes
        .iter()
        .map(|n| Point::new(n.x, max_bottom - n.height))
        .collect()
}

#[must_use]
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

/// Distribute nodes horizontally evenly.
///
/// # Errors
/// Returns an error if less than 3 nodes are selected.
pub fn distribute_horizontally(nodes: &[SnapNode]) -> Result<Vec<Point>, SnapError> {
    if nodes.len() < 3 {
        return Err(SnapError::InsufficientNodesForDistribution(nodes.len()));
    }
    let mut sorted: Vec<usize> = (0..nodes.len()).collect();
    sorted.sort_by(|&a, &b| {
        nodes[a]
            .x
            .partial_cmp(&nodes[b].x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let (first_x, last_x) = (nodes[sorted[0]].x, nodes[sorted[sorted.len() - 1]].x);
    let spacing = (last_x - first_x) / (sorted.len() - 1) as f64;
    let mut result: Vec<(usize, Point)> = sorted
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            (
                idx,
                Point::new((i as f64).mul_add(spacing, first_x), nodes[idx].y),
            )
        })
        .collect();
    result.sort_by_key(|(idx, _)| *idx);
    Ok(result.into_iter().map(|(_, p)| p).collect())
}

/// Distribute nodes vertically evenly.
///
/// # Errors
/// Returns an error if less than 3 nodes are selected.
pub fn distribute_vertically(nodes: &[SnapNode]) -> Result<Vec<Point>, SnapError> {
    if nodes.len() < 3 {
        return Err(SnapError::InsufficientNodesForDistribution(nodes.len()));
    }
    let mut sorted: Vec<usize> = (0..nodes.len()).collect();
    sorted.sort_by(|&a, &b| {
        nodes[a]
            .y
            .partial_cmp(&nodes[b].y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let (first_y, last_y) = (nodes[sorted[0]].y, nodes[sorted[sorted.len() - 1]].y);
    let spacing = (last_y - first_y) / (sorted.len() - 1) as f64;
    let mut result: Vec<(usize, Point)> = sorted
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            (
                idx,
                Point::new(nodes[idx].x, (i as f64).mul_add(spacing, first_y)),
            )
        })
        .collect();
    result.sort_by_key(|(idx, _)| *idx);
    Ok(result.into_iter().map(|(_, p)| p).collect())
}

fn resize_unconstrained(orig: Rect, delta: Point, handle: ResizeHandle) -> Rect {
    match handle {
        ResizeHandle::East => Rect {
            width: orig.width + delta.x,
            ..orig
        },
        ResizeHandle::West => Rect {
            x: orig.x + delta.x,
            width: orig.width - delta.x,
            ..orig
        },
        ResizeHandle::South => Rect {
            height: orig.height + delta.y,
            ..orig
        },
        ResizeHandle::North => Rect {
            y: orig.y + delta.y,
            height: orig.height - delta.y,
            ..orig
        },
        ResizeHandle::NorthEast => Rect {
            y: orig.y + delta.y,
            width: orig.width + delta.x,
            height: orig.height - delta.y,
            ..orig
        },
        ResizeHandle::NorthWest => Rect {
            x: orig.x + delta.x,
            y: orig.y + delta.y,
            width: orig.width - delta.x,
            height: orig.height - delta.y,
        },
        ResizeHandle::SouthEast => Rect {
            width: orig.width + delta.x,
            height: orig.height + delta.y,
            ..orig
        },
        ResizeHandle::SouthWest => Rect {
            x: orig.x + delta.x,
            width: orig.width - delta.x,
            height: orig.height + delta.y,
            ..orig
        },
    }
}

fn resize_east_west(orig: Rect, delta: Point, grid: f64, handle: ResizeHandle) -> Rect {
    match handle {
        ResizeHandle::East => Rect {
            width: (((orig.width + delta.x) / grid).round() * grid).max(1.0),
            ..orig
        },
        ResizeHandle::West => {
            let s_x = snap_to_grid(Point::new(orig.x + delta.x, 0.0), grid).x;
            Rect {
                x: s_x,
                width: (orig.x + orig.width - s_x).max(1.0),
                ..orig
            }
        }
        _ => orig,
    }
}

fn resize_north_south(orig: Rect, delta: Point, grid: f64, handle: ResizeHandle) -> Rect {
    match handle {
        ResizeHandle::South => Rect {
            height: (((orig.height + delta.y) / grid).round() * grid).max(1.0),
            ..orig
        },
        ResizeHandle::North => {
            let s_y = snap_to_grid(Point::new(0.0, orig.y + delta.y), grid).y;
            Rect {
                y: s_y,
                height: (orig.y + orig.height - s_y).max(1.0),
                ..orig
            }
        }
        _ => orig,
    }
}

fn resize_corners(orig: Rect, delta: Point, grid: f64, handle: ResizeHandle) -> Rect {
    match handle {
        ResizeHandle::NorthEast => {
            let s_w = (((orig.width + delta.x) / grid).round() * grid).max(1.0);
            let s_y = snap_to_grid(Point::new(0.0, orig.y + delta.y), grid).y;
            Rect {
                y: s_y,
                width: s_w,
                height: (orig.y + orig.height - s_y).max(1.0),
                ..orig
            }
        }
        ResizeHandle::NorthWest => {
            let s = snap_to_grid(Point::new(orig.x + delta.x, orig.y + delta.y), grid);
            Rect {
                x: s.x,
                y: s.y,
                width: (orig.x + orig.width - s.x).max(1.0),
                height: (orig.y + orig.height - s.y).max(1.0),
            }
        }
        ResizeHandle::SouthEast => {
            let s_w = (((orig.width + delta.x) / grid).round() * grid).max(1.0);
            let s_h = (((orig.height + delta.y) / grid).round() * grid).max(1.0);
            Rect {
                width: s_w,
                height: s_h,
                ..orig
            }
        }
        ResizeHandle::SouthWest => {
            let s_x = snap_to_grid(Point::new(orig.x + delta.x, 0.0), grid).x;
            let s_h = (((orig.height + delta.y) / grid).round() * grid).max(1.0);
            Rect {
                x: s_x,
                width: (orig.x + orig.width - s_x).max(1.0),
                height: s_h,
                ..orig
            }
        }
        _ => orig,
    }
}

#[must_use]
pub fn resize_with_snap(orig: Rect, delta: Point, grid: f64, handle: ResizeHandle) -> Rect {
    if grid <= 0.0 {
        return resize_unconstrained(orig, delta, handle);
    }
    match handle {
        ResizeHandle::East | ResizeHandle::West => resize_east_west(orig, delta, grid, handle),
        ResizeHandle::North | ResizeHandle::South => resize_north_south(orig, delta, grid, handle),
        _ => resize_corners(orig, delta, grid, handle),
    }
}

#[must_use]
pub fn resize_with_aspect_lock(
    original: Rect,
    delta: Point,
    grid_size: f64,
    handle: ResizeHandle,
    constraint: AspectConstraint,
) -> Rect {
    if constraint == AspectConstraint::Unlocked {
        return resize_with_snap(original, delta, grid_size, handle);
    }
    let ratio = original.width / original.height;
    let res = resize_with_snap(original, delta, grid_size, handle);
    match handle {
        ResizeHandle::North | ResizeHandle::South => Rect {
            width: res.height * ratio,
            ..res
        },
        _ => Rect {
            height: res.width / ratio,
            ..res
        },
    }
}
