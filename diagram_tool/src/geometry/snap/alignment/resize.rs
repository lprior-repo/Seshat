use crate::geometry::primitives::Point;
use crate::geometry::snap::grid::snap_to_grid;

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

fn resize_unconstrained(orig: Rect, delta: Point, handle: ResizeHandle) -> Rect {
    let mut rect = orig;
    match handle {
        ResizeHandle::East | ResizeHandle::NorthEast | ResizeHandle::SouthEast => {
            rect.width += delta.x;
        }
        ResizeHandle::West | ResizeHandle::NorthWest | ResizeHandle::SouthWest => {
            rect.x += delta.x;
            rect.width -= delta.x;
        }
        _ => {}
    }
    match handle {
        ResizeHandle::South | ResizeHandle::SouthEast | ResizeHandle::SouthWest => {
            rect.height += delta.y;
        }
        ResizeHandle::North | ResizeHandle::NorthEast | ResizeHandle::NorthWest => {
            rect.y += delta.y;
            rect.height -= delta.y;
        }
        _ => {}
    }
    rect
}

#[must_use]
pub fn resize_with_snap(orig: Rect, delta: Point, grid: f64, handle: ResizeHandle) -> Rect {
    if grid <= 0.0 {
        return resize_unconstrained(orig, delta, handle);
    }
    let mut rect = orig;

    match handle {
        ResizeHandle::East | ResizeHandle::NorthEast | ResizeHandle::SouthEast => {
            rect.width = (((orig.width + delta.x) / grid).round() * grid).max(1.0);
        }
        ResizeHandle::West | ResizeHandle::NorthWest | ResizeHandle::SouthWest => {
            let s_x = snap_to_grid(Point::new(orig.x + delta.x, 0.0), grid).x;
            rect.x = s_x;
            rect.width = (orig.x + orig.width - s_x).max(1.0);
        }
        _ => {}
    }

    match handle {
        ResizeHandle::South | ResizeHandle::SouthEast | ResizeHandle::SouthWest => {
            rect.height = (((orig.height + delta.y) / grid).round() * grid).max(1.0);
        }
        ResizeHandle::North | ResizeHandle::NorthEast | ResizeHandle::NorthWest => {
            let s_y = snap_to_grid(Point::new(0.0, orig.y + delta.y), grid).y;
            rect.y = s_y;
            rect.height = (orig.y + orig.height - s_y).max(1.0);
        }
        _ => {}
    }
    rect
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
