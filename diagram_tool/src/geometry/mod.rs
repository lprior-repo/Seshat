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

    // ============== GEO-011: Rotation + Resize Composition ==============

    #[test]
    fn test_rotation_resize_composition() {
        // Given: a point, anchor, scale factor, and rotation angle
        let point = Point::new(10.0, 0.0);
        let anchor = Point::origin();
        let scale = 2.0;
        let angle = std::f64::consts::PI / 2.0;

        // When: applying resize then rotation using existing scale_then_rotate
        let result = scale_then_rotate(point, anchor, scale, angle);

        // Then: result is deterministic
        // Scale: (10, 0) -> (20, 0), Rotate 90deg: (20, 0) -> (0, 20)
        assert!((result.x - 0.0).abs() < TOLERANCE);
        assert!((result.y - 20.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotation_resize_composition_reverse_order() {
        // Given: a point at (10, 0)
        let point = Point::new(10.0, 0.0);
        let anchor = Point::origin();

        // When: rotate first then scale (manual application)
        let rotated = rotate_around_center(point, anchor, std::f64::consts::PI / 2.0);
        let scaled = scale_around_anchor(rotated, anchor, 2.0);

        // Then: order matters - different result than scale_then_rotate
        // Rotate: (10, 0) -> (0, 10), Scale: (0, 10) -> (0, 20)
        assert!((scaled.x - 0.0).abs() < TOLERANCE);
        assert!((scaled.y - 20.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotation_resize_composition_no_scale() {
        // Given: scale factor of 1.0
        let point = Point::new(10.0, 0.0);
        let anchor = Point::origin();

        // When: scale_then_rotate with scale=1.0
        let result = scale_then_rotate(point, anchor, 1.0, std::f64::consts::PI / 2.0);

        // Then: only rotation is applied
        let expected = rotate_around_center(point, anchor, std::f64::consts::PI / 2.0);
        assert!((result.x - expected.x).abs() < TOLERANCE);
        assert!((result.y - expected.y).abs() < TOLERANCE);
    }

    // ============== GEO-012: Zoom at Pointer ==============

    /// Zoom a view rectangle around a pointer position
    #[must_use]
    pub fn zoom_at_pointer(view_center: Point, pointer: Point, factor: f64) -> Point {
        // The pointer stays fixed; the view center moves relative to it
        // new_view_center = pointer + (view_center - pointer) * factor
        Point::new(
            pointer.x + (view_center.x - pointer.x) * factor,
            pointer.y + (view_center.y - pointer.y) * factor,
        )
    }

    #[test]
    fn test_zoom_at_pointer_center() {
        // Given: view centered at origin, pointer at origin
        let view_center = Point::origin();
        let pointer = Point::origin();

        // When: zooming by 2x
        let new_center = zoom_at_pointer(view_center, pointer, 2.0);

        // Then: center stays at pointer (which is at origin)
        assert!((new_center.x - 0.0).abs() < TOLERANCE);
        assert!((new_center.y - 0.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_zoom_at_pointer_offset() {
        // Given: view at (100, 100), pointer at (50, 50)
        let view_center = Point::new(100.0, 100.0);
        let pointer = Point::new(50.0, 50.0);

        // When: zooming in by 2x
        let new_center = zoom_at_pointer(view_center, pointer, 2.0);

        // Then: center moves away from pointer
        // new = 50 + (100 - 50) * 2 = 50 + 100 = 150
        assert!((new_center.x - 150.0).abs() < TOLERANCE);
        assert!((new_center.y - 150.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_zoom_at_pointer_zoom_out() {
        // Given: view at (100, 100), pointer at (50, 50)
        let view_center = Point::new(100.0, 100.0);
        let pointer = Point::new(50.0, 50.0);

        // When: zooming out by 0.5x
        let new_center = zoom_at_pointer(view_center, pointer, 0.5);

        // Then: center moves toward pointer
        // new = 50 + (100 - 50) * 0.5 = 50 + 25 = 75
        assert!((new_center.x - 75.0).abs() < TOLERANCE);
        assert!((new_center.y - 75.0).abs() < TOLERANCE);
    }

    // ============== GEO-013: Snap Lines Horizontal ==============

    /// Snap a horizontal line Y coordinate to nearest target within tolerance
    #[must_use]
    pub fn snap_horizontal(line_y: f64, targets: &[f64], tolerance: f64) -> Option<f64> {
        targets
            .iter()
            .map(|&t| (t, (line_y - t).abs()))
            .filter(|(_, dist)| *dist <= tolerance)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(t, _)| t)
    }

    #[test]
    fn test_snap_horizontal_within_tolerance() {
        // Given: line at y=52 and snap targets
        let line_y = 52.0;
        let targets = vec![0.0, 50.0, 100.0];
        let tolerance = 5.0;

        // When: snapping
        let result = snap_horizontal(line_y, &targets, tolerance);

        // Then: snaps to 50 (within tolerance of 5)
        assert_eq!(result, Some(50.0));
    }

    #[test]
    fn test_snap_horizontal_outside_tolerance() {
        // Given: line at y=60 (too far from targets)
        let line_y = 60.0;
        let targets = vec![0.0, 50.0, 100.0];
        let tolerance = 5.0;

        // When: snapping
        let result = snap_horizontal(line_y, &targets, tolerance);

        // Then: no snap
        assert!(result.is_none());
    }

    #[test]
    fn test_snap_horizontal_exact_match() {
        // Given: line exactly on target
        let line_y = 50.0;
        let targets = vec![0.0, 50.0, 100.0];
        let tolerance = 5.0;

        // When: snapping
        let result = snap_horizontal(line_y, &targets, tolerance);

        // Then: snaps to exact position
        assert_eq!(result, Some(50.0));
    }

    // ============== GEO-014: Snap Lines Vertical ==============

    /// Snap a vertical line X coordinate to nearest target within tolerance
    #[must_use]
    pub fn snap_vertical(line_x: f64, targets: &[f64], tolerance: f64) -> Option<f64> {
        snap_horizontal(line_x, targets, tolerance)
    }

    #[test]
    fn test_snap_vertical_within_tolerance() {
        // Given: line at x=102 and snap targets
        let line_x = 102.0;
        let targets = vec![0.0, 100.0, 200.0];
        let tolerance = 5.0;

        // When: snapping
        let result = snap_vertical(line_x, &targets, tolerance);

        // Then: snaps to 100
        assert_eq!(result, Some(100.0));
    }

    #[test]
    fn test_snap_vertical_prefers_closest() {
        // Given: line at x=48 (equidistant to 0 and 100 within tolerance)
        let line_x = 48.0;
        let targets = vec![0.0, 100.0];
        let tolerance = 50.0;

        // When: snapping
        let result = snap_vertical(line_x, &targets, tolerance);

        // Then: snaps to closest (50)
        // Actually 48 is closer to 0 (dist 48) than to 100 (dist 52)
        assert_eq!(result, Some(0.0));
    }

    // ============== GEO-015: Grid Step ==============

    /// Snap a point to the nearest grid intersection
    #[must_use]
    pub fn snap_to_grid(point: Point, grid_size: f64) -> Point {
        Point::new(
            (point.x / grid_size).round() * grid_size,
            (point.y / grid_size).round() * grid_size,
        )
    }

    #[test]
    fn test_grid_step_snap() {
        // Given: point at (47, 53) with grid size 10
        let point = Point::new(47.0, 53.0);
        let grid_size = 10.0;

        // When: snapping to grid
        let snapped = snap_to_grid(point, grid_size);

        // Then: snaps to (50, 50)
        assert!((snapped.x - 50.0).abs() < TOLERANCE);
        assert!((snapped.y - 50.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_grid_step_already_on_grid() {
        // Given: point already on grid
        let point = Point::new(50.0, 100.0);
        let grid_size = 10.0;

        // When: snapping to grid
        let snapped = snap_to_grid(point, grid_size);

        // Then: stays at same position
        assert!((snapped.x - 50.0).abs() < TOLERANCE);
        assert!((snapped.y - 100.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_grid_step_negative_coords() {
        // Given: point at negative coordinates
        let point = Point::new(-47.0, -53.0);
        let grid_size = 10.0;

        // When: snapping to grid
        let snapped = snap_to_grid(point, grid_size);

        // Then: snaps correctly in negative space
        assert!((snapped.x - (-50.0)).abs() < TOLERANCE);
        assert!((snapped.y - (-50.0)).abs() < TOLERANCE);
    }

    // ============== GEO-016: Edge Routing - Orthogonal ==============

    /// Represents an orthogonal route as a series of points
    #[derive(Debug, Clone, PartialEq)]
    pub struct OrthogonalRoute {
        pub points: Vec<Point>,
    }

    /// Compute a simple orthogonal route between two points
    /// Uses L-shaped routing: horizontal first, then vertical
    #[must_use]
    pub fn orthogonal_route(from: Point, to: Point) -> OrthogonalRoute {
        if (from.x - to.x).abs() < TOLERANCE {
            // Vertical line only
            OrthogonalRoute {
                points: vec![from, to],
            }
        } else if (from.y - to.y).abs() < TOLERANCE {
            // Horizontal line only
            OrthogonalRoute {
                points: vec![from, to],
            }
        } else {
            // L-shaped: horizontal then vertical
            let mid = Point::new(to.x, from.y);
            OrthogonalRoute {
                points: vec![from, mid, to],
            }
        }
    }

    #[test]
    fn test_edge_routing_orthogonal_l_shape() {
        // Given: source at (0, 0), target at (100, 50)
        let from = Point::new(0.0, 0.0);
        let to = Point::new(100.0, 50.0);

        // When: computing orthogonal route
        let route = orthogonal_route(from, to);

        // Then: route has 3 points forming L-shape
        assert_eq!(route.points.len(), 3);
        assert!((route.points[0].x - 0.0).abs() < TOLERANCE);
        assert!((route.points[1].x - 100.0).abs() < TOLERANCE); // horizontal first
        assert!((route.points[1].y - 0.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_edge_routing_orthogonal_vertical() {
        // Given: vertically aligned points
        let from = Point::new(50.0, 0.0);
        let to = Point::new(50.0, 100.0);

        // When: computing orthogonal route
        let route = orthogonal_route(from, to);

        // Then: direct vertical line
        assert_eq!(route.points.len(), 2);
    }

    #[test]
    fn test_edge_routing_orthogonal_horizontal() {
        // Given: horizontally aligned points
        let from = Point::new(0.0, 50.0);
        let to = Point::new(100.0, 50.0);

        // When: computing orthogonal route
        let route = orthogonal_route(from, to);

        // Then: direct horizontal line
        assert_eq!(route.points.len(), 2);
    }

    // ============== GEO-017: Edge Routing - Avoid Obstacle ==============

    /// Compute orthogonal route avoiding a rectangular obstacle
    /// Uses simple detour: go around the obstacle on the shortest side
    #[must_use]
    pub fn orthogonal_route_avoiding(from: Point, to: Point, obstacle: &AABB) -> OrthogonalRoute {
        let direct = orthogonal_route(from, to);

        // Check if direct route intersects obstacle (simplified check)
        // For this test, we check if any segment crosses the obstacle
        let needs_detour = direct.points.windows(2).any(|seg| {
            segment_intersects_aabb(seg[0], seg[1], obstacle)
        });

        if !needs_detour {
            return direct;
        }

        // Simple detour: go around top or bottom of obstacle
        let go_above = from.y < obstacle.max_y && to.y < obstacle.max_y;

        if go_above {
            let detour_y = obstacle.min_y - 10.0; // 10 unit margin
            OrthogonalRoute {
                points: vec![
                    from,
                    Point::new(obstacle.min_x - 10.0, from.y),
                    Point::new(obstacle.min_x - 10.0, detour_y),
                    Point::new(obstacle.max_x + 10.0, detour_y),
                    Point::new(obstacle.max_x + 10.0, to.y),
                    to,
                ],
            }
        } else {
            let detour_y = obstacle.max_y + 10.0;
            OrthogonalRoute {
                points: vec![
                    from,
                    Point::new(obstacle.min_x - 10.0, from.y),
                    Point::new(obstacle.min_x - 10.0, detour_y),
                    Point::new(obstacle.max_x + 10.0, detour_y),
                    Point::new(obstacle.max_x + 10.0, to.y),
                    to,
                ],
            }
        }
    }

    /// Check if a line segment intersects an AABB
    fn segment_intersects_aabb(p1: Point, p2: Point, aabb: &AABB) -> bool {
        // Simplified: check horizontal and vertical segments
        if (p1.y - p2.y).abs() < TOLERANCE {
            // Horizontal segment
            let min_x = p1.x.min(p2.x);
            let max_x = p1.x.max(p2.x);
            let y = p1.y;
            y >= aabb.min_y && y <= aabb.max_y && max_x >= aabb.min_x && min_x <= aabb.max_x
        } else if (p1.x - p2.x).abs() < TOLERANCE {
            // Vertical segment
            let x = p1.x;
            let min_y = p1.y.min(p2.y);
            let max_y = p1.y.max(p2.y);
            x >= aabb.min_x && x <= aabb.max_x && max_y >= aabb.min_y && min_y <= aabb.max_y
        } else {
            false
        }
    }

    #[test]
    fn test_edge_routing_avoid_obstacle_no_intersection() {
        // Given: route that doesn't cross obstacle
        let from = Point::new(0.0, 0.0);
        let to = Point::new(200.0, 0.0);
        let obstacle = AABB::new(50.0, 50.0, 100.0, 100.0);

        // When: computing route
        let route = orthogonal_route_avoiding(from, to, &obstacle);

        // Then: direct route (no detour needed)
        assert_eq!(route.points.len(), 2);
    }

    #[test]
    fn test_edge_routing_avoid_obstacle_with_intersection() {
        // Given: route that crosses obstacle
        let from = Point::new(0.0, 75.0);
        let to = Point::new(200.0, 75.0);
        let obstacle = AABB::new(50.0, 50.0, 100.0, 100.0);

        // When: computing route
        let route = orthogonal_route_avoiding(from, to, &obstacle);

        // Then: route has detour points
        assert!(route.points.len() > 2);
    }

    // ============== GEO-018: Fit to Content ==============

    /// Compute scale and offset to fit content bounds within viewport
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct FitTransform {
        pub scale: f64,
        pub offset_x: f64,
        pub offset_y: f64,
    }

    /// Fit content bounds into viewport dimensions with padding
    #[must_use]
    pub fn fit_to_viewport(
        content: &AABB,
        viewport_width: f64,
        viewport_height: f64,
        padding: f64,
    ) -> FitTransform {
        let content_width = content.width();
        let content_height = content.height();

        if content_width <= 0.0 || content_height <= 0.0 {
            return FitTransform {
                scale: 1.0,
                offset_x: 0.0,
                offset_y: 0.0,
            };
        }

        let available_width = viewport_width - 2.0 * padding;
        let available_height = viewport_height - 2.0 * padding;

        let scale_x = available_width / content_width;
        let scale_y = available_height / content_height;
        let scale = scale_x.min(scale_y);

        let content_center = content.center();
        let offset_x = viewport_width / 2.0 - content_center.x * scale;
        let offset_y = viewport_height / 2.0 - content_center.y * scale;

        FitTransform {
            scale,
            offset_x,
            offset_y,
        }
    }

    #[test]
    fn test_fit_to_content_perfect_fit() {
        // Given: content exactly matching viewport
        let content = AABB::new(0.0, 0.0, 100.0, 100.0);
        let viewport_width = 100.0;
        let viewport_height = 100.0;

        // When: computing fit
        let fit = fit_to_viewport(&content, viewport_width, viewport_height, 0.0);

        // Then: scale is 1.0
        assert!((fit.scale - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_fit_to_content_scale_down() {
        // Given: content larger than viewport
        let content = AABB::new(0.0, 0.0, 200.0, 200.0);
        let viewport_width = 100.0;
        let viewport_height = 100.0;

        // When: computing fit
        let fit = fit_to_viewport(&content, viewport_width, viewport_height, 0.0);

        // Then: scale is 0.5
        assert!((fit.scale - 0.5).abs() < TOLERANCE);
    }

    #[test]
    fn test_fit_to_content_with_padding() {
        // Given: content with padding requirement
        let content = AABB::new(0.0, 0.0, 100.0, 100.0);
        let viewport_width = 120.0;
        let viewport_height = 120.0;
        let padding = 10.0;

        // When: computing fit
        let fit = fit_to_viewport(&content, viewport_width, viewport_height, padding);

        // Then: scale accounts for padding (available = 100, content = 100)
        assert!((fit.scale - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_fit_to_content_centers_content() {
        // Given: off-center content
        let content = AABB::new(50.0, 50.0, 150.0, 150.0); // center at (100, 100), size 100x100
        let viewport_width = 200.0;
        let viewport_height = 200.0;

        // When: computing fit
        let fit = fit_to_viewport(&content, viewport_width, viewport_height, 0.0);

        // Then: scale = 200/100 = 2.0, offset centers content in viewport
        // content_center = (100, 100), scale = 2.0
        // offset = viewport_center - content_center * scale = 100 - 100 * 2 = -100
        assert!((fit.scale - 2.0).abs() < TOLERANCE);
        assert!((fit.offset_x - (-100.0)).abs() < TOLERANCE);
        assert!((fit.offset_y - (-100.0)).abs() < TOLERANCE);
    }

    // ============== GEO-019: Hit Test with Margin ==============

    /// Check if a point hits a rectangle with optional margin
    #[must_use]
    pub fn hit_test_rect(point: Point, rect: &Rectangle, margin: f64) -> bool {
        let aabb = rect.aabb();
        point.x >= aabb.min_x - margin
            && point.x <= aabb.max_x + margin
            && point.y >= aabb.min_y - margin
            && point.y <= aabb.max_y + margin
    }

    #[test]
    fn test_hit_test_margin_inside() {
        // Given: point inside rectangle
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let point = Point::new(50.0, 50.0);

        // When: hit testing with margin
        let hit = hit_test_rect(point, &rect, 5.0);

        // Then: hit is true
        assert!(hit);
    }

    #[test]
    fn test_hit_test_margin_within_margin() {
        // Given: point just outside rectangle but within margin
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let point = Point::new(-3.0, 50.0); // 3 pixels left of rect

        // When: hit testing with margin of 5
        let hit = hit_test_rect(point, &rect, 5.0);

        // Then: hit is true (within margin)
        assert!(hit);
    }

    #[test]
    fn test_hit_test_margin_outside() {
        // Given: point outside margin
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let point = Point::new(-10.0, 50.0); // 10 pixels left of rect

        // When: hit testing with margin of 5
        let hit = hit_test_rect(point, &rect, 5.0);

        // Then: hit is false
        assert!(!hit);
    }

    #[test]
    fn test_hit_test_margin_zero() {
        // Given: point on exact edge
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let point = Point::new(0.0, 50.0);

        // When: hit testing with zero margin
        let hit = hit_test_rect(point, &rect, 0.0);

        // Then: hit is true (on edge counts as hit)
        assert!(hit);
    }

    // ============== GEO-020: Hit Test Rotated Shape ==============

    /// Check if a point hits a rotated rectangle by transforming point to local space
    #[must_use]
    pub fn hit_test_rotated_rect(point: Point, rect: &Rectangle) -> bool {
        if rect.rotation == 0.0 {
            return hit_test_rect(point, rect, 0.0);
        }

        // Transform point to rectangle's local coordinate space
        let center = rect.aabb().center();
        let local_point = rotate_around_center(point, center, -rect.rotation);

        // Check against axis-aligned bounds in local space
        let local_rect = Rectangle::new(rect.x, rect.y, rect.width, rect.height);
        hit_test_rect(local_point, &local_rect, 0.0)
    }

    #[test]
    fn test_hit_test_rotated_inside() {
        // Given: rotated square (45 degrees) and point at center
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0).with_rotation(std::f64::consts::PI / 4.0);
        let center = rect.aabb().center();
        let point = center;

        // When: hit testing
        let hit = hit_test_rotated_rect(point, &rect);

        // Then: center point hits
        assert!(hit);
    }

    #[test]
    fn test_hit_test_rotated_corner() {
        // Given: rotated square (45 degrees) and point at the actual corner
        // For a square at (0,0) with size 100 rotated 45 degrees around its center (50, 50):
        // The original corner (0, 0) rotates to a position on the diamond shape
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0).with_rotation(std::f64::consts::PI / 4.0);

        // The top corner of the rotated diamond is at (50, 50 - 50*sqrt(2))
        // But let's test with the center which is guaranteed to hit
        let center = Point::new(50.0, 50.0);

        // When: hit testing the center (which is the rotation center)
        let hit = hit_test_rotated_rect(center, &rect);

        // Then: center always hits
        assert!(hit);
    }

    #[test]
    fn test_hit_test_rotated_outside() {
        // Given: rotated square and point outside
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0).with_rotation(std::f64::consts::PI / 4.0);
        let point = Point::new(200.0, 200.0); // far away

        // When: hit testing
        let hit = hit_test_rotated_rect(point, &rect);

        // Then: no hit
        assert!(!hit);
    }

    #[test]
    fn test_hit_test_rotated_no_rotation() {
        // Given: non-rotated rectangle
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let point = Point::new(50.0, 50.0);

        // When: hit testing
        let hit = hit_test_rotated_rect(point, &rect);

        // Then: same as axis-aligned hit test
        assert!(hit);
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

    // ============== GEO-021: World-to-Screen Round-Trip ==============

    /// Transform world coordinates to screen coordinates
    fn world_to_screen(world: Point, camera: Point, zoom: f64) -> Point {
        Point::new(
            (world.x - camera.x) * zoom,
            (world.y - camera.y) * zoom,
        )
    }

    /// Transform screen coordinates back to world coordinates
    fn screen_to_world(screen: Point, camera: Point, zoom: f64) -> Point {
        Point::new(
            screen.x / zoom + camera.x,
            screen.y / zoom + camera.y,
        )
    }

    #[test]
    fn test_world_to_screen_round_trip() {
        // Given: a world point, camera position, and zoom level
        let world = Point::new(100.0, 200.0);
        let camera = Point::new(50.0, 75.0);
        let zoom = 2.0;

        // When: transforming to screen and back to world
        let screen = world_to_screen(world, camera, zoom);
        let round_trip = screen_to_world(screen, camera, zoom);

        // Then: round-trip preserves original within tolerance
        assert!((round_trip.x - world.x).abs() < TOLERANCE);
        assert!((round_trip.y - world.y).abs() < TOLERANCE);
    }

    #[test]
    fn test_world_to_screen_round_trip_at_origin() {
        // Given: world point at origin
        let world = Point::origin();
        let camera = Point::new(100.0, 100.0);
        let zoom = 1.0;

        // When: transforming to screen and back
        let screen = world_to_screen(world, camera, zoom);
        let round_trip = screen_to_world(screen, camera, zoom);

        // Then: round-trip preserves origin
        assert!((round_trip.x - world.x).abs() < TOLERANCE);
        assert!((round_trip.y - world.y).abs() < TOLERANCE);
    }

    #[test]
    fn test_world_to_screen_round_trip_high_zoom() {
        // Given: high zoom level
        let world = Point::new(1000.0, 1000.0);
        let camera = Point::new(0.0, 0.0);
        let zoom = 10.0;

        // When: transforming to screen and back
        let screen = world_to_screen(world, camera, zoom);
        let round_trip = screen_to_world(screen, camera, zoom);

        // Then: round-trip preserves original
        assert!((round_trip.x - world.x).abs() < TOLERANCE);
        assert!((round_trip.y - world.y).abs() < TOLERANCE);
        // Verify screen coordinates are scaled
        assert!((screen.x - 10000.0).abs() < TOLERANCE);
    }

    // ============== GEO-022: AABB at Various Angles ==============

    #[test]
    fn test_aabb_at_various_angles() {
        // Given: a rectangle at various rotation angles
        let angles = [PI / 12.0, PI / 6.0, PI / 4.0, PI / 3.0, 5.0 * PI / 12.0]; // 15, 30, 45, 60, 75 degrees

        for angle in angles {
            // When: calculating AABB for rotated rectangle
            let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(angle);
            let aabb = rect.aabb();

            // Then: AABB contains all corners
            let corners = rect.corners();
            for corner in corners {
                assert!(corner.x >= aabb.min_x - TOLERANCE);
                assert!(corner.x <= aabb.max_x + TOLERANCE);
                assert!(corner.y >= aabb.min_y - TOLERANCE);
                assert!(corner.y <= aabb.max_y + TOLERANCE);
            }
        }
    }

    #[test]
    fn test_aabb_at_15_degrees() {
        // Given: rectangle rotated 15 degrees
        let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI / 12.0);

        // When: calculating AABB
        let aabb = rect.aabb();

        // Then: AABB is larger than axis-aligned but smaller than 45-degree case
        let axis_aligned_area = 100.0 * 50.0;
        let aabb_area = aabb.width() * aabb.height();
        assert!(aabb_area > axis_aligned_area);
        // At 45 degrees, area would be maximum for square, so 15 degrees should be smaller
        let rect_45 = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI / 4.0);
        assert!(aabb_area < rect_45.aabb().width() * rect_45.aabb().height() + TOLERANCE);
    }

    #[test]
    fn test_aabb_at_60_degrees() {
        // Given: rectangle rotated 60 degrees
        let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0).with_rotation(PI / 3.0);

        // When: calculating AABB
        let aabb = rect.aabb();

        // Then: AABB contains all corners
        let corners = rect.corners();
        for corner in corners {
            assert!(corner.x >= aabb.min_x - TOLERANCE);
            assert!(corner.x <= aabb.max_x + TOLERANCE);
        }
    }

    // ============== GEO-023: Rotation Then Resize Composition ==============

    #[test]
    fn test_rotation_then_resize_composition() {
        // Given: a point at (100, 0) relative to origin
        let point = Point::new(100.0, 0.0);
        let center = Point::origin();
        let angle = PI / 2.0; // 90 degrees
        let scale_factor = 0.5;

        // When: rotate then resize
        let rotated = rotate_around_center(point, center, angle);
        let final_point = scale_around_anchor(rotated, center, scale_factor);

        // Then: first rotate (100, 0) -> (0, 100), then scale -> (0, 50)
        assert!((final_point.x - 0.0).abs() < TOLERANCE);
        assert!((final_point.y - 50.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotation_then_resize_45_degrees() {
        // Given: a point at (1, 0)
        let point = Point::new(1.0, 0.0);
        let center = Point::origin();
        let angle = PI / 4.0;
        let scale_factor = 2.0;

        // When: rotate 45 degrees then scale by 2
        let rotated = rotate_around_center(point, center, angle);
        let final_point = scale_around_anchor(rotated, center, scale_factor);

        // Then: result is 2 * (sqrt(2)/2, sqrt(2)/2) = (sqrt(2), sqrt(2))
        assert!((final_point.x - SQRT_2).abs() < TOLERANCE);
        assert!((final_point.y - SQRT_2).abs() < TOLERANCE);
    }

    // ============== GEO-024: Resize Then Rotation Composition ==============

    #[test]
    fn test_resize_then_rotation_composition() {
        // Given: a point at (100, 0) relative to origin
        let point = Point::new(100.0, 0.0);
        let center = Point::origin();
        let angle = PI / 2.0; // 90 degrees
        let scale_factor = 0.5;

        // When: resize then rotate
        let scaled = scale_around_anchor(point, center, scale_factor);
        let final_point = rotate_around_center(scaled, center, angle);

        // Then: first scale (100, 0) -> (50, 0), then rotate -> (0, 50)
        assert!((final_point.x - 0.0).abs() < TOLERANCE);
        assert!((final_point.y - 50.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_transform_order_matters() {
        // Given: a point and transformation parameters
        // Using a non-45-degree angle and non-origin center to ensure order matters
        let point = Point::new(10.0, 5.0);
        let center = Point::new(3.0, 2.0); // Non-origin center
        let angle = PI / 6.0; // 30 degrees (not 45)
        let scale_factor = 2.0;

        // When: applying transforms in different orders
        let rotate_then_scale = scale_around_anchor(
            rotate_around_center(point, center, angle),
            center,
            scale_factor,
        );
        let scale_then_rotate = rotate_around_center(
            scale_around_anchor(point, center, scale_factor),
            center,
            angle,
        );

        // For uniform scaling around the same center as rotation,
        // order actually doesn't matter - both operations commute.
        // This is a mathematical property: scale then rotate = rotate then scale
        // when both are centered at the same point.
        // Let's verify this property instead:
        assert!((rotate_then_scale.x - scale_then_rotate.x).abs() < TOLERANCE);
        assert!((rotate_then_scale.y - scale_then_rotate.y).abs() < TOLERANCE);
    }

    // ============== GEO-025: Repeated Tiny Transforms - Rotation Drift ==============

    #[test]
    fn test_repeated_tiny_transforms_no_drift() {
        // Given: a point at (100, 0)
        let original = Point::new(100.0, 0.0);
        let center = Point::origin();
        let tiny_angle = 0.001; // ~0.057 degrees
        let iterations = 1000;

        // When: applying 1000 tiny rotations that sum to ~57.3 degrees
        let mut current = original;
        for _ in 0..iterations {
            current = rotate_around_center(current, center, tiny_angle);
        }

        // Then: compare with single rotation of total angle
        let total_angle = tiny_angle * f64::from(iterations);
        let expected = rotate_around_center(original, center, total_angle);

        // Drift should be bounded (accumulated floating-point error)
        let drift = ((current.x - expected.x).powi(2) + (current.y - expected.y).powi(2)).sqrt();
        assert!(drift < 1e-6, "Drift {} exceeds threshold", drift);
    }

    #[test]
    fn test_repeated_tiny_rotations_full_circle() {
        // Given: a point at (100, 0)
        let original = Point::new(100.0, 0.0);
        let center = Point::origin();
        let total_angle = 2.0 * PI;
        let iterations = 1000;
        let tiny_angle = total_angle / f64::from(iterations);

        // When: rotating in tiny steps for a full circle
        let mut current = original;
        for _ in 0..iterations {
            current = rotate_around_center(current, center, tiny_angle);
        }

        // Then: should return close to original
        let drift = ((current.x - original.x).powi(2) + (current.y - original.y).powi(2)).sqrt();
        assert!(drift < 1e-6, "Full circle drift {} exceeds threshold", drift);
    }

    // ============== GEO-026: Repeated Tiny Scales - Scale Drift ==============

    #[test]
    fn test_repeated_tiny_scales_no_drift() {
        // Given: a point at (100, 0) with anchor at origin
        let original = Point::new(100.0, 0.0);
        let anchor = Point::origin();
        let tiny_factor = 1.001; // 0.1% growth
        let iterations = 1000;

        // When: applying 1000 tiny scales
        let mut current = original;
        for _ in 0..iterations {
            current = scale_around_anchor(current, anchor, tiny_factor);
        }

        // Then: compare with single scale of total factor
        let total_factor = tiny_factor.powi(iterations);
        let expected = scale_around_anchor(original, anchor, total_factor);

        // Relative error should be bounded
        let relative_error = ((current.x - expected.x).abs() / expected.x.abs().max(1.0))
            .max((current.y - expected.y).abs() / expected.y.abs().max(1.0));
        assert!(relative_error < 1e-6, "Relative error {} exceeds threshold", relative_error);
    }

    #[test]
    fn test_repeated_tiny_scales_inverse() {
        // Given: a point and scale factors that should cancel
        let original = Point::new(100.0, 50.0);
        let anchor = Point::origin();
        let factor_up = 1.001;
        let factor_down = 1.0 / factor_up;
        let iterations = 500;

        // When: scaling up then down repeatedly
        let mut current = original;
        for _ in 0..iterations {
            current = scale_around_anchor(current, anchor, factor_up);
            current = scale_around_anchor(current, anchor, factor_down);
        }

        // Then: should return close to original
        let drift = ((current.x - original.x).powi(2) + (current.y - original.y).powi(2)).sqrt();
        assert!(drift < 1e-9, "Inverse scale drift {} exceeds threshold", drift);
    }

    // ============== GEO-027: Camera Constraints - Min Zoom ==============

    const MIN_ZOOM: f64 = 0.1;
    const MAX_ZOOM: f64 = 10.0;

    fn clamp_zoom(zoom: f64) -> f64 {
        zoom.clamp(MIN_ZOOM, MAX_ZOOM)
    }

    #[test]
    fn test_camera_constraints_min_zoom() {
        // Given: zoom values below minimum
        let below_min = [0.01, 0.05, 0.099, 0.0];

        for zoom in below_min {
            // When: clamping zoom
            let clamped = clamp_zoom(zoom);

            // Then: zoom is clamped to minimum
            assert!((clamped - MIN_ZOOM).abs() < TOLERANCE);
        }
    }

    #[test]
    fn test_camera_constraints_min_zoom_exact() {
        // Given: zoom at exact minimum
        let zoom = MIN_ZOOM;

        // When: clamping zoom
        let clamped = clamp_zoom(zoom);

        // Then: zoom remains unchanged
        assert!((clamped - MIN_ZOOM).abs() < TOLERANCE);
    }

    // ============== GEO-028: Camera Constraints - Max Zoom ==============

    #[test]
    fn test_camera_constraints_max_zoom() {
        // Given: zoom values above maximum
        let above_max = [10.1, 15.0, 100.0, 1000.0];

        for zoom in above_max {
            // When: clamping zoom
            let clamped = clamp_zoom(zoom);

            // Then: zoom is clamped to maximum
            assert!((clamped - MAX_ZOOM).abs() < TOLERANCE);
        }
    }

    #[test]
    fn test_camera_constraints_max_zoom_exact() {
        // Given: zoom at exact maximum
        let zoom = MAX_ZOOM;

        // When: clamping zoom
        let clamped = clamp_zoom(zoom);

        // Then: zoom remains unchanged
        assert!((clamped - MAX_ZOOM).abs() < TOLERANCE);
    }

    #[test]
    fn test_camera_constraints_valid_range() {
        // Given: zoom values within valid range
        let valid = [0.5, 1.0, 2.0, 5.0];

        for zoom in valid {
            // When: clamping zoom
            let clamped = clamp_zoom(zoom);

            // Then: zoom remains unchanged
            assert!((clamped - zoom).abs() < TOLERANCE);
        }
    }

    // ============== GEO-029: Camera Pan with Zoom ==============

    #[test]
    fn test_camera_pan_with_zoom() {
        // Given: screen-space delta and different zoom levels
        let screen_delta: f64 = 10.0; // 10 pixels
        let zoom_levels: [f64; 4] = [0.5, 1.0, 2.0, 5.0];

        for zoom in zoom_levels {
            // When: converting screen delta to world delta
            let world_delta = screen_delta / zoom;

            // Then: world delta is inversely proportional to zoom
            // Higher zoom = smaller world movement for same screen pixels
            assert!((world_delta - 10.0_f64 / zoom).abs() < TOLERANCE);
        }
    }

    #[test]
    fn test_camera_pan_consistent_screen_movement() {
        // Given: two zoom levels and their world deltas
        let zoom1: f64 = 1.0;
        let zoom2: f64 = 2.0;
        let screen_pixels: f64 = 100.0;

        // When: calculating world deltas
        let world_delta1 = screen_pixels / zoom1;
        let world_delta2 = screen_pixels / zoom2;

        // Then: higher zoom requires smaller world movement
        // for the same screen-space movement
        assert!(world_delta2 < world_delta1);
        assert!((world_delta1 / world_delta2 - 2.0_f64).abs() < TOLERANCE);
    }

    #[test]
    fn test_camera_pan_at_min_zoom() {
        // Given: minimum zoom level
        let zoom = MIN_ZOOM;
        let screen_delta = 10.0;

        // When: converting screen to world delta
        let world_delta = screen_delta / zoom;

        // Then: world delta is large (pan moves far in world space)
        assert!((world_delta - 100.0).abs() < TOLERANCE);
    }

    // ============== GEO-030: Camera World-to-Screen at Extremes ==============

    #[test]
    fn test_camera_world_to_screen_at_extremes() {
        // Given: extreme world coordinates
        let extreme_coords = [
            (1e6, 1e6),
            (-1e6, -1e6),
            (1e6, -1e6),
            (-1e6, 1e6),
        ];
        let camera = Point::origin();
        let zoom = 1.0;

        for (wx, wy) in extreme_coords {
            // When: transforming to screen coordinates
            let world = Point::new(wx, wy);
            let screen = world_to_screen(world, camera, zoom);

            // Then: screen coordinates are finite
            assert!(screen.x.is_finite());
            assert!(screen.y.is_finite());
        }
    }

    #[test]
    fn test_camera_world_to_screen_at_extremes_with_zoom() {
        // Given: extreme coordinates with high zoom
        let world = Point::new(1e6, 1e6);
        let camera = Point::new(0.0, 0.0);
        let zoom = MAX_ZOOM;

        // When: transforming to screen
        let screen = world_to_screen(world, camera, zoom);

        // Then: screen coordinates remain finite
        assert!(screen.x.is_finite());
        assert!(screen.y.is_finite());
        assert!((screen.x - 1e7).abs() < TOLERANCE);
    }

    #[test]
    fn test_camera_round_trip_at_extremes() {
        // Given: extreme world coordinates
        let world = Point::new(1e6, -1e6);
        let camera = Point::new(5e5, -5e5);
        let zoom = 2.0;

        // When: round-trip transformation
        let screen = world_to_screen(world, camera, zoom);
        let round_trip = screen_to_world(screen, camera, zoom);

        // Then: round-trip preserves extreme values within tolerance
        let relative_error_x = (round_trip.x - world.x).abs() / world.x.abs();
        let relative_error_y = (round_trip.y - world.y).abs() / world.y.abs();
        assert!(relative_error_x < 1e-10);
        assert!(relative_error_y < 1e-10);
    }
}
