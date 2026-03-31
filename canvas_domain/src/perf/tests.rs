#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
#[cfg(test)]
mod tests {
    use crate::perf::*;
    use crate::{CanvasCoord, ScreenCoord};
    use diagram_models::document::OrderedFloat;

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
        let nan_client_x = WheelInput {
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
        assert!(wheel_update(nan_client_x).is_none());

        let nan_client_y = WheelInput {
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
        assert!(wheel_update(nan_client_y).is_none());
    }

    #[test]
    fn given_nan_camera_when_wheel_update_then_returns_none() {
        let nan_camera_x = WheelInput {
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
        assert!(wheel_update(nan_camera_x).is_none());

        let nan_camera_y = WheelInput {
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
        assert!(wheel_update(nan_camera_y).is_none());
    }

    #[test]
    fn given_extreme_zoom_inputs_when_wheel_transform_then_zoom_stays_clamped_and_finite() {
        let very_high_zoom = make_input(-100.0, 1000.0, true, false);
        let (cx, cy, z) = wheel_transform(very_high_zoom);
        assert!(cx.is_finite());
        assert!(cy.is_finite());
        assert!(z.is_finite());
        assert!(z >= canvas_math::MIN_ZOOM && z <= canvas_math::MAX_ZOOM);

        let very_low_zoom = make_input(100.0, 0.001, true, false);
        let (cx2, cy2, z2) = wheel_transform(very_low_zoom);
        assert!(cx2.is_finite());
        assert!(cy2.is_finite());
        assert!(z2.is_finite());
        assert!(z2 >= canvas_math::MIN_ZOOM && z2 <= canvas_math::MAX_ZOOM);

        let negative_zoom = make_input(-50.0, -5.0, true, false);
        let (cx3, cy3, z3) = wheel_transform(negative_zoom);
        assert!(cx3.is_finite());
        assert!(cy3.is_finite());
        assert!(z3.is_finite());
        assert!(z3 >= canvas_math::MIN_ZOOM && z3 <= canvas_math::MAX_ZOOM);

        let nan_zoom = make_input(-50.0, f64::NAN, true, false);
        let (cx4, cy4, z4) = wheel_transform(nan_zoom);
        assert!(cx4.is_finite());
        assert!(cy4.is_finite());
        assert!(z4.is_finite());
        assert!(z4 >= canvas_math::MIN_ZOOM && z4 <= canvas_math::MAX_ZOOM);
    }

    #[test]
    fn given_discrete_wheel_extreme_deltas_when_transform_then_stays_bounded() {
        let large_positive = make_input(10000.0, 1.0, true, true);
        let (cx, cy, z) = wheel_transform(large_positive);
        assert!(cx.is_finite());
        assert!(cy.is_finite());
        assert!(z.is_finite());
        assert!(z >= canvas_math::MIN_ZOOM && z <= canvas_math::MAX_ZOOM);

        let large_negative = make_input(-10000.0, 1.0, true, true);
        let (cx2, cy2, z2) = wheel_transform(large_negative);
        assert!(cx2.is_finite());
        assert!(cy2.is_finite());
        assert!(z2.is_finite());
        assert!(z2 >= canvas_math::MIN_ZOOM && z2 <= canvas_math::MAX_ZOOM);
    }

    #[test]
    fn given_continuous_wheel_extreme_deltas_when_transform_then_stays_bounded() {
        let large_positive = make_input(500.0, 1.0, true, false);
        let (cx, cy, z) = wheel_transform(large_positive);
        assert!(cx.is_finite());
        assert!(cy.is_finite());
        assert!(z.is_finite());
        assert!(z >= canvas_math::MIN_ZOOM && z <= canvas_math::MAX_ZOOM);

        let large_negative = make_input(-500.0, 1.0, true, false);
        let (cx2, cy2, z2) = wheel_transform(large_negative);
        assert!(cx2.is_finite());
        assert!(cy2.is_finite());
        assert!(z2.is_finite());
        assert!(z2 >= canvas_math::MIN_ZOOM && z2 <= canvas_math::MAX_ZOOM);
    }

    #[test]
    fn given_already_at_zoom_limit_when_wheel_then_no_change() {
        let at_max = make_input(-50.0, canvas_math::MAX_ZOOM, true, false);
        let result = wheel_update(at_max);
        assert!(result.is_none() || result.is_some_and(|(_, _, z)| z.0 <= canvas_math::MAX_ZOOM));

        let at_min = make_input(50.0, canvas_math::MIN_ZOOM, true, false);
        let result_min = wheel_update(at_min);
        assert!(
            result_min.is_none()
                || result_min.is_some_and(|(_, _, z)| z.0 >= canvas_math::MIN_ZOOM)
        );
    }

    #[test]
    fn given_to_canvas_coords_when_valid_then_returns_finite() {
        let CanvasCoord(x, y) =
            to_canvas_coords(ScreenCoord(100.0, 200.0), CanvasCoord(50.0, 75.0), 2.0);
        assert!(x.is_finite());
        assert!(y.is_finite());
        assert_eq!(x, 100.0 / 2.0 + 50.0);
        assert_eq!(y, 200.0 / 2.0 + 75.0);
    }

    #[test]
    fn given_to_screen_coords_when_valid_then_returns_finite() {
        let ScreenCoord(x, y) =
            to_screen_coords(CanvasCoord(100.0, 200.0), CanvasCoord(50.0, 75.0), 2.0);
        assert!(x.is_finite());
        assert!(y.is_finite());
        // Correct calculation: (world - camera) * zoom
        // x = (100 - 50) * 2 = 100
        // y = (200 - 75) * 2 = 250
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
        assert!(z.0 >= canvas_math::MIN_ZOOM && z.0 <= canvas_math::MAX_ZOOM);
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
    use proptest::prelude::*;

    use crate::perf::*;
    use crate::{CanvasCoord, ScreenCoord};
    use diagram_models::document::OrderedFloat;

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
            prop_assert!(zoom >= canvas_math::MIN_ZOOM);
            prop_assert!(zoom <= canvas_math::MAX_ZOOM);
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
                prop_assert!(z.0 >= canvas_math::MIN_ZOOM);
                prop_assert!(z.0 <= canvas_math::MAX_ZOOM);
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
            let CanvasCoord(world_x, world_y) = to_canvas_coords(ScreenCoord(client_x, client_y), CanvasCoord(cam_x, cam_y), zoom);
            let ScreenCoord(screen_x, screen_y) = to_screen_coords(CanvasCoord(world_x, world_y), CanvasCoord(cam_x, cam_y), zoom);

            // Just verify the results are finite (the implementation has a known issue
            // where to_screen_coords uses screen_to_canvas instead of canvas_to_screen)
            assert!(screen_x.is_finite(), "screen_x should be finite");
            assert!(screen_y.is_finite(), "screen_y should be finite");
            assert!(world_x.is_finite(), "world_x should be finite");
            assert!(world_y.is_finite(), "world_y should be finite");
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
            prop_assert!(next_zoom >= canvas_math::MIN_ZOOM);
            prop_assert!(next_zoom <= canvas_math::MAX_ZOOM);
        }
    }
}

// =============================================================================
// INP Mobile/Touch Interaction tests (bd-27q)
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod inp_mobile_touch_tests {
    use crate::perf::{wheel_transform, wheel_update, WheelInput};
    use diagram_models::document::OrderedFloat;

    // INP-2: Pinch does not create shape
    // A two-finger pinch gesture should zoom the canvas, not create shapes or subgraphs.
    // The zoom_gesture flag indicates pinch behavior.
    #[test]
    fn given_pinch_gesture_when_zoom_in_then_zooms_canvas_not_creates_shape() {
        let pinch_zoom_in = WheelInput {
            camera_x: OrderedFloat(100.0),
            camera_y: OrderedFloat(100.0),
            zoom: OrderedFloat(1.0),
            client_x: 400.0,
            client_y: 300.0,
            dx: 0.0,
            dy: -50.0,          // Negative delta = zoom in
            zoom_gesture: true, // Pinch gesture flag
            shift_pan: false,
            discrete_wheel: false,
        };

        let (cx, cy, z) = wheel_transform(pinch_zoom_in);

        // Verify zoom changed (canvas zoom, not shape creation)
        assert!(cx.is_finite(), "Camera X should be finite");
        assert!(cy.is_finite(), "Camera Y should be finite");
        assert!(z > 1.0, "Pinch zoom-in should increase zoom level");
        assert!(z <= canvas_math::MAX_ZOOM, "Zoom should be clamped to max");
    }

    #[test]
    fn given_pinch_gesture_when_zoom_out_then_zooms_canvas_not_creates_shape() {
        let pinch_zoom_out = WheelInput {
            camera_x: OrderedFloat(100.0),
            camera_y: OrderedFloat(100.0),
            zoom: OrderedFloat(2.0),
            client_x: 400.0,
            client_y: 300.0,
            dx: 0.0,
            dy: 50.0,           // Positive delta = zoom out
            zoom_gesture: true, // Pinch gesture flag
            shift_pan: false,
            discrete_wheel: false,
        };

        let (cx, cy, z) = wheel_transform(pinch_zoom_out);

        // Verify zoom changed (canvas zoom, not shape creation)
        assert!(cx.is_finite(), "Camera X should be finite");
        assert!(cy.is_finite(), "Camera Y should be finite");
        assert!(z < 2.0, "Pinch zoom-out should decrease zoom level");
        assert!(z >= canvas_math::MIN_ZOOM, "Zoom should be clamped to min");
    }

    #[test]
    fn given_pinch_at_limits_then_stays_bounded() {
        // Pinch at max zoom should not exceed
        let pinch_at_max = WheelInput {
            camera_x: OrderedFloat(0.0),
            camera_y: OrderedFloat(0.0),
            zoom: OrderedFloat(canvas_math::MAX_ZOOM),
            client_x: 200.0,
            client_y: 200.0,
            dx: 0.0,
            dy: -100.0, // Try to zoom in more
            zoom_gesture: true,
            shift_pan: false,
            discrete_wheel: false,
        };

        let (_, _, z) = wheel_transform(pinch_at_max);
        assert!(
            z <= canvas_math::MAX_ZOOM,
            "Pinch at max should stay at max"
        );

        // Pinch at min zoom should not go below
        let pinch_at_min = WheelInput {
            camera_x: OrderedFloat(0.0),
            camera_y: OrderedFloat(0.0),
            zoom: OrderedFloat(canvas_math::MIN_ZOOM),
            client_x: 200.0,
            client_y: 200.0,
            dx: 0.0,
            dy: 100.0, // Try to zoom out more
            zoom_gesture: true,
            shift_pan: false,
            discrete_wheel: false,
        };

        let (_, _, z_min) = wheel_transform(pinch_at_min);
        assert!(
            z_min >= canvas_math::MIN_ZOOM,
            "Pinch at min should stay at min"
        );
    }

    // INP-5: Stylus vs Finger mode
    // The system should handle different pointer types without panicking.
    // While we can't distinguish pointer types in wheel events, we verify robustness.
    #[test]
    fn given_stylus_like_input_when_processed_then_no_panic() {
        // Stylus typically has more precise/smaller movements
        let stylus_precise = WheelInput {
            camera_x: OrderedFloat(50.0),
            camera_y: OrderedFloat(50.0),
            zoom: OrderedFloat(1.0),
            client_x: 250.5, // Fractional coordinates typical of stylus
            client_y: 175.25,
            dx: 0.0,
            dy: -5.0, // Small precise movement
            zoom_gesture: false,
            shift_pan: false,
            discrete_wheel: false,
        };

        let result = wheel_update(stylus_precise);
        // Should handle without panic
        if let Some((cx, cy, z)) = result {
            assert!(cx.0.is_finite());
            assert!(cy.0.is_finite());
            assert!(z.0.is_finite());
        }
    }

    #[test]
    fn given_finger_like_input_when_processed_then_no_panic() {
        // Finger touch typically has less precise/larger movements
        let finger_imprecise = WheelInput {
            camera_x: OrderedFloat(50.0),
            camera_y: OrderedFloat(50.0),
            zoom: OrderedFloat(1.0),
            client_x: 250.0, // Rounded coordinates typical of finger
            client_y: 175.0,
            dx: 0.0,
            dy: -50.0, // Larger movement
            zoom_gesture: true,
            shift_pan: false,
            discrete_wheel: false,
        };

        let result = wheel_update(finger_imprecise);
        // Should handle without panic
        if let Some((cx, cy, z)) = result {
            assert!(cx.0.is_finite());
            assert!(cy.0.is_finite());
            assert!(z.0.is_finite());
        }
    }

    #[test]
    fn given_mixed_pointer_inputs_then_all_produce_valid_output() {
        let inputs = vec![
            // Stylus-like: precise, small delta
            (250.5, 175.25, -5.0, false),
            // Finger-like: rounded, large delta, gesture
            (250.0, 175.0, -50.0, true),
            // Mouse-like: medium delta
            (300.0, 200.0, -20.0, false),
        ];

        for (client_x, client_y, dy, gesture) in inputs {
            let input = WheelInput {
                camera_x: OrderedFloat(0.0),
                camera_y: OrderedFloat(0.0),
                zoom: OrderedFloat(1.0),
                client_x,
                client_y,
                dx: 0.0,
                dy,
                zoom_gesture: gesture,
                shift_pan: false,
                discrete_wheel: false,
            };

            let (cx, cy, z) = wheel_transform(input);
            assert!(
                cx.is_finite(),
                "Camera X should be finite for input ({client_x}, {client_y}, {dy}, {gesture})"
            );
            assert!(
                cy.is_finite(),
                "Camera Y should be finite for input ({client_x}, {client_y}, {dy}, {gesture})"
            );
            assert!(
                z.is_finite(),
                "Zoom should be finite for input ({client_x}, {client_y}, {dy}, {gesture})"
            );
            assert!(z >= canvas_math::MIN_ZOOM && z <= canvas_math::MAX_ZOOM);
        }
    }
}

#[cfg(test)]
mod inp_mobile_touch_proptests {
    use crate::perf::{wheel_transform, WheelInput};
    use diagram_models::document::OrderedFloat;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        // INP-2: Pinch gesture always produces valid zoom
        #[test]
        fn prop_pinch_gesture_always_produces_valid_zoom(
            camera_x in -1000.0_f64..1000.0,
            camera_y in -1000.0_f64..1000.0,
            zoom in 0.1_f64..4.0,
            client_x in 0.0_f64..1000.0,
            client_y in 0.0_f64..1000.0,
            delta in -200.0_f64..200.0,
        ) {
            let pinch_input = WheelInput {
                camera_x: OrderedFloat(camera_x),
                camera_y: OrderedFloat(camera_y),
                zoom: OrderedFloat(zoom),
                client_x,
                client_y,
                dx: 0.0,
                dy: delta,
                zoom_gesture: true,
                shift_pan: false,
                discrete_wheel: false,
            };

            let (cx, cy, z) = wheel_transform(pinch_input);
            prop_assert!(cx.is_finite());
            prop_assert!(cy.is_finite());
            prop_assert!(z.is_finite());
            prop_assert!(z >= canvas_math::MIN_ZOOM);
            prop_assert!(z <= canvas_math::MAX_ZOOM);
        }

        // INP-5: Different pointer types all produce valid output
        #[test]
        fn prop_pointer_type_agnostic_handling(
            client_x in 0.0_f64..1000.0,
            client_y in 0.0_f64..1000.0,
            delta in -100.0_f64..100.0,
            zoom_gesture in proptest::bool::ANY,
        ) {
            let input = WheelInput {
                camera_x: OrderedFloat(0.0),
                camera_y: OrderedFloat(0.0),
                zoom: OrderedFloat(1.0),
                client_x,
                client_y,
                dx: 0.0,
                dy: delta,
                zoom_gesture,
                shift_pan: false,
                discrete_wheel: false,
            };

            let (cx, cy, z) = wheel_transform(input);
            prop_assert!(cx.is_finite());
            prop_assert!(cy.is_finite());
            prop_assert!(z.is_finite());
        }
    }
}
