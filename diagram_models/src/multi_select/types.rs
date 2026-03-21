use crate::document::Node;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("Empty selection")]
    EmptySelection,
    #[error("Item is locked")]
    ItemLocked,
    #[error("Invalid hierarchy")]
    InvalidHierarchy,
    #[error("Postcondition violated")]
    PostconditionViolated,
    #[error("Node not found")]
    NodeNotFound,
    #[error("Invalid scale factor: must be positive and finite")]
    InvalidScale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyVec<T>(Vec<T>);

impl<T> NonEmptyVec<T> {
    /// Tries to create a `NonEmptyVec`.
    ///
    /// # Errors
    ///
    /// Returns `Error::EmptySelection` if the vector is empty.
    pub fn try_from(vec: Vec<T>) -> Result<Self, Error> {
        if vec.is_empty() {
            Err(Error::EmptySelection)
        } else {
            Ok(Self(vec))
        }
    }

    #[must_use]
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
}

impl<T> IntoIterator for NonEmptyVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardData {
    pub nodes: Vec<Node>,
}
