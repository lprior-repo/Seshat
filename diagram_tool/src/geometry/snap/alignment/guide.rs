use crate::geometry::primitives::Point;
use crate::geometry::snap::mod_types::{NodeId, SnapResult, SnapType};

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
enum Axis {
    Horizontal,
    Vertical,
}

fn closest_guide(val: f64, guides: &[&Guide], threshold: f64, axis: Axis) -> Option<f64> {
    guides
        .iter()
        .filter(|g| match axis {
            Axis::Horizontal => g.is_horizontal(),
            Axis::Vertical => g.is_vertical(),
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

    let snap_y = closest_guide(point.y, &valid_guides, threshold, Axis::Horizontal);
    let snap_x = closest_guide(point.x, &valid_guides, threshold, Axis::Vertical);

    match (snap_x, snap_y) {
        (Some(x), Some(y)) => SnapResult::new(
            SnapType::CenterX,
            NodeId::new("guide".into()),
            Point::new(x, y),
        ),
        (Some(x), None) => SnapResult::new(
            SnapType::CenterX,
            NodeId::new("guide".into()),
            Point::new(x, point.y),
        ),
        (None, Some(y)) => SnapResult::new(
            SnapType::CenterY,
            NodeId::new("guide".into()),
            Point::new(point.x, y),
        ),
        (None, None) => SnapResult::inactive(),
    }
}
