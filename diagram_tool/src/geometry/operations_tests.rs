#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
//! Tests for geometry operations.
//!
//! This module contains unit tests for the operations functions.

// ============== GEO-025: Container Bounds Recomputation ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_single_child_container() {
    // Given: a subgraph container with one child
    let children = vec![(100.0, 100.0, 50.0, 30.0)];

    // When: computing subgraph bounds
    let result = compute_subgraph_bounds(children);

    // Then: result should be the child's bounds
    assert_eq!(result, Some((100.0, 100.0, 50.0, 30.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_multiple_children_horizontal_spread() {
    // Given: children spread horizontally
    let children = vec![
        (0.0, 0.0, 50.0, 50.0),
        (100.0, 0.0, 50.0, 50.0),
        (200.0, 0.0, 50.0, 50.0),
    ];

    // When: computing subgraph bounds
    let result = compute_subgraph_bounds(children);

    // Then: result should encompass all children
    assert_eq!(result, Some((0.0, 0.0, 250.0, 50.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_multiple_children_vertical_spread() {
    // Given: children spread vertically
    let children = vec![
        (0.0, 0.0, 50.0, 50.0),
        (0.0, 100.0, 50.0, 50.0),
        (0.0, 200.0, 50.0, 50.0),
    ];

    // When: computing subgraph bounds
    let result = compute_subgraph_bounds(children);

    // Then: result should encompass all children
    assert_eq!(result, Some((0.0, 0.0, 50.0, 250.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_empty_container() {
    // Given: no children
    let children: Vec<(f64, f64, f64, f64)> = vec![];

    // When: computing subgraph bounds
    let result = compute_subgraph_bounds(children);

    // Then: result should be None
    assert_eq!(result, None);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_child_with_negative_coordinates() {
    // Given: children with negative coordinates
    let children = vec![(-50.0, -50.0, 50.0, 50.0), (0.0, 0.0, 50.0, 50.0)];

    // When: computing subgraph bounds
    let result = compute_subgraph_bounds(children);

    // Then: result should handle negative coords
    assert_eq!(result, Some((-50.0, -50.0, 100.0, 100.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_child_overlap() {
    // Given: overlapping children
    let children = vec![(0.0, 0.0, 100.0, 100.0), (50.0, 50.0, 100.0, 100.0)];

    // When: computing subgraph bounds
    let result = compute_subgraph_bounds(children);

    // Then: result should be the union
    assert_eq!(result, Some((0.0, 0.0, 150.0, 150.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_invalid_child_nan() {
    // Given: one valid child and one with NaN coordinates
    let children = vec![(0.0, 0.0, 50.0, 50.0), (f64::NAN, 0.0, 50.0, 50.0)];

    // When: computing subgraph bounds
    let result = compute_subgraph_bounds(children);

    // Then: result should contain only valid child
    assert_eq!(result, Some((0.0, 0.0, 50.0, 50.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_invalid_child_infinity() {
    // Given: one valid child and one with Infinity coordinates
    let children = vec![(0.0, 0.0, 50.0, 50.0), (f64::INFINITY, 0.0, 50.0, 50.0)];

    // When: computing subgraph bounds
    let result = compute_subgraph_bounds(children);

    // Then: result should contain only valid child
    assert_eq!(result, Some((0.0, 0.0, 50.0, 50.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_single_point_child() {
    // Given: child with zero size
    let children = vec![(50.0, 50.0, 0.0, 0.0)];

    // When: computing subgraph bounds
    let result = compute_subgraph_bounds(children);

    // Then: result should be degenerate but valid
    assert_eq!(result, Some((50.0, 50.0, 0.0, 0.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_all_children_at_same_position() {
    // Given: multiple children at same position with different sizes
    let children = vec![
        (50.0, 50.0, 30.0, 30.0),
        (50.0, 50.0, 50.0, 50.0),
        (50.0, 50.0, 20.0, 40.0),
    ];

    // When: computing subgraph bounds
    let result = compute_subgraph_bounds(children);

    // Then: result should be the union
    assert_eq!(result, Some((50.0, 50.0, 50.0, 50.0)));
}

#[cfg(test)]
mod proptests {
    use crate::geometry::operations::container_bounds::compute_subgraph_bounds;
    use crate::geometry::operations::edge_bounds::{edge_bounds, EdgeArrowType, EdgeBoundsError};
    use crate::geometry::primitives::Point;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_compute_subgraph_bounds_robustness(
            coords in prop::collection::vec(
                (
                    prop::num::f64::ANY,
                    prop::num::f64::ANY,
                    prop::num::f64::ANY,
                    prop::num::f64::ANY
                ),
                0..10
            )
        ) {
            // Function should not panic, returning None or valid AABB
            let result = compute_subgraph_bounds(coords);
            if let Some((min_x, min_y, width, height)) = result {
                assert!(min_x.is_finite());
                assert!(min_y.is_finite());
                assert!(width.is_finite());
                assert!(height.is_finite());
            }
        }

        #[test]
        fn test_edge_bounds_robustness(
            s_x in prop::num::f64::ANY,
            s_y in prop::num::f64::ANY,
            t_x in prop::num::f64::ANY,
            t_y in prop::num::f64::ANY,
            thickness in prop::num::f64::ANY,
            bend_points_x in prop::collection::vec(prop::num::f64::ANY, 0..5),
            bend_points_y in prop::collection::vec(prop::num::f64::ANY, 0..5),
            arrow_type_idx in 0..5usize
        ) {
            let arrow_type = match arrow_type_idx {
                0 => EdgeArrowType::Default,
                1 => EdgeArrowType::Sharp,
                2 => EdgeArrowType::Curved,
                3 => EdgeArrowType::Step,
                _ => EdgeArrowType::Straight,
            };

            let bend_points: Vec<Point> = bend_points_x.iter().zip(bend_points_y.iter())
                .map(|(&x, &y)| Point::new(x, y)).collect();

            // Function should not panic
            let _ = edge_bounds(
                Point::new(s_x, s_y),
                Point::new(t_x, t_y),
                arrow_type,
                thickness,
                &bend_points
            );
        }

        #[test]
        fn test_edge_bounds_valid_range(
            s_x in -10000.0..10000.0f64,
            s_y in -10000.0..10000.0f64,
            t_x in -10000.0..10000.0f64,
            t_y in -10000.0..10000.0f64,
            thickness in 0.1..100.0f64,
            bend_points_x in prop::collection::vec(-10000.0..10000.0f64, 0..5),
            bend_points_y in prop::collection::vec(-10000.0..10000.0f64, 0..5),
            arrow_type_idx in 0..5usize
        ) {
            let arrow_type = match arrow_type_idx {
                0 => EdgeArrowType::Default,
                1 => EdgeArrowType::Sharp,
                2 => EdgeArrowType::Curved,
                3 => EdgeArrowType::Step,
                _ => EdgeArrowType::Straight,
            };

            let bend_points: Vec<Point> = bend_points_x.iter().zip(bend_points_y.iter())
                .map(|(&x, &y)| Point::new(x, y)).collect();

            let result = edge_bounds(
                Point::new(s_x, s_y),
                Point::new(t_x, t_y),
                arrow_type,
                thickness,
                &bend_points
            );

            prop_assert!(result.is_ok());
            if let Ok(bounds) = result {
                prop_assert!(bounds.min_x.is_finite());
                prop_assert!(bounds.min_y.is_finite());
                prop_assert!(bounds.max_x.is_finite());
                prop_assert!(bounds.max_y.is_finite());
                prop_assert!(bounds.min_x <= bounds.max_x);
                prop_assert!(bounds.min_y <= bounds.max_y);
            }
        }
    }
}
