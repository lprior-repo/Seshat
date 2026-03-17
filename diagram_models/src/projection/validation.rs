//! Validation functions for diagram projection
//!
//! This module provides shared validation logic used across projection operations.

use crate::projection::types::ProjectionError;

/// Validates that width and height are valid (finite and positive).
///
/// Returns `Ok(())` if both dimensions are valid, otherwise returns an error.
pub fn validate_dimensions(width: f64, height: f64) -> Result<(), ProjectionError> {
    if !width.is_finite() || width <= 0.0 {
        return Err(ProjectionError::InvalidDimensions(format!(
            "invalid width: {width}"
        )));
    }
    if !height.is_finite() || height <= 0.0 {
        return Err(ProjectionError::InvalidDimensions(format!(
            "invalid height: {height}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_dimensions_when_validated_then_ok() {
        let result = validate_dimensions(100.0, 50.0);
        assert!(result.is_ok());
    }

    #[test]
    fn given_zero_width_when_validated_then_error() {
        let result = validate_dimensions(0.0, 50.0);
        assert!(result.is_err());
    }

    #[test]
    fn given_negative_width_when_validated_then_error() {
        let result = validate_dimensions(-10.0, 50.0);
        assert!(result.is_err());
    }

    #[test]
    fn given_nan_width_when_validated_then_error() {
        let result = validate_dimensions(f64::NAN, 50.0);
        assert!(result.is_err());
    }

    #[test]
    fn given_infinite_width_when_validated_then_error() {
        let result = validate_dimensions(f64::INFINITY, 50.0);
        assert!(result.is_err());
    }

    #[test]
    fn given_zero_height_when_validated_then_error() {
        let result = validate_dimensions(100.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn given_negative_height_when_validated_then_error() {
        let result = validate_dimensions(100.0, -10.0);
        assert!(result.is_err());
    }

    #[test]
    fn given_nan_height_when_validated_then_error() {
        let result = validate_dimensions(100.0, f64::NAN);
        assert!(result.is_err());
    }

    #[test]
    fn given_infinite_height_when_validated_then_error() {
        let result = validate_dimensions(100.0, f64::INFINITY);
        assert!(result.is_err());
    }
}
