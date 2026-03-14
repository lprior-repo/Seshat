#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{de::Error as _, Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

// =============================================================================
// Contract Types: SnapMode and GridSnapError
// =============================================================================

/// Explicit snap state for contract-compliant API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapMode {
    /// Snapping is enabled
    Enabled,
    /// Snapping is disabled - free movement
    Disabled,
}

impl SnapMode {
    /// Converts a bool to SnapMode for backward compatibility with existing callers.
    #[must_use]
    pub const fn from_bool(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }

    /// Returns true if snapping is enabled.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

/// Errors for contract-compliant grid snapping API.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum GridSnapError {
    /// Raw x coordinate is non-finite (NaN or Infinity)
    #[error("x coordinate must be finite, got non-finite value")]
    NonFiniteX,

    /// Raw y coordinate is non-finite (NaN or Infinity)
    #[error("y coordinate must be finite, got non-finite value")]
    NonFiniteY,

    /// Grid size is invalid
    #[error("invalid grid size: {0}")]
    InvalidGridSize(#[from] GridError),

    /// Contract violation detected
    #[error("contract violation in {clause}: {details}")]
    ContractViolation {
        /// The contract clause that was violated
        clause: &'static str,
        /// Details about the violation
        details: String,
    },
}

// =============================================================================
// Contract Functions: snap_node_coordinate, snap_node_coordinates, try_grid_size
// =============================================================================

/// Validates and creates a GridSize from raw f64, returning contract error on failure.
///
/// # Errors
/// Returns `GridSnapError::InvalidGridSize` if the grid size is invalid.
pub fn try_grid_size(raw_step: f64) -> Result<GridSize, GridSnapError> {
    GridSize::new(raw_step).map_err(GridSnapError::from)
}

/// Snaps a single coordinate value to the grid if snapping is enabled.
///
/// # Errors
/// Returns `GridSnapError::NonFiniteX` if the raw value is NaN or Infinity.
pub fn snap_node_coordinate(
    raw_value: f64,
    mode: SnapMode,
    grid: GridSize,
) -> Result<f64, GridSnapError> {
    // P1: Raw x coordinate must be finite
    if !raw_value.is_finite() {
        return Err(GridSnapError::NonFiniteX);
    }

    // Q1-Q5: Apply snapping based on mode
    match mode {
        SnapMode::Disabled => {
            // Q5: Disabled snapping returns value unchanged
            Ok(raw_value)
        }
        SnapMode::Enabled => {
            // Q1-Q4: Enabled snapping applies grid rounding
            Ok(snap_value(raw_value, true, grid))
        }
    }
}

/// Snaps a point (x, y) to the grid if snapping is enabled.
///
/// # Errors
/// Returns `GridSnapError::NonFiniteX` or `GridSnapError::NonFiniteY` if coordinates are non-finite.
pub fn snap_node_coordinates(
    raw_point: (f64, f64),
    mode: SnapMode,
    grid: GridSize,
) -> Result<(f64, f64), GridSnapError> {
    let (raw_x, raw_y) = raw_point;

    // P1: Raw x coordinate must be finite
    if !raw_x.is_finite() {
        return Err(GridSnapError::NonFiniteX);
    }

    // P2: Raw y coordinate must be finite
    if !raw_y.is_finite() {
        return Err(GridSnapError::NonFiniteY);
    }

    // Q1-Q5: Apply snapping based on mode
    match mode {
        SnapMode::Disabled => {
            // Q5: Disabled snapping returns point unchanged
            Ok((raw_x, raw_y))
        }
        SnapMode::Enabled => {
            // Q1-Q4: Enabled snapping applies grid rounding
            Ok(snap_point((raw_x, raw_y), true, grid))
        }
    }
}

// =============================================================================
// Existing Implementation (preserved for backward compatibility)
// =============================================================================

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

/// Snaps a single value to the grid if snapping is enabled.
///
/// # Guarantees
/// - If `snap_to_grid == false`, returns `value` unchanged
/// - If `grid_size` inner value is <= 0 or non-finite, treats `grid_size` as 1.0
/// - Result is always finite if input is finite
/// - NaN input returns NaN
#[must_use]
pub fn snap_value(value: f64, snap_to_grid: bool, grid_size: GridSize) -> f64 {
    if !snap_to_grid {
        return value;
    }

    let step = grid_size.inner().max(1.0);
    (value / step).round() * step
}

/// Snaps a point (x, y) to the grid if snapping is enabled.
///
/// # Guarantees
/// - Applies `snap_value` independently to each coordinate
/// - See [`snap_value`] for additional guarantees
#[must_use]
pub fn snap_point(point: (f64, f64), snap_to_grid: bool, grid_size: GridSize) -> (f64, f64) {
    (
        snap_value(point.0, snap_to_grid, grid_size),
        snap_value(point.1, snap_to_grid, grid_size),
    )
}
