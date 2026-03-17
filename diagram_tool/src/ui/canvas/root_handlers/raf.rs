use crate::history::History;
use crate::ui::canvas::document_ops::{
    flush_pending_pointer_update, flush_pending_wheel_update, WheelSample,
};
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;

pub fn use_raf_handler(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut interaction_mode: Signal<InteractionMode>,
    mut pending_pointer_sample: Signal<Option<(f64, f64)>>,
    mut pending_wheel_sample: Signal<Option<WheelSample>>,
    db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
) {
    use_effect(move || {
        let mut eval = document::eval(
            r"
                if (window.__seshat_canvas_pointer_raf_cleanup) {
                    window.__seshat_canvas_pointer_raf_cleanup();
                }

                let rafId = 0;
                const onFrame = () => {
                    dioxus.send({ type: 'raf' });
                    rafId = window.requestAnimationFrame(onFrame);
                };

                rafId = window.requestAnimationFrame(onFrame);
                window.__seshat_canvas_pointer_raf_cleanup = () => {
                    if (rafId !== 0) {
                        window.cancelAnimationFrame(rafId);
                    }
                };
            ",
        );

        let db_tx = db_tx.clone();
        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                if json["type"].as_str() == Some("raf") {
                    if pending_pointer_sample.read().is_some() {
                        flush_pending_pointer_update(
                            doc_signal,
                            history_signal,
                            interaction_mode,
                            pending_pointer_sample,
                            db_tx.clone(),
                        );
                    }
                    if pending_wheel_sample.read().is_some() {
                        flush_pending_wheel_update(doc_signal, pending_wheel_sample);
                    }
                }
            }
        });
    });
}
