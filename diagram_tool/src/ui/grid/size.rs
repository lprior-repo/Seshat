use serde::{de::Error as _, Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Validated grid size for canvas snapping.
/// Guarantees: value is finite and in range [`Self::MIN`, `Self::MAX`]
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct GridSize(f64);

// Manual Default implementation to return 20.0 (valid default)
impl Default for GridSize {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

// Manual Eq implementation since f64 doesn't implement Eq
// This is safe because GridSize::new() guarantees the value is finite
impl PartialEq for GridSize {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for GridSize {}

impl GridSize {
    /// Minimum allowed grid size (inclusive)
    pub const MIN: f64 = 10.0;

    /// Maximum allowed grid size (inclusive)
    pub const MAX: f64 = 100.0;

    /// Default grid size (20.0)
    pub const DEFAULT: f64 = 20.0;

    /// Creates a new `GridSize`, returning error if out of range or not finite.
    ///
    /// # Errors
    /// - `GridError::OutOfRange` if value < 10.0 or value > 100.0
    /// - `GridError::NotFinite` if value is NaN or Infinity
    pub fn new(value: f64) -> Result<Self, GridError> {
        if !value.is_finite() {
            return Err(GridError::NotFinite {
                kind: classify_non_finite(value),
            });
        }
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(GridError::OutOfRange {
                value: format_float_for_error(value),
            });
        }
        Ok(Self(value))
    }

    /// Returns the inner f64 value.
    #[must_use]
    pub const fn inner(self) -> f64 {
        self.0
    }

    /// Returns the default `GridSize` (20.0).
    #[must_use]
    #[allow(dead_code)]
    pub const fn default_value() -> Self {
        Self(Self::DEFAULT)
    }
}

impl fmt::Display for GridSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for GridSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GridSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::new(value).map_err(|e| D::Error::custom(e.to_string()))
    }
}

/// Errors that can occur when creating or validating a `GridSize`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GridError {
    /// Grid size value is outside the valid range [10.0, 100.0]
    #[error("grid size must be between 10.0 and 100.0, got {value}")]
    OutOfRange { value: String },

    /// Grid size value is not a finite number (NaN or Infinity)
    #[error("grid size must be a finite number, got {kind}")]
    NotFinite { kind: NonFiniteKind },
}

/// Classifies the type of non-finite f64 value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonFiniteKind {
    /// NaN (Not a Number)
    NaN,
    /// Positive infinity
    PositiveInfinity,
    /// Negative infinity
    NegativeInfinity,
}

impl fmt::Display for NonFiniteKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NaN => write!(f, "NaN"),
            Self::PositiveInfinity => write!(f, "Infinity"),
            Self::NegativeInfinity => write!(f, "-Infinity"),
        }
    }
}

const fn classify_non_finite(value: f64) -> NonFiniteKind {
    if value.is_nan() {
        NonFiniteKind::NaN
    } else if value.is_infinite() && value.is_sign_positive() {
        NonFiniteKind::PositiveInfinity
    } else {
        NonFiniteKind::NegativeInfinity
    }
}

fn format_float_for_error(value: f64) -> String {
    if value.is_nan() {
        String::from("NaN")
    } else if value.is_infinite() {
        if value.is_sign_positive() {
            String::from("Infinity")
        } else {
            String::from("-Infinity")
        }
    } else {
        format!("{value}")
    }
}

/// Validates and creates a `GridSize` from a raw f64.
///
/// # Errors
/// - `GridError::OutOfRange` if value < 10.0 or value > 100.0
/// - `GridError::NotFinite` if value is NaN or Infinity
#[allow(dead_code)]
pub fn validated_grid_size(value: f64) -> Result<GridSize, GridError> {
    GridSize::new(value)
}

impl From<diagram_models::document::editor::GridSize> for GridSize {
    fn from(grid: diagram_models::document::editor::GridSize) -> Self {
        GridSize::new(grid.0).unwrap_or_default()
    }
}
