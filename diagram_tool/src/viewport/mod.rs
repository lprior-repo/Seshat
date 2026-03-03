//! Viewport module for camera/viewport operations
//!
//! This module provides the ViewportState struct and operations for managing
//! the camera transformation between screen coordinates and world coordinates.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: Zoom value must be finite and positive (default fallback to 1.0)
//! - P2: Camera coordinates must be finite (clamped if invalid)
//! - P3: Viewport dimensions must be positive (minimum 1.0)
//! - P4: Coordinate transforms require valid zoom/pan state
//! - P5: Zoom bounds: 0.1 <= zoom <= 4.0 (clamped)
//! - P6: Fit-to-viewport requires valid content bounds (returns None if invalid)
//! - P7: Pan delta must be finite
//!
//! ### Postconditions
//! - Q1: After zoom: new zoom within [0.1, 4.0]
//! - Q2: After pan: camera coordinates are finite
//! - Q3: Screen-to-world is inverse of world-to-screen
//! - Q4: Fit-to-viewport preserves aspect ratio
//! - Q5: Zoom around point keeps point under cursor
//! - Q6: State changes return true if modified, false if no change
//! - Q7: Operations are idempotent at boundaries
//!
//! ### Invariants
//! - I1: 0.1 <= zoom <= 4.0
//! - I2: camera_x is always finite
//! - I3: camera_y is always finite
//! - I4: Coordinate transforms are reversible
//! - I5: Viewport dimensions are positive

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::similar_names)]
#![allow(clippy::missing_const_for_fn)]
#![forbid(unsafe_code)]

mod operations;
mod transform;

use serde::{Deserialize, Serialize};

use crate::geometry::AABB;

pub use operations::*;
pub use transform::*;

/// Minimum allowed zoom level
pub const MIN_ZOOM: f64 = 0.1;

/// Maximum allowed zoom level
pub const MAX_ZOOM: f64 = 4.0;

/// Maximum pan distance from origin in world units
pub const MAX_PAN_DISTANCE: f64 = 10000.0;

/// Default zoom factor for zoom in operations
pub const ZOOM_IN_FACTOR: f64 = 1.25;

/// Default zoom factor for zoom out operations
pub const ZOOM_OUT_FACTOR: f64 = 0.8;

/// Viewport state representing camera position and zoom level
///
/// This struct manages the transformation between screen coordinates
/// (pixels on the viewport) and world coordinates (logical diagram space).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewportState {
    /// Camera X position in world coordinates (top-left visible world point)
    camera_x: f64,
    /// Camera Y position in world coordinates (top-left visible world point)
    camera_y: f64,
    /// Zoom level (1.0 = 100%, 2.0 = 200%, etc.)
    zoom: f64,
    /// Viewport width in screen pixels
    viewport_width: f64,
    /// Viewport height in screen pixels
    viewport_height: f64,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self::new(800.0, 600.0)
    }
}

impl ViewportState {
    /// Create a new viewport state with given dimensions
    ///
    /// # Arguments
    /// * `viewport_width` - Width of the viewport in pixels (minimum 1.0)
    /// * `viewport_height` - Height of the viewport in pixels (minimum 1.0)
    ///
    /// # Postconditions
    /// - Camera starts at origin (0, 0)
    /// - Zoom starts at 1.0
    /// - Viewport dimensions are at least 1.0
    #[must_use]
    pub fn new(viewport_width: f64, viewport_height: f64) -> Self {
        Self {
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 1.0,
            viewport_width: viewport_width.max(1.0),
            viewport_height: viewport_height.max(1.0),
        }
    }

    /// Create a viewport state with specific camera and zoom
    #[must_use]
    pub fn with_camera_and_zoom(
        viewport_width: f64,
        viewport_height: f64,
        camera_x: f64,
        camera_y: f64,
        zoom: f64,
    ) -> Self {
        let mut state = Self::new(viewport_width, viewport_height);
        state.set_camera(camera_x, camera_y);
        state.set_zoom(zoom);
        state
    }

