use super::state::ViewportState;
use super::types::{FitTransform, ViewportError, MAX_SAFE_COORDINATE};
use crate::geometry::AABB;
use canvas_math::{MAX_ZOOM, MIN_ZOOM};

pub trait ViewportFit {
    fn fit_to_content(&self, content: &AABB, padding: f64) -> Result<FitTransform, ViewportError>;
    fn apply_fit(&mut self, fit: FitTransform);
}

impl ViewportFit for ViewportState {
    /// Fit content bounds to viewport with padding
    ///
    /// Handles huge coordinates safely without float overflow.
    ///
    /// # Arguments
    /// * `content` - Axis-aligned bounding box of content to fit
    /// * `padding` - Padding around content in screen pixels (must be >= 0)
    ///
    /// # Returns
    /// FitTransform with scale and offset, or Error if input is invalid
    ///
    /// # Errors
    /// Returns `ViewportError::InvalidPadding` if padding is negative
    /// Returns `ViewportError::InvalidContentBounds` if content width/height <= 0
    /// Returns `ViewportError::CoordinateOverflow` if coordinates are too large
    /// Returns `ViewportError::InvalidViewport` if viewport dimensions <= 0
    fn fit_to_content(&self, content: &AABB, padding: f64) -> Result<FitTransform, ViewportError> {
        // P1: Validate padding is non-negative
        if padding < 0.0 {
            return Err(ViewportError::InvalidPadding(padding));
        }

        // P4: Validate viewport dimensions
        if self.viewport_width() <= 0.0 || self.viewport_height() <= 0.0 {
            return Err(ViewportError::InvalidViewport);
        }

        // P2/P3: Validate content bounds
        let content_width = content.width();
        let content_height = content.height();

        if content_width <= 0.0 || content_height <= 0.0 {
            return Err(ViewportError::InvalidContentBounds);
        }

        // Q6: Check for coordinate overflow risk
        let coords = [content.min_x, content.min_y, content.max_x, content.max_y];
        if coords.iter().any(|c| c.abs() > MAX_SAFE_COORDINATE) {
            return Err(ViewportError::CoordinateOverflow);
        }

        // Calculate available space with padding
        let available_width = (self.viewport_width() - 2.0 * padding).max(1.0);
        let available_height = (self.viewport_height() - 2.0 * padding).max(1.0);

        // Calculate scale - check for invalid results
        let scale_x = available_width / content_width;
        let scale_y = available_height / content_height;

        if !scale_x.is_finite() || !scale_y.is_finite() {
            return Err(ViewportError::CoordinateOverflow);
        }

        let scale = scale_x.min(scale_y).clamp(MIN_ZOOM, MAX_ZOOM);

        // Calculate center and offsets - verify all results are finite
        let content_center = content.center();

        // Check center coordinates
        if !content_center.x.is_finite() || !content_center.y.is_finite() {
            return Err(ViewportError::CoordinateOverflow);
        }

        let offset_x = self.viewport_width() / 2.0 - content_center.x * scale;
        let offset_y = self.viewport_height() / 2.0 - content_center.y * scale;

        // Q1, Q2: Verify all output values are finite
        if !scale.is_finite() || !offset_x.is_finite() || !offset_y.is_finite() {
            return Err(ViewportError::CoordinateOverflow);
        }

        Ok(FitTransform {
            scale,
            offset_x,
            offset_y,
        })
    }

    /// Apply a fit transform to this viewport
    fn apply_fit(&mut self, fit: FitTransform) {
        self.set_zoom(fit.scale);
        // The offset represents where the camera should be
        // For fit: camera is at negative of offset (approximately)
        self.set_camera(-fit.offset_x / fit.scale, -fit.offset_y / fit.scale);
    }
}
