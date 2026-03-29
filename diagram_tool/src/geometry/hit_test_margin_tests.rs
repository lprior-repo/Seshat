#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Tests for hit test margin calculations.
//!
//! This module contains unit tests for the hit test margin functions.

use crate::geometry::hit_test_margin::{
    hit_test_with_margin, screen_to_world_margin, HitTestError,
};
use crate::geometry::{Point, Rectangle};
use canvas_math::{MAX_ZOOM, MIN_ZOOM};

// Helper to create point
const fn p(x: f64, y: f64) -> Point {
    Point::new(x, y)
}

// Helper to create rectangle
const fn r(x: f64, y: f64, w: f64, h: f64) -> Rectangle {
    Rectangle::new(x, y, w, h)
}

/// GEO-020-T010: Reject zoom below minimum
#[test]
fn test_reject_zoom_below_minimum() {
    let result = screen_to_world_margin(5.0, 0.05);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, HitTestError::InvalidZoom(_, _, _)));
}

/// GEO-020-T011: Reject zoom above maximum
#[test]
fn test_reject_zoom_above_maximum() {
    let result = screen_to_world_margin(5.0, 5.0);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, HitTestError::InvalidZoom(_, _, _)));
}

/// GEO-020-T012: Reject negative zoom
#[test]
fn test_reject_negative_zoom() {
    let result = screen_to_world_margin(5.0, -1.0);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        HitTestError::InvalidZoom(_, _, _)
    ));
}

/// GEO-020-T013: Reject zero margin
#[test]
fn test_reject_zero_margin() {
    let result = screen_to_world_margin(0.0, 1.0);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        HitTestError::InvalidMargin(0.0)
    ));
}

/// GEO-020-T014: Reject negative margin
#[test]
fn test_reject_negative_margin() {
    let result = screen_to_world_margin(-5.0, 1.0);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        HitTestError::InvalidMargin(-5.0)
    ));
}

/// GEO-020-T015: Reject NaN point coordinates
#[test]
fn test_reject_nan_point() {
    let rect = r(0.0, 0.0, 100.0, 100.0);
    let result = hit_test_with_margin(p(f64::NAN, 50.0), &rect, 1.0, 5.0);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        HitTestError::InvalidPoint(_, _)
    ));
}

/// GEO-020-T016: Reject infinite point coordinates
#[test]
fn test_reject_infinite_point() {
    let rect = r(0.0, 0.0, 100.0, 100.0);
    let result = hit_test_with_margin(p(f64::INFINITY, 50.0), &rect, 1.0, 5.0);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        HitTestError::InvalidPoint(_, _)
    ));
}

/// GEO-020-T030: Verify postcondition Q1 at min zoom
#[test]
fn test_postcondition_q1_min_zoom() {
    let result = screen_to_world_margin(5.0, 0.1).unwrap();
    assert!((result - 50.0).abs() < f64::EPSILON);
}

/// GEO-020-T031: Verify postcondition Q2 at max zoom
#[test]
fn test_postcondition_q2_max_zoom() {
    let result = screen_to_world_margin(5.0, 4.0).unwrap();
    assert!((result - 1.25).abs() < f64::EPSILON);
}

/// GEO-020-T032: Verify postcondition Q3 at unit zoom
#[test]
fn test_postcondition_q3_unit_zoom() {
    let result = screen_to_world_margin(5.0, 1.0).unwrap();
    assert!((result - 5.0).abs() < f64::EPSILON);
}

/// GEO-020-T020: Zoom at exact minimum boundary
#[test]
fn test_zoom_at_minimum_boundary() {
    let result = screen_to_world_margin(5.0, MIN_ZOOM);
    assert!(result.is_ok());
    assert!((result.unwrap() - 50.0).abs() < f64::EPSILON);
}

/// GEO-020-T021: Zoom at exact maximum boundary
#[test]
fn test_zoom_at_maximum_boundary() {
    let result = screen_to_world_margin(5.0, MAX_ZOOM);
    assert!(result.is_ok());
    assert!((result.unwrap() - 1.25).abs() < f64::EPSILON);
}

/// GEO-020-T022: Very small screen margin
#[test]
fn test_very_small_screen_margin() {
    let result = screen_to_world_margin(0.001, 1.0).unwrap();
    assert!((result - 0.001).abs() < f64::EPSILON);
}

/// GEO-020-T023: Very large screen margin at min zoom
#[test]
fn test_very_large_screen_margin() {
    let result = screen_to_world_margin(10000.0, MIN_ZOOM).unwrap();
    assert!((result - 100000.0).abs() < f64::EPSILON);
}

