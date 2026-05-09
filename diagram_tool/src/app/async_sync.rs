//! Async sync layer — provides DB event context to the UI.
//!
//! After removing store_async/store_bridge, this module provides a no-op
//! context on WASM (no persistent store) and native (store removed).

use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
pub fn provide_db_event_context() -> Option<Coroutine<diagram_models::envelope::EventEnvelope>> {
    use_context_provider(|| Option::<Coroutine<diagram_models::envelope::EventEnvelope>>::None);
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn provide_db_event_context() -> Option<Coroutine<diagram_models::envelope::EventEnvelope>> {
    use_context_provider(|| Option::<Coroutine<diagram_models::envelope::EventEnvelope>>::None);
    None
}
