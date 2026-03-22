use crate::history::History;
use crate::ui::canvas::document_ops::{
    flush_pending_pointer_update, flush_pending_wheel_update, WheelSample,
};
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;
use std::time::Duration;

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
                #[cfg(target_arch = "wasm32")]
                gloo_timers::future::sleep(Duration::from_millis(16)).await;

                #[cfg(not(target_arch = "wasm32"))]
                tokio::time::sleep(Duration::from_millis(16)).await;

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
