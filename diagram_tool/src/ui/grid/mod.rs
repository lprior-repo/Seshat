#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_value_when_creating_grid_size_then_returns_ok() {
        let result = GridSize::new(50.0);
        assert!(result.is_ok());
        assert!((result.unwrap().inner() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_minimum_value_when_creating_grid_size_then_returns_ok() {
        let result = GridSize::new(10.0);
        assert!(result.is_ok());
        assert!((result.unwrap().inner() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_maximum_value_when_creating_grid_size_then_returns_ok() {
        let result = GridSize::new(100.0);
        assert!(result.is_ok());
        assert!((result.unwrap().inner() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_default_when_getting_default_grid_size_then_returns_20() {
        let default = GridSize::default();
        assert!((default.inner() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_snap_disabled_when_snapping_value_then_returns_value_unchanged() {
        let value = 37.5;
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(value, false, grid_size);
        assert!((result - 37.5).abs() < f64::EPSILON);
    }

    #[test]
    fn given_snap_enabled_when_snapping_value_then_returns_grid_multiple() {
        let value = 29.0;
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(value, true, grid_size);
        assert!((result - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_point_when_snapping_then_each_coordinate_snapped_independently() {
        let point = (31.0, 49.0);
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_point(point, true, grid_size);
        assert!((result.0 - 40.0).abs() < f64::EPSILON);
        assert!((result.1 - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_valid_json_number_when_deserializing_grid_size_then_succeeds() {
        let json = "25.0";
        let result: Result<GridSize, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        let grid_size = result.unwrap();
        assert!((grid_size.inner() - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_value_below_minimum_when_creating_grid_size_then_returns_out_of_range_error() {
        let result = GridSize::new(5.0);
        assert!(matches!(result, Err(GridError::OutOfRange { .. })));
    }

    #[test]
    fn given_value_above_maximum_when_creating_grid_size_then_returns_out_of_range_error() {
        let result = GridSize::new(150.0);
        assert!(matches!(result, Err(GridError::OutOfRange { .. })));
    }

    #[test]
    fn given_negative_value_when_creating_grid_size_then_returns_out_of_range_error() {
        let result = GridSize::new(-20.0);
        assert!(matches!(result, Err(GridError::OutOfRange { .. })));
    }

    #[test]
    fn given_zero_value_when_creating_grid_size_then_returns_out_of_range_error() {
        let result = GridSize::new(0.0);
        assert!(matches!(result, Err(GridError::OutOfRange { .. })));
    }

    #[test]
    fn given_nan_value_when_creating_grid_size_then_returns_not_finite_error() {
        let result = GridSize::new(f64::NAN);
        assert!(matches!(result, Err(GridError::NotFinite { kind }) if kind == NonFiniteKind::NaN));
    }

    #[test]
    fn given_positive_infinity_when_creating_grid_size_then_returns_not_finite_error() {
        let result = GridSize::new(f64::INFINITY);
        assert!(matches!(
            result,
            Err(GridError::NotFinite { kind }) if kind == NonFiniteKind::PositiveInfinity
        ));
    }

    #[test]
    fn given_negative_infinity_when_creating_grid_size_then_returns_not_finite_error() {
        let result = GridSize::new(f64::NEG_INFINITY);
        assert!(matches!(
            result,
            Err(GridError::NotFinite { kind }) if kind == NonFiniteKind::NegativeInfinity
        ));
    }

    #[test]
    fn given_json_string_when_deserializing_grid_size_then_returns_error() {
        let json = r#""twenty""#;
        let result: Result<GridSize, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn given_out_of_range_json_number_when_deserializing_then_returns_error() {
        let json = "5.0";
        let result: Result<GridSize, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn given_json_null_when_deserializing_grid_size_then_returns_error() {
        let json = "null";
        let result: Result<GridSize, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn given_fractional_value_when_creating_grid_size_then_returns_ok() {
        let result = GridSize::new(50.5);
        assert!(result.is_ok());
        assert!((result.unwrap().inner() - 50.5).abs() < f64::EPSILON);
    }

    #[test]
    fn given_nan_value_when_snapping_then_returns_nan() {
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(f64::NAN, true, grid_size);
        assert!(result.is_nan());
    }

    #[test]
    fn given_infinity_value_when_snapping_then_returns_infinity() {
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(f64::INFINITY, true, grid_size);
        assert!(result.is_infinite());
    }

    #[test]
    fn given_exact_grid_multiple_when_snapping_then_returns_same_value() {
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(40.0, true, grid_size);
        assert!((result - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_value_midway_between_grid_lines_when_snapping_then_rounds_to_nearest() {
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(30.0, true, grid_size);
        assert!((result - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn given_negative_value_when_snapping_then_handles_correctly() {
        let grid_size = GridSize::new(20.0).unwrap();
        let result = snap_value(-15.0, true, grid_size);
        assert!((result - (-20.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_precondition_p1_range_validation() {
        assert!(GridSize::new(9.9).is_err());
        assert!(GridSize::new(10.0).is_ok());
        assert!(GridSize::new(50.0).is_ok());
        assert!(GridSize::new(100.0).is_ok());
        assert!(GridSize::new(100.1).is_err());
    }

    #[test]
    fn test_precondition_p1_finite_validation() {
        assert!(GridSize::new(f64::NAN).is_err());
        assert!(GridSize::new(f64::INFINITY).is_err());
        assert!(GridSize::new(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn test_postcondition_q1_inner_value_preserved() {
        let value = 42.5;
        let grid_size = GridSize::new(value).unwrap();
        assert!((grid_size.inner() - value).abs() < f64::EPSILON);
    }

    #[test]
    fn test_postcondition_q2_snap_disabled_identity() {
        let values = [0.0, -10.0, 37.5, 100.0, f64::NAN, f64::INFINITY];
        let grid_size = GridSize::default();
        for value in values {
            let result = snap_value(value, false, grid_size);
            if value.is_nan() {
                assert!(result.is_nan());
            } else if value.is_infinite() {
                assert!(
                    result.is_infinite() && result.is_sign_positive() == value.is_sign_positive()
                );
            } else {
                assert!((result - value).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_postcondition_q2_snap_enabled_grid_multiple() {
        let grid_size = GridSize::new(20.0).unwrap();
        let grid = grid_size.inner();
        for value in [0.0, 10.0, 20.0, 30.0, 40.0, 50.0] {
            let result = snap_value(value, true, grid_size);
            let remainder = (result / grid).round() * grid - result;
            assert!(remainder.abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_postcondition_q4_serialization_format() {
        let grid_size = GridSize::new(25.0).unwrap();
        let json = serde_json::to_string(&grid_size).unwrap();
        assert_eq!(json, "25.0");
    }

    #[test]
    fn test_postcondition_q5_default_value() {
        let default = GridSize::default();
        assert!((default.inner() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_invariant_i1_range_guaranteed() {
        let test_values = [10.0, 20.0, 50.5, 99.9, 100.0];
        for v in test_values {
            let gs = GridSize::new(v).unwrap();
            assert!(gs.inner() >= 10.0);
            assert!(gs.inner() <= 100.0);
        }
    }

    #[test]
    fn test_invariant_i2_finite_guaranteed() {
        let gs = GridSize::new(50.0).unwrap();
        assert!(gs.inner().is_finite());
    }

    #[test]
    fn given_grid_size_when_serializing_and_deserializing_then_roundtrips() {
        let original = GridSize::new(42.5).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: GridSize = serde_json::from_str(&json).unwrap();
        assert!((parsed.inner() - 42.5).abs() < f64::EPSILON);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_grid_size_value()(x in 10.0_f64..=100.0) -> f64 { x }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn prop_grid_size_invariant_range(value in arb_grid_size_value()) {
            let gs = GridSize::new(value).unwrap();
            prop_assert!(gs.inner() >= 10.0 && gs.inner() <= 100.0);
        }

        #[test]
        fn prop_snap_idempotency(value in -1000.0_f64..1000.0, grid in arb_grid_size_value()) {
            let gs = GridSize::new(grid).unwrap();
            let snap1 = snap_value(value, true, gs);
            let snap2 = snap_value(snap1, true, gs);
            if snap1.is_finite() && snap2.is_finite() {
                prop_assert!((snap1 - snap2).abs() < f64::EPSILON);
            }
        }

        #[test]
        fn prop_snap_grid_alignment(value in -1000.0_f64..1000.0, grid in arb_grid_size_value()) {
            let gs = GridSize::new(grid).unwrap();
            let result = snap_value(value, true, gs);
            if result.is_finite() {
                let effective_grid = grid.max(1.0);
                let remainder = (result / effective_grid).round() * effective_grid - result;
                prop_assert!(remainder.abs() < f64::EPSILON);
            }
        }

        #[test]
        fn prop_snap_disabled_identity(value in -1e6_f64..1e6_f64, grid in arb_grid_size_value()) {
            let gs = GridSize::new(grid).unwrap();
            let result = snap_value(value, false, gs);
            if value.is_nan() {
                prop_assert!(result.is_nan());
            } else {
                prop_assert!((result - value).abs() < f64::EPSILON);
            }
        }

        #[test]
        fn prop_snap_point_consistent_with_snap_value(
            x in -1000.0_f64..1000.0,
            y in -1000.0_f64..1000.0,
            grid in arb_grid_size_value()
        ) {
            let gs = GridSize::new(grid).unwrap();
            let snapped = snap_point((x, y), true, gs);
            let expected_x = snap_value(x, true, gs);
            let expected_y = snap_value(y, true, gs);
            if expected_x.is_finite() && snapped.0.is_finite() {
                prop_assert!((snapped.0 - expected_x).abs() < f64::EPSILON);
            }
            if expected_y.is_finite() && snapped.1.is_finite() {
                prop_assert!((snapped.1 - expected_y).abs() < f64::EPSILON);
            }
        }

        #[test]
        fn prop_serialization_roundtrip(value in arb_grid_size_value()) {
            let gs = GridSize::new(value).unwrap();
            let json = serde_json::to_string(&gs).unwrap();
            let parsed: GridSize = serde_json::from_str(&json).unwrap();
            // Use relative tolerance for floating point comparison
            let diff = (parsed.inner() - value).abs();
            let tolerance = (value.abs() * 1e-10).max(1e-10);
            prop_assert!(diff < tolerance);
        }
    }
}
