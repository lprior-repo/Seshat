use crate::geometry::primitives::Point;
use crate::geometry::snap::mod_types::SnapNode;

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

#[must_use]
pub fn align(nodes: &[SnapNode], anchor: AlignmentAnchor) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let reference_val = match anchor {
        AlignmentAnchor::Left => nodes
            .iter()
            .map(|n| n.x)
            .filter(|x| x.is_finite())
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0),
        AlignmentAnchor::Center => {
            let sum: f64 = nodes
                .iter()
                .map(|n| n.x + n.width / 2.0)
                .filter(|x| x.is_finite())
                .sum();
            sum / nodes.len() as f64
        }
        AlignmentAnchor::Right => nodes
            .iter()
            .map(|n| n.x + n.width)
            .filter(|x| x.is_finite())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0),
        AlignmentAnchor::Top => nodes
            .iter()
            .map(|n| n.y)
            .filter(|y| y.is_finite())
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0),
        AlignmentAnchor::Middle => {
            let sum: f64 = nodes
                .iter()
                .map(|n| n.y + n.height / 2.0)
                .filter(|y| y.is_finite())
                .sum();
            sum / nodes.len() as f64
        }
        AlignmentAnchor::Bottom => nodes
            .iter()
            .map(|n| n.y + n.height)
            .filter(|y| y.is_finite())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0),
    };

    nodes
        .iter()
        .map(|n| match anchor {
            AlignmentAnchor::Left => Point::new(reference_val, n.y),
            AlignmentAnchor::Center => Point::new(reference_val - n.width / 2.0, n.y),
            AlignmentAnchor::Right => Point::new(reference_val - n.width, n.y),
            AlignmentAnchor::Top => Point::new(n.x, reference_val),
            AlignmentAnchor::Middle => Point::new(n.x, reference_val - n.height / 2.0),
            AlignmentAnchor::Bottom => Point::new(n.x, reference_val - n.height),
        })
        .collect()
}

#[must_use]
pub fn align_left(nodes: &[SnapNode]) -> Vec<Point> {
    align(nodes, AlignmentAnchor::Left)
}

#[must_use]
pub fn align_center(nodes: &[SnapNode]) -> Vec<Point> {
    align(nodes, AlignmentAnchor::Center)
}

#[must_use]
pub fn align_right(nodes: &[SnapNode]) -> Vec<Point> {
    align(nodes, AlignmentAnchor::Right)
}

#[must_use]
pub fn align_top(nodes: &[SnapNode]) -> Vec<Point> {
    align(nodes, AlignmentAnchor::Top)
}

#[must_use]
pub fn align_middle(nodes: &[SnapNode]) -> Vec<Point> {
    align(nodes, AlignmentAnchor::Middle)
}

#[must_use]
pub fn align_bottom(nodes: &[SnapNode]) -> Vec<Point> {
    align(nodes, AlignmentAnchor::Bottom)
}
