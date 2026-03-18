use dioxus::prelude::*;

#[cfg(all(feature = "async-db", not(target_arch = "wasm32")))]
use crate::ui::toast::{AiConflictState, ToastQueue};
use diagram_models::document::DiagramDocument;
use diagram_models::envelope::EventEnvelope;

#[cfg(all(feature = "async-db", not(target_arch = "wasm32")))]
use crate::ui::toast::show_conflict_toast;
#[cfg(all(feature = "async-db", not(target_arch = "wasm32")))]
use futures_util::stream::StreamExt;

#[cfg(all(feature = "async-db", not(target_arch = "wasm32")))]
pub(crate) fn provide_db_event_context() -> Option<Coroutine<EventEnvelope>> {
    let store_bridge = use_context::<std::sync::Arc<crate::store_bridge::StoreBridge>>();
    let store_bridge_tx = store_bridge.clone();
    let db_tx = use_coroutine(move |mut rx: UnboundedReceiver<EventEnvelope>| {
        let store_bridge = store_bridge_tx.clone();
        async move {
            while let Some(env) = rx.next().await {
                let _ = store_bridge.append_event_sync(&env, None);
            }
        }
    });

    use_context_provider(|| Some(db_tx));
    Some(db_tx)
}

#[cfg(any(not(feature = "async-db"), target_arch = "wasm32"))]
pub(crate) fn provide_db_event_context() -> Option<Coroutine<EventEnvelope>> {
    use_context_provider(|| Option::<Coroutine<EventEnvelope>>::None);
    None
}

#[cfg(all(feature = "async-db", not(target_arch = "wasm32")))]
pub(crate) fn use_conflict_toast_effect() {
    let toast_queue = use_context::<Signal<ToastQueue>>();
    let ai_conflict_state = use_context::<Signal<Option<AiConflictState>>>();
    let mut conflict_toast_shown = use_context::<Signal<bool>>();

    use_effect(move || {
        let has_conflict = ai_conflict_state.read().is_some();
        let already_shown = *conflict_toast_shown.read();
        if has_conflict && !already_shown {
            if let Some(conflict_state) = ai_conflict_state.read().as_ref() {
                let toast_api = crate::ui::toast::ToastApi::from_signal(toast_queue);
                let _ = show_conflict_toast(conflict_state, toast_api);
                conflict_toast_shown.set(true);
            }
        }
        if !has_conflict {
            conflict_toast_shown.set(false);
        }
    });
}

#[cfg(any(not(feature = "async-db"), target_arch = "wasm32"))]
pub(crate) fn use_conflict_toast_effect() {}

#[cfg(all(feature = "async-db", not(target_arch = "wasm32")))]
pub(crate) fn use_store_sync_loop(doc_signal: Signal<DiagramDocument>) {
    let store_bridge = use_context::<std::sync::Arc<crate::store_bridge::StoreBridge>>();
    let last_sync_revision = use_signal(|| 0_i64);
    let pending_ai_ops = use_context::<Signal<std::collections::HashSet<String>>>();
    let _ai_conflict_state = use_context::<Signal<Option<AiConflictState>>>();

    use_future(move || {
        let store_bridge = store_bridge.clone();
        let mut doc_signal = doc_signal;
        let mut last_sync_revision = last_sync_revision;
        let mut pending_ai_ops = pending_ai_ops;
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let current_rev = *last_sync_revision.read();
                if let Ok(events) = store_bridge.fetch_events_since_sync(current_rev) {
                    if !events.is_empty() {
                        let mut next_rev = current_rev;
                        doc_signal.with_mut(|doc| {
                            for event in events {
                                next_rev = next_rev.max(event.revision);
                                if let Ok(envelope) =
                                    diagram_models::envelope::parse_event_envelope(&event.payload)
                                {
                                    let op_id = event.op_id.clone();
                                    let is_ai = !diagram_models::projection::types::is_human_author(
                                        &envelope.author,
                                    );
                                    if is_ai {
                                        pending_ai_ops.with_mut(|ops| {
                                            let _ = ops.remove(&op_id);
                                        });
                                    }

                                    let proj_event = diagram_models::projection::EventRecord {
                                        op_id: event.op_id.clone(),
                                        revision: doc.revision.value(),
                                        operation: envelope.operation,
                                        author: envelope.author,
                                        timestamp: event.timestamp,
                                    };
                                    let proj = diagram_models::projection::DiagramProjection {
                                        version: doc.version,
                                        revision: doc.revision.value(),
                                        nodes: doc.document.nodes.clone(),
                                        edges: doc.document.edges.clone(),
                                        author_priority: im::HashMap::new(),
                                        cycle_policy:
                                            diagram_models::projection::CyclePolicy::default(),
                                    };
                                    if let Ok(new_proj) =
                                        diagram_models::projection::apply_event(proj, &proj_event)
                                    {
                                        doc.document.nodes = new_proj.nodes;
                                        doc.document.edges = new_proj.edges;
                                        doc.revision = diagram_models::document::Revision::new(
                                            new_proj.revision,
                                        );
                                        doc.version = new_proj.version;
                                    }
                                }
                            }
                        });
                        last_sync_revision.set(next_rev);
                    }
                }
            }
        }
    });
}

#[cfg(any(not(feature = "async-db"), target_arch = "wasm32"))]
pub(crate) fn use_store_sync_loop(_doc_signal: Signal<DiagramDocument>) {}
