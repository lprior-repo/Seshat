use crate::ui::canvas::domain::types::{CanvasPoint, CanvasVector};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid timing threshold: must be strictly greater than zero")]
    InvalidTimingThreshold,
    #[error("Negative hit padding: touch padding must be >= 0")]
    NegativeHitPadding,
    #[error("Untracked pointer: pointer ID {0} is not currently tracked")]
    UntrackedPointer(PointerId),
    #[error("Too many simultaneous pointers")]
    TooManyPointers,
    #[error("Duplicate pointer ID")]
    DuplicatePointerId,
    #[error("Postcondition violation")]
    PostconditionViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointerId(pub u64);

impl fmt::Display for PointerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeMs(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerType {
    Mouse,
    Touch,
    Pen,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    PanCamera { vector: CanvasVector },
    MoveShape { vector: CanvasVector },
    DoubleTap,
    SingleTap,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct InputState {
    pub active_pointers: im::HashMap<PointerId, PointerData>,
    pub last_tap: Option<TapHistory>,
}

impl InputState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerData {
    pub id: PointerId,
    pub pointer_type: PointerType,
    pub start_pos: CanvasPoint,
    pub current_pos: CanvasPoint,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TapHistory {
    pub pos: CanvasPoint,
    pub time: TimeMs,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerEvent {
    Down {
        id: PointerId,
        pointer_type: PointerType,
        pos: CanvasPoint,
        time: TimeMs,
    },
    Move {
        id: PointerId,
        pos: CanvasPoint,
    },
    Up {
        id: PointerId,
        pos: CanvasPoint,
        time: TimeMs,
    },
}

pub struct Handle {
    pub center: CanvasPoint,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TwoFingerGesture {
    pub id1: PointerId,
    pub id2: PointerId,
}

impl TwoFingerGesture {
    pub fn new(id1: PointerId, id2: PointerId) -> Result<Self, Error> {
        if id1 == id2 {
            Err(Error::DuplicatePointerId)
        } else {
            Ok(Self { id1, id2 })
        }
    }
}
