use crate::document::{EdgeId, NodeId};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElementId {
    Node(NodeId),
    Edge(EdgeId),
}

impl fmt::Display for ElementId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(id) => write!(f, "{id}"),
            Self::Edge(id) => write!(f, "{id}"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct SelectModifiers {
    pub alt: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub right_click: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SelectionError {
    #[error("Node not found in document")]
    NodeNotFound,
    #[error("Movement exceeded drag threshold")]
    MovementExceededDragThreshold,
    #[error("Node is not editable")]
    NodeNotEditable,
    #[error("Invalid marquee bounds: negative width or height")]
    InvalidMarqueeBounds,
    #[error("Element is locked")]
    ElementLocked,
    #[error("Element is hidden")]
    ElementHidden,
    #[error("Element not found")]
    ElementNotFound,
    #[error("Node has no parent container")]
    NoParentContainer,
    #[error("Precondition violated")]
    PreconditionViolated,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    /// # Errors
    /// Returns an error if the marquee bounds have negative width or height.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, SelectionError> {
        if width < 0.0 || height < 0.0 {
            Err(SelectionError::InvalidMarqueeBounds)
        } else {
            Ok(Self {
                x,
                y,
                width,
                height,
            })
        }
    }
}
