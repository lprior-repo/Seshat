use crate::ui::canvas::document_ops::WheelSample;
use canvas_domain::interaction_reducer::InteractionMode;
use dioxus::prelude::*;

pub fn use_touch_handler(
    mut multi_touch_active: Signal<bool>,
    mut pending_pointer_sample: Signal<Option<(f64, f64)>>,
    mut pending_wheel_sample: Signal<Option<WheelSample>>,
    mut space_pan_active: Signal<bool>,
    mut interaction_mode: Signal<InteractionMode>,
) {
    use_effect(move || {
        let mut eval = document::eval(
            r"
                if (window.__seshat_canvas_touch_guard_cleanup) {
                    window.__seshat_canvas_touch_guard_cleanup();
                }

                const reportTouches = (event) => {
                    const target = event.target;
                    const inCanvas = target && target.closest && target.closest('.canvas-container');
                    if (!inCanvas) {
                        return;
                    }
                    const touches = event.touches ? event.touches.length : 0;
                    dioxus.send({ type: 'touchmeta', touches });
                };

                const onTouchStart = (event) => reportTouches(event);
                const onTouchMove = (event) => reportTouches(event);
                const onTouchEnd = (event) => reportTouches(event);
                const onTouchCancel = (event) => reportTouches(event);

                window.addEventListener('touchstart', onTouchStart, { passive: true, capture: true });
                window.addEventListener('touchmove', onTouchMove, { passive: true, capture: true });
                window.addEventListener('touchend', onTouchEnd, { passive: true, capture: true });
                window.addEventListener('touchcancel', onTouchCancel, { passive: true, capture: true });

                window.__seshat_canvas_touch_guard_cleanup = () => {
                    window.removeEventListener('touchstart', onTouchStart, true);
                    window.removeEventListener('touchmove', onTouchMove, true);
                    window.removeEventListener('touchend', onTouchEnd, true);
                    window.removeEventListener('touchcancel', onTouchCancel, true);
                };
            ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                if json["type"].as_str() != Some("touchmeta") {
                    continue;
                }

                let touch_count = json["touches"].as_u64().map_or(0_u64, |v| v);
                let is_multi_touch = touch_count >= 2;
                multi_touch_active.set(is_multi_touch);

                if is_multi_touch {
                    pending_pointer_sample.set(None);
                    pending_wheel_sample.set(None);
                    space_pan_active.set(false);
                    interaction_mode.set(InteractionMode::Select);
                }
            }
        });
    });
}
