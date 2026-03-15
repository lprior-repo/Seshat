use super::mod_types::SnapMode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridError {
    OutOfRange,
    NotFinite,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridSnapError {
    NotFinite,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridSize(f64);

impl GridSize {
    #[must_use]
    pub fn try_grid_size(raw_step: f64) -> Result<Self, GridError> {
        if !raw_step.is_finite() {
            return Err(GridError::NotFinite);
        }
        if raw_step < 10.0 || raw_step > 100.0 {
            return Err(GridError::OutOfRange);
        }
        Ok(Self(raw_step))
    }

    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }
}

#[must_use]
pub fn snap_node_coordinate(
    raw_value: f64,
    mode: SnapMode,
    grid: GridSize,
) -> Result<f64, GridSnapError> {
    if !raw_value.is_finite() {
        return Err(GridSnapError::NotFinite);
    }
    match mode {
        SnapMode::Disabled => Ok(raw_value),
        SnapMode::Enabled => {
            let grid_val = grid.value();
            let snapped = (raw_value / grid_val).round() * grid_val;
            Ok(snapped)
        }
    }
}

#[must_use]
pub fn snap_node_coordinates(
    raw_point: (f64, f64),
    mode: SnapMode,
    grid: GridSize,
) -> Result<(f64, f64), GridSnapError> {
    if !raw_point.0.is_finite() || !raw_point.1.is_finite() {
        return Err(GridSnapError::NotFinite);
    }
    let x = snap_node_coordinate(raw_point.0, mode, grid)?;
    let y = snap_node_coordinate(raw_point.1, mode, grid)?;
    Ok((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Happy Path Tests
    #[test]
    fn test_coordinates_are_snapped_to_grid_when_enabled() {
        let grid = GridSize::try_grid_size(20.0).unwrap();
        assert_eq!(
            snap_node_coordinate(29.0, SnapMode::Enabled, grid),
            Ok(20.0)
        );
        assert_eq!(
            snap_node_coordinates((29.0, 41.0), SnapMode::Enabled, grid),
            Ok((20.0, 40.0))
        );
    }

    #[test]
    fn test_coordinates_remain_unchanged_when_snapping_disabled() {
        let grid = GridSize::try_grid_size(20.0).unwrap();
        assert_eq!(
            snap_node_coordinate(15.7, SnapMode::Disabled, grid),
            Ok(15.7)
        );
        assert_eq!(
            snap_node_coordinates((15.7, -42.3), SnapMode::Disabled, grid),
            Ok((15.7, -42.3))
        );
    }

    #[test]
    fn test_grid_size_is_created_with_valid_dimensions() {
        assert!(GridSize::try_grid_size(10.0).is_ok());
        assert!(GridSize::try_grid_size(50.0).is_ok());
        assert!(GridSize::try_grid_size(100.0).is_ok());
    }

    // Error Path Tests
    #[test]
    fn test_rejects_out_of_bounds_grid_sizes() {
        assert_eq!(GridSize::try_grid_size(9.9), Err(GridError::OutOfRange));
        assert_eq!(GridSize::try_grid_size(100.1), Err(GridError::OutOfRange));
    }

    #[test]
    fn test_rejects_non_finite_grid_sizes() {
        assert_eq!(GridSize::try_grid_size(f64::NAN), Err(GridError::NotFinite));
        assert_eq!(
            GridSize::try_grid_size(f64::INFINITY),
            Err(GridError::NotFinite)
        );
        assert_eq!(
            GridSize::try_grid_size(f64::NEG_INFINITY),
            Err(GridError::NotFinite)
        );
    }

    #[test]
    fn test_rejects_non_finite_coordinates_x_nonfinite() {
        let grid = GridSize::try_grid_size(20.0).unwrap();
        assert_eq!(
            snap_node_coordinates((f64::NAN, 15.0), SnapMode::Enabled, grid),
            Err(GridSnapError::NotFinite)
        );
    }

    #[test]
    fn test_rejects_non_finite_coordinates_y_nonfinite() {
        let grid = GridSize::try_grid_size(20.0).unwrap();
        assert_eq!(
            snap_node_coordinates((15.0, f64::NAN), SnapMode::Enabled, grid),
            Err(GridSnapError::NotFinite)
        );
    }

    #[test]
    fn test_rejects_non_finite_coordinates_both_nonfinite() {
        let grid = GridSize::try_grid_size(20.0).unwrap();
        assert_eq!(
            snap_node_coordinates((f64::NAN, f64::INFINITY), SnapMode::Enabled, grid),
            Err(GridSnapError::NotFinite)
        );
    }

    // Edge Case Tests
    #[test]
    fn test_snapping_handles_exact_grid_multiples() {
        let grid = GridSize::try_grid_size(10.0).unwrap();
        assert_eq!(
            snap_node_coordinate(20.0, SnapMode::Enabled, grid),
            Ok(20.0)
        );
    }

    #[test]
    fn test_snapping_resolves_midway_ties_deterministically_positive() {
        let grid = GridSize::try_grid_size(20.0).unwrap();
        assert_eq!(
            snap_node_coordinate(10.0, SnapMode::Enabled, grid),
            Ok(20.0)
        );
    }

    #[test]
    fn test_snapping_resolves_midway_ties_deterministically_negative() {
        let grid = GridSize::try_grid_size(20.0).unwrap();
        assert_eq!(
            snap_node_coordinate(-10.0, SnapMode::Enabled, grid),
            Ok(-20.0)
        );
    }

    #[test]
    fn test_snapping_handles_negative_coordinates() {
        let grid = GridSize::try_grid_size(10.0).unwrap();
        assert_eq!(
            snap_node_coordinate(-15.0, SnapMode::Enabled, grid),
            Ok(-20.0)
        );
        assert_eq!(
            snap_node_coordinate(-14.9, SnapMode::Enabled, grid),
            Ok(-10.0)
        );
    }

    #[test]
    fn test_snapping_works_at_minimum_and_maximum_grid_boundaries() {
        let grid_min = GridSize::try_grid_size(10.0).unwrap();
        let grid_max = GridSize::try_grid_size(100.0).unwrap();
        assert_eq!(
            snap_node_coordinate(5.0, SnapMode::Enabled, grid_min),
            Ok(10.0)
        );
        assert_eq!(
            snap_node_coordinate(50.0, SnapMode::Enabled, grid_max),
            Ok(100.0)
        );
    }

    #[test]
    fn test_snapping_handles_zero_coordinate() {
        let grid = GridSize::try_grid_size(10.0).unwrap();
        assert_eq!(snap_node_coordinate(0.0, SnapMode::Enabled, grid), Ok(0.0));
    }

    // Contract Verification Tests
    #[test]
    fn test_precondition_finite_coordinates_are_required() {
        let grid = GridSize::try_grid_size(10.0).unwrap();
        assert_eq!(
            snap_node_coordinate(f64::NAN, SnapMode::Enabled, grid),
            Err(GridSnapError::NotFinite)
        );
    }

    #[test]
    fn test_precondition_grid_size_must_be_within_bounds() {
        assert_eq!(GridSize::try_grid_size(9.999), Err(GridError::OutOfRange));
        assert_eq!(GridSize::try_grid_size(100.001), Err(GridError::OutOfRange));
    }

    #[test]
    fn test_snapping_is_ignored_when_disabled() {
        let grid = GridSize::try_grid_size(10.0).unwrap();
        assert_eq!(
            snap_node_coordinate(12.34, SnapMode::Disabled, grid),
            Ok(12.34)
        );
    }

    #[test]
    fn test_snapping_aligns_to_grid_multiples() {
        let grid = GridSize::try_grid_size(10.0).unwrap();
        let val = snap_node_coordinate(12.0, SnapMode::Enabled, grid).unwrap();
        assert_eq!(val % 10.0, 0.0);
    }

    #[test]
    fn test_snapping_distance_never_exceeds_half_grid() {
        let grid = GridSize::try_grid_size(10.0).unwrap();
        let val = snap_node_coordinate(14.9, SnapMode::Enabled, grid).unwrap();
        assert!((val - 14.9).abs() <= 5.0);
    }

    proptest! {
        #[test]
        fn proptest_q1_disabled_snap_returns_exact_raw_coordinate(
            val in proptest::num::f64::NORMAL,
            grid_val in 10.0..=100.0f64
        ) {
            if let Ok(grid) = GridSize::try_grid_size(grid_val) {
                prop_assert_eq!(snap_node_coordinate(val, SnapMode::Disabled, grid), Ok(val));
            }
        }

        #[test]
        fn proptest_q2_enabled_snap_is_multiple_of_grid(
            val in proptest::num::f64::NORMAL,
            grid_val in 10.0..=100.0f64
        ) {
            if let Ok(grid) = GridSize::try_grid_size(grid_val) {
                let snapped = snap_node_coordinate(val, SnapMode::Enabled, grid).unwrap();
                let remainder = (snapped % grid_val).abs();
                prop_assert!(remainder < 1e-10 || (remainder - grid_val).abs() < 1e-10);
            }
        }

        #[test]
        fn proptest_q3_snap_distance_never_exceeds_half_grid(
            val in proptest::num::f64::NORMAL,
            grid_val in 10.0..=100.0f64
        ) {
            if let Ok(grid) = GridSize::try_grid_size(grid_val) {
                let snapped = snap_node_coordinate(val, SnapMode::Enabled, grid).unwrap();
                prop_assert!((snapped - val).abs() <= grid_val / 2.0 + 1e-10);
            }
        }

        #[test]
        fn proptest_q4_midway_ties_always_round_away_from_zero(
            multiplier in -100..100,
            grid_val in 10.0..=100.0f64
        ) {
            if let Ok(grid) = GridSize::try_grid_size(grid_val) {
                let base = multiplier as f64 * grid_val;
                let tie = base + (grid_val / 2.0);
                let snapped = snap_node_coordinate(tie, SnapMode::Enabled, grid).unwrap();
                if tie >= 0.0 {
                    prop_assert_eq!(snapped, base + grid_val);
                } else {
                    prop_assert_eq!(snapped, base - grid_val);
                }
            }
        }

        #[test]
        fn proptest_q5_finite_inputs_always_yield_finite_outputs(
            val in proptest::num::f64::NORMAL,
            grid_val in 10.0..=100.0f64
        ) {
            if let Ok(grid) = GridSize::try_grid_size(grid_val) {
                let snapped = snap_node_coordinate(val, SnapMode::Enabled, grid).unwrap();
                prop_assert!(snapped.is_finite());
            }
        }
    }
}
