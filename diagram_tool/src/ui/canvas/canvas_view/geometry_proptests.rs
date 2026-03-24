use super::*;
use proptest::prelude::*;

fn finite_f64() -> impl Strategy<Value = f64> {
    -1000.0_f64..=1000.0_f64
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[cfg(kani)]
#[kani::proof]
    fn quadratic_bezier_point_returns_finite_for_finite_inputs(
        p0x in finite_f64(), p0y in finite_f64(),
        p1x in finite_f64(), p1y in finite_f64(),
        p2x in finite_f64(), p2y in finite_f64(),
        t in 0.0_f64..=1.0_f64,
    ) {
        let (x, y) = quadratic_bezier_point((p0x, p0y), (p1x, p1y), (p2x, p2y), t);
        prop_assert!(x.is_finite());
        prop_assert!(y.is_finite());
    }

    #[cfg(kani)]
#[kani::proof]
    fn quadratic_bezier_point_t_zero_returns_p0(
        p0x in finite_f64(), p0y in finite_f64(),
        p1x in finite_f64(), p1y in finite_f64(),
        p2x in finite_f64(), p2y in finite_f64(),
    ) {
        let (x, y) = quadratic_bezier_point((p0x, p0y), (p1x, p1y), (p2x, p2y), 0.0);
        prop_assert!((x - p0x).abs() < 1e-10);
        prop_assert!((y - p0y).abs() < 1e-10);
    }

    #[cfg(kani)]
#[kani::proof]
    fn quadratic_bezier_point_t_one_returns_p2(
        p0x in finite_f64(), p0y in finite_f64(),
        p1x in finite_f64(), p1y in finite_f64(),
        p2x in finite_f64(), p2y in finite_f64(),
    ) {
        let (x, y) = quadratic_bezier_point((p0x, p0y), (p1x, p1y), (p2x, p2y), 1.0);
        prop_assert!((x - p2x).abs() < 1e-10);
        prop_assert!((y - p2y).abs() < 1e-10);
    }

    #[cfg(kani)]
#[kani::proof]
    fn interpolate_polyline_point_t_zero_returns_first(
        x1 in finite_f64(), y1 in finite_f64(),
        x2 in finite_f64(), y2 in finite_f64(),
    ) {
        let points = vec![(x1, y1), (x2, y2)];
        let (px, py) = interpolate_polyline_point(&points, 0.0);
        prop_assert!((px - x1).abs() < 1e-10);
        prop_assert!((py - y1).abs() < 1e-10);
    }

    #[cfg(kani)]
#[kani::proof]
    fn interpolate_polyline_point_t_one_returns_last(
        x1 in finite_f64(), y1 in finite_f64(),
        x2 in finite_f64(), y2 in finite_f64(),
    ) {
        let points = vec![(x1, y1), (x2, y2)];
        let (px, py) = interpolate_polyline_point(&points, 1.0);
        prop_assert!((px - x2).abs() < 1e-10);
        prop_assert!((py - y2).abs() < 1e-10);
    }

    #[cfg(kani)]
#[kani::proof]
    fn interpolate_polyline_point_single_point_returns_that_point(
        x in finite_f64(), y in finite_f64(),
        t in 0.0_f64..=1.0_f64,
    ) {
        let points = vec![(x, y)];
        let (px, py) = interpolate_polyline_point(&points, t);
        prop_assert!((px - x).abs() < 1e-10);
        prop_assert!((py - y).abs() < 1e-10);
    }

    #[cfg(kani)]
#[kani::proof]
    fn interpolate_polyline_point_empty_returns_zero(t in 0.0_f64..=1.0_f64) {
        let points: Vec<(f64, f64)> = vec![];
        let (px, py) = interpolate_polyline_point(&points, t);
        prop_assert!((px - 0.0).abs() < 1e-10);
        prop_assert!((py - 0.0).abs() < 1e-10);
    }

    #[cfg(kani)]
#[kani::proof]
    fn quadratic_control_returns_finite_for_finite_input(
        sx in finite_f64(), sy in finite_f64(),
        tx in finite_f64(), ty in finite_f64(),
    ) {
        let (cx, cy) = quadratic_control(sx, sy, tx, ty);
        prop_assert!(cx.is_finite());
        prop_assert!(cy.is_finite());
    }

    #[cfg(kani)]
#[kani::proof]
    fn quadratic_control_lies_on_perpendicular_through_midpoint(
        sx in finite_f64(), sy in finite_f64(),
        tx in finite_f64(), ty in finite_f64(),
    ) {
        let (cx, cy) = quadratic_control(sx, sy, tx, ty);
        let mx = f64::midpoint(sx, tx);
        let my = f64::midpoint(sy, ty);
        let to_control_x = cx - mx;
        let to_control_y = cy - my;
        let edge_x = tx - sx;
        let edge_y = ty - sy;
        let dot = to_control_x * edge_x + to_control_y * edge_y;
        let scale = (edge_x.abs().max(edge_y.abs())).max(1.0);
        prop_assert!(dot.abs() < scale * 1e-9);
    }

    #[cfg(kani)]
#[kani::proof]
    fn dist_to_segment_zero_length_returns_distance_to_point(
        px in finite_f64(), py in finite_f64(),
        x in finite_f64(), y in finite_f64(),
    ) {
        let dist = dist_to_segment(px, py, x, y, x, y);
        let expected = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
        let tolerance = expected.abs().max(1.0) * 1e-10;
        prop_assert!((dist - expected).abs() <= tolerance);
    }

    #[cfg(kani)]
#[kani::proof]
    fn dist_to_segment_point_on_endpoint_returns_zero(
        x1 in finite_f64(), y1 in finite_f64(),
        x2 in finite_f64(), y2 in finite_f64(),
    ) {
        let dist_start = dist_to_segment(x1, y1, x1, y1, x2, y2);
        let dist_end = dist_to_segment(x2, y2, x1, y1, x2, y2);
        prop_assert!(dist_start < 1e-9);
        prop_assert!(dist_end < 1e-9);
    }

    #[cfg(kani)]
#[kani::proof]
    fn dist_to_segment_always_non_negative(
        px in finite_f64(), py in finite_f64(),
        x1 in finite_f64(), y1 in finite_f64(),
        x2 in finite_f64(), y2 in finite_f64(),
    ) {
        let dist = dist_to_segment(px, py, x1, y1, x2, y2);
        prop_assert!(dist >= 0.0);
    }

    #[cfg(kani)]
#[kani::proof]
    fn interpolate_polyline_point_midpoint_two_points(
        x1 in finite_f64(), y1 in finite_f64(),
        x2 in finite_f64(), y2 in finite_f64(),
    ) {
        let points = vec![(x1, y1), (x2, y2)];
        let (px, py) = interpolate_polyline_point(&points, 0.5);
        let expected_x = (x1 + x2) / 2.0;
        let expected_y = (y1 + y2) / 2.0;
        prop_assert!((px - expected_x).abs() < 1e-10);
        prop_assert!((py - expected_y).abs() < 1e-10);
    }

    #[cfg(kani)]
#[kani::proof]
    fn interpolate_polyline_point_returns_finite(
        x1 in finite_f64(), y1 in finite_f64(),
        x2 in finite_f64(), y2 in finite_f64(),
        x3 in finite_f64(), y3 in finite_f64(),
        t in 0.0_f64..=1.0_f64,
    ) {
        let points = vec![(x1, y1), (x2, y2), (x3, y3)];
        let (px, py) = interpolate_polyline_point(&points, t);
        prop_assert!(px.is_finite());
        prop_assert!(py.is_finite());
    }

    #[cfg(kani)]
#[kani::proof]
    fn interpolate_polyline_point_clamped_t_stays_in_bounds(
        x1 in finite_f64(), y1 in finite_f64(),
        x2 in finite_f64(), y2 in finite_f64(),
        t in finite_f64(),
    ) {
        let points = vec![(x1, y1), (x2, y2)];
        let t = t.clamp(0.0, 1.0);
        let (px, py) = interpolate_polyline_point(&points, t);
        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);
        prop_assert!(px >= min_x - 1e-10 && px <= max_x + 1e-10);
        prop_assert!(py >= min_y - 1e-10 && py <= max_y + 1e-10);
    }
}
