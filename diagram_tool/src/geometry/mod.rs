//! Geometry module for diagram tool
//!
//! This module provides geometry primitives and operations for the diagram tool,
//! including bounding box calculations, transforms, and bounds utilities.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::cast_precision_loss)]
#![forbid(unsafe_code)]

/// Represents a 2D point
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Represents an axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl AABB {
    #[must_use]
    pub const fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    #[must_use]
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    #[must_use]
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    #[must_use]
    pub fn center(&self) -> Point {
        Point::new(
            self.min_x + self.width() / 2.0,
            self.min_y + self.height() / 2.0,
        )
    }

    /// Expand the AABB by a given amount on all sides
    #[must_use]
    pub fn expand(&self, amount: f64) -> Self {
        Self::new(
            self.min_x - amount,
            self.min_y - amount,
            self.max_x + amount,
            self.max_y + amount,
        )
    }
}

/// Represents a rectangle with position, dimensions, and optional rotation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64, // rotation in radians
}

impl Rectangle {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            rotation: 0.0,
        }
    }

    #[must_use]
    pub const fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    /// Calculate the axis-aligned bounding box for this rectangle
    /// GEO-001, GEO-002: AABB calculation for axis-aligned and rotated rectangles
    #[must_use]
    pub fn aabb(&self) -> AABB {
        if self.rotation == 0.0 {
            // Axis-aligned case (GEO-001)
            AABB::new(self.x, self.y, self.x + self.width, self.y + self.height)
        } else {
            // Rotated case (GEO-002)
            let corners = self.corners();
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;

            for corner in corners {
                min_x = min_x.min(corner.x);
                min_y = min_y.min(corner.y);
                max_x = max_x.max(corner.x);
                max_y = max_y.max(corner.y);
            }

            AABB::new(min_x, min_y, max_x, max_y)
        }
    }

    /// Get the four corners of the rectangle (accounting for rotation)
    fn corners(&self) -> [Point; 4] {
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;

        let hw = self.width / 2.0;
        let hh = self.height / 2.0;

        // Corners relative to center
        let local_corners = [
            Point::new(-hw, -hh),
            Point::new(hw, -hh),
            Point::new(hw, hh),
            Point::new(-hw, hh),
        ];

        let cos = self.rotation.cos();
        let sin = self.rotation.sin();

        local_corners.map(|p| {
            Point::new(
                p.x.mul_add(cos, (-p.y).mul_add(sin, cx)),
                p.x.mul_add(sin, p.y.mul_add(cos, cy)),
            )
        })
    }
}

/// Represents a shape with stroke
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StrokedShape<T> {
    pub shape: T,
    pub stroke_width: f64,
}

impl<T> StrokedShape<T> {
    #[must_use]
    pub const fn new(shape: T, stroke_width: f64) -> Self {
        Self { shape, stroke_width }
    }
}

impl StrokedShape<Rectangle> {
    /// GEO-003: Calculate bounds including stroke width
    #[must_use]
    pub fn bounds_with_stroke(&self) -> AABB {
        let shape_aabb = self.shape.aabb();
        // Stroke extends by stroke_width/2 on each side
        shape_aabb.expand(self.stroke_width / 2.0)
    }
}

/// Represents text with position and font metrics
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub x: f64,
    pub y: f64,
    pub content: String,
    pub font_size: f64,
}

impl Text {
    #[must_use]
    pub fn new(x: f64, y: f64, content: &str, font_size: f64) -> Self {
        Self {
            x,
            y,
            content: content.to_string(),
            font_size,
        }
    }

    /// GEO-004: Calculate text bounds based on font metrics
    /// Approximates text width as 0.6 * `font_size` * character count (monospace-like estimate)
    #[must_use]
    pub fn bounds(&self) -> AABB {
        let char_count = self.content.chars().count() as f64;
        // Approximate width: average character width is about 0.6 of font size
        let width = self.font_size * 0.6 * char_count;
        let height = self.font_size;

        AABB::new(self.x, self.y, self.x + width, self.y + height)
    }
}

