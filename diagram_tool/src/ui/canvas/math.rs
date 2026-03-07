#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

#[must_use]
pub fn safe_zoom(zoom: f64) -> Option<f64> {
    (zoom.is_finite() && zoom >= f64::EPSILON).then_some(zoom)
}

#[must_use]
pub fn sanitize_zoom(zoom: f64, min: f64, max: f64) -> Option<f64> {
    let valid = safe_zoom(zoom)?;
    if valid < min {
        Some(min)
    } else if valid > max {
        Some(max)
    } else {
        Some(valid)
    }
}

#[must_use]
pub fn within(subgraph: (f64, f64, f64, f64), node: (f64, f64, f64, f64)) -> bool {
    let (sx, sy, sw, sh) = subgraph;
    let (nx, ny, nw, nh) = node;
    if sw.is_infinite() && sh.is_infinite() && sw > 0.0 && sh > 0.0 {
        return nx >= sx && ny >= sy;
    }
    if sw.is_infinite() || sh.is_infinite() || sw <= 0.0 || sh <= 0.0 {
        return false;
    }
    nx >= sx && ny >= sy && nx + nw <= sx + sw && ny + nh <= sy + sh
}

#[must_use]
pub fn screen_to_canvas(
    client_x: f64,
    client_y: f64,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
) -> Option<(f64, f64)> {
    if !client_x.is_finite() || !client_y.is_finite() {
        return None;
    }
    let valid_zoom = safe_zoom(zoom)?;
    let cx = (client_x / valid_zoom) + camera_x;
    let cy = (client_y / valid_zoom) + camera_y;
    Some((cx, cy))
}

#[must_use]
pub fn canvas_to_screen(
    canvas_x: f64,
    canvas_y: f64,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
) -> Option<(f64, f64)> {
    let valid_zoom = safe_zoom(zoom)?;
    let sx = (canvas_x - camera_x) * valid_zoom;
    let sy = (canvas_y - camera_y) * valid_zoom;
    Some((sx, sy))
}

