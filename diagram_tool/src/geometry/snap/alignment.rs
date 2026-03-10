use crate::geometry::primitives::Point;
use crate::geometry::snap::grid::snap_to_grid;
use crate::geometry::snap::mod_types::{SnapError, SnapNode};

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

#[must_use]
pub fn snap_to_guides(point: Point, guides: &[Guide], threshold: f64) -> Option<Point> {
    if threshold < 0.0 || !threshold.is_finite() {
        return None;
    }

    let valid_guides: Vec<&Guide> = guides
        .iter()
        .filter(|g| g.coordinate().is_finite())
        .collect();

    if valid_guides.is_empty() {
        return None;
    }

    let mut snapped_x: Option<f64> = None;
    let mut snapped_y: Option<f64> = None;

    for guide in valid_guides.iter().filter(|g| g.is_horizontal()) {
        let target = guide.coordinate();
        let distance = (point.y - target).abs();

        if distance <= threshold {
            let should_snap =
                snapped_y.map_or(true, |current| distance < (point.y - current).abs());
            if should_snap {
                snapped_y = Some(target);
            }
        }
    }

    for guide in valid_guides.iter().filter(|g| g.is_vertical()) {
        let target = guide.coordinate();
        let distance = (point.x - target).abs();

        if distance <= threshold {
            let should_snap =
                snapped_x.map_or(true, |current| distance < (point.x - current).abs());
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

#[must_use]
pub fn snap_to_nodes(active: &SnapNode, targets: &[SnapNode], threshold: f64) -> Option<Point> {
    if threshold < 0.0 || !threshold.is_finite() || targets.is_empty() {
        return None;
    }

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
        if target.id == active.id {
            continue;
        }

        for target_x in [target.left(), target.center_x(), target.right()] {
            let dist = (active.center_x() - target_x).abs();
            if dist <= threshold && dist < min_dist_x {
                min_dist_x = dist;
                snap_x = Some(target_x);
            }
        }

        for target_y in [target.top(), target.center_y(), target.bottom()] {
            let dist = (active.center_y() - target_y).abs();
            if dist <= threshold && dist < min_dist_y {
                min_dist_y = dist;
                snap_y = Some(target_y);
            }
        }
    }

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
        .map(|n| n.center_y())
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
        .map(|n| n.bottom())
        .filter(|y| y.is_finite())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);
    nodes
        .iter()
        .map(|n| Point::new(n.x, max_bottom - n.height))
        .collect()
}

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

    let spacing = (last_x - first_x) / (sorted_indices.len() - 1) as f64;

    let mut result = Vec::with_capacity(nodes.len());
    for (i, &idx) in sorted_indices.iter().enumerate() {
        let new_x = first_x + (i as f64 * spacing);
        result.push((idx, Point::new(new_x, nodes[idx].y)));
    }

    result.sort_by_key(|(idx, _)| *idx);
    Ok(result.into_iter().map(|(_, p)| p).collect())
}

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

    let spacing = (last_y - first_y) / (sorted_indices.len() - 1) as f64;

    let mut result = Vec::with_capacity(nodes.len());
    for (i, &idx) in sorted_indices.iter().enumerate() {
        let new_y = first_y + (i as f64 * spacing);
        result.push((idx, Point::new(nodes[idx].x, new_y)));
    }

    result.sort_by_key(|(idx, _)| *idx);
    Ok(result.into_iter().map(|(_, p)| p).collect())
}

pub fn resize_with_snap(
    original: Rect,
    delta: Point,
    grid_size: f64,
    handle: ResizeHandle,
) -> Rect {
    if grid_size <= 0.0 {
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

    match handle {
        ResizeHandle::East => {
            let new_width = original.width + delta.x;
            Rect {
                width: ((new_width / grid_size).round() * grid_size).max(1.0),
                ..original
            }
        }
        ResizeHandle::West => {
            let new_x = original.x + delta.x;
            let snapped_x = snap_to_grid(Point::new(new_x, 0.0), grid_size).x;
            Rect {
                x: snapped_x,
                width: (original.x + original.width - snapped_x).max(1.0),
                ..original
            }
        }
        ResizeHandle::South => {
            let new_height = original.height + delta.y;
            Rect {
                height: ((new_height / grid_size).round() * grid_size).max(1.0),
                ..original
            }
        }
        ResizeHandle::North => {
            let new_y = original.y + delta.y;
            let snapped_y = snap_to_grid(Point::new(0.0, new_y), grid_size).y;
            Rect {
                y: snapped_y,
                height: (original.y + original.height - snapped_y).max(1.0),
                ..original
            }
        }
        ResizeHandle::NorthEast => {
            let new_width = original.width + delta.x;
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
            let snapped = snap_to_grid(
                Point::new(original.x + delta.x, original.y + delta.y),
                grid_size,
            );
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
            Rect {
                width: ((new_width / grid_size).round() * grid_size).max(1.0),
                height: ((new_height / grid_size).round() * grid_size).max(1.0),
                ..original
            }
        }
        ResizeHandle::SouthWest => {
            let new_x = original.x + delta.x;
            let new_height = original.height + delta.y;
            let snapped_x = snap_to_grid(Point::new(new_x, 0.0), grid_size).x;
            Rect {
                x: snapped_x,
                width: (original.x + original.width - snapped_x).max(1.0),
                height: ((new_height / grid_size).round() * grid_size).max(1.0),
                ..original
            }
        }
    }
}

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
    let result = resize_with_snap(original, delta, grid_size, handle);
    match handle {
        ResizeHandle::East | ResizeHandle::West => Rect {
            height: result.width / aspect_ratio,
            ..result
        },
        ResizeHandle::North | ResizeHandle::South => Rect {
            width: result.height * aspect_ratio,
            ..result
        },
        ResizeHandle::NorthEast | ResizeHandle::SouthEast => Rect {
            height: result.width / aspect_ratio,
            ..result
        },
        ResizeHandle::NorthWest | ResizeHandle::SouthWest => Rect {
            height: result.width / aspect_ratio,
            ..result
        },
    }
}
