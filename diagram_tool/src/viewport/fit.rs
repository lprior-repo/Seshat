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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_fit_to_content_invalid_padding() {
        let viewport = ViewportState::new(800.0, 600.0);
        let content = AABB::new(0.0, 0.0, 100.0, 100.0);
        assert_eq!(
            viewport.fit_to_content(&content, -10.0),
            Err(ViewportError::InvalidPadding(-10.0))
        );
    }

    #[test]
    fn test_fit_to_content_invalid_content_bounds() {
        let viewport = ViewportState::new(800.0, 600.0);
        let content = AABB::new(0.0, 0.0, 0.0, 0.0); // width 0, height 0
        assert_eq!(
            viewport.fit_to_content(&content, 10.0),
            Err(ViewportError::InvalidContentBounds)
        );
    }

    #[test]
    fn test_fit_to_content_coordinate_overflow() {
        let viewport = ViewportState::new(800.0, 600.0);
        let content = AABB::new(0.0, 0.0, MAX_SAFE_COORDINATE + 1.0, 100.0);
        assert_eq!(
            viewport.fit_to_content(&content, 10.0),
            Err(ViewportError::CoordinateOverflow)
        );
    }

    #[test]
    fn test_fit_to_content_success() {
        let viewport = ViewportState::new(800.0, 600.0);
        let content = AABB::new(0.0, 0.0, 400.0, 300.0);

        let fit = viewport.fit_to_content(&content, 0.0).unwrap();
        assert_eq!(fit.scale, 2.0); // 800/400 = 2, 600/300 = 2
        assert_eq!(fit.offset_x, 0.0); // 800/2 - 200*2 = 0
        assert_eq!(fit.offset_y, 0.0); // 600/2 - 150*2 = 0
    }

    #[test]
    fn test_apply_fit() {
        let mut viewport = ViewportState::new(800.0, 600.0);
        let fit = FitTransform {
            scale: 2.0,
            offset_x: -100.0,
            offset_y: -200.0,
        };
        viewport.apply_fit(fit);
        assert_eq!(viewport.zoom(), 2.0);
        assert_eq!(viewport.camera_x(), 50.0); // -(-100)/2
        assert_eq!(viewport.camera_y(), 100.0); // -(-200)/2
    }
}
