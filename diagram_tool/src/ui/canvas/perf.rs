#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::OrderedFloat;

const ZOOM_MIN: f64 = 0.1;
const ZOOM_MAX: f64 = 4.0;
const VIEWPORT_EPSILON: f64 = 0.5;

#[must_use]
fn sanitize_zoom(zoom: f64) -> Option<f64> {
    (zoom.is_finite() && zoom > f64::EPSILON).then_some(zoom.clamp(ZOOM_MIN, ZOOM_MAX))
}

#[derive(Clone, Copy, Debug)]
pub(super) struct WheelInput {
    pub camera_x: OrderedFloat,
    pub camera_y: OrderedFloat,
    pub zoom: OrderedFloat,
    pub client_x: f64,
    pub client_y: f64,
    pub dx: f64,
    pub dy: f64,
    pub zoom_gesture: bool,
    pub shift_pan: bool,
    pub discrete_wheel: bool,
}

#[must_use]
pub(super) fn to_canvas_coords(
    client_x: f64,
    client_y: f64,
    cam_x: f64,
    cam_y: f64,
    zoom: f64,
) -> (f64, f64) {
    ((client_x / zoom) + cam_x, (client_y / zoom) + cam_y)
}

#[must_use]
pub(super) fn to_screen_coords(
    world_x: f64,
    world_y: f64,
    cam_x: f64,
    cam_y: f64,
    zoom: f64,
) -> (f64, f64) {
    ((world_x - cam_x) * zoom, (world_y - cam_y) * zoom)
}

#[must_use]
pub(super) fn wheel_transform(input: WheelInput) -> (f64, f64, f64) {
    let current_zoom = sanitize_zoom(input.zoom.0).unwrap_or(1.0);
    let wheel_delta = if input.shift_pan {
        if input.dx.abs() > f64::EPSILON {
            input.dx
        } else {
            input.dy
        }
    } else if input.dy.abs() > f64::EPSILON {
        input.dy
    } else {
        input.dx
    };

    let next_zoom = if input.discrete_wheel {
        let factor = if wheel_delta > 0.0 { 0.9 } else { 1.1 };
        (current_zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX)
    } else {
        let intensity = if input.zoom_gesture { 0.01 } else { 0.006 };
        let factor = wheel_delta.mul_add(-intensity, 1.0);
        (current_zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX)
    };

    if (next_zoom - current_zoom).abs() <= f64::EPSILON {
        (input.camera_x.0, input.camera_y.0, current_zoom)
    } else {
        let factor = current_zoom / next_zoom;
        let (wx, wy) = to_canvas_coords(
            input.client_x,
            input.client_y,
            input.camera_x.0,
            input.camera_y.0,
            current_zoom,
        );
        (
            (wx - input.camera_x.0).mul_add(-factor, wx),
            (wy - input.camera_y.0).mul_add(-factor, wy),
            next_zoom,
        )
    }
}

#[must_use]
pub(super) fn wheel_update(
    input: WheelInput,
) -> Option<(OrderedFloat, OrderedFloat, OrderedFloat)> {
    if !input.client_x.is_finite()
        || !input.client_y.is_finite()
        || !input.dx.is_finite()
        || !input.dy.is_finite()
        || !input.camera_x.0.is_finite()
        || !input.camera_y.0.is_finite()
    {
        return None;
    }

    let (next_x, next_y, next_zoom) = wheel_transform(input);

    if !next_x.is_finite() || !next_y.is_finite() || !next_zoom.is_finite() {
        return None;
    }

    if (next_x - input.camera_x.0).abs() <= f64::EPSILON
        && (next_y - input.camera_y.0).abs() <= f64::EPSILON
        && (next_zoom - input.zoom.0).abs() <= f64::EPSILON
    {
        None
    } else {
        Some((
            OrderedFloat(next_x),
            OrderedFloat(next_y),
            OrderedFloat(next_zoom),
        ))
    }
}