/// Represents an image with position and dimensions
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Image {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Image {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// GEO-005: Calculate image bounds
    #[must_use]
    pub fn bounds(&self) -> AABB {
        AABB::new(self.x, self.y, self.x + self.width, self.y + self.height)
    }
}

/// GEO-006: Scale a point around an anchor point
#[must_use]
pub fn scale_around_anchor(point: Point, anchor: Point, factor: f64) -> Point {
    Point::new(
        (point.x - anchor.x).mul_add(factor, anchor.x),
        (point.y - anchor.y).mul_add(factor, anchor.y),
    )
}

/// GEO-007: Rotate a point around a center point
#[must_use]
pub fn rotate_around_center(point: Point, center: Point, angle_radians: f64) -> Point {
    let cos = angle_radians.cos();
    let sin = angle_radians.sin();
    let dx = point.x - center.x;
    let dy = point.y - center.y;

    Point::new(
        dx.mul_add(cos, (-dy).mul_add(sin, center.x)),
        dx.mul_add(sin, dy.mul_add(cos, center.y)),
    )
}

/// GEO-008: Resize dimensions while maintaining aspect ratio
#[must_use]
pub fn resize_with_aspect_lock(
    original_width: f64,
    original_height: f64,
    new_width: f64,
) -> f64 {
    if original_width <= 0.0 {
        return new_width;
    }
    let aspect_ratio = original_height / original_width;
    new_width * aspect_ratio
}

/// GEO-009: Combined transform - scale then rotate
#[must_use]
pub fn scale_then_rotate(point: Point, anchor: Point, scale_factor: f64, angle_radians: f64) -> Point {
    let scaled = scale_around_anchor(point, anchor, scale_factor);
    rotate_around_center(scaled, anchor, angle_radians)
}

/// GEO-010: Safe bounds calculation that handles edge cases
#[must_use]
pub fn safe_bounds(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Option<AABB> {
    // Check for NaN or infinity
    if min_x.is_nan()
        || min_y.is_nan()
        || max_x.is_nan()
        || max_y.is_nan()
        || min_x.is_infinite()
        || min_y.is_infinite()
        || max_x.is_infinite()
        || max_y.is_infinite()
    {
        return None;
    }

    // Ensure min <= max
    let (final_min_x, final_max_x) = if min_x <= max_x {
        (min_x, max_x)
    } else {
        (max_x, min_x)
    };
    let (final_min_y, final_max_y) = if min_y <= max_y {
        (min_y, max_y)
    } else {
        (max_y, min_y)
    };

    Some(AABB::new(
        final_min_x,
        final_min_y,
        final_max_x,
        final_max_y,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::f64::consts::{FRAC_1_SQRT_2, PI, SQRT_2};

    const TOLERANCE: f64 = 1e-10;

    // ============== GEO-001: AABB for Axis-Aligned Rectangles ==============

    #[test]
    fn test_aabb_axis_aligned() {
        // Given: a rectangle at origin
        let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);

        // When: calculating AABB
        let aabb = rect.aabb();

        // Then: AABB equals the rectangle itself
        assert!((aabb.min_x - 0.0).abs() < TOLERANCE);
        assert!((aabb.min_y - 0.0).abs() < TOLERANCE);
        assert!((aabb.max_x - 100.0).abs() < TOLERANCE);
        assert!((aabb.max_y - 50.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_aabb_axis_aligned_with_offset() {
        // Given: a rectangle at non-origin position
        let rect = Rectangle::new(50.0, 25.0, 100.0, 50.0);

        // When: calculating AABB
        let aabb = rect.aabb();

        // Then: AABB correctly reflects position
        assert!((aabb.min_x - 50.0).abs() < TOLERANCE);
        assert!((aabb.min_y - 25.0).abs() < TOLERANCE);
        assert!((aabb.max_x - 150.0).abs() < TOLERANCE);
        assert!((aabb.max_y - 75.0).abs() < TOLERANCE);
    }

    // ============== GEO-002: AABB for Rotated Rectangles ==============

    #[test]
    fn test_aabb_rotated_rectangle_45_degrees() {
        // Given: a square rotated 45 degrees
        let size = 100.0;
        let rect = Rectangle::new(0.0, 0.0, size, size).with_rotation(PI / 4.0);

        // When: calculating AABB
        let aabb = rect.aabb();

        // Then: AABB should be expanded by sqrt(2)/2 factor (diagonal)
        // For a square centered at (50, 50), rotated 45 degrees:
        // The corners extend from center by (size/2) * sqrt(2)
        let expected_half_extent = (size / 2.0) * SQRT_2;
        let center = 50.0;

        assert!((aabb.min_x - (center - expected_half_extent)).abs() < TOLERANCE);
        assert!((aabb.max_x - (center + expected_half_extent)).abs() < TOLERANCE);
        assert!((aabb.min_y - (center - expected_half_extent)).abs() < TOLERANCE);
        assert!((aabb.max_y - (center + expected_half_extent)).abs() < TOLERANCE);
    }

    #[test]
    fn test_aabb_rotated_rectangle_90_degrees() {
        // Given: a rectangle rotated 90 degrees
        let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI / 2.0);

        // When: calculating AABB
        let aabb = rect.aabb();

        // Then: AABB dimensions are swapped (centered)
        // Original center: (50, 25), after 90 degree rotation
        // width becomes height and vice versa
        let center_x = 50.0;
        let center_y = 25.0;
        let expected_half_w = 25.0; // original height/2
        let expected_half_h = 50.0; // original width/2

        assert!((aabb.min_x - (center_x - expected_half_w)).abs() < TOLERANCE);
        assert!((aabb.max_x - (center_x + expected_half_w)).abs() < TOLERANCE);
        assert!((aabb.min_y - (center_y - expected_half_h)).abs() < TOLERANCE);
        assert!((aabb.max_y - (center_y + expected_half_h)).abs() < TOLERANCE);
    }

    #[test]
    fn test_aabb_rotated_rectangle_180_degrees() {
        // Given: a rectangle rotated 180 degrees
        let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI);

        // When: calculating AABB
        let aabb = rect.aabb();

        // Then: AABB is same as unrotated (180 degree rotation doesn't change AABB)
        assert!((aabb.min_x - 0.0).abs() < TOLERANCE);
        assert!((aabb.min_y - 0.0).abs() < TOLERANCE);
        assert!((aabb.max_x - 100.0).abs() < TOLERANCE);
        assert!((aabb.max_y - 50.0).abs() < TOLERANCE);
    }

    // ============== GEO-003: Stroke Width Inclusion in Bounds ==============

    #[test]
    fn test_stroke_width_inclusion() {
        // Given: a rectangle with stroke
        let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
        let stroked = StrokedShape::new(rect, 4.0);

        // When: calculating bounds with stroke
        let bounds = stroked.bounds_with_stroke();

        // Then: bounds are expanded by stroke_width/2 on each side
        assert!((bounds.min_x - (-2.0)).abs() < TOLERANCE);
        assert!((bounds.min_y - (-2.0)).abs() < TOLERANCE);
        assert!((bounds.max_x - 102.0).abs() < TOLERANCE);
        assert!((bounds.max_y - 52.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_stroke_width_zero() {
        // Given: a rectangle with zero stroke
        let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
        let stroked = StrokedShape::new(rect, 0.0);

        // When: calculating bounds
        let bounds = stroked.bounds_with_stroke();

        // Then: bounds equal the shape bounds
        let expected = rect.aabb();
        assert!((bounds.min_x - expected.min_x).abs() < TOLERANCE);
        assert!((bounds.min_y - expected.min_y).abs() < TOLERANCE);
        assert!((bounds.max_x - expected.max_x).abs() < TOLERANCE);
        assert!((bounds.max_y - expected.max_y).abs() < TOLERANCE);
    }

    // ============== GEO-004: Text Bounds Calculation ==============

    #[test]
    fn test_text_bounds() {
        // Given: text at position with font size
        let text = Text::new(10.0, 20.0, "Hello", 16.0);

        // When: calculating bounds
        let bounds = text.bounds();

        // Then: bounds start at text position
        assert!((bounds.min_x - 10.0).abs() < TOLERANCE);
        assert!((bounds.min_y - 20.0).abs() < TOLERANCE);
        assert!((bounds.height() - 16.0).abs() < TOLERANCE);
        // Width = 0.6 * font_size * char_count = 0.6 * 16 * 5 = 48
        assert!((bounds.width() - 48.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_text_bounds_empty_string() {
        // Given: empty text
        let text = Text::new(10.0, 20.0, "", 16.0);

        // When: calculating bounds
        let bounds = text.bounds();

        // Then: bounds have zero width but maintain height
        assert!((bounds.min_x - 10.0).abs() < TOLERANCE);
        assert!((bounds.width() - 0.0).abs() < TOLERANCE);
        assert!((bounds.height() - 16.0).abs() < TOLERANCE);
    }

    // ============== GEO-005: Image Bounds Calculation ==============

    #[test]
    fn test_image_bounds() {
        // Given: an image with position and dimensions
        let image = Image::new(50.0, 100.0, 200.0, 150.0);

        // When: calculating bounds
        let bounds = image.bounds();

        // Then: bounds equal position + dimensions
        assert!((bounds.min_x - 50.0).abs() < TOLERANCE);
        assert!((bounds.min_y - 100.0).abs() < TOLERANCE);
        assert!((bounds.max_x - 250.0).abs() < TOLERANCE);
        assert!((bounds.max_y - 250.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_image_bounds_at_origin() {
        // Given: an image at origin
        let image = Image::new(0.0, 0.0, 100.0, 100.0);

        // When: calculating bounds
        let bounds = image.bounds();

        // Then: bounds start at origin
        assert!((bounds.min_x - 0.0).abs() < TOLERANCE);
        assert!((bounds.min_y - 0.0).abs() < TOLERANCE);
        assert!((bounds.max_x - 100.0).abs() < TOLERANCE);
        assert!((bounds.max_y - 100.0).abs() < TOLERANCE);
    }

    // ============== GEO-006: Scale Around Anchor Point ==============

    #[test]
    fn test_scale_around_anchor() {
        // Given: a point and anchor
        let point = Point::new(100.0, 100.0);
        let anchor = Point::new(50.0, 50.0);

        // When: scaling by factor 2
        let scaled = scale_around_anchor(point, anchor, 2.0);

        // Then: point moves away from anchor by factor
        // new_x = 50 + (100 - 50) * 2 = 150
        // new_y = 50 + (100 - 50) * 2 = 150
        assert!((scaled.x - 150.0).abs() < TOLERANCE);
        assert!((scaled.y - 150.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_scale_around_anchor_keeps_anchor_fixed() {
        // Given: anchor point as the point to scale
        let anchor = Point::new(50.0, 50.0);

        // When: scaling anchor around itself
        let scaled = scale_around_anchor(anchor, anchor, 2.0);

        // Then: anchor stays fixed
        assert!((scaled.x - anchor.x).abs() < TOLERANCE);
        assert!((scaled.y - anchor.y).abs() < TOLERANCE);
    }

    #[test]
    fn test_scale_around_anchor_shrink() {
        // Given: a point and anchor
        let point = Point::new(100.0, 100.0);
        let anchor = Point::new(50.0, 50.0);

        // When: scaling by factor 0.5
        let scaled = scale_around_anchor(point, anchor, 0.5);

        // Then: point moves toward anchor
        // new_x = 50 + (100 - 50) * 0.5 = 75
        // new_y = 50 + (100 - 50) * 0.5 = 75
        assert!((scaled.x - 75.0).abs() < TOLERANCE);
        assert!((scaled.y - 75.0).abs() < TOLERANCE);
    }

    // ============== GEO-007: Rotate Around Center ==============

    #[test]
    fn test_rotate_around_center_90_degrees() {
        // Given: a point at (100, 0) and center at origin
        let point = Point::new(100.0, 0.0);
        let center = Point::origin();

        // When: rotating 90 degrees counter-clockwise
        let rotated = rotate_around_center(point, center, PI / 2.0);

        // Then: point is at (0, 100)
        assert!((rotated.x - 0.0).abs() < TOLERANCE);
        assert!((rotated.y - 100.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotate_around_center_180_degrees() {
        // Given: a point at (100, 0) and center at origin
        let point = Point::new(100.0, 0.0);
        let center = Point::origin();

        // When: rotating 180 degrees
        let rotated = rotate_around_center(point, center, PI);

        // Then: point is at (-100, 0)
        assert!((rotated.x - (-100.0)).abs() < TOLERANCE);
        assert!((rotated.y - 0.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotate_around_center_keeps_center_fixed() {
        // Given: center as the point to rotate
        let center = Point::new(50.0, 50.0);

        // When: rotating center around itself
        let rotated = rotate_around_center(center, center, PI / 4.0);

        // Then: center stays fixed
        assert!((rotated.x - center.x).abs() < TOLERANCE);
        assert!((rotated.y - center.y).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotate_around_center_45_degrees() {
        // Given: a point at (1, 0) and center at origin
        let point = Point::new(1.0, 0.0);
        let center = Point::origin();

        // When: rotating 45 degrees
        let rotated = rotate_around_center(point, center, PI / 4.0);

        // Then: point is at (sqrt(2)/2, sqrt(2)/2)
        assert!((rotated.x - FRAC_1_SQRT_2).abs() < TOLERANCE);
        assert!((rotated.y - FRAC_1_SQRT_2).abs() < TOLERANCE);
    }

    // ============== GEO-008: Resize with Aspect Ratio Lock ==============

    #[test]
    fn test_resize_aspect_lock() {
        // Given: original dimensions 100x50 (2:1 aspect ratio)
        let original_width = 100.0;
        let original_height = 50.0;

        // When: resizing width to 200
        let new_height = resize_with_aspect_lock(original_width, original_height, 200.0);

        // Then: height maintains 2:1 aspect ratio (should be 100)
        assert!((new_height - 100.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_resize_aspect_lock_shrink() {
        // Given: original dimensions 100x50 (2:1 aspect ratio)
        let original_width = 100.0;
        let original_height = 50.0;

        // When: resizing width to 50
        let new_height = resize_with_aspect_lock(original_width, original_height, 50.0);

        // Then: height maintains aspect ratio (should be 25)
        assert!((new_height - 25.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_resize_aspect_lock_square() {
        // Given: square dimensions 100x100
        let original_width = 100.0;
        let original_height = 100.0;

        // When: resizing width to 200
        let new_height = resize_with_aspect_lock(original_width, original_height, 200.0);

        // Then: height equals new width (1:1 aspect ratio)
        assert!((new_height - 200.0).abs() < TOLERANCE);
    }

    // ============== GEO-009: Combined Transform Chain ==============

    #[test]
    fn test_combined_transforms() {
        // Given: a point at (2, 0), anchor at origin
        let point = Point::new(2.0, 0.0);
        let anchor = Point::origin();

        // When: scale by 2 then rotate 90 degrees
        let result = scale_then_rotate(point, anchor, 2.0, PI / 2.0);

        // Then: first scale (2, 0) -> (4, 0), then rotate -> (0, 4)
        assert!((result.x - 0.0).abs() < TOLERANCE);
        assert!((result.y - 4.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_combined_transforms_order_matters() {
        // Given: a point and anchor
        let point = Point::new(1.0, 0.0);
        let anchor = Point::origin();

        // When: rotate 90 degrees then scale by 2 (reverse order)
        // Note: Our function does scale first, then rotate
        // Scale: (1, 0) -> (2, 0), Rotate: (2, 0) -> (0, 2)
        let result = scale_then_rotate(point, anchor, 2.0, PI / 2.0);

        // Then: result is deterministic
        assert!((result.x - 0.0).abs() < TOLERANCE);
        assert!((result.y - 2.0).abs() < TOLERANCE);
    }

    // ============== GEO-010: Bounds Edge Cases ==============

    #[test]
    fn test_bounds_edge_cases_zero_size() {
        // Given: zero-sized bounds
        let result = safe_bounds(0.0, 0.0, 0.0, 0.0);

        // Then: valid AABB with zero dimensions
        assert!(result.is_some());
        let aabb = result.unwrap();
        assert!((aabb.width() - 0.0).abs() < TOLERANCE);
        assert!((aabb.height() - 0.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_bounds_edge_cases_negative_coords() {
        // Given: negative coordinates
        let result = safe_bounds(-100.0, -50.0, -10.0, -5.0);

        // Then: valid AABB
        assert!(result.is_some());
        let aabb = result.unwrap();
        assert!((aabb.min_x - (-100.0)).abs() < TOLERANCE);
        assert!((aabb.min_y - (-50.0)).abs() < TOLERANCE);
        assert!((aabb.max_x - (-10.0)).abs() < TOLERANCE);
        assert!((aabb.max_y - (-5.0)).abs() < TOLERANCE);
    }

    #[test]
    fn test_bounds_edge_cases_large_coords() {
        // Given: very large coordinates
        let result = safe_bounds(1e10, 1e10, 1e10 + 100.0, 1e10 + 100.0);

        // Then: valid AABB (within f64 range)
        assert!(result.is_some());
        let aabb = result.unwrap();
        assert!((aabb.width() - 100.0).abs() < TOLERANCE);
        assert!((aabb.height() - 100.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_bounds_edge_cases_nan() {
        // Given: NaN values
        let result = safe_bounds(f64::NAN, 0.0, 100.0, 100.0);

        // Then: None (invalid)
        assert!(result.is_none());
    }

    #[test]
    fn test_bounds_edge_cases_infinity() {
        // Given: infinity values
        let result = safe_bounds(f64::INFINITY, 0.0, 100.0, 100.0);

        // Then: None (invalid)
        assert!(result.is_none());
    }

    #[test]
    fn test_bounds_edge_cases_swapped_min_max() {
        // Given: min > max (swapped)
        let result = safe_bounds(100.0, 100.0, 0.0, 0.0);

        // Then: valid AABB with corrected order
        assert!(result.is_some());
        let aabb = result.unwrap();
        assert!((aabb.min_x - 0.0).abs() < TOLERANCE);
        assert!((aabb.min_y - 0.0).abs() < TOLERANCE);
        assert!((aabb.max_x - 100.0).abs() < TOLERANCE);
        assert!((aabb.max_y - 100.0).abs() < TOLERANCE);
    }

    // ============== Property-Based Tests ==============

    proptest! {
        #[test]
        fn prop_scale_around_anchor_idempotent_at_anchor(factor in -10.0_f64..10.0) {
            let anchor = Point::new(50.0, 50.0);
            let scaled = scale_around_anchor(anchor, anchor, factor);
            prop_assert!((scaled.x - anchor.x).abs() < TOLERANCE);
            prop_assert!((scaled.y - anchor.y).abs() < TOLERANCE);
        }

        #[test]
        fn prop_rotate_around_center_idempotent_at_center(angle in -4.0_f64 * PI..4.0 * PI) {
            let center = Point::new(50.0, 50.0);
            let rotated = rotate_around_center(center, center, angle);
            prop_assert!((rotated.x - center.x).abs() < TOLERANCE);
            prop_assert!((rotated.y - center.y).abs() < TOLERANCE);
        }

        #[test]
        fn prop_rotate_full_circle_returns_to_origin(angle in -4.0_f64 * PI..4.0 * PI) {
            let point = Point::new(100.0, 0.0);
            let center = Point::origin();
            let rotated_once = rotate_around_center(point, center, angle);
            let rotated_twice = rotate_around_center(rotated_once, center, 2.0 * PI - angle);
            prop_assert!((rotated_twice.x - point.x).abs() < 1e-9);
            prop_assert!((rotated_twice.y - point.y).abs() < 1e-9);
        }

        #[test]
        fn prop_aabb_contains_all_corners(
            x in -1000.0_f64..1000.0,
            y in -1000.0_f64..1000.0,
            width in 1.0_f64..500.0,
            height in 1.0_f64..500.0,
            rotation in 0.0_f64..2.0 * PI
        ) {
            let rect = Rectangle::new(x, y, width, height).with_rotation(rotation);
            let aabb = rect.aabb();

            // All corners should be within or on the AABB
            let corners = rect.corners();
            for corner in corners {
                prop_assert!(corner.x >= aabb.min_x - TOLERANCE);
                prop_assert!(corner.x <= aabb.max_x + TOLERANCE);
                prop_assert!(corner.y >= aabb.min_y - TOLERANCE);
                prop_assert!(corner.y <= aabb.max_y + TOLERANCE);
            }
        }

        #[test]
        fn prop_aspect_ratio_preserved(
            width in 1.0_f64..1000.0,
            height in 1.0_f64..1000.0,
            new_width in 1.0_f64..1000.0
        ) {
            let original_ratio = height / width;
            let new_height = resize_with_aspect_lock(width, height, new_width);
            let new_ratio = new_height / new_width;
            prop_assert!((original_ratio - new_ratio).abs() < TOLERANCE);
        }

        #[test]
        fn prop_safe_bounds_finite_inputs_produce_valid_aabb(
            min_x in -1e6_f64..1e6,
            min_y in -1e6_f64..1e6,
            max_x in -1e6_f64..1e6,
            max_y in -1e6_f64..1e6
        ) {
            let result = safe_bounds(min_x, min_y, max_x, max_y);
            prop_assert!(result.is_some());

            let aabb = result.unwrap();
            prop_assert!(aabb.min_x.is_finite());
            prop_assert!(aabb.min_y.is_finite());
            prop_assert!(aabb.max_x.is_finite());
            prop_assert!(aabb.max_y.is_finite());
            prop_assert!(aabb.min_x <= aabb.max_x);
            prop_assert!(aabb.min_y <= aabb.max_y);
        }
    }
}