/// GEO-020-T024: Point exactly on margin boundary
#[test]
fn test_point_on_margin_boundary() {
    let rect = r(0.0, 0.0, 100.0, 100.0);
    let point = p(105.0, 50.0); // exactly 5 units from edge
    let result = hit_test_with_margin(point, &rect, 1.0, 5.0).unwrap();
    assert!(result);
}

/// GEO-020-T025: Point just outside margin
#[test]
fn test_point_just_outside_margin() {
    let rect = r(0.0, 0.0, 100.0, 100.0);
    let point = p(105.1, 50.0); // just outside 5-unit margin
    let result = hit_test_with_margin(point, &rect, 1.0, 5.0).unwrap();
    assert!(!result);
}

/// GEO-020-T001: Easy node selection when zoomed out
#[test]
fn test_easy_selection_zoomed_out() {
    let rect = r(0.0, 0.0, 100.0, 100.0);
    let point = p(105.0, 50.0); // 5 pixels from edge in screen space
                                // At zoom 0.1, margin = 50.0 world units, so 5 screen pixels = 50 world units
    let result = hit_test_with_margin(point, &rect, 0.1, 5.0).unwrap();
    assert!(result, "Should select node when zoomed out");
}

/// GEO-020-T002: Precise node selection when zoomed in
#[test]
fn test_precise_selection_zoomed_in() {
    let rect = r(0.0, 0.0, 100.0, 100.0);
    let point = p(105.0, 50.0); // 5 pixels from edge in screen space
                                // At zoom 4.0, margin = 1.25 world units, so 5 screen pixels = 1.25 world units
    let result = hit_test_with_margin(point, &rect, 4.0, 5.0).unwrap();
    assert!(!result, "Should NOT select node when zoomed in");
}

/// GEO-020-T003: Consistent selection at default zoom
#[test]
fn test_consistent_selection_default_zoom() {
    let rect = r(0.0, 0.0, 100.0, 100.0);
    let point = p(105.0, 50.0); // 5 pixels from edge in screen space
    let result = hit_test_with_margin(point, &rect, 1.0, 5.0).unwrap();
    assert!(result, "Should select node at default zoom");
}

/// GEO-020-T004: Hit test margin scales with zoom inversely
#[test]
fn test_margin_scales_with_zoom_inversely() {
    let margin_01 = screen_to_world_margin(5.0, 0.1).unwrap();
    let margin_10 = screen_to_world_margin(5.0, 1.0).unwrap();
    let margin_40 = screen_to_world_margin(5.0, 4.0).unwrap();

    assert!((margin_01 - 50.0).abs() < f64::EPSILON);
    assert!((margin_10 - 5.0).abs() < f64::EPSILON);
    assert!((margin_40 - 1.25).abs() < f64::EPSILON);

    // Monotonically decreasing
    assert!(margin_01 > margin_10);
    assert!(margin_10 > margin_40);
}

/// GEO-020-T033: Verify invariant I1 - screen-space consistency
#[test]
fn test_invariant_screen_space_consistency() {
    let rect = r(0.0, 0.0, 100.0, 100.0);
    // Point at 5 screen pixels from edge: at different zooms, this hits differently
    // because world margin changes to maintain same screen hit area

    // At zoom 0.1: world margin = 50, point at 5 world units from edge is WITHIN margin
    let hit_01 = hit_test_with_margin(p(105.0, 50.0), &rect, 0.1, 5.0).unwrap();

    // At zoom 1.0: world margin = 5, point at 5 world units from edge is ON boundary
    let hit_10 = hit_test_with_margin(p(105.0, 50.0), &rect, 1.0, 5.0).unwrap();

    // At zoom 4.0: world margin = 1.25, point at 5 world units from edge is OUTSIDE
    let hit_40 = hit_test_with_margin(p(105.0, 50.0), &rect, 4.0, 5.0).unwrap();

    // Screen-space consistency: lower zoom = larger hit area
    assert!(hit_01);
    assert!(hit_10);
    assert!(!hit_40);
}

/// GEO-020-T034: Verify invariant I2 - world margin decreases with zoom
#[test]
fn test_invariant_world_margin_decreases_with_zoom() {
    let margin_01 = screen_to_world_margin(5.0, 0.1).unwrap();
    let margin_10 = screen_to_world_margin(5.0, 1.0).unwrap();
    let margin_40 = screen_to_world_margin(5.0, 4.0).unwrap();

    assert!(margin_01 > margin_10);
    assert!(margin_10 > margin_40);
}
