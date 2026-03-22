#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]
#![allow(clippy::float_cmp)]

use crate::geometry::path::{
    simplify_path, simplify_path_default, PathError, PathSimplificationConfig,
};
use crate::geometry::Point;
use proptest::prelude::*;

// -----------------------------------------------------------------------------
// Proptest Generators
// -----------------------------------------------------------------------------

prop_compose! {
    fn arb_finite_f64()(f in any::<f64>()) -> f64 {
        if f.is_finite() { f } else { 0.0 }
    }
}

prop_compose! {
    fn arb_finite_point()(x in arb_finite_f64(), y in arb_finite_f64()) -> Point {
        Point::new(x, y)
    }
}

prop_compose! {
    fn arb_finite_points(min: usize, max: usize)(points in prop::collection::vec(arb_finite_point(), min..=max)) -> Vec<Point> {
        points
    }
}

prop_compose! {
    fn arb_epsilon()(eps in 0.0f64..1000.0f64) -> f64 {
        eps
    }
}

prop_compose! {
    fn arb_min_points()(min_pts in 2usize..10usize) -> usize {
        min_pts
    }
}

// -----------------------------------------------------------------------------
// Proptest Suites
// -----------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_valid_finite_inputs_never_panic(
        points in arb_finite_points(2, 50),
        epsilon in arb_epsilon(),
        min_points in arb_min_points(),
    ) {
        let config = PathSimplificationConfig::new(epsilon, min_points);
        if let Some(cfg) = config {
            let _ = simplify_path(&points, cfg);
            // We don't care about the result, only that it didn't panic.
        }
    }

    #[test]
    fn test_empty_and_small_inputs_return_insufficient_points(
        points in arb_finite_points(0, 5),
        min_points in 6usize..10usize, // Ensure min_points > points.len()
    ) {
        let config = PathSimplificationConfig::new(1.0, min_points).unwrap();
        let result = simplify_path(&points, config);
        assert_eq!(result, Err(PathError::InsufficientPoints));
    }

    #[test]
    fn test_negative_epsilon_handled_gracefully(
        points in arb_finite_points(2, 10),
        epsilon in -100.0f64..-0.0001f64
    ) {
        let config = PathSimplificationConfig::new(epsilon, 2);
        assert!(config.is_none(), "Config should fail to create with negative epsilon");
    }
}

// -----------------------------------------------------------------------------
// Explicit NaN and Infinity boundary tests
// -----------------------------------------------------------------------------

#[test]
fn test_nan_coordinates_rejected() {
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(f64::NAN, 1.0),
        Point::new(2.0, 2.0),
    ];
    let config = PathSimplificationConfig::default_config();
    let result = simplify_path(&points, config);
    assert_eq!(result, Err(PathError::InvalidPoint));
}

#[test]
fn test_infinity_coordinates_rejected() {
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(f64::INFINITY, 1.0),
        Point::new(2.0, 2.0),
    ];
    let config = PathSimplificationConfig::default_config();
    let result = simplify_path(&points, config);
    assert_eq!(result, Err(PathError::InvalidPoint));
}

#[test]
fn test_neg_infinity_coordinates_rejected() {
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(1.0, f64::NEG_INFINITY),
        Point::new(2.0, 2.0),
    ];
    let config = PathSimplificationConfig::default_config();
    let result = simplify_path(&points, config);
    assert_eq!(result, Err(PathError::InvalidPoint));
}

#[test]
fn test_simplify_path_default_wrapper() {
    let points = vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
    let result = simplify_path_default(&points);
    assert!(result.is_some());
    assert_eq!(result.unwrap().len(), 2);

    let nan_points = vec![Point::new(f64::NAN, 0.0), Point::new(1.0, 1.0)];
    let nan_result = simplify_path_default(&nan_points);
    assert!(nan_result.is_none());
}