#[must_use]
pub fn safe_zoom_clamped(zoom: f64, min: f64, max: f64) -> Option<f64> {
    safe_zoom(zoom).map(|z| z.clamp(min, max))
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_safe_zoom_rejects_extreme_values(zoom: f64) {
            let result = safe_zoom(zoom);
            if zoom.is_nan() || zoom.is_infinite() || zoom <= f64::EPSILON {
                prop_assert!(result.is_none());
            } else {
                prop_assert!(result.is_some());
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_handles_nan_subgraph_coords(
            sx in prop::option::of(any::<f64>()),
            sy in prop::option::of(any::<f64>()),
            sw in prop::option::of(any::<f64>()),
            sh in prop::option::of(any::<f64>()),
        ) {
            let subgraph = (
                sx.unwrap_or(f64::NAN),
                sy.unwrap_or(f64::NAN),
                sw.unwrap_or(f64::NAN),
                sh.unwrap_or(f64::NAN),
            );
            let node = (10.0, 10.0, 50.0, 50.0);
            let result = within(subgraph, node);
            if sx.is_none() || sy.is_none() || sw.is_none() || sh.is_none() {
                prop_assert!(!result);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_handles_nan_node_coords(
            nx in prop::option::of(any::<f64>()),
            ny in prop::option::of(any::<f64>()),
            nw in prop::option::of(any::<f64>()),
            nh in prop::option::of(any::<f64>()),
        ) {
            let subgraph = (0.0, 0.0, 100.0, 100.0);
            let node = (
                nx.unwrap_or(f64::NAN),
                ny.unwrap_or(f64::NAN),
                nw.unwrap_or(f64::NAN),
                nh.unwrap_or(f64::NAN),
            );
            let result = within(subgraph, node);
            if nx.is_none() || ny.is_none() || nw.is_none() || nh.is_none() {
                prop_assert!(!result);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_degenerate_rectangles(
            sx: f64, sy: f64, sw: f64, sh: f64,
            nx: f64, ny: f64, nw: f64, nh: f64,
        ) {
            prop_assume!(sw.is_finite() && sh.is_finite() && nw.is_finite() && nh.is_finite());
            let subgraph = (sx, sy, sw, sh);
            let node = (nx, ny, nw, nh);
            let result = within(subgraph, node);
            if sw > 0.0 && sh > 0.0 && nw > 0.0 && nh > 0.0 && sx.is_finite() && sy.is_finite() && nx.is_finite() && ny.is_finite() {
                let nx_end = nx + nw;
                let ny_end = ny + nh;
                let sx_end = sx + sw;
                let sy_end = sy + sh;
                let should_be_within = nx >= sx && ny >= sy && nx_end <= sx_end && ny_end <= sy_end;
                prop_assert_eq!(result, should_be_within);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_infinite_dims(
            sw in prop::sample::select(&[f64::INFINITY, f64::NEG_INFINITY]),
            sh in prop::sample::select(&[f64::INFINITY, f64::NEG_INFINITY]),
        ) {
            let subgraph = (0.0, 0.0, sw, sh);
            let node = (10.0, 10.0, 10.0, 10.0);
            let result = within(subgraph, node);
            if sw.is_infinite() && sh.is_infinite() && sw > 0.0 && sh > 0.0 {
                prop_assert!(result);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_exact_boundary(x in -1e6_f64..1e6_f64, y in -1e6_f64..1e6_f64, w in 1e-6_f64..1e6_f64, h in 1e-6_f64..1e6_f64) {
            let subgraph = (x, y, w, h);
            let node = (x, y, w, h);
            prop_assert!(within(subgraph, node));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_node_on_edge(x in -1e6_f64..1e6_f64, y in -1e6_f64..1e6_f64, w in 1e-6_f64..1e6_f64, h in 1e-6_f64..1e6_f64) {
            let subgraph = (x, y, w, h);
            let node = (x, y, (w - f64::EPSILON).max(0.0), (h - f64::EPSILON).max(0.0));
            prop_assert!(within(subgraph, node));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_within_exceeds_by_epsilon(x in -1e6_f64..1e6_f64, y in -1e6_f64..1e6_f64, w in 1e-6_f64..1e6_f64, h in 1e-6_f64..1e6_f64) {
            let subgraph = (x, y, w, h);
            let exceed_amount = (w * 0.01).max(f64::EPSILON * 100.0);
            let node = (x, y, w + exceed_amount, h);
            prop_assert!(!within(subgraph, node));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_safe_zoom_boundary(
            zoom in prop::sample::select(&[
                f64::EPSILON,
                f64::EPSILON * 0.5,
                f64::EPSILON * 2.0,
                -f64::EPSILON,
                0.0,
                -0.0,
                f64::MIN_POSITIVE,
            ])
        ) {
            let result = safe_zoom(zoom);
            if zoom > f64::EPSILON && zoom.is_finite() {
                prop_assert!(result.is_some());
            } else {
                prop_assert!(result.is_none());
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_overflow_safety(
            x in prop::sample::select(&[f64::MAX / 2.0, f64::MAX * 0.99]),
            y in prop::sample::select(&[f64::MAX / 2.0, f64::MAX * 0.99]),
        ) {
            let subgraph = (x, y, f64::MAX / 4.0, f64::MAX / 4.0);
            let node = (x + 1.0, y + 1.0, 1.0, 1.0);
            let _ = within(subgraph, node);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_subnormal_floats(sw in prop::sample::select(&[f64::MIN_POSITIVE, 1e-310]), sh in prop::sample::select(&[f64::MIN_POSITIVE, 1e-310])) {
            let subgraph = (0.0, 0.0, sw, sh);
            let node = (0.0, 0.0, sw / 2.0, sh / 2.0);
            if sw > 0.0 && sh > 0.0 {
                let result = within(subgraph, node);
                prop_assert!(result);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn prop_screen_to_canvas_basic(
            cx in -1e6_f64..1e6_f64,
            cy in -1e6_f64..1e6_f64,
            camera_x in -1e6_f64..1e6_f64,
            camera_y in -1e6_f64..1e6_f64,
            zoom in 0.01_f64..100.0_f64,
        ) {
            let result = super::screen_to_canvas(
                (cx - camera_x) * zoom,
                (cy - camera_y) * zoom,
                camera_x,
                camera_y,
                zoom,
            );
            prop_assert!(result.is_some());
            let (rx, ry) = result.unwrap();
            prop_assert!((rx - cx).abs() < 1e-6);
            prop_assert!((ry - cy).abs() < 1e-6);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]
        #[test]
        fn prop_screen_to_canvas_rejects_invalid_zoom(zoom in prop::sample::select(&[0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::EPSILON, -f64::EPSILON])) {
            let result = super::screen_to_canvas(100.0, 100.0, 0.0, 0.0, zoom);
            prop_assert!(result.is_none());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn prop_screen_to_canvas_within_bounds(
            client_x in -1e6_f64..1e6_f64,
            client_y in -1e6_f64..1e6_f64,
            zoom in 0.01_f64..100.0_f64,
        ) {
            let result = super::screen_to_canvas(client_x, client_y, 0.0, 0.0, zoom);
            prop_assert!(result.is_some());
            let (cx, cy) = result.unwrap();
            let expected_x = client_x / zoom;
            let expected_y = client_y / zoom;
            prop_assert!((cx - expected_x).abs() < 1e-6);
            prop_assert!((cy - expected_y).abs() < 1e-6);
        }
    }

    #[test]
    fn screen_to_canvas_zero_camera() {
        let result = super::screen_to_canvas(100.0, 200.0, 0.0, 0.0, 2.0);
        let (cx, cy) = result.expect("valid zoom should produce Some");
        assert!((cx - 50.0).abs() < 1e-6, "x coord mismatch");
        assert!((cy - 100.0).abs() < 1e-6, "y coord mismatch");
    }

    #[test]
    fn screen_to_canvas_nonzero_camera() {
        let result = super::screen_to_canvas(100.0, 200.0, 50.0, 75.0, 2.0);
        let (cx, cy) = result.expect("valid zoom should produce Some");
        assert!((cx - 100.0).abs() < 1e-6, "x coord mismatch");
        assert!((cy - 175.0).abs() < 1e-6, "y coord mismatch");
    }

    #[test]
    fn screen_to_canvas_invalid_zoom_zero() {
        assert!(super::screen_to_canvas(100.0, 100.0, 0.0, 0.0, 0.0).is_none());
    }

    #[test]
    fn screen_to_canvas_invalid_zoom_nan() {
        assert!(super::screen_to_canvas(100.0, 100.0, 0.0, 0.0, f64::NAN).is_none());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn prop_canvas_to_screen_roundtrip(
            cx in -1e6_f64..1e6_f64,
            cy in -1e6_f64..1e6_f64,
            camera_x in -1e6_f64..1e6_f64,
            camera_y in -1e6_f64..1e6_f64,
            zoom in 0.01_f64..100.0_f64,
        ) {
            let screen_result = super::canvas_to_screen(cx, cy, camera_x, camera_y, zoom);
            prop_assert!(screen_result.is_some());
            let (sx, sy) = screen_result.unwrap();
            let back_result = super::screen_to_canvas(sx, sy, camera_x, camera_y, zoom);
            prop_assert!(back_result.is_some());
            let (rx, ry) = back_result.unwrap();
            prop_assert!((rx - cx).abs() < 1e-6);
            prop_assert!((ry - cy).abs() < 1e-6);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]
        #[test]
        fn prop_canvas_to_screen_rejects_invalid_zoom(zoom in prop::sample::select(&[0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::EPSILON, -f64::EPSILON])) {
            let result = super::canvas_to_screen(100.0, 100.0, 0.0, 0.0, zoom);
            prop_assert!(result.is_none());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn prop_safe_zoom_clamped_basic(
            zoom in 0.001_f64..10.0_f64,
            min in 0.1_f64..1.0_f64,
            max in 1.0_f64..5.0_f64,
        ) {
            prop_assume!(min < max);
            let result = super::safe_zoom_clamped(zoom, min, max);
            prop_assert!(result.is_some());
            let clamped = result.unwrap();
            prop_assert!(clamped >= min);
            prop_assert!(clamped <= max);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        fn prop_safe_zoom_clamped_rejects_invalid(zoom in prop::sample::select(&[0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY])) {
            let result = super::safe_zoom_clamped(zoom, 0.1, 4.0);
            prop_assert!(result.is_none());
        }
    }

    #[test]
    fn canvas_to_screen_zero_camera() {
        let result = super::canvas_to_screen(50.0, 100.0, 0.0, 0.0, 2.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), (100.0, 200.0));
    }

    #[test]
    fn canvas_to_screen_nonzero_camera() {
        let result = super::canvas_to_screen(100.0, 175.0, 50.0, 75.0, 2.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), (100.0, 200.0));
    }

    #[test]
    fn canvas_to_screen_clamp_below_min() {
        let result = super::safe_zoom_clamped(0.05, 0.1, 4.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 0.1);
    }

    #[test]
    fn canvas_to_screen_clamp_above_max() {
        let result = super::safe_zoom_clamped(10.0, 0.1, 4.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 4.0);
    }

    #[test]
    fn screen_to_canvas_equiv_to_perf_to_canvas_coords() {
        let test_cases = [
            (100.0, 200.0, 50.0, 75.0, 2.0),
            (0.0, 0.0, 0.0, 0.0, 1.0),
            (50.0, 100.0, 10.0, 20.0, 0.5),
            (1000.0, 500.0, 100.0, 200.0, 1.5),
        ];
        for (cx, cy, cam_x, cam_y, zoom) in test_cases {
            let math_result = super::screen_to_canvas(cx, cy, cam_x, cam_y, zoom);
            let perf_result = super::super::perf::to_canvas_coords(cx, cy, cam_x, cam_y, zoom);
            assert_eq!(
                math_result,
                Some(perf_result),
                "math::screen_to_canvas should match perf::to_canvas_coords for valid zoom"
            );
        }
    }
}
