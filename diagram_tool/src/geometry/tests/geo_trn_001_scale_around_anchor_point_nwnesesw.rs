use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-TRN-001: Scale Around Anchor Point (NW/NE/SE/SW) ==============
//
// These tests verify scaling operations that use corner anchor points.
// When scaling around a corner, that corner remains fixed while other
// corners move toward or away from it.

/// Get corner anchor point for a rectangle
#[derive(Debug, Clone, Copy, PartialEq)]
enum Corner {
    NorthWest,
    NorthEast,
    SouthEast,
    SouthWest,
}

fn get_corner_point(rect: &Rectangle, corner: Corner) -> Point {
    match corner {
        Corner::NorthWest => Point::new(rect.x, rect.y),
        Corner::NorthEast => Point::new(rect.x + rect.width, rect.y),
        Corner::SouthEast => Point::new(rect.x + rect.width, rect.y + rect.height),
        Corner::SouthWest => Point::new(rect.x, rect.y + rect.height),
    }
}

/// Scale a rectangle around a corner anchor point
fn scale_rect_around_corner(rect: &Rectangle, corner: Corner, factor: f64) -> Rectangle {
    let anchor = get_corner_point(rect, corner);

    // Scale all corners around the anchor
    let nw = scale_around_anchor(get_corner_point(rect, Corner::NorthWest), anchor, factor);
    let se = scale_around_anchor(get_corner_point(rect, Corner::SouthEast), anchor, factor);

    // Compute new rectangle from scaled corners
    // Width and height are the differences between opposite corners
    let new_width = (se.x - nw.x).abs();
    let new_height = (se.y - nw.y).abs();

    // Determine the new origin (top-left corner)
    let (new_x, new_y) = match corner {
        Corner::NorthWest => (anchor.x, anchor.y),
        Corner::NorthEast => (anchor.x - new_width, anchor.y),
        Corner::SouthEast => (anchor.x - new_width, anchor.y - new_height),
        Corner::SouthWest => (anchor.x, anchor.y - new_height),
    };

    Rectangle::new(new_x, new_y, new_width, new_height)
}

#[cfg(kani)]
#[kani::proof]
fn test_scale_around_anchor_nw() {
    // Given: a rectangle at origin with size 100x50
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
    let factor = 2.0;

    // When: scaling around NW corner (top-left)
    let scaled = scale_rect_around_corner(&rect, Corner::NorthWest, factor);

    // Then: NW corner stays fixed, others move away
    // Original: NW at (0, 0), SE at (100, 50)
    // After 2x scale around NW: NW stays (0, 0), SE moves to (200, 100)
    assert!((scaled.x - 0.0).abs() < TOLERANCE);
    assert!((scaled.y - 0.0).abs() < TOLERANCE);
    assert!((scaled.width - 200.0).abs() < TOLERANCE);
    assert!((scaled.height - 100.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_scale_around_anchor_ne() {
    // Given: a rectangle at origin with size 100x50
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
    let factor = 2.0;

    // When: scaling around NE corner (top-right)
    let scaled = scale_rect_around_corner(&rect, Corner::NorthEast, factor);

    // Then: NE corner stays fixed at (100, 0)
    // New width is 200, so x becomes 100 - 200 = -100
    assert!((scaled.x - (-100.0)).abs() < TOLERANCE);
    assert!((scaled.y - 0.0).abs() < TOLERANCE);
    assert!((scaled.width - 200.0).abs() < TOLERANCE);
    assert!((scaled.height - 100.0).abs() < TOLERANCE);
    // NE corner should still be at (100, 0)
    assert!((get_corner_point(&scaled, Corner::NorthEast).x - 100.0).abs() < TOLERANCE);
    assert!((get_corner_point(&scaled, Corner::NorthEast).y - 0.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_scale_around_anchor_se() {
    // Given: a rectangle at origin with size 100x50
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
    let factor = 2.0;

    // When: scaling around SE corner (bottom-right)
    let scaled = scale_rect_around_corner(&rect, Corner::SouthEast, factor);

    // Then: SE corner stays fixed at (100, 50)
    // New width is 200, height is 100
    // x = 100 - 200 = -100, y = 50 - 100 = -50
    assert!((scaled.x - (-100.0)).abs() < TOLERANCE);
    assert!((scaled.y - (-50.0)).abs() < TOLERANCE);
    assert!((scaled.width - 200.0).abs() < TOLERANCE);
    assert!((scaled.height - 100.0).abs() < TOLERANCE);
    // SE corner should still be at (100, 50)
    let se = get_corner_point(&scaled, Corner::SouthEast);
    assert!((se.x - 100.0).abs() < TOLERANCE);
    assert!((se.y - 50.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_scale_around_anchor_sw() {
    // Given: a rectangle at origin with size 100x50
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
    let factor = 2.0;

    // When: scaling around SW corner (bottom-left)
    let scaled = scale_rect_around_corner(&rect, Corner::SouthWest, factor);

    // Then: SW corner stays fixed at (0, 50)
    // New width is 200, height is 100
    // x stays 0, y = 50 - 100 = -50
    assert!((scaled.x - 0.0).abs() < TOLERANCE);
    assert!((scaled.y - (-50.0)).abs() < TOLERANCE);
    assert!((scaled.width - 200.0).abs() < TOLERANCE);
    assert!((scaled.height - 100.0).abs() < TOLERANCE);
    // SW corner should still be at (0, 50)
    let sw = get_corner_point(&scaled, Corner::SouthWest);
    assert!((sw.x - 0.0).abs() < TOLERANCE);
    assert!((sw.y - 50.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_scale_around_anchor_shrink_nw() {
    // Given: a rectangle at origin with size 100x50
    let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
    let factor = 0.5;

    // When: shrinking around NW corner
    let scaled = scale_rect_around_corner(&rect, Corner::NorthWest, factor);

    // Then: NW corner stays fixed, size halves
    assert!((scaled.x - 0.0).abs() < TOLERANCE);
    assert!((scaled.y - 0.0).abs() < TOLERANCE);
    assert!((scaled.width - 50.0).abs() < TOLERANCE);
    assert!((scaled.height - 25.0).abs() < TOLERANCE);
}
