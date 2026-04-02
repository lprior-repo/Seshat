use diagram_models::document::{DiagramDocument, NodeId};
use dioxus::prelude::*;
use im::HashMap;

use crate::ui::dispatch::create::create_node_move_envelope;

pub fn dispatch_drag_move_batch(
    original_positions: &HashMap<NodeId, (f64, f64)>,
    doc: &DiagramDocument,
    db_tx: &Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
) {
    if let Some(tx) = db_tx {
        original_positions
            .iter()
            .for_each(|(id, (original_x, original_y))| {
                let maybe_envelope = doc.document.nodes.get(id).and_then(|node| {
                    let moved = (node.x.0 - *original_x).abs() > f64::EPSILON
                        || (node.y.0 - *original_y).abs() > f64::EPSILON;
                    if moved {
                        create_node_move_envelope(
                            id.clone(),
                            *original_x,
                            *original_y,
                            node.x.0,
                            node.y.0,
                        )
                        .ok()
                    } else {
                        None
                    }
                });
                if let Some(envelope) = maybe_envelope {
                    tx.send(envelope);
                }
            });
    }
}
