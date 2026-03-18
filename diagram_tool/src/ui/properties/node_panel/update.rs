#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::history::History;
use diagram_models::document::{DiagramDocument, Node, NodeId};
use dioxus::prelude::*;

/// Apply Scott Wlaschin DDD principles: model explicit state transitions
/// without duplicating identical mutation boilerplate.
pub fn update_node_if_changed(
    doc_signal: &mut Signal<DiagramDocument>,
    history: &mut Signal<History>,
    node_id: &NodeId,
    has_changes: impl FnOnce(&Node) -> bool,
    update: impl FnOnce(&mut Node),
) {
    let nid = node_id.clone();
    let changed = doc_signal
        .read()
        .document
        .nodes
        .get(&nid)
        .is_some_and(has_changes);

    if changed {
        let current = doc_signal.read().clone();
        let next_h = history.read().push(current);
        *history.write() = next_h;

        doc_signal.with_mut(|doc| {
            if let Some(n) = doc.document.nodes.get_mut(&nid) {
                update(n);
                doc.revision = doc.revision.increment();
            }
        });
    }
}