    /// Get the camera X position
    #[must_use]
    pub const fn camera_x(&self) -> f64 {
        self.camera_x
    }

    /// Get the camera Y position
    #[must_use]
    pub const fn camera_y(&self) -> f64 {
        self.camera_y
    }

    /// Get the current zoom level
    #[must_use]
    pub const fn zoom(&self) -> f64 {
        self.zoom
    }

    /// Get the viewport width
    #[must_use]
    pub const fn viewport_width(&self) -> f64 {
        self.viewport_width
    }

    /// Get the viewport height
    #[must_use]
    pub const fn viewport_height(&self) -> f64 {
        self.viewport_height
    }

    /// Set the camera position with bounds checking
    ///
    /// # Postconditions
    /// - Camera coordinates are clamped to [-MAX_PAN_DISTANCE, MAX_PAN_DISTANCE]
    /// - NaN values are replaced with 0.0
    pub fn set_camera(&mut self, x: f64, y: f64) {
        self.camera_x = if x.is_finite() {
            x.clamp(-MAX_PAN_DISTANCE, MAX_PAN_DISTANCE)
        } else {
            0.0
        };
        self.camera_y = if y.is_finite() {
            y.clamp(-MAX_PAN_DISTANCE, MAX_PAN_DISTANCE)
        } else {
            0.0
        };
    }

    /// Set the zoom level with bounds checking
    ///
    /// # Returns
    /// true if zoom was changed, false if already at bounds
    pub fn set_zoom(&mut self, zoom: f64) -> bool {
        let new_zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom.clamp(MIN_ZOOM, MAX_ZOOM)
        } else {
            1.0
        };

