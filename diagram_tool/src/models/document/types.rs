//! NewType definitions for diagram document identifiers and values.
//!
//! These types eliminate primitive obsession by wrapping raw values in semantic types
//! with validation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Newtype for Node Identifier to prevent primitive obsession
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(String);

impl NodeId {
    #[must_use]
    pub const fn new(id: String) -> Self {
        Self(id)
    }

    /// Create a new `NodeId`, returning error for empty strings
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is an empty string.
    pub fn try_new(id: String) -> Result<Self, &'static str> {
        if id.is_empty() {
            Err("NodeId cannot be empty")
        } else {
            Ok(Self(id))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype for Edge Identifier
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeId(String);

impl EdgeId {
    #[must_use]
    pub const fn new(id: String) -> Self {
        Self(id)
    }

    /// Create a new `EdgeId`, returning error for empty strings
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is an empty string.
    pub fn try_new(id: String) -> Result<Self, &'static str> {
        if id.is_empty() {
            Err("EdgeId cannot be empty")
        } else {
            Ok(Self(id))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype for Author Identifier to prevent primitive obsession
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AuthorId(String);

impl AuthorId {
    #[must_use]
    pub const fn new(id: String) -> Self {
        Self(id)
    }

    /// Create a new `AuthorId`, returning error for empty strings
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is an empty string.
    pub fn try_new(id: String) -> Result<Self, &'static str> {
        if id.is_empty() {
            Err("AuthorId cannot be empty")
        } else {
            Ok(Self(id))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AuthorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype for Timestamp (Unix timestamp in seconds)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp {
    #[must_use]
    pub const fn new(timestamp: i64) -> Self {
        Self(timestamp)
    }

    /// Create a new `Timestamp`, returning error for negative timestamps
    ///
    /// # Errors
    ///
    /// Returns an error if `timestamp` is negative.
    pub fn try_new(timestamp: i64) -> Result<Self, &'static str> {
        if timestamp < 0 {
            Err("Timestamp cannot be negative")
        } else {
            Ok(Self(timestamp))
        }
    }

    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Document revision counter - monotonically increasing
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(u64);

impl Revision {
    pub const INITIAL: Self = Self(0);

    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn increment(self) -> Self {
        Self(self.0 + 1)
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Helper to make floats Eq - wraps f64 with total ordering guarantees
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct OrderedFloat(pub f64);

/// Error type for `OrderedFloat` construction
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum OrderedFloatError {
    #[error("NaN is not a valid value for OrderedFloat")]
    NaN,
    #[error("Infinity is not a valid value for OrderedFloat")]
    Infinite,
}

impl<'de> Deserialize<'de> for OrderedFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Ok(Self::new_unchecked(value))
    }
}

impl Serialize for OrderedFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl OrderedFloat {
    /// Creates a new `OrderedFloat` from a `f64` value.
    ///
    /// # Errors
    ///
    /// Returns `OrderedFloatError::NaN` if value is NaN.
    /// Returns `OrderedFloatError::Infinite` if value is infinite.
    pub const fn new(value: f64) -> Result<Self, OrderedFloatError> {
        if value.is_nan() {
            Err(OrderedFloatError::NaN)
        } else if value.is_infinite() {
            Err(OrderedFloatError::Infinite)
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }
}

impl Eq for OrderedFloat {}

impl std::hash::Hash for OrderedFloat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl fmt::Display for OrderedFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add for OrderedFloat {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(self.0 + rhs.0)
    }
}

impl std::ops::Sub for OrderedFloat {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(self.0 - rhs.0)
    }
}

impl std::ops::Sub<f64> for OrderedFloat {
    type Output = Self;
    fn sub(self, rhs: f64) -> Self::Output {
        Self::new_unchecked(self.0 - rhs)
    }
}

impl std::ops::Mul<f64> for OrderedFloat {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new_unchecked(self.0 * rhs)
    }
}

impl std::ops::Div<f64> for OrderedFloat {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new_unchecked(self.0 / rhs)
    }
}
