use canvas_domain::perf::{normalize_viewport, viewport_changed};
use dioxus::prelude::*;
use std::time::Duration;

/// Poll interval for detecting canvas resize events (~10fps is sufficient for layout changes).
const RESIZE_POLL_INTERVAL_MS: u64 = 100;

#[cfg(target_arch = "wasm32")]
fn get_canvas_rect() -> Option<(f64, f64, f64, f64)> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let element = document.query_selector(".canvas-container").ok()??;
    let rect = element.get_bounding_client_rect();
    Some((rect.left(), rect.top(), rect.width(), rect.height()))
}

#[cfg(not(target_arch = "wasm32"))]
fn get_canvas_rect() -> Option<(f64, f64, f64, f64)> {
    Some((0.0, 0.0, 1200.0, 800.0))
}

pub fn use_resize_handler(
    mut canvas_origin: Signal<(f64, f64)>,
    mut viewport_size: Signal<(f64, f64)>,
) {
    use_hook(move || {
        spawn(async move {
            let mut last_left = f64::NAN;
            let mut last_top = f64::NAN;
            let mut last_width = f64::NAN;
            let mut last_height = f64::NAN;

            loop {
                #[cfg(target_arch = "wasm32")]
                gloo_timers::future::sleep(Duration::from_millis(RESIZE_POLL_INTERVAL_MS)).await;

                #[cfg(not(target_arch = "wasm32"))]
                tokio::time::sleep(Duration::from_millis(RESIZE_POLL_INTERVAL_MS)).await;

                if let Some((left, top, width, height)) = get_canvas_rect() {
                    if last_left.is_nan()
                        || (left - last_left).abs() > 0.5
                        || (top - last_top).abs() > 0.5
                        || (width - last_width).abs() > 0.5
                        || (height - last_height).abs() > 0.5
                    {
                        last_left = left;
                        last_top = top;
                        last_width = width;
                        last_height = height;

                        canvas_origin.set((left, top));
                        let next = normalize_viewport(width, height);
                        let current = *viewport_size.peek();
                        if viewport_changed(current, next) {
                            viewport_size.set(next);
                        }
                    }
                }
            }
        });
    });
}