        let changed = (self.zoom - new_zoom).abs() >= f64::EPSILON;
        self.zoom = new_zoom;
        changed
    }

    /// Update viewport dimensions
    pub fn set_viewport_size(&mut self, width: f64, height: f64) {
        self.viewport_width = width.max(1.0);
        self.viewport_height = height.max(1.0);
    }

    /// Pan the viewport by the given screen delta
    ///
    /// # Arguments
    /// * `dx` - Pan delta in screen pixels (positive = pan right)
    /// * `dy` - Pan delta in screen pixels (positive = pan down)
    ///
    /// # Postconditions
    /// - World appears to move opposite to pan direction
    /// - Camera is clamped to valid bounds
    pub fn pan(&mut self, dx: f64, dy: f64) -> bool {
        if !dx.is_finite() || !dy.is_finite() {
            return false;
        }

        // Convert screen delta to world delta (inverse of zoom)
        let world_dx = dx / self.zoom;
        let world_dy = dy / self.zoom;

        let new_x = self.camera_x - world_dx;
        let new_y = self.camera_y - world_dy;

        let old_x = self.camera_x;
        let old_y = self.camera_y;

        self.set_camera(new_x, new_y);

        (self.camera_x - old_x).abs() >= f64::EPSILON
            || (self.camera_y - old_y).abs() >= f64::EPSILON
    }

    /// Zoom in by the default factor
    pub fn zoom_in(&mut self) -> bool {
        self.zoom_by_factor(ZOOM_IN_FACTOR)
    }

    /// Zoom out by the default factor
    pub fn zoom_out(&mut self) -> bool {
        self.zoom_by_factor(ZOOM_OUT_FACTOR)
    }

    /// Zoom by a specific factor
    pub fn zoom_by_factor(&mut self, factor: f64) -> bool {
        if !factor.is_finite() || factor <= 0.0 {
            return false;
        }
        let new_zoom = self.zoom * factor;
        self.set_zoom(new_zoom)
    }

    /// Center the viewport on a world point
    pub fn center_on(&mut self, world_x: f64, world_y: f64) {
        if !world_x.is_finite() || !world_y.is_finite() {
            return;
        }

        // Camera position such that world point is at viewport center
        let new_camera_x = world_x - self.viewport_width / 2.0 / self.zoom;
        let new_camera_y = world_y - self.viewport_height / 2.0 / self.zoom;

        self.set_camera(new_camera_x, new_camera_y);
    }

    /// Zoom around a specific screen point (e.g., mouse position)
    ///
    /// This keeps the world point under the screen point stationary
    /// while zooming.
    pub fn zoom_around_point(&mut self, new_zoom: f64, screen_x: f64, screen_y: f64) -> bool {
        if !screen_x.is_finite()
            || !screen_y.is_finite()
            || !new_zoom.is_finite()
            || new_zoom <= 0.0
        {
            return false;
        }

        // Get the world point under the screen point before zoom
        let world_before = self.screen_to_world(screen_x, screen_y);

        // Apply the new zoom
        if !self.set_zoom(new_zoom) {
            return false;
        }

        // Adjust camera so the world point is still under the screen point
        // world_x = camera_x + screen_x / zoom
        // camera_x = world_x - screen_x / zoom
        let new_camera_x = world_before.x - screen_x / self.zoom;
        let new_camera_y = world_before.y - screen_y / self.zoom;

        self.set_camera(new_camera_x, new_camera_y);
        true
    }

    /// Fit content bounds to viewport with padding
    ///
    /// # Returns
    /// FitTransform with scale and offset, or None if content is invalid
    #[must_use]
    pub fn fit_to_content(&self, content: &AABB, padding: f64) -> Option<FitTransform> {
        let content_width = content.width();
        let content_height = content.height();

        if content_width <= 0.0 || content_height <= 0.0 {
            return None;
        }

        let available_width = (self.viewport_width - 2.0 * padding).max(1.0);
        let available_height = (self.viewport_height - 2.0 * padding).max(1.0);

        let scale_x = available_width / content_width;
        let scale_y = available_height / content_height;
        let scale = scale_x.min(scale_y).clamp(MIN_ZOOM, MAX_ZOOM);

        let content_center = content.center();
        let offset_x = self.viewport_width / 2.0 - content_center.x * scale;
        let offset_y = self.viewport_height / 2.0 - content_center.y * scale;

        Some(FitTransform {
            scale,
            offset_x,
            offset_y,
        })
    }

    /// Apply a fit transform to this viewport
    pub fn apply_fit(&mut self, fit: FitTransform) {
        self.set_zoom(fit.scale);
        // The offset represents where the camera should be
        // For fit: camera is at negative of offset (approximately)
        self.set_camera(-fit.offset_x / fit.scale, -fit.offset_y / fit.scale);
    }

    /// Convert screen coordinates to world coordinates
    #[must_use]
    pub fn screen_to_world(&self, screen_x: f64, screen_y: f64) -> WorldPoint {
        let world_x = self.camera_x + screen_x / self.zoom;
        let world_y = self.camera_y + screen_y / self.zoom;
        WorldPoint { x: world_x, y: world_y }
    }

    /// Convert world coordinates to screen coordinates
    #[must_use]
    pub fn world_to_screen(&self, world_x: f64, world_y: f64) -> ScreenPoint {
        let screen_x = (world_x - self.camera_x) * self.zoom;
        let screen_y = (world_y - self.camera_y) * self.zoom;
        ScreenPoint { x: screen_x, y: screen_y }
    }

    /// Get the visible world bounds (AABB)
    #[must_use]
    pub fn visible_world_bounds(&self) -> AABB {
        let top_left = self.screen_to_world(0.0, 0.0);
        let bottom_right = self.screen_to_world(self.viewport_width, self.viewport_height);
        AABB::new(
            top_left.x.min(bottom_right.x),
            top_left.y.min(bottom_right.y),
            top_left.x.max(bottom_right.x),
            top_left.y.max(bottom_right.y),
        )
    }
}

/// A point in world coordinates
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WorldPoint {
    pub x: f64,
    pub y: f64,
}

/// A point in screen coordinates
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

/// Result of fit-to-content calculation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitTransform {
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[cfg(test)]
mod tests;
