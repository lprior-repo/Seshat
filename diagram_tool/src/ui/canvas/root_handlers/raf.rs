use crate::history::History;
use crate::ui::canvas::document_ops::{
    flush_pending_pointer_update, flush_pending_wheel_update, WheelSample,
};
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;

/// Flush pointer/wheel samples on every `requestAnimationFrame` for vsync-aligned updates.
/// Falls back to 16ms polling only on non-WASM targets where rAF is unavailable.
pub fn use_raf_handler(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    interaction_mode: Signal<InteractionMode>,
    pending_pointer_sample: Signal<Option<(f64, f64)>>,
    pending_wheel_sample: Signal<Option<WheelSample>>,
    db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
) {
    use_hook(move || {
        spawn(async move {
            loop {
                // On WASM: use requestAnimationFrame for vsync-aligned frame updates.
                // This eliminates timer drift and synchronizes with the display refresh
                // rate (60/120/144Hz), producing smoother drag feedback.
                #[cfg(target_arch = "wasm32")]
                wait_for_animation_frame().await;

                #[cfg(not(target_arch = "wasm32"))]
                tokio::time::sleep(std::time::Duration::from_millis(16)).await;

                if pending_pointer_sample.read().is_some() {
                    flush_pending_pointer_update(
                        doc_signal,
                        history_signal,
                        interaction_mode,
                        pending_pointer_sample,
                        db_tx,
                    );
                }
                if pending_wheel_sample.read().is_some() {
                    flush_pending_wheel_update(doc_signal, pending_wheel_sample);
                }
            }
        });
    });
}

/// Await the next `requestAnimationFrame` tick via a oneshot channel.
///
/// This aligns the update loop with the display refresh rate, eliminating drift
/// from the old `gloo_timers::sleep(16ms)` approach (which accumulated drift each
/// frame since processing time wasn't subtracted from the interval).
///
/// Uses `Closure::wrap` because `requestAnimationFrame` expects a `&js_sys::Function`.
/// The oneshot channel silently drops duplicate resolves after the first fire.
#[cfg(target_arch = "wasm32")]
async fn wait_for_animation_frame() {
    use wasm_bindgen::JsCast;

    let (tx, rx) = futures_channel::oneshot::channel::<()>();
    // Closure::wrap requires a Box<dyn FnMut>. We wrap tx in Option and .take() it
    // on the first invocation so the closure is technically `FnMut` (subsequent calls
    // are no-ops, which is correct — oneshot ignores extra sends).
    let cb = wasm_bindgen::prelude::Closure::wrap(Box::new({
        let mut tx = Some(tx);
        move || {
            if let Some(tx) = tx.take() {
                let _ = tx.send(());
            }
        }
    }) as Box<dyn FnMut()>);
    // Forget the closure so it isn't dropped before rAF fires.
    // This is safe: rAF invokes the callback exactly once (or not at all if the
    // tab is hidden), after which the JS GC reclaims it. The closure captures only
    // the oneshot `tx` sender, so forgetting it here is a deliberate resource transfer.
    // SAFETY: The Closure is converted to a raw JS function pointer via `into_js_value`.
    // The browser holds the reference until rAF invokes it, then drops it. We must
    // not reclaim it in Rust, hence `forget`.
    let cb_fn: js_sys::Function = cb.into_js_value().unchecked_into();
    if let Some(window) = web_sys::window() {
        let _ = window.request_animation_frame(&cb_fn);
    }
    let _ = rx.await;
}
