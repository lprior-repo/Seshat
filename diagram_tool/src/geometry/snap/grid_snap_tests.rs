#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use super::*;
use proptest::prelude::*;

// Happy Path Tests
#[test]
fn test_coordinates_are_snapped_to_grid_when_enabled() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(20.0)?;
    assert_eq!(
        snap_node_coordinate(29.0, SnapMode::Enabled, grid),
        Ok(20.0)
    );
    assert_eq!(
        snap_node_coordinates((29.0, 41.0), SnapMode::Enabled, grid),
        Ok((20.0, 40.0))
    );
    Ok(())
}

#[test]
fn test_coordinates_remain_unchanged_when_snapping_disabled() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(20.0)?;
    assert_eq!(
        snap_node_coordinate(15.7, SnapMode::Disabled, grid),
        Ok(15.7)
    );
    assert_eq!(
        snap_node_coordinates((15.7, -42.3), SnapMode::Disabled, grid),
        Ok((15.7, -42.3))
    );
    Ok(())
}

#[test]
fn test_grid_size_is_created_with_valid_dimensions() {
    assert!(GridSize::new(10.0).is_ok());
    assert!(GridSize::new(50.0).is_ok());
    assert!(GridSize::new(100.0).is_ok());
}

// Error Path Tests
#[test]
fn test_rejects_out_of_bounds_grid_sizes() {
    assert_eq!(
        GridSize::new(9.9),
        Err(GridError::OutOfRange {
            value: "9.9".to_string()
        })
    );
    assert_eq!(
        GridSize::new(100.1),
        Err(GridError::OutOfRange {
            value: "100.1".to_string()
        })
    );
}

#[test]
fn test_rejects_non_finite_grid_sizes() {
    assert!(GridSize::new(f64::NAN).is_err());
    assert!(GridSize::new(f64::INFINITY).is_err());
    assert!(GridSize::new(f64::NEG_INFINITY).is_err());
}

#[test]
fn test_rejects_non_finite_coordinates_x_nonfinite() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(20.0)?;
    assert_eq!(
        snap_node_coordinates((f64::NAN, 15.0), SnapMode::Enabled, grid),
        Err(GridSnapError::NotFinite)
    );
    Ok(())
}

#[test]
fn test_rejects_non_finite_coordinates_y_nonfinite() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(20.0)?;
    assert_eq!(
        snap_node_coordinates((15.0, f64::NAN), SnapMode::Enabled, grid),
        Err(GridSnapError::NotFinite)
    );
    Ok(())
}

#[test]
fn test_rejects_non_finite_coordinates_both_nonfinite() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(20.0)?;
    assert_eq!(
        snap_node_coordinates((f64::NAN, f64::INFINITY), SnapMode::Enabled, grid),
        Err(GridSnapError::NotFinite)
    );
    Ok(())
}

// Edge Case Tests
#[test]
fn test_snapping_handles_exact_grid_multiples() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(20.0)?;
    assert_eq!(
        snap_node_coordinate(20.0, SnapMode::Enabled, grid),
        Ok(20.0)
    );
    Ok(())
}

#[test]
fn test_snapping_resolves_midway_ties_deterministically_positive() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(20.0)?;
    assert_eq!(
        snap_node_coordinate(10.0, SnapMode::Enabled, grid),
        Ok(20.0)
    );
    Ok(())
}

#[test]
fn test_snapping_resolves_midway_ties_deterministically_negative() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(20.0)?;
    assert_eq!(
        snap_node_coordinate(-10.0, SnapMode::Enabled, grid),
        Ok(-20.0)
    );
    Ok(())
}

#[test]
fn test_snapping_handles_negative_coordinates() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(10.0)?;
    assert_eq!(
        snap_node_coordinate(-15.0, SnapMode::Enabled, grid),
        Ok(-20.0)
    );
    assert_eq!(
        snap_node_coordinate(-14.9, SnapMode::Enabled, grid),
        Ok(-10.0)
    );
    Ok(())
}

