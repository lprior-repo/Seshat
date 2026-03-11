use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

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

