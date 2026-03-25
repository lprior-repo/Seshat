use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_fuzz_pan(dx in prop::num::f64::ANY, dy in prop::num::f64::ANY) {
        let mut state = ViewportState::default();
        let _ = state.pan(dx, dy);
        assert!(state.camera_x().is_finite() || state.camera_x() == 0.0);
        assert!(state.camera_y().is_finite() || state.camera_y() == 0.0);
    }

    #[test]
    fn test_fuzz_zoom(zoom in prop::num::f64::ANY) {
        let mut state = ViewportState::default();
        let _ = state.set_zoom(zoom);
        assert!(state.zoom().is_finite());
        assert!(state.zoom() > 0.0);
    }

    #[test]
    fn test_fuzz_zoom_by_factor(factor in prop::num::f64::ANY) {
        let mut state = ViewportState::default();
        let _ = state.zoom_by_factor(factor);
        assert!(state.zoom().is_finite());
        assert!(state.zoom() > 0.0);
    }

    #[test]
    fn test_fuzz_zoom_around_point(zoom in prop::num::f64::ANY, x in prop::num::f64::ANY, y in prop::num::f64::ANY) {
        let mut state = ViewportState::default();
        let _ = state.zoom_around_point(zoom, x, y);
        assert!(state.zoom().is_finite());
        assert!(state.zoom() > 0.0);
        assert!(state.camera_x().is_finite() || state.camera_x() == 0.0);
        assert!(state.camera_y().is_finite() || state.camera_y() == 0.0);
    }

    #[test]
    fn test_fuzz_set_camera(x in prop::num::f64::ANY, y in prop::num::f64::ANY) {
        let mut state = ViewportState::default();
        state.set_camera(x, y);
        assert!(state.camera_x().is_finite() || state.camera_x() == 0.0);
        assert!(state.camera_y().is_finite() || state.camera_y() == 0.0);
    }

    #[test]
    fn test_fuzz_center_on(x in prop::num::f64::ANY, y in prop::num::f64::ANY) {
        let mut state = ViewportState::default();
        state.center_on(x, y);
        assert!(state.camera_x().is_finite() || state.camera_x() == 0.0);
        assert!(state.camera_y().is_finite() || state.camera_y() == 0.0);
    }

    #[test]
    fn test_fuzz_screen_to_world(x in prop::num::f64::ANY, y in prop::num::f64::ANY) {
        let state = ViewportState::default();
        let _ = state.screen_to_world(x, y);
    }

    #[test]
    fn test_fuzz_world_to_screen(x in prop::num::f64::ANY, y in prop::num::f64::ANY) {
        let state = ViewportState::default();
        let _ = state.world_to_screen(x, y);
    }
}