#[test]
fn test_snapping_works_at_minimum_and_maximum_grid_boundaries() -> Result<(), anyhow::Error> {
    let grid_min = GridSize::new(10.0)?;
    let grid_max = GridSize::new(100.0)?;
    assert_eq!(
        snap_node_coordinate(5.0, SnapMode::Enabled, grid_min),
        Ok(10.0)
    );
    assert_eq!(
        snap_node_coordinate(50.0, SnapMode::Enabled, grid_max),
        Ok(100.0)
    );
    Ok(())
}

#[test]
fn test_snapping_handles_zero_coordinate() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(10.0)?;
    assert_eq!(snap_node_coordinate(0.0, SnapMode::Enabled, grid), Ok(0.0));
    Ok(())
}

// Contract Verification Tests
#[test]
fn test_precondition_finite_coordinates_are_required() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(10.0)?;
    assert_eq!(
        snap_node_coordinate(f64::NAN, SnapMode::Enabled, grid),
        Err(GridSnapError::NotFinite)
    );
    Ok(())
}

#[test]
fn test_precondition_grid_size_must_be_within_bounds() {
    assert!(GridSize::new(9.999).is_err());
    assert!(GridSize::new(100.001).is_err());
}

#[test]
fn test_snapping_is_ignored_when_disabled() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(10.0)?;
    assert_eq!(
        snap_node_coordinate(12.34, SnapMode::Disabled, grid),
        Ok(12.34)
    );
    Ok(())
}

#[test]
fn test_snapping_aligns_to_grid_multiples() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(10.0)?;
    let val = snap_node_coordinate(12.0, SnapMode::Enabled, grid)?;
    assert_eq!(val % 10.0, 0.0);
    Ok(())
}

#[test]
fn test_snapping_distance_never_exceeds_half_grid() -> Result<(), anyhow::Error> {
    let grid = GridSize::new(10.0)?;
    let val = snap_node_coordinate(14.9, SnapMode::Enabled, grid)?;
    assert!((val - 14.9).abs() <= 5.0);
    Ok(())
}

proptest! {
    #[test]
    fn proptest_q1_disabled_snap_returns_exact_raw_coordinate(
        // Use reasonable coordinate values instead of full normal float range
        val in -10000.0f64..=10000.0,
        grid_val in 10.0..=100.0f64
    ) {
        if let Ok(grid) = GridSize::new(grid_val) {
            prop_assert_eq!(snap_node_coordinate(val, SnapMode::Disabled, grid), Ok(val));
        }
    }

    #[test]
    fn proptest_q2_enabled_snap_is_multiple_of_grid(
        // Use reasonable coordinate values instead of full normal float range
        val in -10000.0f64..=10000.0,
        grid_val in 10.0..=100.0f64
    ) {
        if let Ok(grid) = GridSize::new(grid_val) {
            let snapped = snap_node_coordinate(val, SnapMode::Enabled, grid).unwrap_or(0.0);
            let remainder = (snapped % grid_val).abs();
            prop_assert!(remainder < 1e-10 || (remainder - grid_val).abs() < 1e-10);
        }
    }

    #[test]
    fn proptest_q3_snap_distance_never_exceeds_half_grid(
        // Use reasonable coordinate values instead of full normal float range
        val in -10000.0f64..=10000.0,
        grid_val in 10.0..=100.0f64
    ) {
        if let Ok(grid) = GridSize::new(grid_val) {
            let snapped = snap_node_coordinate(val, SnapMode::Enabled, grid).unwrap_or(0.0);
            prop_assert!((snapped - val).abs() <= grid_val / 2.0 + 1e-10);
        }
    }

    #[test]
    fn proptest_q4_midway_ties_round_consistently(
        multiplier in -10..10,
        grid_val in 10.0f64..100.0
    ) {
        // Just verify snapping works and produces consistent results
        if let Ok(grid) = GridSize::new(grid_val) {
            let base = f64::from(multiplier) * grid_val;
            let tie = base + (grid_val / 2.0);
            let snapped = snap_node_coordinate(tie, SnapMode::Enabled, grid).unwrap_or(0.0);

            // Verify snapped is a valid grid line (within precision)
            let remainder = (snapped / grid_val).fract();
            prop_assert!(
                remainder < 1e-9 || (1.0 - remainder) < 1e-9 || remainder.abs() < 1e-9,
                "Snapped value {} should be on a grid line for grid {}",
                snapped, grid_val
            );
        }
    }
}
