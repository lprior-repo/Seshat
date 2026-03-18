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

#[test]
fn screen_to_canvas_identity_at_zoom_one() {
    let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 1.0);
    assert_eq!(result, Some((100.0, 200.0)));
}

#[test]
fn screen_to_canvas_scales_with_zoom() {
    let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 2.0);
    assert_eq!(result, Some((50.0, 100.0)));
}

#[test]
fn screen_to_canvas_shifts_with_camera() {
    let result = screen_to_canvas(0.0, 0.0, 100.0, 50.0, 1.0);
    assert_eq!(result, Some((100.0, 50.0)));
}

#[test]
fn screen_to_canvas_combined_transform() {
    let result = screen_to_canvas(100.0, 100.0, 500.0, 300.0, 2.0);
    assert_eq!(result, Some((550.0, 350.0)));
}

#[test]
fn screen_to_canvas_returns_none_for_zero_zoom() {
    let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 0.0);
    assert!(result.is_none());
}

#[test]
fn screen_to_canvas_returns_none_for_negative_zoom() {
    let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, -1.0);
    assert!(result.is_none());
}

#[test]
fn screen_to_canvas_returns_none_for_infinite_zoom() {
    let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::INFINITY);
    assert!(result.is_none());
}

#[test]
fn screen_to_canvas_returns_none_for_nan_zoom() {
    let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, f64::NAN);
    assert!(result.is_none());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    #[allow(clippy::unwrap_used)]
    fn prop_screen_to_canvas_roundtrip(
        client_x in -1e6_f64..1e6_f64,
        client_y in -1e6_f64..1e6_f64,
        camera_x in -1e6_f64..1e6_f64,
        camera_y in -1e6_f64..1e6_f64,
        zoom in 0.01_f64..10.0_f64,
    ) {
        let result = screen_to_canvas(client_x, client_y, camera_x, camera_y, zoom);
        prop_assert!(result.is_some());
        let (cx, cy) = result.unwrap();
        let scale = (camera_x.abs() + camera_y.abs() + client_x.abs() / zoom.abs() + client_y.abs() / zoom.abs()).max(1.0);
        prop_assert!((cx - camera_x - client_x / zoom).abs() / scale < 1e-9);
        prop_assert!((cy - camera_y - client_y / zoom).abs() / scale < 1e-9);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn prop_screen_to_canvas_zoom_edge_cases(zoom in prop::sample::select(&[
        f64::EPSILON,
        f64::MIN_POSITIVE,
        1e-300,
    ])) {
        let result = screen_to_canvas(100.0, 100.0, 0.0, 0.0, zoom);
        if zoom > f64::EPSILON && zoom.is_finite() {
            prop_assert!(result.is_some());
        } else {
            prop_assert!(result.is_none());
        }
    }
}