#[must_use]
pub(super) const fn normalize_viewport(width: f64, height: f64) -> (f64, f64) {
    (width.max(1.0), height.max(1.0))
}

#[must_use]
pub(super) fn viewport_changed(current: (f64, f64), next: (f64, f64)) -> bool {
    (current.0 - next.0).abs() > VIEWPORT_EPSILON || (current.1 - next.1).abs() > VIEWPORT_EPSILON
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn make_input(dy: f64, zoom: f64, zoom_gesture: bool, discrete_wheel: bool) -> WheelInput {
        WheelInput {
            camera_x: OrderedFloat(100.0),
            camera_y: OrderedFloat(100.0),
            zoom: OrderedFloat(zoom),
            client_x: 400.0,
            client_y: 300.0,
            dx: 0.0,
            dy,
            zoom_gesture,
            shift_pan: false,
            discrete_wheel,
        }
    }

    #[test]
    fn given_nan_wheel_delta_when_wheel_update_then_no_invalid_state_is_emitted() {
        let nan_input = make_input(f64::NAN, 1.0, true, false);
        let result = wheel_update(nan_input);
        assert!(result.is_none());

        let nan_input_no_gesture = make_input(f64::NAN, 1.0, false, false);
        let result_no_gesture = wheel_update(nan_input_no_gesture);
        assert!(result_no_gesture.is_none());

        let inf_input = make_input(f64::INFINITY, 1.0, true, false);
        let inf_result = wheel_update(inf_input);
        assert!(inf_result.is_none());

        let neg_inf_input = make_input(f64::NEG_INFINITY, 1.0, true, false);
        let neg_inf_result = wheel_update(neg_inf_input);
        assert!(neg_inf_result.is_none());
    }

    #[test]
    fn given_nan_client_coords_when_wheel_update_then_returns_none() {
        let nan_x_input = WheelInput {
            camera_x: OrderedFloat(100.0),
            camera_y: OrderedFloat(100.0),
            zoom: OrderedFloat(1.0),
            client_x: f64::NAN,
            client_y: 300.0,
            dx: 0.0,
            dy: -50.0,
            zoom_gesture: true,
            shift_pan: false,
            discrete_wheel: false,
        };
        assert!(wheel_update(nan_x_input).is_none());

        let nan_y_input = WheelInput {
            camera_x: OrderedFloat(100.0),
            camera_y: OrderedFloat(100.0),
            zoom: OrderedFloat(1.0),
            client_x: 400.0,
            client_y: f64::NAN,
            dx: 0.0,
            dy: -50.0,
            zoom_gesture: true,
            shift_pan: false,
            discrete_wheel: false,
        };
        assert!(wheel_update(nan_y_input).is_none());
    }

    #[test]
    fn given_nan_camera_when_wheel_update_then_returns_none() {
        let nan_cam_x_input = WheelInput {
            camera_x: OrderedFloat(f64::NAN),
            camera_y: OrderedFloat(100.0),
            zoom: OrderedFloat(1.0),
            client_x: 400.0,
            client_y: 300.0,
            dx: 0.0,
            dy: -50.0,
            zoom_gesture: true,
            shift_pan: false,
            discrete_wheel: false,
        };
        assert!(wheel_update(nan_cam_x_input).is_none());

        let nan_cam_y_input = WheelInput {
            camera_x: OrderedFloat(100.0),
            camera_y: OrderedFloat(f64::NAN),
            zoom: OrderedFloat(1.0),
            client_x: 400.0,
            client_y: 300.0,
            dx: 0.0,
            dy: -50.0,
            zoom_gesture: true,
            shift_pan: false,
            discrete_wheel: false,
        };
        assert!(wheel_update(nan_cam_y_input).is_none());
    }

    #[test]
    fn given_extreme_zoom_inputs_when_wheel_transform_then_zoom_stays_clamped_and_finite() {
        let very_high_zoom = make_input(-100.0, 1000.0, true, false);
        let (cx, cy, z) = wheel_transform(very_high_zoom);
        assert!(cx.is_finite());
        assert!(cy.is_finite());
        assert!(z.is_finite());
        assert!(z >= ZOOM_MIN && z <= ZOOM_MAX);

        let very_low_zoom = make_input(100.0, 0.001, true, false);
        let (cx2, cy2, z2) = wheel_transform(very_low_zoom);
        assert!(cx2.is_finite());
        assert!(cy2.is_finite());
        assert!(z2.is_finite());
        assert!(z2 >= ZOOM_MIN && z2 <= ZOOM_MAX);

        let negative_zoom = make_input(-50.0, -5.0, true, false);
        let (cx3, cy3, z3) = wheel_transform(negative_zoom);
        assert!(cx3.is_finite());
        assert!(cy3.is_finite());
        assert!(z3.is_finite());
        assert!(z3 >= ZOOM_MIN && z3 <= ZOOM_MAX);

        let nan_zoom = make_input(-50.0, f64::NAN, true, false);
        let (cx4, cy4, z4) = wheel_transform(nan_zoom);
        assert!(cx4.is_finite());
        assert!(cy4.is_finite());
        assert!(z4.is_finite());
        assert!(z4 >= ZOOM_MIN && z4 <= ZOOM_MAX);
    }

    #[test]
    fn given_discrete_wheel_extreme_deltas_when_transform_then_stays_bounded() {
        let large_positive = make_input(10000.0, 1.0, true, true);
        let (cx, cy, z) = wheel_transform(large_positive);
        assert!(cx.is_finite());
        assert!(cy.is_finite());
        assert!(z.is_finite());
        assert!(z >= ZOOM_MIN && z <= ZOOM_MAX);

        let large_negative = make_input(-10000.0, 1.0, true, true);
        let (cx2, cy2, z2) = wheel_transform(large_negative);
        assert!(cx2.is_finite());
        assert!(cy2.is_finite());
        assert!(z2.is_finite());
        assert!(z2 >= ZOOM_MIN && z2 <= ZOOM_MAX);
    }

    #[test]
    fn given_continuous_wheel_extreme_deltas_when_transform_then_stays_bounded() {
        let large_positive = make_input(500.0, 1.0, true, false);
        let (cx, cy, z) = wheel_transform(large_positive);
        assert!(cx.is_finite());
        assert!(cy.is_finite());
        assert!(z.is_finite());
        assert!(z >= ZOOM_MIN && z <= ZOOM_MAX);

        let large_negative = make_input(-500.0, 1.0, true, false);
        let (cx2, cy2, z2) = wheel_transform(large_negative);
        assert!(cx2.is_finite());
        assert!(cy2.is_finite());
        assert!(z2.is_finite());
        assert!(z2 >= ZOOM_MIN && z2 <= ZOOM_MAX);
    }

    #[test]
    fn given_already_at_zoom_limit_when_wheel_then_no_change() {
        let at_max = make_input(-50.0, ZOOM_MAX, true, false);
        let result = wheel_update(at_max);
        assert!(result.is_none() || result.is_some_and(|(_, _, z)| z.0 <= ZOOM_MAX));

        let at_min = make_input(50.0, ZOOM_MIN, true, false);
        let result_min = wheel_update(at_min);
        assert!(result_min.is_none() || result_min.is_some_and(|(_, _, z)| z.0 >= ZOOM_MIN));
    }

    #[test]
    fn given_sanitize_zoom_when_invalid_input_then_returns_none() {
        assert!(sanitize_zoom(f64::NAN).is_none());
        assert!(sanitize_zoom(f64::INFINITY).is_none());
        assert!(sanitize_zoom(f64::NEG_INFINITY).is_none());
        assert!(sanitize_zoom(0.0).is_none());
        assert!(sanitize_zoom(-1.0).is_none());
        assert!(sanitize_zoom(f64::EPSILON / 2.0).is_none());
    }

    #[test]
    fn given_sanitize_zoom_when_valid_input_then_clamps_to_range() {
        assert_eq!(sanitize_zoom(1.0), Some(1.0));
        assert_eq!(sanitize_zoom(2.0), Some(2.0));
        assert_eq!(sanitize_zoom(5.0), Some(ZOOM_MAX));
        assert_eq!(sanitize_zoom(0.05), Some(ZOOM_MIN));
        assert_eq!(sanitize_zoom(0.5), Some(0.5));
    }

    #[test]
    fn given_to_canvas_coords_when_valid_then_returns_finite() {
        let (x, y) = to_canvas_coords(100.0, 200.0, 50.0, 75.0, 2.0);
        assert!(x.is_finite());
        assert!(y.is_finite());
        assert_eq!(x, 100.0 / 2.0 + 50.0);
        assert_eq!(y, 200.0 / 2.0 + 75.0);
    }

    #[test]
    fn given_to_screen_coords_when_valid_then_returns_finite() {
        let (x, y) = to_screen_coords(100.0, 200.0, 50.0, 75.0, 2.0);
        assert!(x.is_finite());
        assert!(y.is_finite());
        assert_eq!(x, (100.0 - 50.0) * 2.0);
        assert_eq!(y, (200.0 - 75.0) * 2.0);
    }

    #[test]
    fn given_wheel_update_when_all_valid_then_returns_some_with_finite_values() {
        let input = make_input(-50.0, 1.0, true, false);
        let result = wheel_update(input);
        assert!(result.is_some());
        let (cx, cy, z) = result.unwrap();
        assert!(cx.0.is_finite());
        assert!(cy.0.is_finite());
        assert!(z.0.is_finite());
        assert!(z.0 >= ZOOM_MIN && z.0 <= ZOOM_MAX);
    }

    #[test]
    fn given_wheel_update_when_zero_delta_then_returns_none() {
        let input = make_input(0.0, 1.0, true, false);
        let result = wheel_update(input);
        assert!(result.is_none());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_finite_f64()(x in -1e6_f64..1e6_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_zoom_f64()(x in 0.001_f64..100.0_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_wheel_delta()(x in -500.0_f64..500.0_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_wheel_input()(
            camera_x in arb_finite_f64(),
            camera_y in arb_finite_f64(),
            zoom in arb_zoom_f64(),
            client_x in arb_finite_f64(),
            client_y in arb_finite_f64(),
            dx in arb_wheel_delta(),
            dy in arb_wheel_delta(),
            zoom_gesture in any::<bool>(),
            shift_pan in any::<bool>(),
            discrete_wheel in any::<bool>(),
        ) -> WheelInput {
            WheelInput {
                camera_x: OrderedFloat(camera_x),
                camera_y: OrderedFloat(camera_y),
                zoom: OrderedFloat(zoom),
                client_x,
                client_y,
                dx,
                dy,
                zoom_gesture,
                shift_pan,
                discrete_wheel,
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_wheel_transform_zoom_always_clamped(input in arb_wheel_input()) {
            let (_, _, zoom) = wheel_transform(input);
            prop_assert!(zoom.is_finite());
            prop_assert!(zoom >= ZOOM_MIN);
            prop_assert!(zoom <= ZOOM_MAX);
        }

        #[test]
        fn prop_wheel_transform_coords_always_finite(input in arb_wheel_input()) {
            let (cx, cy, _) = wheel_transform(input);
            prop_assert!(cx.is_finite());
            prop_assert!(cy.is_finite());
        }

        #[test]
        fn prop_wheel_update_returns_finite_or_none(input in arb_wheel_input()) {
            if let Some((cx, cy, z)) = wheel_update(input) {
                prop_assert!(cx.0.is_finite());
                prop_assert!(cy.0.is_finite());
                prop_assert!(z.0.is_finite());
                prop_assert!(z.0 >= ZOOM_MIN);
                prop_assert!(z.0 <= ZOOM_MAX);
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_sanitize_zoom_valid_clamped(zoom in 0.001_f64..100.0_f64) {
            let result = sanitize_zoom(zoom);
            prop_assert!(result.is_some());
            let sanitized = result.unwrap();
            prop_assert!(sanitized >= ZOOM_MIN);
            prop_assert!(sanitized <= ZOOM_MAX);
        }

        #[test]
        fn prop_sanitize_zoom_invalid_returns_none(zoom in proptest::num::f64::INFINITE) {
            if !zoom.is_finite() || zoom <= f64::EPSILON {
                prop_assert!(sanitize_zoom(zoom).is_none());
            }
        }

        #[test]
        fn prop_coord_transform_roundtrip(
            client_x in arb_finite_f64(),
            client_y in arb_finite_f64(),
            cam_x in arb_finite_f64(),
            cam_y in arb_finite_f64(),
            zoom in 0.1_f64..4.0_f64,
        ) {
            let (world_x, world_y) = to_canvas_coords(client_x, client_y, cam_x, cam_y, zoom);
            let (screen_x, screen_y) = to_screen_coords(world_x, world_y, cam_x, cam_y, zoom);

            let tolerance = 1e-9;
            prop_assert!((screen_x - client_x).abs() < tolerance);
            prop_assert!((screen_y - client_y).abs() < tolerance);
        }

        #[test]
        fn prop_viewport_changed_detects_significant_change(
            w1 in 1.0_f64..2000.0_f64,
            h1 in 1.0_f64..2000.0_f64,
            delta in 0.0_f64..5.0_f64,
        ) {
            let current = (w1, h1);
            let next = (w1 + delta, h1 + delta);

            if delta > VIEWPORT_EPSILON {
                prop_assert!(viewport_changed(current, next));
            }
        }

        #[test]
        fn prop_normalize_viewport_always_positive(
            width in -1000.0_f64..1000.0_f64,
            height in -1000.0_f64..1000.0_f64,
        ) {
            let (w, h) = normalize_viewport(width, height);
            prop_assert!(w >= 1.0);
            prop_assert!(h >= 1.0);
        }

        #[test]
        fn prop_discrete_wheel_zoom_factor_consistency(
            zoom in 0.2_f64..3.0_f64,
            positive_delta in proptest::bool::ANY,
        ) {
            let input = WheelInput {
                camera_x: OrderedFloat(0.0),
                camera_y: OrderedFloat(0.0),
                zoom: OrderedFloat(zoom),
                client_x: 100.0,
                client_y: 100.0,
                dx: 0.0,
                dy: if positive_delta { 10.0 } else { -10.0 },
                zoom_gesture: false,
                shift_pan: false,
                discrete_wheel: true,
            };

            let (_, _, next_zoom) = wheel_transform(input);

            if positive_delta {
                prop_assert!(next_zoom <= zoom);
            } else {
                prop_assert!(next_zoom >= zoom);
            }
        }

        #[test]
        fn prop_continuous_wheel_zoom_respects_bounds(
            zoom in 0.2_f64..3.0_f64,
            delta in -200.0_f64..200.0_f64,
        ) {
            let input = WheelInput {
                camera_x: OrderedFloat(0.0),
                camera_y: OrderedFloat(0.0),
                zoom: OrderedFloat(zoom),
                client_x: 100.0,
                client_y: 100.0,
                dx: 0.0,
                dy: delta,
                zoom_gesture: false,
                shift_pan: false,
                discrete_wheel: false,
            };

            let (_, _, next_zoom) = wheel_transform(input);
            prop_assert!(next_zoom >= ZOOM_MIN);
            prop_assert!(next_zoom <= ZOOM_MAX);
        }
    }
}
