//! Tests for geometry operations.
//!
//! This module contains unit tests for the operations functions.

use crate::geometry::operations::compute_subgraph_bounds;

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
